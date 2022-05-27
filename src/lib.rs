#[cfg(not(feature = "std"))]
#[macro_use]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

mod constants;
mod types;
mod calendar;
mod utils;

pub mod prelude {
    pub use super::types::*;
    pub use super::calendar::*;
}

#[cfg(test)]
mod tests;
#[cfg(feature = "std")]
#[cfg(test)]
mod tests_props;
