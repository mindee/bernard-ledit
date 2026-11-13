from bernard_ledit.image import Image
from bernard_ledit.pdf import PdfDocument


def test_page_render_returns_image(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    image = doc.get_page(0).render()
    assert isinstance(image, Image)
    width, height = image.size
    assert width > 0
    assert height > 0


def test_page_render_scale_affects_size(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    small = doc.get_page(0).render(scale=1.0).size
    large = doc.get_page(0).render(scale=2.0).size
    assert large[0] >= small[0] * 2 - 1
    assert large[1] >= small[1] * 2 - 1


def test_page_text(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    page = doc.get_page(0)
    assert page.text() == ""


def test_page_repr(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    doc = PdfDocument(data)
    assert repr(doc.get_page(0)) == "PdfPage(index=0)"
