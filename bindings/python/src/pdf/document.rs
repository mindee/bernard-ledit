//! PDF document.
use super::{page::PyPdfPage, text_char::PyTextChar};
use crate::pdf::error::{closed_err, format_pdf_err};
use bernard_ledit::pdf::{Document, TextChar as RustTextChar};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::path::PathBuf;
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
    /// Path to a file.
    Path(PathBuf),
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
            PdfInput::Path(path) => std::fs::read(&path).map_err(|e| {
                pyo3::exceptions::PyIOError::new_err(format!(
                    "Failed to read file at {}: {}",
                    path.display(),
                    e
                ))
            })?,
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

    /// Returns true if the document contains neither text nor objects.
    fn is_blank(&self) -> PyResult<bool> {
        self.with_doc(|d| d.is_blank().map_err(|e| format_pdf_err(&e)))?
    }

    /// Sequence protocol: allows indexing like `pdf[0]` and iteration like `for page in pdf:`
    fn __getitem__(slf: Bound<'_, Self>, index: isize) -> PyResult<PyPdfPage> {
        let len_usize = slf.borrow().__len__()?;
        let len = isize::try_from(len_usize).map_err(|_| {
            pyo3::exceptions::PyIndexError::new_err("Document length exceeds index limits")
        })?;

        let mut idx = index;
        if idx < 0 {
            idx += len;
        }

        if idx < 0 || idx >= len {
            return Err(pyo3::exceptions::PyIndexError::new_err(
                "Page index out of range",
            ));
        }
        let page_idx = u16::try_from(idx).map_err(|_| {
            pyo3::exceptions::PyIndexError::new_err("Page index out of valid range (max 65535)")
        })?;

        Self::get_page(slf, page_idx)
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

    /// Add text to a page.
    #[allow(clippy::needless_pass_by_value)]
    fn add_text(&self, page_idx: u16, chars: Vec<Bound<'_, PyTextChar>>) -> PyResult<()> {
        let chars: Vec<RustTextChar> = chars
            .iter()
            .map(|c| RustTextChar::from(&*c.borrow()))
            .collect();
        self.with_doc_mut(|d| {
            d.add_text(i32::from(page_idx), &chars)
                .map_err(|e| format_pdf_err(&e))
        })??;
        Ok(())
    }

    /// Append a JPEG page to the document.
    fn append_jpeg_page(&self, jpeg_byte: &[u8]) -> PyResult<()> {
        self.with_doc_mut(|d| {
            d.append_jpeg_page(jpeg_byte)
                .map_err(|e| format_pdf_err(&e))
        })??;
        Ok(())
    }

    /// Append a JPEG page to the document.
    #[allow(clippy::needless_pass_by_value)]
    fn append_multiple_jpeg_pages(&self, jpegs: Vec<Vec<u8>>) -> PyResult<()> {
        self.with_doc_mut(|d| {
            d.append_multiple_jpeg_pages(&jpegs.iter().map(Vec::as_slice).collect::<Vec<&[u8]>>())
                .map_err(|e| format_pdf_err(&e))
        })??;
        Ok(())
    }

    /// Checks if the document contains any text.
    fn has_text(&self) -> PyResult<bool> {
        let has_text = self.with_doc(|d| d.has_text().map_err(|e| format_pdf_err(&e)))??;
        Ok(has_text)
    }

    /// Save the document into a file-like object.
    fn save(&self, buf: &Bound<'_, PyAny>) -> PyResult<()> {
        let mut bytes: Vec<u8> = Vec::new();
        self.with_doc(|d| d.save(&mut bytes).map_err(|e| format_pdf_err(&e)))??;
        buf.call_method1("write", (PyBytes::new(buf.py(), &bytes),))?;
        Ok(())
    }

    /// Rasterizes a PDF page and returns the JPEG bytes.
    fn rasterize_page(&self, page_index: u16, quality: u8) -> PyResult<Vec<u8>> {
        self.with_doc(|d| {
            d.rasterize_page(page_index, quality)
                .map_err(|e| format_pdf_err(&e))
        })?
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

    pub(crate) fn with_doc_mut<R>(&self, f: impl FnOnce(&mut Document) -> R) -> PyResult<R> {
        let mut g = self.inner.lock().map_err(|_| closed_err())?;
        let d = g.as_mut().ok_or_else(closed_err)?;
        let result = f(d);
        drop(g);
        Ok(result)
    }
}
