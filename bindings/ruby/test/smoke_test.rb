# frozen_string_literal: true

# Calls each exported function once through the generated Ruby bindings.

require 'minitest/autorun'
require 'bernard_ledit'

class SmokeTest < Minitest::Test
  FILE_TYPES = File.expand_path('../../../tests/data/file_types', __dir__)
  FULL_PAGE = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]].map do |x, y|
    BernardLedit::Point.new(x: x, y: y)
  end

  def read(name)
    File.binread(File.join(FILE_TYPES, name))
  end

  def test_decodes_and_encodes_images
    jpeg = read('receipt.jpg')
    assert_equal 'JPEG', BernardLedit.guess_format(jpeg)
    image = BernardLedit::Image.new(jpeg)
    assert_operator image.width, :>, 0
    assert_equal 'JPEG', image.format
    assert_equal "\x89PNG".b, image.encode('PNG', 85)[0, 4].b
  end

  def test_compresses_images
    compressed = BernardLedit.compress_image(read('receipt.jpg'), 85, 100, nil)
    assert_operator compressed.width, :<=, 100
  end

  def test_reports_errors_with_their_kind
    error = assert_raises(BernardLedit::BernardLeditError::InvalidArgument) do
      BernardLedit.compress_image(read('receipt.jpg'), 300, nil, nil)
    end
    assert_includes error.message, 'quality must be between 5 and 100, got 300'
    assert_raises(BernardLedit::BernardLeditError::Image) do
      BernardLedit.guess_format(read('receipt.heic'))
    end
  end

  def test_reads_and_splits_pdfs
    pdf = read('pdf/multipage.pdf')
    assert_equal 12, BernardLedit::PdfDocument.new(pdf).page_count
    parts = BernardLedit.extract_from_page_ranges(pdf, [[0], [1, 2]])
    assert_equal [1, 2], parts.map(&:page_count)
    crops = BernardLedit.extract_from_polygons(pdf, 0, [FULL_PAGE], 72.0)
    assert_equal 'JPEG', crops.first.format
  end

  def test_fixes_and_compresses_pdfs
    pdf = read('pdf/multipage.pdf')
    assert_equal '%PDF-', BernardLedit.fix_pdf("junk#{pdf}", 500)[0, 5]
    assert_equal '%PDF-', BernardLedit.compress_pdf(read('pdf/not_blank_image_only.pdf'), 85, 150.0)[0, 5]
  end
end
