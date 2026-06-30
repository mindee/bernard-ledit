import io

import pytest

from bernard_ledit.pdf import (
    PdfDocument,
    PdfiumError,
    PdfPage,
    TextChar,
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


def test_add_text_updates_text_and_chars(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    doc.add_text(
        0,
        [
            TextChar(
                "A",
                "Arial",
                12.0,
                300,
                bounds=(0.0, 0.0, 10.0, 10.0),
            )
        ],
    )

    page = doc.get_page(0)
    assert page.text() == "A"
    chars = page.chars()
    assert len(chars) == 1
    assert chars[0].char == "A"
    assert chars[0].font_name == "Helvetica"


def test_append_jpeg_page(test_data_dir):
    doc = PdfDocument.new()
    jpeg_data = (test_data_dir / "file_types/receipt.jpg").read_bytes()
    doc.append_jpeg_page(jpeg_data)
    assert len(doc) == 1


def test_append_jpeg_page_invalid_data():
    doc = PdfDocument.new()
    with pytest.raises(PdfiumError):
        doc.append_jpeg_page(b"not a valid jpeg")


def test_append_multiple_jpeg_pages(test_data_dir):
    doc = PdfDocument.new()
    jpeg_data = (test_data_dir / "file_types/receipt.jpg").read_bytes()
    doc.append_multiple_jpeg_pages([jpeg_data, jpeg_data, jpeg_data])
    assert len(doc) == 3


def test_has_text_true(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()
    doc = PdfDocument(data)
    assert doc.has_text() is True


def test_has_text_false(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank.pdf").read_bytes()
    doc = PdfDocument(data)
    assert doc.has_text() is False


def test_rasterize_page_returns_jpeg_bytes(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)

    jpeg_bytes = doc.rasterize_page(0, 85)

    assert isinstance(jpeg_bytes, bytes)
    assert len(jpeg_bytes) > 0
    assert jpeg_bytes.startswith(b"\xff\xd8")


def test_rasterize_page_out_of_bounds_raises(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)

    with pytest.raises(PdfiumError):
        doc.rasterize_page(99, 85)


def test_rasterize_page_fails_on_closed_doc(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    doc.close()

    with pytest.raises(RuntimeError):
        doc.rasterize_page(0, 85)


def test_document_getitem_returns_page(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()
    doc = PdfDocument(data)

    page = doc[0]
    assert isinstance(page, PdfPage)


def test_document_getitem_negative_index(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()
    doc = PdfDocument(data)
    last_page = doc[-1]
    assert isinstance(last_page, PdfPage)


def test_document_getitem_out_of_bounds_raises(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    with pytest.raises(IndexError, match="Page index out of range"):
        _ = doc[1]

    with pytest.raises(IndexError, match="Page index out of range"):
        _ = doc[-2]


def test_document_is_iterable(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()
    doc = PdfDocument(data)
    pages = [page for page in doc]

    assert len(pages) == len(doc)
    assert all(isinstance(page, PdfPage) for page in pages)


def test_document_iteration_empty():
    doc = PdfDocument.new()
    pages = [page for page in doc]
    assert len(pages) == 0


def test_document_from_str_path(test_data_dir):
    path_str = str(test_data_dir / "file_types/pdf/blank.pdf")
    doc = PdfDocument(path_str)
    assert len(doc) == 10


def test_document_from_pathlib_path(test_data_dir):
    path_obj = test_data_dir / "file_types/pdf/blank_1.pdf"
    doc = PdfDocument(path_obj)
    assert len(doc) == 1


def test_document_from_nonexistent_path():
    with pytest.raises(OSError, match="Failed to read file"):
        PdfDocument("this_file_does_not_exist_xyz.pdf")


def test_document_is_blank_true():
    doc = PdfDocument.new()
    assert doc.is_blank() is True


def test_document_is_blank_false(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()
    doc = PdfDocument(data)
    assert doc.is_blank() is False
