from bernard_ledit.pdf import (
    PdfBitmap,
    PdfDocument,
)


def test_page_render_returns_bitmap(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    bitmap = doc.get_page(0).render_to_bitmap()
    assert isinstance(bitmap, PdfBitmap)
    assert bitmap.width > 0
    assert bitmap.height > 0


def test_page_render_scale_affects_size(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    small = doc.get_page(0).render_to_bitmap(scale=1.0)
    large = doc.get_page(0).render_to_bitmap(scale=2.0)
    assert large.width >= small.width * 2 - 1
    assert large.height >= small.height * 2 - 1


def test_page_text(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    page = doc.get_page(0)
    assert page.text() == ""


def test_page_repr(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    assert repr(doc.get_page(0)) == "PdfPage(index=0)"
