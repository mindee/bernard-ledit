/// Errors related to attempt to PDF fixing.
#[derive(Debug, thiserror::Error)]
pub enum PdfFixError {
    /// Failed to validate PDF-like object.
    #[error("Provided stream isn't a valid PDF-like object.")]
    NotPdf,
    /// PDF can't be fixed.
    #[error("PDF couldn't be fixed. PDF tag was found at position {pos}.")]
    TagTooFar {
        /// Position of the fixing tag.
        pos: usize,
    },
}

/// Returns `data` starting at the first `%PDF-` marker, if it sits before `max_offset`.
/// # Errors
/// Returns [`PdfFixError`] if there is no marker, or it is at or past `max_offset`.
pub fn fix_pdf(data: &[u8], max_offset: usize) -> Result<&[u8], PdfFixError> {
    let pos = data
        .windows(5)
        .position(|w| w == b"%PDF-")
        .ok_or(PdfFixError::NotPdf)?;
    if pos >= max_offset {
        return Err(PdfFixError::TagTooFar { pos });
    }
    Ok(&data[pos..])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::{Document, pdfium};

    #[test]
    fn returns_input_unchanged_when_marker_is_first() {
        let data = b"%PDF-1.4 rest";
        assert_eq!(fix_pdf(data, 500).unwrap(), data);
    }

    #[test]
    fn strips_bytes_before_marker() {
        let data = b"junk\r\n%PDF-1.4 rest";
        assert_eq!(fix_pdf(data, 500).unwrap(), b"%PDF-1.4 rest");
    }

    #[test]
    fn uses_first_marker_when_several_exist() {
        let data = b"ab%PDF-1 cd%PDF-2";
        assert_eq!(fix_pdf(data, 500).unwrap(), b"%PDF-1 cd%PDF-2");
    }

    #[test]
    fn marker_just_below_limit_is_accepted() {
        let mut data = vec![b'x'; 9];
        data.extend_from_slice(b"%PDF-1.4");
        assert_eq!(fix_pdf(&data, 10).unwrap(), b"%PDF-1.4");
    }

    #[test]
    fn marker_at_limit_is_rejected() {
        let mut data = vec![b'x'; 10];
        data.extend_from_slice(b"%PDF-1.4");
        assert!(matches!(
            fix_pdf(&data, 10),
            Err(PdfFixError::TagTooFar { pos: 10 })
        ));
    }

    #[test]
    fn zero_limit_rejects_everything() {
        // `pos >= max_offset` means even a marker at 0 fails. Matches the Python `pos < max` rule.
        assert!(matches!(
            fix_pdf(b"%PDF-1.4", 0),
            Err(PdfFixError::TagTooFar { pos: 0 })
        ));
    }

    #[test]
    fn missing_marker_errors() {
        assert!(matches!(
            fix_pdf(b"just some text", 500),
            Err(PdfFixError::NotPdf)
        ));
    }

    #[test]
    fn marker_is_case_sensitive() {
        assert!(matches!(
            fix_pdf(b"xx%pdf-1.4", 500),
            Err(PdfFixError::NotPdf)
        ));
    }

    #[test]
    fn marker_without_dash_is_not_a_marker() {
        assert!(matches!(fix_pdf(b"%PDF", 500), Err(PdfFixError::NotPdf)));
    }

    #[test]
    fn empty_and_tiny_inputs_error() {
        assert!(matches!(fix_pdf(b"", 500), Err(PdfFixError::NotPdf)));
        assert!(matches!(fix_pdf(b"%PD", 500), Err(PdfFixError::NotPdf)));
    }

    #[test]
    fn fixed_bytes_load_as_a_real_pdf() {
        pdfium();
        let pdf: &[u8] = test_data_bytes!("file_types/pdf/multipage.pdf");
        let mut data = b"GARBAGE-HEADER\r\n".to_vec();
        data.extend_from_slice(pdf);

        let fixed = fix_pdf(&data, 500).unwrap();
        assert_eq!(fixed, pdf);
        let doc = Document::from_bytes(fixed.to_vec()).unwrap();
        assert_eq!(doc.page_count().unwrap(), 12);
    }
}
