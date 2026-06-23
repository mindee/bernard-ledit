from bernard_ledit.pdf import (
    PageSize,
    PdfBitmap,
    PdfDocument,
)


def test_page_get_size(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    size = doc.get_page(0).get_size()
    assert isinstance(size, PageSize)
    assert size.width > 0
    assert size.height > 0


def test_page_is_empty(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    assert doc.get_page(0).is_empty() is True


def test_page_is_not_empty_for_multipage(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()
    doc = PdfDocument(data)
    assert doc.get_page(0).is_empty() is False


def test_page_render_returns_bitmap(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    bitmap = doc.get_page(0).render()
    assert isinstance(bitmap, PdfBitmap)
    assert bitmap.width > 0
    assert bitmap.height > 0


def test_page_render_scale_affects_size(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    small = doc.get_page(0).render(scale=1.0)
    large = doc.get_page(0).render(scale=2.0)
    assert large.width >= small.width * 2 - 1
    assert large.height >= small.height * 2 - 1


def test_page_repr(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    assert repr(doc.get_page(0)) == "PdfPage(index=0)"
