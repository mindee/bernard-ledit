//! Range checks for integer and float arguments, with the messages the PHP binding uses.

use crate::BernardLeditError;
use std::fmt::Display;

/// Lowest DPI pages are rendered at.
pub const MIN_DPI: f64 = 1.0;
/// Highest DPI pages are rendered at: a page's bitmap grows with the square of its DPI.
pub const MAX_DPI: f64 = 1200.0;

type Result<T> = std::result::Result<T, BernardLeditError>;

/// Returns `value` as a `T` if it's between `min` and `max`.
pub fn int<T>(name: &str, value: i64, min: T, max: T) -> Result<T>
where
    T: Copy + Display + Into<i64> + TryFrom<i64>,
{
    if (min.into()..=max.into()).contains(&value)
        && let Ok(narrowed) = T::try_from(value)
    {
        return Ok(narrowed);
    }
    Err(BernardLeditError::InvalidArgument(format!(
        "{name} must be between {min} and {max}, got {value}"
    )))
}

/// Checks each of `values` with [`int`], naming them `name[i]`.
pub fn ints<T>(name: &str, values: &[i32], min: T, max: T) -> Result<Vec<T>>
where
    T: Copy + Display + Into<i64> + TryFrom<i64>,
{
    values
        .iter()
        .enumerate()
        .map(|(i, &value)| int(&format!("{name}[{i}]"), value.into(), min, max))
        .collect()
}

/// A page index: between 0 and 65535.
pub fn page_index(name: &str, value: i32) -> Result<u16> {
    int(name, value.into(), 0, u16::MAX)
}

/// A JPEG quality between `min` and 100.
pub fn quality(value: i32, min: u8) -> Result<u8> {
    int("quality", value.into(), min, 100)
}

/// A render resolution between [`MIN_DPI`] and [`MAX_DPI`], as core takes it.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the range fits in f32, which core takes"
)]
pub fn dpi(value: f64) -> Result<f32> {
    if (MIN_DPI..=MAX_DPI).contains(&value) {
        Ok(value as f32)
    } else {
        Err(BernardLeditError::InvalidArgument(format!(
            "dpi must be a finite number between {MIN_DPI} and {MAX_DPI}, got {value}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_in_range_are_narrowed() {
        assert_eq!(int("quality", 100, 1_u8, 100).ok(), Some(100));
        assert_eq!(page_index("index", 65535).ok(), Some(u16::MAX));
        assert_eq!(dpi(1200.0).ok(), Some(1200.0));
    }

    #[test]
    fn values_out_of_range_name_the_argument_and_range() {
        let err = quality(300, 5).unwrap_err();
        assert_eq!(
            err.to_string(),
            "quality must be between 5 and 100, got 300"
        );
        assert!(page_index("index", -1).is_err());
        assert_eq!(
            ints("indexes", &[0, 65536], 0_u16, u16::MAX)
                .unwrap_err()
                .to_string(),
            "indexes[1] must be between 0 and 65535, got 65536"
        );
        for bad in [0.0, -1.0, 5000.0, f64::NAN, f64::INFINITY] {
            assert!(dpi(bad).is_err(), "{bad}");
        }
    }
}
