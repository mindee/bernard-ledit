use crate::pdf::PdfError;
use pdfium_render::prelude::{PdfFontWeight, PdfPageTextChar};

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
    pub bounds: [f32; 4],
}

impl TryFrom<PdfPageTextChar<'_>> for TextChar {
    type Error = PdfError;
    fn try_from(char: PdfPageTextChar<'_>) -> Result<Self, Self::Error> {
        let char_bounds = char.loose_bounds()?;
        Ok(Self {
            char: char
                .unicode_char()
                .ok_or_else(|| PdfError::Other("Missing unicode character".to_string()))?,
            font_name: char.font_name(),
            font_size: char.unscaled_font_size().value,
            font_weight: font_weight_to_u32(char.font_weight()),
            stroke_color: char
                .stroke_color()
                .ok()
                .map(|c| [c.red(), c.green(), c.blue(), c.alpha()]),
            fill_color: char
                .fill_color()
                .ok()
                .map(|c| [c.red(), c.green(), c.blue(), c.alpha()]),
            font_flags: 0,
            bounds: [
                char_bounds.bottom().value,
                char_bounds.left().value,
                char_bounds.top().value,
                char_bounds.right().value,
            ],
        })
    }
}

/// Convert a `PdfFontWeight` to a u32.
#[must_use]
pub const fn font_weight_to_u32(weight: Option<PdfFontWeight>) -> u32 {
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
        None => 0,
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
            bounds: [0.0, 0.0, 0.0, 0.0],
        }
    }
}
