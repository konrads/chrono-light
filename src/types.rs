#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "scale")]
use codec::{Decode, Encode};
#[cfg(feature = "scale")]
use scale_info::TypeInfo;

use super::constants::*;

/// DateTime representation from year to ms. Valid values are:
/// - year:   [1970, 4000]
/// - month:  [1, 12]
/// - day:    [1, 31] (depending on month, leap year)
/// - hour:   [0: 23]
/// - minute: [0, 59]
/// - second: [0, 59]
/// - ms:     [0, 999]
/// 
/// Note: other values will be accepted, but will be classified invalid by the calendar, and if used,
/// appropriate values will be added on top, eg. 32/01 -> 01/02.
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo))]
pub struct DateTime {
    // date
    pub year:   u16,
    pub month:  u8,
    pub day:    u8,

    // time
    pub hour:   u8,
    pub minute: u8,
    pub second: u8,
    pub ms:     u16,
}

impl DateTime {
    /// Calculates ms for the day
    pub fn to_day_unixtime(&self) -> u64 {
        self.day.checked_sub(1).expect("failed to calc day - 1") as u64 * MS_IN_DAY
            + self.hour as u64 * MS_IN_HOUR
            + self.minute as u64 * MS_IN_MIN
            + self.second as u64 * MS_IN_SEC
            + self.ms as u64
    }
}

/// Schedule, represented by a `start` `DateTime`, optional `end` `DateTime`, and multiple pairs of (`Frequency`, `multiplier`).
/// Next occurrence of trigger time is calculated by taking the earliest occurrence of `Frequency` * `multiplier`, from `start`, but before `end`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo))]
pub struct Schedule {
    pub start: DateTime,
    pub items: Vec<(Frequency, u32)>,  // frequency with multiplier
    pub end: Option<DateTime>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "scale", derive(Encode, Decode, TypeInfo))]
#[repr(u32)]
pub enum Frequency {
    Year   = 666_u32,
    Month  = 999_u32,
    Week   = 7 * MS_IN_DAY as u32,
    Day    = MS_IN_DAY as u32,
    Hour   = MS_IN_HOUR as u32,
    Minute = MS_IN_MIN as u32,
    Second = MS_IN_SEC as u32,
    Ms     = 1_u32,
}

#[derive(PartialEq, Eq, Debug)]
pub enum ValidationError {
    /// `DateTime` not covered by this library, eg. 01/01/1000 00:00:00:000, 01/01/5000 00:00:00:000
    OutOfScope,
    /// Invalid `DateTime`, eg. 32/13/2000 66:66:66:6666, 29/02/2021 10:10:10:000 (non leap year)
    Invalid
}
