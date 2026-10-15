import io

import pytest

from bernard_ledit.pdf import PdfDocument, fix_pdf


def test_fix_pdf_returns_input_unchanged_when_marker_is_first():
    data = b"%PDF-1.4 rest of file"
    assert fix_pdf(data) == data


def test_fix_pdf_strips_garbage_bytes_before_marker():
    data = b"junk\r\n%PDF-1.4 rest of file"
    assert fix_pdf(data) == b"%PDF-1.4 rest of file"


def test_fix_pdf_uses_first_marker_when_several_exist():
    data = b"ab%PDF-1 cd%PDF-2"
    assert fix_pdf(data) == b"%PDF-1 cd%PDF-2"


def test_fix_pdf_binary_io_support():
    stream = io.BytesIO(b"junk%PDF-1.5 stream data")
    assert fix_pdf(stream) == b"%PDF-1.5 stream data"


def test_fix_pdf_respects_maximum_offset():
    data = (b"x" * 10) + b"%PDF-1.4"

    assert fix_pdf(data, maximum_offset=11) == b"%PDF-1.4"

    with pytest.raises(Exception, match="PDF couldn't be fixed"):
        fix_pdf(data, maximum_offset=10)


def test_fix_pdf_missing_marker_raises_error():
    with pytest.raises(Exception, match="valid PDF-like object"):
        fix_pdf(b"just some random text without the marker", maximum_offset=500)


def test_fix_pdf_short_and_empty_inputs_raise_error():
    with pytest.raises(Exception):
        fix_pdf(b"")
    with pytest.raises(Exception):
        fix_pdf(b"%PD")


def test_fix_pdf_integration_with_real_document(test_data_dir):
    clean_pdf = (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()

    garbage_data = b"GARBAGE-HEADER\r\n" + clean_pdf
    fixed_bytes = fix_pdf(garbage_data)

    assert fixed_bytes == clean_pdf

    doc = PdfDocument(fixed_bytes)
    assert doc.get_page(0) is not None
