import io

import pytest

from bernard_ledit.image import Image, extract_polygons

FULL = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
TOP_LEFT_QUARTER = [(0.0, 0.0), (0.5, 0.0), (0.5, 0.5), (0.0, 0.5)]


@pytest.fixture
def receipt(test_data_dir):
    return (test_data_dir / "file_types/receipt.jpg").read_bytes()


def test_extract_full_page_polygon_keeps_size(receipt):
    (img,) = extract_polygons(receipt, 0, [FULL])
    assert img.size == Image(receipt).size
    assert img.format == "JPEG"


def test_extract_quarter_polygon_is_half_size(receipt):
    width, height = Image(receipt).size
    (img,) = extract_polygons(receipt, 0, [TOP_LEFT_QUARTER])
    cw, ch = img.size
    assert abs(cw - width / 2) <= 1
    assert abs(ch - height / 2) <= 1


def test_extract_returns_one_image_per_polygon(receipt):
    images = extract_polygons(receipt, 0, [FULL, TOP_LEFT_QUARTER])
    assert len(images) == 2


def test_extract_accepts_file_like(receipt):
    (img,) = extract_polygons(io.BytesIO(receipt), 0, [FULL])
    assert img.size == Image(receipt).size


def test_extract_from_pdf_page_is_jpeg(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()
    (img,) = extract_polygons(data, 0, [FULL])
    width, height = img.size
    assert width > 0 and height > 0
    assert img.format == "JPEG"


def test_extract_image_page_out_of_range(receipt):
    with pytest.raises(Exception):
        extract_polygons(receipt, 1, [FULL])


def test_extract_pdf_page_out_of_range(test_data_dir):
    data = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()
    with pytest.raises(Exception):
        extract_polygons(data, 99, [FULL])


def test_extract_empty_polygon_errors(receipt):
    with pytest.raises(Exception):
        extract_polygons(receipt, 0, [[]])


def test_extract_polygon_outside_page_errors(receipt):
    with pytest.raises(Exception):
        extract_polygons(receipt, 0, [[(2.0, 2.0), (3.0, 3.0)]])


def test_extract_garbage_errors():
    with pytest.raises(Exception):
        extract_polygons(b"not an image", 0, [FULL])
