#[cfg(not(feature = "std"))]
use super::Vec;
use super::{
    constants::*,
    types::*,
    utils::*,
};

/// Representation of Calendar without timezones, has awareness of leap years, days in a given month for leap and non-leap years.
/// 
/// Typical usage
/// ```rust
/// use chrono_light::prelude::*;
/// let c = Calendar::create();
/// let now_in_ms: u64 = 1650412800000;  // represents 20/04/2022 00:00:00:000
/// let schedule = Schedule {
///     start: DateTime { year: 2020, month: 4, day: 30, hour: 0, minute: 0, second: 0, ms: 0 },
///     items: vec![(Frequency::Year, 1)],
///     end: Some(DateTime { year: 2025, month: 4, day: 30, hour: 0, minute: 0, second: 0, ms: 0 })
/// };
/// assert_eq!(Some(10*24*60*60*1000), c.next_occurrence_ms(&c.from_unixtime(now_in_ms), &schedule));  // triggers in 10 days
/// ```
/// 
/// Beware `c.to_unixtime()` may panic, use `c.validate()` and/or `c.to_unixtime_opt()` to guarantee safety.
pub struct Calendar {
    // values required for the lookup of the years/months, considering leap Februaries
    // - year_offset_ms, taking into account leap/non leap years. store in array with implied index starting at 1970
    // - month_offset_ms, for every year, taking into account leap februaries
    year_ms_offsets:             Vec<u64>,
    leap_year_month_offsets:     Vec<u64>,
    non_leap_year_month_offsets: Vec<u64>,
}

impl Calendar {
    /// Constructor for the calendar.
    pub fn create() -> Self {
        let mut year_ms_offsets = vec![0_u64];
        let mut leap_year_month_offsets = vec![0_u64];
        let mut non_leap_year_month_offsets = vec![0_u64];

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

        (0..MONTH_MS_OFFSET_FOR_NON_LEAP_YEAR.len()).fold((0, 0), |acc, i| {
            let (so_far_leap, so_far_non_leap) = acc;
            let new_acc_leap = so_far_leap + MONTH_MS_OFFSET_FOR_LEAP_YEAR[i];
            let new_acc_non_leap = so_far_non_leap + MONTH_MS_OFFSET_FOR_NON_LEAP_YEAR[i];
            leap_year_month_offsets.push(new_acc_leap);
            non_leap_year_month_offsets.push(new_acc_non_leap);
            (new_acc_leap, new_acc_non_leap)
        });

        Self {
            year_ms_offsets,
            leap_year_month_offsets,
            non_leap_year_month_offsets,
        }
    }

    /// Converts a `&DateTime` to ms from epoch. Note: may panic if invalid `DateTime` specified.
    /// ```rust
    /// # use chrono_light::prelude::*;
    /// let c = Calendar::create();
    /// assert_eq!(c.to_unixtime(&DateTime {year: 2010, month: 10, day: 10, hour: 10, minute: 10, second: 10, ms: 10}), 1286705410010);
    /// ```
    pub fn to_unixtime(&self, dt: &DateTime) -> u64 {
        let year = dt.year as usize - EPOCH_YEAR;
        let year_offset = self.year_ms_offsets[year];
        let month_offset = if LEAP_YEARS.contains(&(dt.year as u16)) {
            self.leap_year_month_offsets[dt.month.checked_sub(1).expect("failed to calc month - 1") as usize]
        } else {
            self.non_leap_year_month_offsets[dt.month.checked_sub(1).expect("failed to calc month - 1") as usize]
        };
        let day_offset = dt.day.checked_sub(1).expect("failed to calc day - 1") as u64 * MS_IN_DAY;
        let hour_offset = dt.hour as u64 * MS_IN_HOUR;
        let minute_offset = dt.minute as u64 * MS_IN_MIN;
        let second_offset = dt.second as u64 * MS_IN_SEC;
        let ms_offset = dt.ms as u64;

        year_offset + month_offset + day_offset + hour_offset + minute_offset + second_offset + ms_offset
    }

    /// Converts a `&DateTime` to ms from epoch, returning `Some()` if supplied `DateTime` was valid, `None` otherwise.
    /// ```rust
    /// # use chrono_light::prelude::*;
    /// let c = Calendar::create();
    /// assert_eq!(c.to_unixtime_opt(&DateTime {year: 2010, month: 10, day: 10, hour: 10, minute: 10, second: 10, ms: 10}), Some(1286705410010));
    /// assert_eq!(c.to_unixtime_opt(&DateTime {year: 2010, month:  0, day: 10, hour: 10, minute: 10, second: 10, ms: 10}), None);
    /// assert_eq!(c.to_unixtime_opt(&DateTime {year: 2010, month: 10, day:  0, hour: 10, minute: 10, second: 10, ms: 10}), None);
    /// ```
    pub fn to_unixtime_opt(&self, dt: &DateTime) -> Option<u64> {
        match self.validate(dt) {
            ValidationResult::Valid => Some(self.to_unixtime(dt)),
            _ => None
        }
    }

    /// Converts ms from epoch to `DateTime`.
    pub fn from_unixtime(&self, ts: u64) -> DateTime {
        // find year
        let mut year = CURRENT_YEAR - EPOCH_YEAR;
        if ts > self.year_ms_offsets[year] {
            while ts > self.year_ms_offsets[year+1] {
                year += 1;
            }
        } else {
            year -= 1;
            while ts < self.year_ms_offsets[year] {
                year -= 1;
            }
        }
        let year_offset = ts - self.year_ms_offsets[year];
        let month_offsets = if LEAP_YEARS.contains(&(year as u16 + EPOCH_YEAR as u16)) {
            &self.leap_year_month_offsets
        } else {
            &self.non_leap_year_month_offsets
        };
        
        let mut month = 1_usize;
        if year_offset > 0 {
            while year_offset > month_offsets[month-1] {
                month += 1;
            }
            month -= 1;
        }

        let day_offset = year_offset - month_offsets[month-1];
        let day = day_offset / MS_IN_DAY + 1;
        let hour = (day_offset % MS_IN_DAY) / MS_IN_HOUR;
        let minute = (day_offset % MS_IN_HOUR) / MS_IN_MIN;
        let second = (day_offset % MS_IN_MIN) / MS_IN_SEC;
        let ms = day_offset % MS_IN_SEC;

        DateTime {
            year: (year + EPOCH_YEAR) as u16,
            month: month as u8,
            day: day as u8,
            hour: hour as u8,
            minute: minute as u8,
            second: second as u8,
            ms: ms as u16
        }
    }

    /// Given a `now` `DateTime` and `Schedule`, finds ms delta when the next occurrence should trigger.
    /// If cut of by `Schedule.end`, returns a `None`.
    pub fn next_occurrence_ms(&self, now: &DateTime, schedule: &Schedule) -> Option<u64> /* delta_in_ms */ {
        let now_in_ms = self.to_unixtime(now);
        let start_in_ms = self.to_unixtime(&schedule.start);
        let is_expired = || schedule.end.as_ref().map_or(false, |end_dt| now_in_ms > self.to_unixtime(end_dt));

        if now_in_ms < start_in_ms {
            Some(start_in_ms - now_in_ms)
        } else if is_expired() {
            None
        } else {
            schedule.items.iter().map(|(freq, multiplier)| {
                match freq {
                    Frequency::Year => {
                        let m_delta = now.month as i64 - schedule.start.month as i64 + i64::from(now.to_day_unixtime() > schedule.start.to_day_unixtime());
                        let y_delta = now.year as i64 - schedule.start.year as i64 + i64::from(m_delta > 0);
                        let total_y_from_start = ceil_div(y_delta as u32, *multiplier) * multiplier;
                        let next_occurrence = DateTime {
                            year: schedule.start.year + total_y_from_start as u16,
                            month: schedule.start.month,
                            day: schedule.start.day,
                            hour: schedule.start.hour,
                            minute: schedule.start.minute,
                            second: schedule.start.second,
                            ms: schedule.start.ms,
                        };
                        self.to_unixtime(&next_occurrence) - self.to_unixtime(now)
                    }
                    Frequency::Month => {
                        let m_delta = now.month as i64 - schedule.start.month as i64 + i64::from(now.to_day_unixtime() > schedule.start.to_day_unixtime());
                        let total_m_delta = (now.year as i64 - schedule.start.year as i64) * 12 + m_delta;
                        let total_m_from_start = ceil_div(total_m_delta as u32, *multiplier) * multiplier;
                        let next_occurrence = DateTime {
                            year: schedule.start.year + ((schedule.start.month as u32 + total_m_from_start - 1)/12) as u16,
                            month: ((schedule.start.month as u32 + total_m_from_start - 1) % 12) as u8 + 1,
                            day: schedule.start.day,
                            hour: schedule.start.hour,
                            minute: schedule.start.minute,
                            second: schedule.start.second,
                            ms: schedule.start.ms,
                        };
                        self.to_unixtime(&next_occurrence) - self.to_unixtime(now)
                    }
                    Frequency::Week | Frequency::Day | Frequency::Hour | Frequency::Minute | Frequency::Second | Frequency::Ms => {
                        let freq_in_ms = *freq as u64 * *multiplier as u64;
                        let ms_in_this_period = (now_in_ms - start_in_ms) % freq_in_ms;
                        if ms_in_this_period == 0 {
                            0
                        } else {
                            freq_in_ms - ms_in_this_period
                        }
                    },
                }
            }).min()
        }
    }

    /// Finds ms delta between 2 `DateTime`s.
    pub fn ms_between(&self, from: &DateTime, to: &DateTime) -> i64 {
        (self.to_unixtime(to) as i64).checked_sub(self.to_unixtime(from) as i64).expect("failed to calc ms_between")
    }

    /// Validates `DateTime` for correctness of fields, checking in respect to leap years.
    pub fn validate(&self, dt: &DateTime) -> ValidationResult {
        // scope check
        (EPOCH_YEAR..=EPOCH_YEAR+self.year_ms_offsets.len()-1).contains(&(dt.year as usize));
        if !(EPOCH_YEAR..=EPOCH_YEAR+self.year_ms_offsets.len()-1).contains(&(dt.year as usize)) {
            return ValidationResult::OutOfScope;
        }

        // static valid check
        if !(1..=12).contains(&dt.month) || !(1..=31).contains(&dt.day) || dt.hour >= 24 || dt.minute >= 60 || dt.second >= 60 || dt.ms >= 1000 {
            return ValidationResult::Invalid;
        }

        // leap year check
        let is_leap_year = LEAP_YEARS.contains(&(dt.year as u16));
        if (is_leap_year && dt.day > MONTH_FOR_LEAP_YEAR[dt.month.checked_sub(1).expect("failed to calc month - 1") as usize]) ||
            (!is_leap_year && dt.day > MONTH_FOR_NON_LEAP_YEAR[dt.month.checked_sub(1).expect("failed to calc month - 1") as usize]) {
            return ValidationResult::Invalid;
        }
        ValidationResult::Valid
    }
}
