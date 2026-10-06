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
            bounds: (
                value.bounds[0],
                value.bounds[1],
                value.bounds[2],
                value.bounds[3],
            ),
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
            bounds: (
                value.bounds[0],
                value.bounds[1],
                value.bounds[2],
                value.bounds[3],
            ),
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
