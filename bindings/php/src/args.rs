//! Range checks for `int` and `float` arguments.
//!
//! Integer parameters are taken as `i64`, PHP's `int`, and narrowed here: ext-php-rs would reject an
//! `int` that doesn't fit the Rust type with a `TypeError` that doesn't give the valid range.

use ext_php_rs::{
    exception::{PhpException, PhpResult},
    zend::ce,
};
use std::fmt::Display;

/// Lowest DPI pages are rendered at.
pub const MIN_DPI: f64 = 1.0;
/// Highest DPI pages are rendered at: a page's bitmap grows with the square of its DPI.
pub const MAX_DPI: f64 = 1200.0;
/// Lowest scale `PdfPage::renderToBitmap` takes.
pub const MIN_SCALE: f64 = 0.01;
/// Highest scale `PdfPage::renderToBitmap` takes, about `MAX_DPI` at 72 DPI per unit.
pub const MAX_SCALE: f64 = 16.0;

/// A `\ValueError` with `message`.
#[must_use]
pub fn value_error(message: String) -> PhpException {
    PhpException::new(message, 0, ce::value_error())
}

/// Returns `value` as a `T` if it's between `min` and `max`.
///
/// # Errors
/// A `\ValueError` naming `name` and the range otherwise.
pub fn int<T>(name: &str, value: i64, min: T, max: T) -> PhpResult<T>
where
    T: Copy + Display + Into<i64> + TryFrom<i64>,
{
    if (min.into()..=max.into()).contains(&value)
        && let Ok(narrowed) = T::try_from(value)
    {
        return Ok(narrowed);
    }
    Err(value_error(format!(
        "{name} must be between {min} and {max}, got {value}"
    )))
}

/// Checks each of `values` with [`int`], naming them `name[i]`.
///
/// # Errors
/// A `\ValueError` for the first value out of range.
pub fn ints<T>(name: &str, values: &[i64], min: T, max: T) -> PhpResult<Vec<T>>
where
    T: Copy + Display + Into<i64> + TryFrom<i64>,
{
    values
        .iter()
        .enumerate()
        .map(|(i, &value)| int(&format!("{name}[{i}]"), value, min, max))
        .collect()
}

/// A page index: between 0 and 65535.
///
/// # Errors
/// A `\ValueError` if `value` is out of range.
pub fn page_index(name: &str, value: i64) -> PhpResult<u16> {
    int(name, value, 0, u16::MAX)
}

/// A JPEG quality between `min` and 100.
///
/// # Errors
/// A `\ValueError` if `value` is out of range.
pub fn quality(value: i64, min: u8) -> PhpResult<u8> {
    int("quality", value, min, 100)
}

/// Returns `value` as a `usize` if it isn't negative.
///
/// # Errors
/// A `\ValueError` naming `name` otherwise.
pub fn non_negative(name: &str, value: i64) -> PhpResult<usize> {
    if value < 0 {
        return Err(value_error(format!(
            "{name} must be at least 0, got {value}"
        )));
    }
    Ok(usize::try_from(value).unwrap_or(usize::MAX))
}

/// Returns `value` if it's a finite number between `min` and `max`.
///
/// # Errors
/// A `\ValueError` naming `name` and the range otherwise, `NAN` and infinities included.
pub fn float(name: &str, value: f64, min: f64, max: f64) -> PhpResult<f64> {
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(value_error(format!(
            "{name} must be a finite number between {min} and {max}, got {value}"
        )))
    }
}

/// A render resolution between [`MIN_DPI`] and [`MAX_DPI`].
///
/// # Errors
/// A `\ValueError` if `value` is out of range.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the range fits in f32, which core takes"
)]
pub fn dpi(value: f64) -> PhpResult<f32> {
    float("dpi", value, MIN_DPI, MAX_DPI).map(|dpi| dpi as f32)
}
