import io

import pytest

import bernard_ledit.image as bl_image
import bernard_ledit.pdf as bl_pdf

ImageError = bl_image.ImageError
ExtractionError = bl_pdf.PdfExtractionError


def test_extract_no_groups_errors(multipage):
    with pytest.raises(ExtractionError):
        bl_pdf.extract_from_page_ranges(multipage, [])


def test_extract_empty_group_errors(multipage):
    with pytest.raises(ExtractionError):
        bl_pdf.extract_from_page_ranges(multipage, [[0], []])


def test_extract_index_out_of_range_errors(multipage):
    with pytest.raises(IndexError):
        bl_pdf.extract_from_page_ranges(multipage, [[12]])


def test_extract_garbage_errors():
    # Not a PDF, so it falls through to image decoding.
    with pytest.raises(ImageError):
        bl_pdf.extract_from_page_ranges(b"garbage", [[0]])


@pytest.fixture
def multipage(test_data_dir):
    return (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()


def stars(page) -> int:
    return page.text().replace("\r\n", "").count("*")


def test_extract_groups_page_counts(multipage):
    extracted_documents = bl_pdf.extract_from_page_ranges(multipage, [[0], [1, 2]])
    assert [len(document) for document in extracted_documents] == [1, 2]


def test_extract_keeps_requested_pages_in_order(multipage):
    (doc,) = bl_pdf.extract_from_page_ranges(multipage, [[2, 0]])
    assert stars(doc[0]) == 9
    assert stars(doc[1]) == 1


def test_extract_accepts_file_like(multipage):
    docs = bl_pdf.extract_from_page_ranges(io.BytesIO(multipage), [[0]])
    assert len(docs) == 1


def test_extract_output_roundtrips(multipage):
    (doc,) = bl_pdf.extract_from_page_ranges(multipage, [[0, 1]])
    buf = io.BytesIO()
    doc.save(buf)
    assert buf.getvalue().startswith(b"%PDF-")
    assert len(bl_pdf.PdfDocument(buf.getvalue())) == 2


@pytest.mark.parametrize("fixture", ["receipt.jpg", "receipt.png"])
def test_extract_from_image_is_one_page_document(test_data_dir, fixture):
    data = (test_data_dir / "file_types" / fixture).read_bytes()
    (doc,) = bl_pdf.extract_from_page_ranges(data, [[0]])
    assert len(doc) == 1


def test_extract_truncated_pdf_raises_pdfium_error():
    # Has the %PDF- signature, so it is routed to pdfium, not the image decoder.
    # Replaces the weak Rust test that accepts both Ok and Err.
    with pytest.raises(bl_pdf.PdfiumError):
        bl_pdf.extract_from_page_ranges(b"%PDF-1.5\n...", [[0]])


def test_extract_empty_bytes_errors():
    with pytest.raises(ImageError):
        bl_pdf.extract_from_page_ranges(b"", [[0]])


@pytest.mark.parametrize("bad", [-1, 70000])
def test_extract_index_outside_u16_raises_overflow(multipage, bad):
    with pytest.raises(OverflowError):
        bl_pdf.extract_from_page_ranges(multipage, [[bad]])


def test_extract_each_group_has_its_own_content(multipage):
    group_a, group_b = bl_pdf.extract_from_page_ranges(multipage, [[0, 1], [2, 3]])
    assert [stars(group_a[i]) for i in range(2)] == [1, 4]
    assert [stars(group_b[i]) for i in range(2)] == [9, 16]


def test_extract_documents_are_independent(multipage):
    group_a, group_b = bl_pdf.extract_from_page_ranges(multipage, [[0], [0]])
    group_a.close()
    assert len(group_b) == 1  # closing one must not affect the other


def test_extract_does_not_consume_or_modify_source(multipage):
    bl_pdf.extract_from_page_ranges(multipage, [[0]])
    assert len(bl_pdf.PdfDocument(multipage)) == 12


def test_extract_preserves_text_layer(multipage):
    (doc,) = bl_pdf.extract_from_page_ranges(multipage, [[3]])
    assert doc.has_text()


def test_extract_from_image_page_size_matches_pixels(test_data_dir):
    data = (test_data_dir / "file_types/receipt.jpg").read_bytes()
    width, height = bl_image.Image(data).size
    (doc,) = bl_pdf.extract_from_page_ranges(data, [[0]])
    size = doc[0].get_size()
    assert (size.width, size.height) == (width, height)


def test_extract_image_with_page_index_1_errors(test_data_dir):
    data = (test_data_dir / "file_types/receipt.jpg").read_bytes()
    with pytest.raises(IndexError):
        bl_pdf.extract_from_page_ranges(data, [[1]])
