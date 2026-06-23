//! Resolve textual font descriptors to PDF built-in fonts.
//!
//! # DISCLAIMER
//!
//! This module was mostly LLM-generated, as we do not have sufficient font manipulation knowledge
//! to do so ourselves.
//! # Content
//! PDF defines 14 built-in (standard) fonts that don't require font
//! embedding (Helvetica, Times, Courier — each in four regular/bold/italic
//! variants — plus Symbol and ZapfDingbats). When adding text we don't have
//! arbitrary font files at hand, so we pick the closest built-in for the
//! requested name.

use pdfium_render::prelude::PdfFontBuiltin;

/// PDF font flags bit 7 (1 << 6) indicates an italic font.
const PDF_FONT_FLAG_ITALIC: i32 = 1 << 6;

/// Resolve a textual font name (plus weight and flags) to one of the 14
/// PDF built-in font variants.
///
/// The mapping picks a family from the font name (Times / Courier / Symbol /
/// ZapfDingbats, defaulting to Helvetica for any sans-serif or unknown name)
/// and then picks a bold/italic variant based on `font_weight` (>= 600 is
/// considered bold) and the italic flag in `font_flags` (or "italic" /
/// "oblique" appearing in the font name).
pub fn resolve_builtin_font(font_name: &str, font_weight: u32, font_flags: i32) -> PdfFontBuiltin {
    let lower = font_name.to_ascii_lowercase();

    if lower.contains("zapf") || lower.contains("dingbat") {
        return PdfFontBuiltin::ZapfDingbats;
    }
    if lower.contains("symbol") {
        return PdfFontBuiltin::Symbol;
    }

    let bold = font_weight >= 600
        || lower.contains("bold")
        || lower.contains("black")
        || lower.contains("heavy");
    let italic = (font_flags & PDF_FONT_FLAG_ITALIC) != 0
        || lower.contains("italic")
        || lower.contains("oblique");

    let is_times = lower.contains("times")
        || lower.contains("serif") && !lower.contains("sans")
        || lower.contains("roman")
        || lower.contains("georgia");
    let is_courier = lower.contains("courier")
        || lower.contains("mono")
        || lower.contains("consolas")
        || lower.contains("menlo");

    if is_courier {
        match (bold, italic) {
            (false, false) => PdfFontBuiltin::Courier,
            (true, false) => PdfFontBuiltin::CourierBold,
            (false, true) => PdfFontBuiltin::CourierOblique,
            (true, true) => PdfFontBuiltin::CourierBoldOblique,
        }
    } else if is_times {
        match (bold, italic) {
            (false, false) => PdfFontBuiltin::TimesRoman,
            (true, false) => PdfFontBuiltin::TimesBold,
            (false, true) => PdfFontBuiltin::TimesItalic,
            (true, true) => PdfFontBuiltin::TimesBoldItalic,
        }
    } else {
        match (bold, italic) {
            (false, false) => PdfFontBuiltin::Helvetica,
            (true, false) => PdfFontBuiltin::HelveticaBold,
            (false, true) => PdfFontBuiltin::HelveticaOblique,
            (true, true) => PdfFontBuiltin::HelveticaBoldOblique,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_arial_to_helvetica() {
        assert_eq!(
            resolve_builtin_font("Arial", 400, 0),
            PdfFontBuiltin::Helvetica
        );
    }

    #[test]
    fn detects_bold_from_weight() {
        assert_eq!(
            resolve_builtin_font("Arial", 700, 0),
            PdfFontBuiltin::HelveticaBold
        );
    }

    #[test]
    fn detects_italic_from_flags() {
        assert_eq!(
            resolve_builtin_font("Arial", 400, 1 << 6),
            PdfFontBuiltin::HelveticaOblique
        );
    }

    #[test]
    fn detects_bold_italic_from_name() {
        assert_eq!(
            resolve_builtin_font("Helvetica Bold Italic", 400, 0),
            PdfFontBuiltin::HelveticaBoldOblique
        );
    }

    #[test]
    fn maps_times_family() {
        assert_eq!(
            resolve_builtin_font("Times New Roman", 400, 0),
            PdfFontBuiltin::TimesRoman
        );
        assert_eq!(
            resolve_builtin_font("Georgia", 700, 0),
            PdfFontBuiltin::TimesBold
        );
    }

    #[test]
    fn maps_courier_family() {
        assert_eq!(
            resolve_builtin_font("Courier New", 400, 0),
            PdfFontBuiltin::Courier
        );
        assert_eq!(
            resolve_builtin_font("Consolas", 400, 1 << 6),
            PdfFontBuiltin::CourierOblique
        );
    }

    #[test]
    fn maps_symbol_and_dingbats() {
        assert_eq!(
            resolve_builtin_font("Symbol", 400, 0),
            PdfFontBuiltin::Symbol
        );
        assert_eq!(
            resolve_builtin_font("ZapfDingbats", 400, 0),
            PdfFontBuiltin::ZapfDingbats
        );
    }
}
