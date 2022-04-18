use super::prelude::*;
#[cfg(feature = "no_std")]
use sp_std::vec;

#[cfg(feature = "std")]
use std::vec;

#[test]
fn test_roundtrip() {
    let ts_and_dt = [
        (1648515689162, DateTime { year: 2022, month:  3, day: 29, hour:  1, minute:  1, second: 29, ms: 162 }),
        (1646092675123, DateTime { year: 2022, month:  2, day: 28, hour: 23, minute: 57, second: 55, ms: 123 }),
        (1583020675456, DateTime { year: 2020, month:  2, day: 29, hour: 23, minute: 57, second: 55, ms: 456 }),
        (1731665410010, DateTime { year: 2024, month: 11, day: 15, hour: 10, minute: 10, second: 10, ms: 010 }),
        (1648515689162, DateTime { year: 2022, month:  3, day: 29, hour:  1, minute:  1, second: 29, ms: 162 }),
        (1650863010000, DateTime { year: 2022, month:  4, day: 25, hour:  5, minute:  3, second: 30, ms:   0 })
    ];

    // roundtrip checks
    let c = Calendar::create();
    for (ts, dt) in ts_and_dt {
        assert_eq!(dt, c.from_unixtime(ts));
        assert_eq!(ts, c.to_unixtime(&c.from_unixtime(ts)));
    }
}

#[test]
fn test_invalid_dates() {
    let c = Calendar::create();

    let dt_wrong = DateTime { year: 2022, month:  2, day: 29, hour: 23, minute: 57, second: 55, ms: 123 };  // 2022 not a leap year, no 29th of Feb
    let dt_right = DateTime { year: 2022, month:  3, day:  1, hour: 23, minute: 57, second: 55, ms: 123 };  // it's really 1st of March
    assert_eq!(dt_right, c.from_unixtime(c.to_unixtime(&dt_wrong)));
}

/// Code for days/hours/minutes/seconds/ms is the same, hence treating it as such
#[test]
fn test_next_occurrence_day_to_ms() {
    let c = Calendar::create();
    let start = DateTime { year: 2022, month: 3, day: 29, hour: 5, minute: 1, second: 29, ms: 162 };

    // now before schedule start
    let now = DateTime { year: 2022, month:  3, day: 29, hour: 1, minute: 1, second: 29, ms: 162 };
    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Minute, 2)],
        end: None
    });
    assert_eq!(Some(4*60*60*1000), next_occurrence);

    // now 2:00:001 after schedule start
    let now = DateTime { year: 2022, month: 3, day: 29, hour: 5, minute: 3, second: 29, ms: 163 };  // 2:00:001 after start, ie. 3mins-1ms to go
    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Minute, 5)],
        end: None
    });
    assert_eq!(Some(3*60*1000-1), next_occurrence);

    // now 2:00:001 after schedule start
    let now = DateTime { year: 2022, month: 3, day: 29, hour: 5, minute: 3, second: 29, ms: 162 };  // 1:59:999 after start, ie. 2:58:00:000 to go
    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Hour, 3)],
        end: None
    });
    assert_eq!(Some(2*60*60*1000+58*60*1000), next_occurrence);

    let now = DateTime { year: 2022, month: 3, day: 29, hour: 6, minute: 1, second: 29, ms: 162 };  // 1:59:999 after start, ie. 2:58:00:000 to go
    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Day, 2)],
        end: None
    });
    assert_eq!(Some((24+24-1)*60*60*1000), next_occurrence);

    let now = DateTime { year: 2022, month: 3, day: 29, hour: 6, minute: 1, second: 30, ms: 162 };  // 1:59:999 after start, ie. 2:58:00:000 to go
    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Second, 10)],
        end: None
    });
    assert_eq!(Some(9000), next_occurrence);

    let now = DateTime { year: 2022, month: 3, day: 29, hour: 6, minute: 1, second: 30, ms: 172 };  // 1:59:999 after start, ie. 2:58:00:000 to go
    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Ms, 100)],
        end: None
    });
    assert_eq!(Some(90), next_occurrence);
}

#[test]
fn test_with_schedule_end() {
    let c = Calendar::create();
    let start = DateTime { year: 2022, month: 3, day: 29, hour:  5, minute: 1, second: 29, ms: 162 };
    let end   = DateTime { year: 2025, month: 3, day: 29, hour: 10, minute: 1, second: 29, ms: 162 };

    // now after schedule end
    let now = DateTime { year: 2032, month: 3, day: 29, hour: 1, minute: 1, second: 29, ms: 162 };
    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Minute, 2)],
        end: Some(end.clone())
    });
    assert_eq!(None, next_occurrence);
}

#[test]
fn test_next_occurrence_months() {
    let c = Calendar::create();
    let start = DateTime { year: 2022, month: 1, day: 25, hour: 5, minute: 3, second: 30, ms: 0 };
    let now = DateTime { year: 2022, month: 1, day: 30, hour: 5, minute: 3, second: 30, ms: 0 };

    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Month, 1)],
        end: None
    });
    assert_eq!(Some((1+25)*24*60*60*1000), next_occurrence);

    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Month, 2)],
        end: None
    });
    assert_eq!(Some((1+28+25)*24*60*60*1000), next_occurrence);

    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Month, 3)],
        end: None
    });
    assert_eq!(Some((1+28+31+25)*24*60*60*1000), next_occurrence);

    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Month, 36)],
        end: None
    });
    assert_eq!(Some((365+366+365-5)*24*60*60*1000), next_occurrence);
}

#[test]
fn test_next_occurrence_years() {
    let c = Calendar::create();
    let start = DateTime { year: 2022, month: 1, day: 25, hour: 5, minute: 3, second: 30, ms: 0 };
    let now = DateTime { year: 2022, month: 1, day: 30, hour: 5, minute: 3, second: 30, ms: 0 };

    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Year, 1)],
        end: None
    });
    assert_eq!(Some((365-5)*24*60*60*1000), next_occurrence);

    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Year, 2)],
        end: None
    });
    assert_eq!(Some((365+365-5)*24*60*60*1000), next_occurrence);

    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Year, 3)],
        end: None
    });
    assert_eq!(Some((365+365+366-5)*24*60*60*1000), next_occurrence);
}

#[test]
fn test_validation() {
    let c = Calendar::create();
    assert_eq!(ValidationResult::OutOfScope, c.validate(&DateTime { year: 1969, month: 1, day: 1, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::OutOfScope, c.validate(&DateTime { year: 4001, month: 1, day: 1, hour: 0, minute: 0, second: 0, ms: 0 }));

    // static validation
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month:  0, day:  1, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month:  1, day:  1, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month: 12, day:  1, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month: 13, day:  1, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month: 1,  day:  0, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month: 1,  day:  1, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month: 1,  day: 28, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month: 1,  day: 32, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month: 1,  day: 10, hour: 23, minute:  0, second:  0, ms:   0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month: 1,  day: 10, hour: 24, minute:  0, second:  0, ms:   0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month: 1,  day: 10, hour:  0, minute: 59, second:  0, ms:   0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month: 1,  day: 10, hour:  0, minute: 60, second:  0, ms:   0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month: 1,  day: 10, hour:  0, minute:  0, second: 59, ms:   0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month: 1,  day: 10, hour:  0, minute:  0, second: 60, ms:   0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month: 1,  day: 10, hour:  0, minute:  0, second:  0, ms: 999 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month: 1,  day: 10, hour:  0, minute:  0, second:  0, ms: 1000 }));

    // months, including leap
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month:  1,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month:  1,  day: 32, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month:  2,  day: 29, hour: 0, minute: 0, second: 0, ms: 0 })); // leap
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month:  2,  day: 30, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2001, month:  2,  day: 28, hour: 0, minute: 0, second: 0, ms: 0 })); // non leap
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2001, month:  2,  day: 29, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month:  3,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month:  3,  day: 32, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month:  4,  day: 30, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month:  4,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month:  5,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month:  5,  day: 32, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month:  6,  day: 30, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month:  6,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month:  7,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month:  7,  day: 32, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month:  8,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month:  8,  day: 32, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month:  9,  day: 30, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month:  9,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month: 10,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month: 10,  day: 32, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month: 11,  day: 30, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month: 11,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Valid,   c.validate(&DateTime { year: 2000, month: 12,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(ValidationResult::Invalid, c.validate(&DateTime { year: 2000, month: 12,  day: 32, hour: 0, minute: 0, second: 0, ms: 0 }));
}

#[test]
fn test_earliest_schedule_selected() {
    let c = Calendar::create();
    let start = DateTime { year: 2022, month: 1, day: 25, hour: 5, minute: 3, second: 30, ms: 0 };
    let now = DateTime { year: 2022, month: 1, day: 30, hour: 5, minute: 3, second: 31, ms: 0 };

    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![
            (Frequency::Year, 3),
            (Frequency::Month, 3),
            (Frequency::Day, 3),
            (Frequency::Second, 3),  // earliest schedule (earlier and 5000ms)
            (Frequency::Hour, 3),
            (Frequency::Minute, 3),
            (Frequency::Ms, 5000),
        ],
        end: None
    });
    assert_eq!(Some(2000), next_occurrence);
}

#[test]
fn test_invalid_datetimes() {
    let c = Calendar::create();
    let dt = DateTime { year: 2020, month: 1, day: 0, hour: 0, minute: 0, second: 0, ms: 0 };
    assert_eq!(None, c.to_unixtime_opt(&dt));
    let dt = DateTime { year: 2020, month: 0, day: 1, hour: 0, minute: 0, second: 0, ms: 0 };
    assert_eq!(None, c.to_unixtime_opt(&dt));
}