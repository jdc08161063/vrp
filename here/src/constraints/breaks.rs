#[cfg(test)]
#[path = "../../tests/unit/constraints/breaks_test.rs"]
mod breaks_test;

use core::construction::constraints::*;
use core::construction::states::{ActivityContext, RouteContext, SolutionContext};
use core::models::common::{Cost, IdDimension, ValueDimension};
use core::models::problem::{Job, Single};
use core::models::solution::Activity;
use std::collections::HashSet;
use std::slice::Iter;
use std::sync::Arc;

pub struct BreakModule {
    conditional: ConditionalJobModule,
    constraints: Vec<ConstraintVariant>,
    /// Controls whether break should be considered as unassigned job
    demote_breaks_from_unassigned: bool,
}

impl BreakModule {
    pub fn new(code: i32, extra_break_cost: Option<Cost>, demote_breaks_from_unassigned: bool) -> Self {
        Self {
            conditional: ConditionalJobModule::new(Box::new(|ctx, job| is_required_job(ctx, job))),
            constraints: vec![
                ConstraintVariant::HardActivity(Arc::new(BreakHardActivityConstraint { code })),
                ConstraintVariant::SoftActivity(Arc::new(BreakSoftActivityConstraint { extra_break_cost })),
            ],
            demote_breaks_from_unassigned,
        }
    }
}

impl ConstraintModule for BreakModule {
    fn accept_route_state(&self, ctx: &mut RouteContext) {
        self.conditional.accept_route_state(ctx);
    }

    fn accept_solution_state(&self, ctx: &mut SolutionContext) {
        self.conditional.accept_solution_state(ctx);

        remove_orphan_breaks(ctx);

        if self.demote_breaks_from_unassigned {
            demote_unassigned_breaks(ctx);
        }
    }

    fn state_keys(&self) -> Iter<i32> {
        self.conditional.state_keys()
    }

    fn get_constraints(&self) -> Iter<ConstraintVariant> {
        self.constraints.iter()
    }
}

struct BreakHardActivityConstraint {
    code: i32,
}

impl BreakHardActivityConstraint {
    fn stop(&self) -> Option<ActivityConstraintViolation> {
        Some(ActivityConstraintViolation { code: self.code, stopped: false })
    }
}

impl HardActivityConstraint for BreakHardActivityConstraint {
    fn evaluate_activity(
        &self,
        route_ctx: &RouteContext,
        activity_ctx: &ActivityContext,
    ) -> Option<ActivityConstraintViolation> {
        let break_job = as_break_job(activity_ctx.target);

        if let Some(break_job) = break_job {
            // avoid assigning break right after departure
            if activity_ctx.prev.job.is_none() {
                return self.stop();
            } else {
                // lock break to specific vehicle
                let is_correct_vehicle = get_vehicle_id_from_break(&break_job)
                    .map_or(false, |vehicle_id| vehicle_id == get_vehicle_id_from_ctx(route_ctx));
                if !is_correct_vehicle {
                    return self.stop();
                }
            }
        }

        None
    }
}

struct BreakSoftActivityConstraint {
    /// Allows to control whether break should be preferable for insertion
    extra_break_cost: Option<Cost>,
}

impl SoftActivityConstraint for BreakSoftActivityConstraint {
    fn estimate_activity(&self, _route_ctx: &RouteContext, activity_ctx: &ActivityContext) -> f64 {
        if let Some(cost) = self.extra_break_cost {
            (if as_break_job(activity_ctx.target).is_some() { cost } else { 0. })
        } else {
            0.
        }
    }
}

/// Mark job as ignored only if it has break type and vehicle id is not present in routes
fn is_required_job(ctx: &SolutionContext, job: &Arc<Job>) -> bool {
    match job.as_ref() {
        Job::Single(job) => {
            if is_break_job(job) {
                let vehicle_id = get_vehicle_id_from_break(job.as_ref()).unwrap();
                !ctx.required.is_empty() && ctx.routes.iter().any(move |rc| get_vehicle_id_from_ctx(rc) == vehicle_id)
            } else {
                true
            }
        }
        Job::Multi(_) => true,
    }
}

fn is_break_job(job: &Arc<Single>) -> bool {
    job.dimens.get_value::<String>("type").map_or(false, |t| t == "break")
}

fn as_break_job(activity: &Activity) -> Option<Arc<Single>> {
    activity.job.as_ref().and_then(|job| match job.as_ref() {
        Job::Single(job) => {
            if is_break_job(job) {
                Some(job.clone())
            } else {
                None
            }
        }
        _ => None,
    })
}

/// Remove some breaks from required jobs as we don't want to consider breaks
/// as unassigned jobs if they are outside of vehicle's time window
fn demote_unassigned_breaks(ctx: &mut SolutionContext) {
    if ctx.unassigned.is_empty() {
        return;
    }

    // NOTE remove all breaks from list of unassigned jobs
    let breaks_set: HashSet<_> = ctx
        .unassigned
        .iter()
        .filter_map(|(job, _)| match job.as_ref() {
            Job::Single(single) => get_vehicle_id_from_break(single.as_ref()).map(|_| job.clone()),
            Job::Multi(_) => None,
        })
        .collect();

    ctx.unassigned.retain(|job, _| breaks_set.get(job).is_none());
    ctx.ignored.extend(breaks_set.into_iter());
}

fn get_vehicle_id_from_ctx(ctx: &RouteContext) -> String {
    ctx.route.actor.vehicle.dimens.get_id().unwrap().clone()
}

fn get_vehicle_id_from_break(job: &Single) -> Option<String> {
    job.dimens.get_value::<String>("vehicle_id").cloned()
}

/// Removes breaks without location served separately.They are left-overs
/// from ruin methods when original job is removed, but break is kept.
fn remove_orphan_breaks(ctx: &mut SolutionContext) {
    let breaks_set = ctx.routes.iter_mut().fold(HashSet::new(), |mut acc, rc: &mut RouteContext| {
        // NOTE assume that first activity is never break (should be always departure)
        let (_, breaks_set) =
            rc.route.tour.all_activities().fold((0, HashSet::new()), |(prev, mut breaks), activity| {
                let current = activity.place.location;

                if let Some(break_job) = as_break_job(activity) {
                    // TODO support multiple places for break
                    assert_eq!(break_job.places.len(), 1);

                    if prev != current && break_job.places.first().and_then(|p| p.location).is_none() {
                        breaks.insert(activity.job.as_ref().unwrap().clone());
                    }
                }

                (current, breaks)
            });

        acc.extend(breaks_set.into_iter());

        acc
    });

    ctx.required.extend(breaks_set.into_iter());
}
