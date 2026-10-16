use crate::pdf::{Document, PdfError};
use std::collections::BTreeSet;
use std::str::FromStr;

/// Page operation to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageOperation {
    /// Keep only the specified pages.
    KeepOnly,
    /// Remove the specified pages.
    Remove,
}

impl FromStr for PageOperation {
    type Err = PageOptionsError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "KEEP_ONLY" => Ok(Self::KeepOnly),
            "REMOVE" => Ok(Self::Remove),
            _ => Err(PageOptionsError::InvalidOperation(s.to_owned())),
        }
    }
}

/// Page options errors.
#[derive(Debug, thiserror::Error)]
pub enum PageOptionsError {
    /// PDF errors.
    #[error(transparent)]
    Pdf(#[from] PdfError),
    #[error("PDF pages are empty")]
    /// PDF pages are empty.
    EmptyPdf,
    #[error("Invalid cut behavior specified: {0}")]
    /// Invalid cut behavior specified.
    InvalidOperation(String),
    #[error("Resulting PDF would have no pages left.")]
    /// Resulting PDF would have no pages left.
    NoPagesLeft,
}

/// Page options outcome.
pub struct PageOptionsOutcome {
    /// `None` when nothing was done (document has fewer pages than `on_min_pages`).
    pub document: Option<Document>,
    /// Requested indexes that don't exist in the source, so callers can warn.
    pub skipped: Vec<i32>,
}

/// Keeps or removes pages. Negative indexes count from the end, as in Python.
/// Kept pages are emitted in ascending order, without duplicates.
/// # Errors
/// See [`PageOptionsError`] if the resulting document would have no pages.
pub fn apply_page_options(
    doc: &Document,
    operation: PageOperation,
    on_min_pages: usize,
    indexes: &[i32],
) -> Result<PageOptionsOutcome, PageOptionsError> {
    if doc.has_no_content()? {
        return Err(PageOptionsError::EmptyPdf);
    }
    let count = doc.page_count()?;
    if on_min_pages > usize::from(count) {
        return Ok(PageOptionsOutcome {
            document: None,
            skipped: vec![],
        });
    }

    let mut selected = BTreeSet::new();
    let mut skipped = Vec::new();
    for &i in indexes {
        let idx = if i < 0 {
            i64::from(i) + i64::from(count)
        } else {
            i64::from(i)
        };
        match u16::try_from(idx).ok().filter(|&p| p < count) {
            Some(p) => {
                selected.insert(p);
            }
            None => skipped.push(i),
        }
    }

    let keep: Vec<u16> = match operation {
        PageOperation::KeepOnly => selected.into_iter().collect(),
        PageOperation::Remove => (0..count).filter(|p| !selected.contains(p)).collect(),
    };
    if keep.is_empty() {
        return Err(PageOptionsError::NoPagesLeft);
    }

    let mut out = Document::new()?;
    out.import_pages(doc, &keep)?;
    Ok(PageOptionsOutcome {
        document: Some(out),
        skipped,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::pdfium;

    fn multipage() -> Document {
        pdfium();
        Document::from_bytes(test_data_bytes!("file_types/pdf/multipage.pdf").to_vec()).unwrap()
    }

    /// Page `i` of multipage.pdf holds (i + 1)^2 asterisks, so the counts identify the pages.
    fn star_counts(doc: &Document) -> Vec<usize> {
        (0..doc.page_count().unwrap())
            .map(|i| doc.page(i).unwrap().text().unwrap().matches('*').count())
            .collect()
    }

    fn run(
        doc: &Document,
        op: PageOperation,
        on_min: usize,
        idx: &[i32],
    ) -> Result<PageOptionsOutcome, PageOptionsError> {
        apply_page_options(doc, op, on_min, idx)
    }

    #[test]
    fn keep_only_selects_pages() {
        let doc = multipage();
        let out = run(&doc, PageOperation::KeepOnly, 0, &[0, 2]).unwrap();
        assert_eq!(star_counts(&out.document.unwrap()), vec![1, 9]);
        assert_eq!(out.skipped, []);
    }

    #[test]
    fn keep_only_emits_ascending_order_regardless_of_input_order() {
        let doc = multipage();
        let out = run(&doc, PageOperation::KeepOnly, 0, &[2, 0]).unwrap();
        assert_eq!(star_counts(&out.document.unwrap()), vec![1, 9]);
    }

    #[test]
    fn keep_only_deduplicates() {
        let doc = multipage();
        let out = run(&doc, PageOperation::KeepOnly, 0, &[1, 1, 1]).unwrap();
        assert_eq!(star_counts(&out.document.unwrap()), vec![4]);
    }

    #[test]
    fn remove_drops_pages() {
        let doc = multipage();
        let out = run(&doc, PageOperation::Remove, 0, &[0, 11]).unwrap();
        let counts = star_counts(&out.document.unwrap());
        assert_eq!(counts.len(), 10);
        assert!(!counts.contains(&1) && !counts.contains(&144));
    }

    #[test]
    fn negative_indexes_count_from_the_end() {
        let doc = multipage();
        let kept = run(&doc, PageOperation::KeepOnly, 0, &[-1]).unwrap();
        assert_eq!(star_counts(&kept.document.unwrap()), vec![144]);

        let removed = run(&doc, PageOperation::Remove, 0, &[-1]).unwrap();
        let counts = star_counts(&removed.document.unwrap());
        assert_eq!(counts.len(), 11);
        assert!(!counts.contains(&144));
    }

    #[test]
    fn out_of_range_indexes_are_reported_not_fatal() {
        let doc = multipage();
        // 12 is past the end; -13 resolves to -1, before the start.
        let out = run(&doc, PageOperation::KeepOnly, 0, &[0, 12, -13]).unwrap();
        assert_eq!(out.skipped, vec![12, -13]);
        assert_eq!(star_counts(&out.document.unwrap()), vec![1]);
    }

    #[test]
    fn remove_ignores_out_of_range_indexes() {
        let doc = multipage();
        let out = run(&doc, PageOperation::Remove, 0, &[99, 0]).unwrap();
        assert_eq!(out.skipped, vec![99]);
        assert_eq!(out.document.unwrap().page_count().unwrap(), 11);
    }

    #[test]
    fn below_on_min_pages_does_nothing() {
        let doc = multipage();
        let out = run(&doc, PageOperation::KeepOnly, 13, &[0]).unwrap();
        assert!(out.document.is_none());
        assert_eq!(out.skipped, []);
    }

    #[test]
    fn on_min_pages_equal_to_page_count_still_applies() {
        let doc = multipage();
        let out = run(&doc, PageOperation::KeepOnly, 12, &[0]).unwrap();
        assert!(out.document.is_some());
    }

    #[test]
    fn keep_only_with_nothing_valid_errors() {
        let doc = multipage();
        assert!(matches!(
            run(&doc, PageOperation::KeepOnly, 0, &[99]),
            Err(PageOptionsError::NoPagesLeft)
        ));
        assert!(matches!(
            run(&doc, PageOperation::KeepOnly, 0, &[]),
            Err(PageOptionsError::NoPagesLeft)
        ));
    }

    #[test]
    fn removing_every_page_errors() {
        let doc = multipage();
        let all: Vec<i32> = (0..12).collect();
        assert!(matches!(
            run(&doc, PageOperation::Remove, 0, &all),
            Err(PageOptionsError::NoPagesLeft)
        ));
    }

    #[test]
    fn remove_with_no_indexes_keeps_everything() {
        let doc = multipage();
        let out = run(&doc, PageOperation::Remove, 0, &[]).unwrap();
        assert_eq!(out.document.unwrap().page_count().unwrap(), 12);
    }

    #[test]
    fn empty_pdf_errors_before_on_min_pages_check() {
        pdfium();
        let blank =
            Document::from_bytes(test_data_bytes!("file_types/pdf/blank.pdf").to_vec()).unwrap();
        // on_min_pages is huge: the empty check must still win, as in the old SDK flow.
        assert!(matches!(
            run(&blank, PageOperation::KeepOnly, 1000, &[0]),
            Err(PageOptionsError::EmptyPdf)
        ));
    }

    #[test]
    fn source_document_is_not_modified() {
        let doc = multipage();
        run(&doc, PageOperation::Remove, 0, &[0, 1, 2]).unwrap();
        assert_eq!(doc.page_count().unwrap(), 12);
    }

    #[test]
    fn output_survives_save_and_reload() {
        let doc = multipage();
        let out = run(&doc, PageOperation::KeepOnly, 0, &[3, 4]).unwrap();
        let bytes = out.document.unwrap().to_bytes().unwrap();
        let reloaded = Document::from_bytes(bytes).unwrap();
        assert_eq!(star_counts(&reloaded), vec![16, 25]);
    }

    #[test]
    fn operation_parses_case_insensitively() {
        assert_eq!(
            "KEEP_ONLY".parse::<PageOperation>().unwrap(),
            PageOperation::KeepOnly
        );
        assert_eq!(
            "keep_only".parse::<PageOperation>().unwrap(),
            PageOperation::KeepOnly
        );
        assert_eq!(
            "Remove".parse::<PageOperation>().unwrap(),
            PageOperation::Remove
        );
    }

    #[test]
    fn unknown_operation_errors_with_the_given_name() {
        match "shuffle".parse::<PageOperation>() {
            Err(PageOptionsError::InvalidOperation(name)) => assert_eq!(name, "shuffle"),
            other => panic!("unexpected: {other:?}"),
        }
    }
}
