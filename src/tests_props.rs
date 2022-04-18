use crate::constants::{EPOCH_YEAR, MS_IN_DAY};
use super::prelude::*;

use chrono::{NaiveDateTime, TimeZone, Utc, Datelike, Timelike};
use quickcheck::QuickCheck;
use std::panic;

const PROPS_TESTS: u64 = 500;  // 5 x the norm

/// For default quickcheck behaviour, with not panic hook initialization or test run settings, could also just do:
/// ```no_run
/// use quickcheck::quickcheck;
/// 
/// #[quickcheck]
/// fn validate_light_vs_chrono(...) {...}
/// ```
fn validate_light_vs_chrono(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8, ms: u16) -> bool {
    let c = Calendar::create();  // FIXME: should initialize once only!

    // ensure values get some (but not too much) margin for error
    let year = year % (4000 - EPOCH_YEAR as u16) + EPOCH_YEAR as u16;
    let month = month % 15;
    let day = day % 35;
    let minute = minute % 65;
    let second = second % 65;
    let ms = ms % 1100;
    let dt_light = DateTime { year, month, day, hour, minute, second, ms};
    let dt_chrono_res = panic::catch_unwind(|| Utc.ymd(year as i32, month as u32, day as u32).and_hms_milli(hour as u32, minute as u32, second as u32, ms as u32));
    let dt_chrono_light_res = panic::catch_unwind(|| c.to_unixtime(&dt_light));
    let validation_result = c.validate(&dt_light);

    if validation_result == ValidationResult::Invalid && dt_chrono_res.is_err() {
        true
    } else if validation_result == ValidationResult::Valid && dt_chrono_res.is_ok() && dt_chrono_light_res.is_ok() { // Note: dt_chrono_light_res may be ok when overflowing months/days, but never underflowing
        let dt_light_ms = c.to_unixtime(&dt_light);
        let dt_light2 = c.from_unixtime(dt_light_ms);
        let dt_chrono_ms = dt_chrono_res.unwrap().timestamp_millis() as u64;
        if dt_light_ms == dt_chrono_ms && dt_light == dt_light2 {
            true
        } else {
            println!("Failed on dt: {:?}, light ms: {:?}, chrono ms: {:?}", dt_light, dt_light_ms, dt_chrono_ms);
            false
        }
    } else {
        println!("Failed on dt: {:?}, validation_result: {:?}, dt_chrono_res: {:?}, dt_chrono_light_res: {:?}", dt_light, validation_result, dt_chrono_res, dt_chrono_light_res);
        false
    }
}

fn validate_scheduler_after_start(start_ms: u64, delta_ms: u64, freq: u8, freq_multiplier: u8) -> bool {
    let c = Calendar::create();  // FIXME: should initialize once only!

    fn get_exact_months(start_ms: u64, now_ms: u64, next_occurrence: u64) -> Option<u32> {
        let start = NaiveDateTime::from_timestamp(start_ms as i64 / 1000, ((start_ms % 1000) * 1_000_000) as u32);
        let end_ms = now_ms + next_occurrence;
        let end = NaiveDateTime::from_timestamp(end_ms as i64 / 1000, ((end_ms % 1000) * 1_000_000) as u32);
        if start.day() == end.day() && start.hour() == end.hour() && start.minute() == end.minute() && start.second() == end.second()
           && start.timestamp_subsec_millis() == end.timestamp_subsec_millis() {
            Some(((end.year() - start.year()) * 12 + end.month() as i32 - start.month() as i32) as u32)
        } else {
            None
        }
    }

    let freq = match freq % 8 {
        0 => Frequency::Year,
        1 => Frequency::Month,
        2 => Frequency::Week,
        3 => Frequency::Day,
        4 => Frequency::Hour,
        5 => Frequency::Minute,
        6 => Frequency::Second,
        _ => Frequency::Ms,
    };
    let freq_multiplier = freq_multiplier + 1;  // start at 1
    let mut start_ms = start_ms % 100*365*24*60*60*1000;
    let mut start = c.from_unixtime(start_ms);
    if freq == Frequency::Month {
        start.day = start.day.min(28);
        start_ms = c.to_unixtime(&start);
    }
    let delta_ms = delta_ms % 10*365*24*60*60*1000 + 1;
    let now_ms = start_ms + delta_ms;

    let now = c.from_unixtime(now_ms);
    let next_occurrence = c.next_occurrence_ms(&now, &Schedule { start: start.clone(), items: vec![(freq, freq_multiplier as u32)], end: None }).unwrap();

    match freq {
        Frequency::Year => {
            if let Some(months) = get_exact_months(start_ms, now_ms, next_occurrence) {
                months % (12 * freq_multiplier as u32) == 0 && next_occurrence <= freq_multiplier as u64 * MS_IN_DAY * 366
            } else {
                false
            }
        },
        Frequency::Month => {
            if let Some(months) = get_exact_months(start_ms, now_ms, next_occurrence) {
                months % freq_multiplier as u32 == 0 && next_occurrence <= freq_multiplier as u64 * MS_IN_DAY * 31
            } else {
                false
            }
        },
        _ => {
            let end_ms = now_ms + next_occurrence;
            let freq_ms = freq as u64 * freq_multiplier as u64;
            (end_ms - start_ms) % freq_ms == 0 && next_occurrence <= freq_ms
        }
    }
}


#[test]
fn test_validate_vs_chrono() {
    panic::set_hook(Box::new(|_info| { /* reduces the panic::catch_unwind() log noise */ }));
    QuickCheck::new().tests(PROPS_TESTS).max_tests(PROPS_TESTS * 100).quickcheck(validate_light_vs_chrono as fn(u16, u8, u8, u8, u8, u8, u16) -> bool)
}

#[test]
fn test_validate_scheduler_vs_chrono() {
    QuickCheck::new().tests(PROPS_TESTS).max_tests(PROPS_TESTS * 100).quickcheck(validate_scheduler_after_start as fn(u64, u64, u8, u8) -> bool)
}

#[test]
fn test_zeros() {
    fn validate_zero_next_occurrence(ts: u64, freq: u8) -> bool {
        let c = Calendar::create();  // FIXME: should initialize once only!
        let ts = ts % 60913560719000;
        let now = c.from_unixtime(ts);
        let freq = match freq % 8 {
            0 => Frequency::Year,
            1 => Frequency::Month,
            2 => Frequency::Week,
            3 => Frequency::Day,
            4 => Frequency::Hour,
            5 => Frequency::Minute,
            6 => Frequency::Second,
            _ => Frequency::Ms,
        };

        let freq_multiplier = freq as u32;

        let res = c.next_occurrence_ms(&now.clone(), &Schedule {
            start: now,
            items: vec![(freq, freq_multiplier)],
            end: None,
        });

        res.map_or(false, |x| x == 0)
    }
    QuickCheck::new().tests(PROPS_TESTS / 10).max_tests(PROPS_TESTS * 10).quickcheck(validate_zero_next_occurrence as fn(u64, u8) -> bool)
}