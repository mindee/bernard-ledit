use pdfium_render::prelude::{PdfFontWeight, PdfPageTextChar, PdfRect};

/// Represents a character in a PDF text string.
pub struct TextChar {
    /// The Unicode codepoint of the character.
    pub char: char,
    /// Name of the font used to render the character.
    pub font_name: String,
    /// Font size in points.
    pub font_size: f32,
    /// Font weight.
    pub font_weight: u32,
    /// Font stroke color.
    pub stroke_color: Option<[u8; 4]>,
    /// Font fill color.
    pub fill_color: Option<[u8; 4]>,
    /// Flags associated with the font.
    pub font_flags: i32,
    /// Character bounding box.
    pub bounds: PdfRect,
}

impl TextChar {
    /// Creates a new `TextChar`.
    pub fn new(
        char: char,
        font_name: impl Into<String>,
        font_size: f32,
        font_weight: u32,
        stroke_color: Option<[u8; 4]>,
        fill_color: Option<[u8; 4]>,
        font_flags: i32,
        bounds: PdfRect,
    ) -> Self {
        Self {
            char,
            font_name: font_name.into(),
            font_size,
            font_weight,
            stroke_color,
            fill_color,
            font_flags,
            bounds,
        }
    }
}

/// Convert a `PdfFontWeight` to a u32.
pub fn font_weight_to_u32(weight: Option<PdfFontWeight>) -> u32 {
    match weight {
        Some(PdfFontWeight::Weight100) => 100,
        Some(PdfFontWeight::Weight200) => 200,
        Some(PdfFontWeight::Weight300) => 300,
        Some(PdfFontWeight::Weight400Normal) => 400,
        Some(PdfFontWeight::Weight500) => 500,
        Some(PdfFontWeight::Weight600) => 600,
        Some(PdfFontWeight::Weight700Bold) => 700,
        Some(PdfFontWeight::Weight800) => 800,
        Some(PdfFontWeight::Weight900) => 900,
        Some(PdfFontWeight::Custom(val)) => val,
        None => 400,
    }
}

impl From<PdfPageTextChar<'_>> for TextChar {
    fn from(char: PdfPageTextChar<'_>) -> Self {
        Self::new(
            char.unicode_char().unwrap(),
            char.font_name(),
            char.unscaled_font_size().value,
            font_weight_to_u32(char.font_weight()),
            char.stroke_color()
                .ok()
                .map(|c| [c.red(), c.green(), c.blue(), c.alpha()]),
            char.fill_color()
                .ok()
                .map(|c| [c.red(), c.green(), c.blue(), c.alpha()]),
            0,
            char.loose_bounds().unwrap(),
        )
    }
}

impl Default for TextChar {
    /// Default `TextChar` values.
    fn default() -> Self {
        Self {
            char: '\0',
            font_name: String::from("Helvetica"),
            font_size: 12.0,
            font_weight: 400,
            stroke_color: None,
            fill_color: Some([0, 0, 0, 255]),
            font_flags: 0,
            bounds: PdfRect::new_from_values(0.0, 0.0, 0.0, 0.0),
        }
    }
}
