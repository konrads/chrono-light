/// Division with round up of result.
pub fn ceil_div(x: u32, y: u32) -> u32 {
    x / y + u32::from(x % y != 0)
}