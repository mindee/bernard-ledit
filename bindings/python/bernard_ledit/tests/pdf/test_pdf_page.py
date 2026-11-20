import pytest

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


def test_page_dimensions_and_closed_geometry(test_data_dir):
    data = (test_data_dir / "file_types/pdf/blank_1.pdf").read_bytes()
    with PdfDocument(data) as document:
        page = document.get_page(0)
        width, height = page.size
        assert page.render().size == (round(width), round(height))
        assert page.rotation == 0
        assert page.bounding_box == pytest.approx((0, 0, width, height))
        assert page.chars() == []
        assert not page.is_native()
    with pytest.raises(RuntimeError, match="closed"):
        _ = page.size


def test_page_native_character_geometry(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()
    with PdfDocument(data) as document:
        page = document.get_page(0)
        characters = page.chars()
        assert "".join(character.char for character in characters) == page.text()
        assert page.is_native()
        for character in characters:
            bottom, left, top, right = character.bounds
            assert bottom <= top
            assert left <= right
