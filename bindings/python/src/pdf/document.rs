//! PDF document.
use super::page::PyPdfPage;
use crate::pdf::error::{closed_err, format_pdf_err};
use bernard_ledit::pdf::Document;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::sync::Mutex;

/// A PDF document.
#[pyclass(name = "PdfDocument", module = "bernard_ledit.pdf")]
pub struct PyPdfDocument {
    inner: Mutex<Option<Document>>,
}

/// PDF input.
#[derive(FromPyObject)]
pub enum PdfInput<'py> {
    /// Raw bytes.
    Bytes(Bound<'py, PyBytes>),
    /// File-like object.
    FileLike(Bound<'py, PyAny>),
}

/// PDF document methods.
#[pymethods]
impl PyPdfDocument {
    /// Load a PDF from a file-like object or raw bytes.
    #[new]
    fn new(input: PdfInput<'_>) -> PyResult<Self> {
        let bytes: Vec<u8> = match input {
            PdfInput::Bytes(b) => b.as_bytes().to_vec(),
            PdfInput::FileLike(obj) => {
                let data = obj.call_method0("read")?;
                data.extract::<Vec<u8>>()?
            }
        };
        let doc = Document::from_bytes(bytes).map_err(|e| format_pdf_err(&e))?;
        Ok(Self {
            inner: Mutex::new(Some(doc)),
        })
    }

    /// Create a new empty PDF document.
    #[pyo3(name = "new")]
    #[staticmethod]
    fn create() -> PyResult<Self> {
        let doc = Document::new().map_err(|e| format_pdf_err(&e))?;
        Ok(Self {
            inner: Mutex::new(Some(doc)),
        })
    }

    /// Number of pages in the document.
    fn __len__(&self) -> PyResult<usize> {
        self.with_doc(|d| {
            d.page_count()
                .map(usize::from)
                .map_err(|e| format_pdf_err(&e))
        })?
    }

    /// Get a specific page from the document.
    /// First checks if the index exists within the document by opening the page then discards it.
    fn get_page(slf: Bound<'_, Self>, index: u16) -> PyResult<PyPdfPage> {
        slf.borrow().with_doc(|d| {
            d.page(index)
                .map(|_page| ())
                .map_err(|e| format_pdf_err(&e))
        })??;
        Ok(PyPdfPage {
            doc: slf.unbind(),
            index,
        })
    }

    /// Import a subset of pages from `src` into `self`, appending them.
    // While clippy complains about pass-by-value, pyo3 can't handle references.
    #[allow(clippy::needless_pass_by_value)]
    fn import_pages(&self, src: &Self, indexes: Vec<u16>) -> PyResult<()> {
        if std::ptr::eq(self, src) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Cannot import pages from a document into itself",
            ));
        }
        let mut self_guard = self.inner.lock().map_err(|_| closed_err())?;
        let src_guard = src.inner.lock().map_err(|_| closed_err())?;
        let dst = self_guard.as_mut().ok_or_else(closed_err)?;
        let s = src_guard.as_ref().ok_or_else(closed_err)?;
        dst.import_pages(s, &indexes)
            .map_err(|e| format_pdf_err(&e))?;
        drop(src_guard);
        drop(self_guard);
        Ok(())
    }

    /// Save the document into a file-like object.
    fn save(&self, buf: &Bound<'_, PyAny>) -> PyResult<()> {
        let mut bytes: Vec<u8> = Vec::new();
        self.with_doc(|d| d.save(&mut bytes).map_err(|e| format_pdf_err(&e)))??;
        buf.call_method1("write", (PyBytes::new(buf.py(), &bytes),))?;
        Ok(())
    }

    /// Close the document.
    fn close(&self) -> PyResult<()> {
        let mut guard = self.inner.lock().map_err(|_| closed_err())?;
        *guard = None;
        drop(guard);
        Ok(())
    }

    /// String representation showing page count, or "closed" if already closed.
    fn __repr__(&self) -> String {
        self.inner.lock().map_or_else(
            |_| "PdfDocument(<locked>)".to_string(),
            |guard| {
                guard.as_ref().map_or_else(
                    || "PdfDocument(closed)".to_string(),
                    |doc| {
                        doc.page_count().map_or_else(
                            |_| "PdfDocument(<error reading page count>)".to_string(),
                            |n| format!("PdfDocument({n} pages)"),
                        )
                    },
                )
            },
        )
    }

    /// Context manager for closing the document.
    // `const fn` breaks pyo3's `#[pymethods]` macro expansion (pyo3 #[pymethods] issue with NonNull)
    #[allow(clippy::missing_const_for_fn)]
    fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    /// Exit context manager.
    fn __exit__(
        &self,
        _exc_type: Option<&Bound<'_, PyAny>>,
        _exc_val: Option<&Bound<'_, PyAny>>,
        _exc_tb: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<bool> {
        self.close()?;
        Ok(false) // do not suppress exceptions
    }
}

impl PyPdfDocument {
    pub(crate) fn with_doc<R>(&self, f: impl FnOnce(&Document) -> R) -> PyResult<R> {
        let g = self.inner.lock().map_err(|_| closed_err())?;
        let d = g.as_ref().ok_or_else(closed_err)?;
        let result = f(d);
        drop(g);
        Ok(result)
    }
}
