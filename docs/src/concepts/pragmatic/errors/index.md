# Error Index

This page lists errors produced by the solver.


## E0xxx Error

Errors from E0xxx range are generic.


### E0000

`cannot deserialize problem` is returned when problem definition cannot be deserialized from the input stream.


### E0001

`cannot deserialize matrix` is returned when routing matrix definition cannot be deserialized from the input stream.


### E0002

`cannot create transport costs` is returned when problem cannot be matched within routing matrix data passed.

### E0003

`cannot find any solution` is returned when no solution is found. In this case, please submit a bug and share original
problem and routing matrix.

### E0004

`cannot read config` is returned when algorithm configuration cannot be created. To fix it, make sure that config has
a valid json schema and valid parameters.


## E1xxx: Validation errors

Errors from E1xxx range are used by validation engine which checks logical correctness of the rich VRP definition.


### E11xx: Jobs

These errors are related to `plan.jobs` property definition.


#### E1100

`duplicated job ids` error is returned when `plan.jobs` has jobs with the same ids:

```json
{
  "plan": {
    "jobs": [
      {
        "id": "job1",
        /** omitted **/
      },
      {
        /** Error: this id is already used by another job **/
        "id": "job1",
        /** omitted **/
      }
      /** omitted **/
    ]
  }
}
```

Duplicated job ids are not allowed, so you need to remove all duplicates in order to fix the error.


#### E1101

`invalid job task demand` error is returned when job has invalid demand: `pickup`, `delivery`, `replacement` job types should
have demand specified on each job task, `service` type should have no demand specified:

```json
{
  "id": "job1",
  "deliveries": [
    {
      /** omitted **/
      /** Error: delivery task should have demand set**/
      "demand": null
    }
 ],
 "services": [
   {
     /** omitted **/
     /** Error: service task should have no demand specified**/
     "demand": [1]
   }
 ]
}
```

To fix the error, make sure that each job task has proper demand.


#### E1102

`invalid pickup and delivery demand` error code is returned when job has both pickups and deliveries, but the sum of
pickups demand does not match to the sum of deliveries demand:

```json
{
  "id": "job",
  "pickups": [
    {
      "places": [/** omitted **/],
      "demand": [1],
    },
    {
      "places": [/** omitted **/],
      "demand": [1]
    }
  ],
  "deliveries": [
    {
      "places": [/** omitted **/],
      /** Error: should be 2 as the sum of pickups is 2 **/
      "demand": [1]
    }
  ]
}
```


#### E1103

`invalid time windows in jobs` error is returned when there is a job which has invalid time windows, e.g.:

```json
{
  /** Error: end time is one hour earlier than start time**/
  "times": [
    [
      "2020-07-04T12:00:00Z",
      "2020-07-04T11:00:00Z"
    ]
  ]
}
```

Each time window must satisfy the following criteria:

* array of two strings each of these specifies date in RFC3339 format. The first is considered as start,
the second - as end
* start date is earlier than end date
* if multiple time windows are specified, they must not intersect, e.g.:

```json
{
  /** Error: second time window intersects with first one: [13:00, 14:00] **/
  "times": [
    [
      "2020-07-04T10:00:00Z",
      "2020-07-04T14:00:00Z"
    ],
    [
      "2020-07-04T13:00:00Z",
      "2020-07-04T17:00:00Z"
    ]
  ]
}
```


#### E1104

`reserved job id is used` error is returned when there is a job which has reserved job id:

```json
{
  /** Error: 'departure' is reserved job id **/
  "id": "departure"
}
```

To avoid confusion, the following ids are reserved: `departure`, `arrival`, `break`, and `reload`. These ids are not
allowed to be used within `job.id` property.


#### E1105

`empty job` error is returned when there is a job which has no or empty job tasks:

```json
{
  /** Error: at least one job task has to be defined **/
  "id": "job1",
  "pickups": null,
  "deliveries": []
}
```

To fix the error, remove job from the plan or add at least one job task to it.


#### E1106

`job has negative duration` error is returned when there is a job place with negative duration:

```json
{
  "id": "job",
  "pickups": [
    {
      "places": [{
        /** Error: negative duration does not make sense **/
        "duration": -10,
        "location": {/* omitted */}
       }]
       /* omitted */
    }
  ]
}
```

To fix the error, make sure that all durations are non negative.


#### E1107

`job has negative demand` error is returned when there is a job with negative demand in any of dimensions:

```json
{
  "id": "job",
  "pickups": [
    {
      "places": [/* omitted */],
      /** Error: negative demand is not allowed **/
      "demand": [10, -1]
    }
  ]
}
```

To fix the error, make sure that all demand values are non negative.


### E12xx: Relations

These errors are related to `plan.relations` property definition.


#### E1200

`relation has job id which does not present in the plan` error is returned when `plan.relations` has relations with
job ids, not present in `plan.jobs`.


#### E1201

`relation has vehicle id which does not present in the fleet` error is returned when `plan.relations` has relations with
vehicle ids, not present in `plan.fleet`.


#### E1202

`relation has empty job id list` error is returned when `plan.relations` has relations with empty `jobs` list.


#### E1203

`strict or sequence relation has job with multiple places or time windows` error is returned when `plan.relations` has
strict or sequence relation which refers one or many jobs with multiple places and/or time windows.

This is currently not allowed due to matching problem.


#### E1204

`job is assigned to different vehicles in relations` error is returned when `plan.relations` has a job assigned to several
relations with different vehicle ids:

```json
{
  "plan": {
    "relations": [
      {
        "vehicleId": "vehicle_1",
        "jobs": ["job1"],
        /** omitted **/
      },
      {
        /** Error: this job id is already assigned to another vehicle **/
        "vehicleId": "vehicle_2",
        "jobs": ["job1"],
        /** omitted **/
      }
    ]
  }
}
```

To fix this, remove job id from one of relations.


### E13xx: Vehicles

These errors are related to `fleet.vehicles` property definition.


#### E1300

`duplicated vehicle type ids` error is returned when `fleet.vehicles` has vehicle types with the same `typeId`:

```json
{
  "fleet": {
    "vehicles": [
      {
        "typeId": "vehicle_1",
        /** omitted **/
      },
      {
        /** Error: this id is already used by another vehicle type **/
        "typeId": "vehicle_1",
        /** omitted **/
      }
      /** omitted **/
    ]
  }
}
```


#### E1301

`duplicated vehicle ids` error is returned when `fleet.vehicles` has vehicle types with the same `vehicleIds`:

```json
{
  "fleet": {
    "vehicles": [
      {
        "typeId": "vehicle_1",
        "vehicleIds": [
          "vehicle_1_a",
          "vehicle_1_b",
          /** Error: vehicle_1_b is used second time **/
          "vehicle_1_b"
        ],
        /** omitted **/
      },
      {
        "typeId": "vehicle_2",
        "vehicleIds": [
          /** Error: vehicle_1_a is used second time **/
          "vehicle_1_a",
          "vehicle_2_b"
        ],
        /** omitted **/
      }
      /** omitted **/
    ]
  }
}
```

Please note that vehicle id should be unique across all vehicle types.


#### E1302

`invalid start or end times in vehicle shift` error is returned when vehicle has start/end shift times violating one of
time windows rules defined for jobs in E1103.


#### E1303

`invalid break time windows in vehicle shift` error is returned when vehicle has invalid time window of a break. List of
break should follow time window rules defined for jobs in E1103. Additionally, break time should be inside vehicle shift
it is specified:

```json
{
  "start": {
    "time": "2019-07-04T08:00:00Z",
    /** omitted **/
  },
  "end": {
    "time": "2019-07-04T15:00:00Z",
    /** omitted **/
  },
  "breaks": [
    {
      /** Error: break is outside of vehicle shift times **/
      "times": [
        [
          "2019-07-04T17:00:00Z",
          "2019-07-04T18:00:00Z"
        ]
      ],
      "duration": 3600.0
    }
  ]
}
```


#### E1304

`invalid reload time windows in vehicle shift` error is returned when vehicle has invalid time window of a reload. Reload
list should follow time window rules defined for jobs in E1003 except multiple reloads can have time window intersections.
Additionally, reload time should be inside vehicle shift it is specified:

```json
{
  "start": {
    "time": "2019-07-04T08:00:00Z",
    /** omitted **/
  },
  "end": {
    "time": "2019-07-04T15:00:00Z",
    /** omitted **/
  },
  "reloads": [
    {
      /** Error: reload is outside of vehicle shift times **/
      "times": [
        [
          "2019-07-04T17:00:00Z",
          "2019-07-04T18:00:00Z"
        ]
      ],
      "location": { /** omitted **/ },
      "duration": 3600.0
    }
  ]
}
```

#### E1305

`invalid allowed area definition in vehicle limits` error is returned when `allowedArea` property in `fleet.vehicles`
violates one of the following rules:

* no empty arrays
* each area has more than 2 coordinates

```json
{
  "limits": {
    "allowedAreas": [
      /** Error: at least three locations has to be defined **/
      [
        { "lat": 52.12, "lng":  13.14 },
        { "lat": 52.13, "lng":  13.15 }
      ]
    ]
  }
}
```


### E15xx: Profiles

These errors are related to `fleet.profiles` property definition.


#### E1500

`duplicate profile names` error is returned when `fleet.profiles` has more than one profile with the same name:

```json
{
  "profiles": [
    {
      "name": "vehicle_profile",
      "type": "car"
    },
    {
      "name": "vehicle_profile",
      "type": "truck"
    }
  ]
}
```

To fix the issue, remove all duplicates.


#### E1501

`empty profile collection` error is returned when `fleet.profiles` is empty:

```json
{
  "profiles": []
}
```


### E16xx: Objectives

These errors are related to `objectives` property definition.


#### E1600

`an empty objective specified` error is returned when objective property is present in the problem, but no single
objective is set, e.g.:

```json
{
  "objectives": {
    "primary":[]
  }
}
```

`objectives` property is optional, just remove it to fix the problem and use default objectives.


#### E1610

`duplicate objective specified` error is returned when objective of specific type specified more than once:

```json
{
  "objectives": {
    "primary": [
      {
        "type": "minimize-unassigned"
      },
      {
        "type": "minimize-unassigned"
      }
    ],
    "secondary": [
      {
        "type": "minimize-cost"
      }
    ]
  }
}
```

To fix this issue, just remove one, e.g. `minimize-unassigned`.


#### E1611

`missing cost objective` error is returned when no cost objective specified (at the moment, only `minimize-cost` supported):

```json
{
  "objectives": {
    "primary": [
      {
        "type": "minimize-unassigned"
      }
    ]
  }
}
```

This objective is used to calculate final costs, so it is required to be specified.
