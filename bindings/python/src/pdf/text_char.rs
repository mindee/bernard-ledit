use bernard_ledit::pdf::TextChar as RustTextChar;
use pyo3::prelude::*;

/// A text character extracted from or written to a PDF page.
#[pyclass(name = "TextChar", module = "bernard_ledit.pdf")]
pub struct PyTextChar {
    #[pyo3(get)]
    char: char,
    #[pyo3(get)]
    font_name: String,
    #[pyo3(get)]
    font_size: f32,
    #[pyo3(get)]
    font_weight: u32,
    #[pyo3(get)]
    stroke_color: Option<[u8; 4]>,
    #[pyo3(get)]
    fill_color: Option<[u8; 4]>,
    #[pyo3(get)]
    font_flags: i32,
    #[pyo3(get)]
    bounds: (f32, f32, f32, f32),
}

#[pymethods]
impl PyTextChar {
    #[new]
    #[pyo3(signature = (
        char,
        font_name,
        font_size,
        font_weight,
        stroke_color = None,
        fill_color = None,
        font_flags = 0,
        bounds = (0.0, 0.0, 0.0, 0.0),
    ))]
    #[allow(clippy::too_many_arguments)]
    const fn new(
        char: char,
        font_name: String,
        font_size: f32,
        font_weight: u32,
        stroke_color: Option<[u8; 4]>,
        fill_color: Option<[u8; 4]>,
        font_flags: i32,
        bounds: (f32, f32, f32, f32),
    ) -> Self {
        Self {
            char,
            font_name,
            font_size,
            font_weight,
            stroke_color,
            fill_color,
            font_flags,
            bounds,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "TextChar(char={:?}, font_name={:?}, font_size={}, font_weight={}, font_flags={})",
            self.char, self.font_name, self.font_size, self.font_weight, self.font_flags
        )
    }
}

impl From<RustTextChar> for PyTextChar {
    fn from(value: RustTextChar) -> Self {
        Self {
            char: value.char,
            font_name: value.font_name,
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

impl From<&PyTextChar> for RustTextChar {
    fn from(value: &PyTextChar) -> Self {
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
