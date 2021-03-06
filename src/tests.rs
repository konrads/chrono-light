use super::prelude::*;
#[cfg(not(feature = "std"))]
use alloc::vec;

#[cfg(feature = "std")]
use std::vec;

use crate::constants::*;

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
    assert_eq!(Err(ValidationError::OutOfScope), c.validate_datetime(&DateTime { year: 1969, month: 1, day: 1, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Err(ValidationError::OutOfScope), c.validate_datetime(&DateTime { year: 4001, month: 1, day: 1, hour: 0, minute: 0, second: 0, ms: 0 }));

    // static validation
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month:  0, day:  1, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month:  1, day:  1, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month: 12, day:  1, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month: 13, day:  1, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month: 1,  day:  0, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month: 1,  day:  1, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month: 1,  day: 28, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month: 1,  day: 32, hour:  0, minute:  0, second:  0, ms:   0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month: 1,  day: 10, hour: 23, minute:  0, second:  0, ms:   0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month: 1,  day: 10, hour: 24, minute:  0, second:  0, ms:   0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month: 1,  day: 10, hour:  0, minute: 59, second:  0, ms:   0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month: 1,  day: 10, hour:  0, minute: 60, second:  0, ms:   0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month: 1,  day: 10, hour:  0, minute:  0, second: 59, ms:   0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month: 1,  day: 10, hour:  0, minute:  0, second: 60, ms:   0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month: 1,  day: 10, hour:  0, minute:  0, second:  0, ms: 999 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month: 1,  day: 10, hour:  0, minute:  0, second:  0, ms: 1000 }));

    // months, including leap
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month:  1,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month:  1,  day: 32, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month:  2,  day: 29, hour: 0, minute: 0, second: 0, ms: 0 })); // leap
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month:  2,  day: 30, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2001, month:  2,  day: 28, hour: 0, minute: 0, second: 0, ms: 0 })); // non leap
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2001, month:  2,  day: 29, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month:  3,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month:  3,  day: 32, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month:  4,  day: 30, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month:  4,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month:  5,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month:  5,  day: 32, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month:  6,  day: 30, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month:  6,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month:  7,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month:  7,  day: 32, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month:  8,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month:  8,  day: 32, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month:  9,  day: 30, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month:  9,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month: 10,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month: 10,  day: 32, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month: 11,  day: 30, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month: 11,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Ok(()),                        c.validate_datetime(&DateTime { year: 2000, month: 12,  day: 31, hour: 0, minute: 0, second: 0, ms: 0 }));
    assert_eq!(Err(ValidationError::Invalid), c.validate_datetime(&DateTime { year: 2000, month: 12,  day: 32, hour: 0, minute: 0, second: 0, ms: 0 }));
}

#[test]
fn test_no_schedules_for_now() {
    let c = Calendar::create();
    let start = DateTime { year: 2022, month: 1, day: 25, hour: 5, minute: 3, second: 30, ms: 0 };
    let start_plus_2s = DateTime { second: start.second + 2, ..start };
    let start_plus_3s = DateTime { second: start.second + 3, ..start };
    let now = start.clone();
    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Second, 3)],
        end: Some(start_plus_2s.clone())
    });
    assert_eq!(None, next_occurrence);

    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start.clone(),
        items: vec![(Frequency::Second, 3)],
        end: Some(start_plus_3s)
    });
    assert_eq!(Some(3000), next_occurrence);

    let start_plus_1s = DateTime { second: start.second + 1, ..start.clone() };
    let next_occurrence = c.next_occurrence_ms(&now, &Schedule {
        start: start_plus_1s.clone(),
        items: vec![(Frequency::Second, 3)],
        end: Some(start_plus_2s)
    });
    assert_eq!(Some(1000), next_occurrence);
}

#[test]
fn test_schedule_valid() {
    let c = Calendar::create();
    let t1 = DateTime { year: 2022, month: 1, day: 25, hour: 5, minute: 3, second: 30, ms: 0 };
    let t2 = DateTime { second: t1.second + 1, ..t1 };
    assert_eq!(Ok(()),                        c.validate_schedule(&Schedule { start: t1.clone(), items: vec![], end: None}));
    assert_eq!(Ok(()),                        c.validate_schedule(&Schedule { start: t1.clone(), items: vec![], end: Some(t2.clone())}));
    assert_eq!(Err(ValidationError::Invalid), c.validate_schedule(&Schedule { start: t2.clone(), items: vec![], end: Some(t1.clone())}));
    assert_eq!(Ok(()),                        c.validate_schedule(&Schedule { start: t1.clone(), items: vec![(Frequency::Hour, 1)], end: None}));
    assert_eq!(Err(ValidationError::Invalid), c.validate_schedule(&Schedule { start: t1.clone(), items: vec![(Frequency::Hour, 0)], end: None}));
}

#[test]
fn test_next_occurrence_ms_with_past_triggers() {
    let c = Calendar::create();
    let start = DateTime { year: 2000, month: 1, day: 1, hour: 2, minute: 0, second: 0, ms: 0 };
    let now = DateTime { hour: 6, ..start };
    let end = DateTime { hour: 23, ..start };
    let schedule = Schedule {
        start: start.clone(),
        items: vec![(Frequency::Hour, 3)],
        end: Some(end.clone())
    };

    let (triggers, next_occurrence) = c.next_occurrence_ms_with_past_triggers(None, &now.clone(), &schedule);
    assert_eq!(triggers, vec![
        c.to_unixtime(&DateTime { hour: 2, ..start }),
        c.to_unixtime(&DateTime { hour: 5, ..start }),
    ]);
    assert_eq!(next_occurrence, Some(2*60*60*1000));

    let last_run = DateTime { hour: 4, ..now };
    let now = DateTime { hour: 7, ..start };
    let (triggers, next_occurrence) = c.next_occurrence_ms_with_past_triggers(Some(&last_run), &now.clone(), &schedule);
    assert_eq!(triggers, vec![
        c.to_unixtime(&DateTime { hour: 5, ..start }),
    ]);
    assert_eq!(next_occurrence, Some(60*60*1000));

    let last_run = DateTime { hour: 16, ..now };
    let now = DateTime { hour: 24, ..start };
    let (triggers, next_occurrence) = c.next_occurrence_ms_with_past_triggers(Some(&last_run), &now.clone(), &schedule);
    assert_eq!(triggers, vec![
        c.to_unixtime(&DateTime { hour: 17, ..start }),
        c.to_unixtime(&DateTime { hour: 20, ..start }),
        c.to_unixtime(&DateTime { hour: 23, ..start }),
    ]);
    assert_eq!(next_occurrence, None);
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
    assert_eq!(Err(ValidationError::Invalid), c.to_unixtime_res(&dt));
    let dt = DateTime { year: 2020, month: 0, day: 1, hour: 0, minute: 0, second: 0, ms: 0 };
    assert_eq!(Err(ValidationError::Invalid), c.to_unixtime_res(&dt));
}

pub(crate) const NON_LEAP_YEAR_IN_MS: u64 = 365 * MS_IN_DAY;
pub(crate) const LEAP_YEAR_IN_MS: u64     = 366 * MS_IN_DAY;

pub(crate) const MONTH_MS_OFFSET_FOR_NON_LEAP_YEAR: &[u64] = &[31*MS_IN_DAY, 28*MS_IN_DAY, 31*MS_IN_DAY, 30*MS_IN_DAY, 31*MS_IN_DAY, 30*MS_IN_DAY, 31*MS_IN_DAY, 31*MS_IN_DAY, 30*MS_IN_DAY, 31*MS_IN_DAY, 30*MS_IN_DAY, 31*MS_IN_DAY];
pub(crate) const MONTH_MS_OFFSET_FOR_LEAP_YEAR: &[u64]     = &[31*MS_IN_DAY, 29*MS_IN_DAY, 31*MS_IN_DAY, 30*MS_IN_DAY, 31*MS_IN_DAY, 30*MS_IN_DAY, 31*MS_IN_DAY, 31*MS_IN_DAY, 30*MS_IN_DAY, 31*MS_IN_DAY, 30*MS_IN_DAY, 31*MS_IN_DAY];

/// Purpose of this test is to confirm the correctness of the hardcoded offset constants.
#[test]
fn test_gen_calendar_offsets() {
    let mut leap_year_month_offsets = vec![0_u64];
    let mut non_leap_year_month_offsets = vec![0_u64];
    let mut year_ms_offsets = vec![0_u64];

    (0..MONTH_MS_OFFSET_FOR_NON_LEAP_YEAR.len()).fold((0, 0), |acc, i| {
        let (so_far_leap, so_far_non_leap) = acc;
        let new_acc_leap = so_far_leap + MONTH_MS_OFFSET_FOR_LEAP_YEAR[i];
        let new_acc_non_leap = so_far_non_leap + MONTH_MS_OFFSET_FOR_NON_LEAP_YEAR[i];
        leap_year_month_offsets.push(new_acc_leap);
        non_leap_year_month_offsets.push(new_acc_non_leap);
        (new_acc_leap, new_acc_non_leap)
    });

    (1970_u16..4000_u16).fold(0, | acc, y| {
        let ms_in_y = if LEAP_YEARS.contains(&y) {
            LEAP_YEAR_IN_MS
        } else {
            NON_LEAP_YEAR_IN_MS
        };
        let new_acc = acc + ms_in_y;
        year_ms_offsets.push(new_acc);
        new_acc
    });

    assert_eq!(LEAP_YEAR_MONTH_OFFSETS, leap_year_month_offsets);
    assert_eq!(NON_LEAP_YEAR_MONTH_OFFSETS, non_leap_year_month_offsets);
    assert_eq!(YEAR_MS_OFFSETS, year_ms_offsets);
}
