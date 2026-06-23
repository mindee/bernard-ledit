import io

import pytest

from bernard_ledit.pdf import (
    PdfDocument,
    PdfiumError,
    PdfPage,
)


def test_document_from_bytes(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank.pdf").read_bytes()
    doc = PdfDocument(data)
    assert len(doc) == 10


def test_document_from_file_like(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(io.BytesIO(data))
    assert len(doc) == 1


def test_document_from_invalid_bytes():
    with pytest.raises(PdfiumError):
        PdfDocument(b"not a pdf")


def test_document_new_creates_empty():
    doc = PdfDocument.new()
    assert len(doc) == 0


def test_document_repr(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    assert repr(doc) == "PdfDocument(1 pages)"
    doc.close()
    assert repr(doc) == "PdfDocument(closed)"


def test_get_page_returns_page(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()
    doc = PdfDocument(data)
    page = doc.get_page(0)
    assert isinstance(page, PdfPage)


def test_get_page_out_of_bounds_raises(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    with pytest.raises(PdfiumError):
        doc.get_page(99)


def test_import_pages_appends(test_data_dir):
    src_data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()
    dst_data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    src = PdfDocument(src_data)
    dst = PdfDocument(dst_data)
    initial = len(dst)
    dst.import_pages(src, [0, 1, 2])
    assert len(dst) == initial + 3


def test_import_pages_out_of_bounds_raises(test_data_dir):
    src_data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    dst_data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    src = PdfDocument(src_data)
    dst = PdfDocument(dst_data)
    with pytest.raises(IndexError):
        dst.import_pages(src, [100])


def test_import_pages_from_self_raises(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    with pytest.raises(ValueError):
        doc.import_pages(doc, [0])


def test_save_writes_reloadable_pdf(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    buf = io.BytesIO()
    doc.save(buf)
    saved = buf.getvalue()
    assert saved.startswith(b"%PDF-")
    reloaded = PdfDocument(saved)
    assert len(reloaded) == len(doc)


def test_close_makes_operations_fail(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    doc.close()
    with pytest.raises(RuntimeError):
        _ = len(doc)


def test_close_is_idempotent(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    doc.close()
    doc.close()


def test_context_manager_closes(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    with PdfDocument(data) as doc:
        assert len(doc) == 1
    with pytest.raises(RuntimeError):
        _ = len(doc)


def test_context_manager_does_not_suppress_exceptions(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    with pytest.raises(ZeroDivisionError):
        with PdfDocument(data):
            _ = 1 / 0
