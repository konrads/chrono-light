# Simple DateTime/Scheduler library

[![test](https://github.com/konrads/chrono-light/workflows/test/badge.svg)](https://github.com/konrads/chrono-light/actions/workflows/test.yml)

Provides `DateTime` and `schedule` capabilities with minimal dependency (only requires `Vec` from `std` or [sp_std](https://docs.rs/sp-std/latest/sp_std)).

Designed to work with `Substrate` framework, to extend its scheduling capabilities.

Functionality provided:
* conversion from unixtime (mills from epoch) to `DateTime`, and vice versa
* finding next occurrence for a schedule comprising:
  * start `DateTime`
  * repeat frequency (multiples of year/month/week/day/hour/minute/second/millis)
  * optional end `DateTime`

## Scope
This library works with `DateTime`s and `schedule`s within years of [1970, 4000].

Does not support timezones.

Overflow of months (>12), days (>28, >30, >31), hour (>23), minute/second (>59), millis (>999) is discouraged yet allowed, with excess added eg. 31 April ~= 1 May. Underflow of month/day (=0) causes panic. To avoid panic, validate hand crafted `DateTime` via `Calendar::validate()` or convert to unixtime via `Calendar::to_unixtime_opt()`.

Reasoning behind these restrictions is to keep the footprint reasonably compact and performance reasonably fast, and cater for real life scenarios.

### Typical usage
```rust
use chrono_light::prelude::*;

let c = Calendar::create();
let now_in_ms: u64 = 1650412800000;  // represents 20/04/2022 00:00:00:000
let schedule = Schedule {
    start: DateTime { year: 2020, month: 4, day: 30, hour: 0, minute: 0, second: 0, ms: 0 },
    items: vec![(Frequency::Year, 1)],
    end: Some(DateTime { year: 2025, month: 4, day: 30, hour: 0, minute: 0, second: 0, ms: 0 })
};
assert_eq!(Some(10*24*60*60*1000), c.next_occurrence_ms(&c.from_unixtime(now_in_ms), &schedule));  // triggers in 10 days
```