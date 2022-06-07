#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
mod calendar;
mod constants;
mod types;
mod utils;

pub mod prelude {
    pub use super::calendar::*;
    pub use super::types::*;
}

#[cfg(test)]
mod tests;
#[cfg(feature = "std")]
#[cfg(test)]
mod tests_props;
