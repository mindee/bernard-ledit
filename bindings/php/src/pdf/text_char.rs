use bernard_ledit::pdf::TextChar as RustTextChar;
use ext_php_rs::{php_class, php_impl};
use std::collections::HashMap;

/// Text character representation.
#[php_class]
#[derive(Debug, Clone)]
pub struct PhpTextChar {
    /// Actual character.
    char: char,
    /// Font name.
    font_name: String,
    /// Font size.
    font_size: f32,
    /// Font weight.
    font_weight: u32,
    /// Stroke color.
    stroke_color: Option<[u8; 4]>,
    /// Fill color.
    fill_color: Option<[u8; 4]>,
    /// Font flags.
    font_flags: i32,
    /// Bounds of the character.
    bounds: (f32, f32, f32, f32),
}

/// Helper to safely convert a Vec into a fixed 4-byte array
fn vec_to_array<T: Copy + Default>(vec: Option<Vec<T>>) -> Option<[T; 4]> {
    vec.and_then(|v| v.try_into().ok())
}

/// Helper to safely convert a Vec into a fixed 4-tuple
fn parse_bounds(bounds: Option<Vec<f32>>) -> (f32, f32, f32, f32) {
    if let Some(v) = bounds
        && v.len() >= 4
    {
        return (v[0], v[1], v[2], v[3]);
    }
    (0.0, 0.0, 0.0, 0.0)
}

#[php_impl]
impl PhpTextChar {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn __construct(
        char: String,
        font_name: String,
        font_size: f32,
        font_weight: u32,
        stroke_color: Option<Vec<u8>>,
        fill_color: Option<Vec<u8>>,
        font_flags: i32,
        bounds: Option<Vec<f32>>,
    ) -> Self {
        Self {
            char: char.chars().next().unwrap_or(' '),
            font_name,
            font_size,
            font_weight,
            stroke_color: vec_to_array(stroke_color),
            fill_color: vec_to_array(fill_color),
            font_flags,
            bounds: parse_bounds(bounds),
        }
    }

    /// Default debug info.
    #[must_use]
    pub fn __debug_info(&self) -> HashMap<&'static str, String> {
        let mut info = HashMap::new();

        info.insert("char", self.char.to_string());
        info.insert("font_name", self.font_name.clone());
        info.insert("font_size", self.font_size.to_string());
        info.insert("font_weight", self.font_weight.to_string());
        info.insert("font_flags", self.font_flags.to_string());

        info.insert("bounds", format!("{:?}", self.bounds));

        info.insert(
            "stroke_color",
            self.stroke_color
                .map_or_else(|| "null".to_string(), |c| format!("{c:?}")),
        );
        info.insert(
            "fill_color",
            self.fill_color
                .map_or_else(|| "null".to_string(), |c| format!("{c:?}")),
        );

        info
    }

    /// String rep.
    #[must_use]
    pub fn __to_string(&self) -> String {
        self.char.to_string()
    }
}

impl From<RustTextChar> for PhpTextChar {
    fn from(value: RustTextChar) -> Self {
        Self {
            char: value.char,
            font_name: value.font_name.clone(),
            font_size: value.font_size,
            font_weight: value.font_weight,
            stroke_color: value.stroke_color,
            fill_color: value.fill_color,
            font_flags: value.font_flags,
            bounds: value.bounds.into(),
        }
    }
}

impl From<&RustTextChar> for PhpTextChar {
    fn from(value: &RustTextChar) -> Self {
        Self {
            char: value.char,
            font_size: value.font_size,
            font_weight: value.font_weight,
            stroke_color: value.stroke_color,
            fill_color: value.fill_color,
            font_flags: value.font_flags,
            font_name: value.font_name.clone(),
            bounds: value.bounds.into(),
        }
    }
}
impl From<&PhpTextChar> for RustTextChar {
    fn from(value: &PhpTextChar) -> Self {
        Self {
            char: value.char,
            font_name: value.font_name.clone(),
            font_size: value.font_size,
            font_weight: value.font_weight,
            stroke_color: value.stroke_color,
            fill_color: value.fill_color,
            font_flags: value.font_flags,
            bounds: value.bounds.into(),
        }
    }
}

/// These tests intentionally stay in plain-Rust because ext-php-rs classes rely on a live Zend
/// engine for anything that touches FFI.
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rust_text_char() -> RustTextChar {
        RustTextChar {
            char: 'A',
            font_name: "Arial".to_string(),
            font_size: 12.5,
            font_weight: 400,
            stroke_color: Some([1, 2, 3, 4]),
            fill_color: Some([5, 6, 7, 8]),
            font_flags: 32,
            bounds: [0.0, 1.0, 2.0, 3.0],
        }
    }

    #[test]
    fn from_rust_text_char_preserves_all_fields() {
        let php_char = PhpTextChar::from(sample_rust_text_char());
        let rust_char = sample_rust_text_char();

        assert_eq!(php_char.char, rust_char.char);
        assert_eq!(php_char.font_name, rust_char.font_name);
        assert!((php_char.font_size - rust_char.font_size).abs() < f32::EPSILON);
        assert_eq!(php_char.font_weight, rust_char.font_weight);
        assert_eq!(php_char.stroke_color, rust_char.stroke_color);
        assert_eq!(php_char.fill_color, rust_char.fill_color);
        assert_eq!(php_char.font_flags, rust_char.font_flags);
        assert_eq!(php_char.bounds, rust_char.bounds.into());
    }

    #[test]
    fn from_ref_rust_text_char_matches_owned_conversion() {
        let rust_char = sample_rust_text_char();
        let via_ref = PhpTextChar::from(&rust_char);
        let via_owned = PhpTextChar::from(rust_char);

        assert_eq!(via_ref.char, via_owned.char);
        assert_eq!(via_ref.font_name, via_owned.font_name);
        assert_eq!(via_ref.bounds, via_owned.bounds);
    }

    #[test]
    fn round_trip_php_to_rust_to_php_preserves_fields() {
        let original = sample_rust_text_char();
        let php_char = PhpTextChar::from(&original);
        let back_to_rust = RustTextChar::from(&php_char);

        assert_eq!(back_to_rust.char, original.char);
        assert_eq!(back_to_rust.font_name, original.font_name);
        assert!((back_to_rust.font_size - original.font_size).abs() < f32::EPSILON);
        assert_eq!(back_to_rust.font_weight, original.font_weight);
        assert_eq!(back_to_rust.stroke_color, original.stroke_color);
        assert_eq!(back_to_rust.fill_color, original.fill_color);
        assert_eq!(back_to_rust.font_flags, original.font_flags);
        for (a, b) in back_to_rust.bounds.iter().zip(original.bounds.iter()) {
            assert!((a - b).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn vec_to_array_converts_matching_length() {
        let v = Some(vec![10u8, 20, 30, 40]);
        assert_eq!(vec_to_array(v), Some([10, 20, 30, 40]));
    }

    #[test]
    fn vec_to_array_rejects_wrong_length() {
        assert_eq!(vec_to_array::<u8>(Some(vec![1, 2, 3])), None);
        assert_eq!(vec_to_array::<u8>(Some(vec![1, 2, 3, 4, 5])), None);
    }

    #[test]
    fn vec_to_array_none_stays_none() {
        assert_eq!(vec_to_array::<u8>(None), None);
    }

    #[test]
    fn parse_bounds_extracts_first_four_values() {
        assert_eq!(
            parse_bounds(Some(vec![1.0, 2.0, 3.0, 4.0])),
            (1.0, 2.0, 3.0, 4.0)
        );
    }

    #[test]
    fn parse_bounds_ignores_extra_values() {
        assert_eq!(
            parse_bounds(Some(vec![1.0, 2.0, 3.0, 4.0, 5.0])),
            (1.0, 2.0, 3.0, 4.0)
        );
    }

    #[test]
    fn parse_bounds_defaults_to_zeros_when_missing_or_short() {
        assert_eq!(parse_bounds(None), (0.0, 0.0, 0.0, 0.0));
        assert_eq!(parse_bounds(Some(vec![1.0, 2.0])), (0.0, 0.0, 0.0, 0.0));
    }

    #[test]
    fn construct_truncates_multi_char_string_to_first_char() {
        let php_char = PhpTextChar::__construct(
            "AB".to_string(),
            "Arial".to_string(),
            10.0,
            400,
            None,
            None,
            0,
            None,
        );
        assert_eq!(php_char.char, 'A');
    }

    #[test]
    fn construct_defaults_empty_string_to_space() {
        let php_char = PhpTextChar::__construct(
            String::new(),
            "Arial".to_string(),
            10.0,
            400,
            None,
            None,
            0,
            None,
        );
        assert_eq!(php_char.char, ' ');
    }
}
