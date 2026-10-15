import io

import pytest

from bernard_ledit.pdf import PdfDocument, extract_documents


@pytest.fixture
def multipage(test_data_dir):
    return (test_data_dir / "file_types/pdf/multipage.pdf").read_bytes()


def stars(page) -> int:
    return page.text().replace("\r\n", "").count("*")


def test_extract_groups_page_counts(multipage):
    extracted_documents = extract_documents(multipage, [[0], [1, 2]])
    assert [len(document) for document in extracted_documents] == [1, 2]


def test_extract_keeps_requested_pages_in_order(multipage):
    (doc,) = extract_documents(multipage, [[2, 0]])
    assert stars(doc[0]) == 9
    assert stars(doc[1]) == 1


def test_extract_accepts_file_like(multipage):
    docs = extract_documents(io.BytesIO(multipage), [[0]])
    assert len(docs) == 1


def test_extract_output_roundtrips(multipage):
    (doc,) = extract_documents(multipage, [[0, 1]])
    buf = io.BytesIO()
    doc.save(buf)
    assert buf.getvalue().startswith(b"%PDF-")
    assert len(PdfDocument(buf.getvalue())) == 2


@pytest.mark.parametrize("fixture", ["receipt.jpg", "receipt.png"])
def test_extract_from_image_is_one_page_document(test_data_dir, fixture):
    data = (test_data_dir / "file_types" / fixture).read_bytes()
    (doc,) = extract_documents(data, [[0]])
    assert len(doc) == 1


def test_extract_no_groups_errors(multipage):
    with pytest.raises(Exception):
        extract_documents(multipage, [])


def test_extract_empty_group_errors(multipage):
    with pytest.raises(Exception):
        extract_documents(multipage, [[0], []])


def test_extract_index_out_of_range_errors(multipage):
    with pytest.raises(Exception):
        extract_documents(multipage, [[12]])  # 12 pages, so max valid index is 11


def test_extract_garbage_errors():
    with pytest.raises(Exception):
        extract_documents(b"garbage", [[0]])
