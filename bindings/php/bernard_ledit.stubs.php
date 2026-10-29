<?php

// Stubs for bernard_ledit

namespace BernardLedit\Image {
    class Image {
        /**
         * Decodes an image from its encoded bytes: `new Image(string $data)`.
         *
         * @param string $data
         * @throws \BernardLedit\Image\ImageException if `$data` isn't an identifiable image, or can't be decoded.
         */
        public function __construct(string $data) {}

        /**
         * Crops a rendered page's image from a given bounding box.
         *
         * @param int $left
         * @param int $top
         * @param int $right
         * @param int $bottom
         * @return \BernardLedit\Image\Image
         * @throws \ValueError if the box is empty or extends past the image.
         */
        public function crop(int $left, int $top, int $right, int $bottom): \BernardLedit\Image\Image {}

        /**
         * Returns the encoded bytes as a PHP string.
         *
         * @param string $format
         * @param int $quality
         * @param bool $optimize
         * @return string
         * @throws \ValueError if `$format` isn't a supported output format.
         * @throws \BernardLedit\Image\ImageException if encoding fails, or `$quality` is outside 1-100 for JPEG and PDF.
         */
        public function encode(string $format, int $quality = 85, bool $optimize = true): string {}

        /**
         * Upper-case format name (`"JPEG"`, `"PNG"`, etc.) or `null`.
         *
         * @return string|null
         */
        public function getFormat(): ?string {}

        /**
         * Height.
         *
         * @return int
         */
        public function getHeight(): int {}

        /**
         * Dimensions as `[width, height]`.
         *
         * @return array
         */
        public function getSize(): array {}

        /**
         * Width.
         *
         * @return int
         */
        public function getWidth(): int {}

        /**
         * Resizes an image using dimensions and a filter.
         *
         * @param int $width
         * @param int $height
         * @return \BernardLedit\Image\Image
         * @throws \ValueError if `$width` or `$height` is 0.
         */
        public function resize(int $width, int $height): \BernardLedit\Image\Image {}
    }

    /**
     * `BernardLedit\Image\ImageException extends \Exception`: input that isn't an identifiable or
     * decodable image, and encoding failures.
     */
    class ImageException extends \Exception {
        public function __construct() {}
    }

    /**
     * Compresses an image, as `[string $bytes, int $width, int $height]`.
     *
     * @param string $data
     * @param int $quality
     * @param int|null $max_width
     * @param int|null $max_height
     * @param bool|null $optimize
     * @return array
     * @throws \BernardLedit\Image\ImageException if `$data` isn't an identifiable image, or can't be decoded; if `$quality` is outside 5-100; or if encoding fails.
     */
    function compress(string $data, int $quality = 85, ?int $max_width = null, ?int $max_height = null, ?bool $optimize = null): array {}

    /**
     * `BernardLedit\Image\decode(string $data): Image`
     *
     * @param string $data
     * @return \BernardLedit\Image\Image
     * @throws \BernardLedit\Image\ImageException if `$data` isn't an identifiable image, or can't be decoded.
     */
    function decode(string $data): \BernardLedit\Image\Image {}

    /**
     * Extracts polygons from page `$page_id` of a PDF or an image.
     *
     * @param string $data
     * @param int $page_id
     * @param array $polygons
     * @param float $dpi
     * @return array
     * @throws \ValueError if a point doesn't have exactly two coordinates, a polygon is empty or falls outside the page, or there's no page `$page_id`.
     * @throws \BernardLedit\Image\ImageException if `$data` is neither a PDF nor an identifiable, decodable image.
     * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't be loaded, or can't read or render the PDF.
     */
    function extractFromPolygons(string $data, int $page_id, array $polygons, float $dpi = 72.0): array {}

    /**
     * `BernardLedit\Image\guessFormat(string $data): string`
     *
     * @param string $data
     * @return string
     * @throws \BernardLedit\Image\ImageException if `$data` isn't an identifiable image.
     */
    function guessFormat(string $data): string {}
}

namespace BernardLedit\Pdf {
    class PdfDocument {
        /**
         * Create a new instance from bytes.
         *
         * @param string $data
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't be loaded, or `$data` isn't a PDF it can read.
         */
        public function __construct(string $data) {}

        /**
         * Append text to a page.
         *
         * @param int $page_idx
         * @param array $chars
         * @return void
         * @throws \ValueError if there's no page `$page_idx`.
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't add the text.
         * @throws \Exception if the document is closed.
         */
        public function addText(int $page_idx, array $chars): void {}

        /**
         * Appends a JPEG as a page of the document.
         *
         * @param string $jpeg
         * @return void
         * @throws \BernardLedit\Image\ImageException if `$jpeg` is invalid, truncated or in an unsupported color space.
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't append the page.
         * @throws \Exception if the document is closed.
         */
        public function appendJpegPage(string $jpeg): void {}

        /**
         * Append multiple JPEG pages to the document.
         *
         * @param array $jpegs
         * @return void
         * @throws \BernardLedit\Image\ImageException if one of `$jpegs` is invalid, truncated or in an unsupported color space.
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't append a page.
         * @throws \Exception if the document is closed.
         */
        public function appendMultipleJpegPages(array $jpegs): void {}

        /**
         * Returns the PDF bytes (PHP writes files itself and keeps I/O out of Rust).
         *
         * @return string
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't save the document.
         * @throws \Exception if the document is closed.
         */
        public function asBytes(): string {}

        /**
         * Close the document. Closing it again does nothing.
         *
         * @return void
         */
        public function close(): void {}

        /**
         * Recompresses pages that have no text layer (scans) as JPEG.
         * Returns `null` when there is nothing to gain (e.g. text-only documents).
         *
         * @param int $quality
         * @param bool $optimize
         * @param float $dpi
         * @return \BernardLedit\Pdf\PdfDocument|null
         * @throws \BernardLedit\Image\ImageException if `$quality` is outside 5-100, or JPEG encoding fails.
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't render a page, or build the new document.
         * @throws \Exception if the document is closed.
         */
        public function compress(int $quality = 85, bool $optimize = true, float $dpi = 150.0): ?\BernardLedit\Pdf\PdfDocument {}

        /**
         * `PdfDocument::create()` empty document (new is reserved).
         *
         * @return \BernardLedit\Pdf\PdfDocument
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't be loaded, or can't create a document.
         */
        public static function create(): \BernardLedit\Pdf\PdfDocument {}

        /**
         * Return the specified page.
         *
         * @param int $index
         * @return \BernardLedit\Pdf\PdfPage
         * @throws \ValueError if there's no page `$index`.
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't load the page.
         * @throws \Exception if the document is closed.
         */
        public function getPage(int $index): \BernardLedit\Pdf\PdfPage {}

        /**
         * Returns the page count.
         *
         * @return int
         * @throws \ValueError if the document has more than 65535 pages.
         * @throws \Exception if the document is closed.
         */
        public function getPageCount(): int {}

        /**
         * Retrieves the entire text from a document as a string.
         *
         * @return string
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't read a page's text.
         * @throws \Exception if the document is closed.
         */
        public function getText(): string {}

        /**
         * Returns true if the document has no content.
         *
         * @return bool
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't read a page.
         * @throws \Exception if the document is closed.
         */
        public function hasNoContent(): bool {}

        /**
         * Check if the document has any text.
         *
         * @return bool
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't read a page.
         * @throws \Exception if the document is closed.
         */
        public function hasText(): bool {}

        /**
         * Append pages `indexes` of `$src` to `$this`.
         *
         * @param \BernardLedit\Pdf\PdfDocument $src
         * @param array $indexes
         * @return void
         * @throws \ValueError if `$src` is this document, or has no page at one of `$indexes`.
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't copy the pages.
         * @throws \Exception if either document is closed.
         */
        public function importPages(\BernardLedit\Pdf\PdfDocument $src, array $indexes): void {}

        /**
         * Rasterizes a page from a given index as a JPEG.
         *
         * @param int $index
         * @param int $quality
         * @return string
         * @throws \ValueError if there's no page `$index`.
         * @throws \BernardLedit\Image\ImageException if `$quality` is above 100, or JPEG encoding fails.
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't render the page.
         * @throws \Exception if the document is closed.
         */
        public function rasterizePageToJpegBytes(int $index, int $quality): string {}
    }

    class PdfPage {
        public function __construct() {}

        /**
         * Returns the characters on the page.
         *
         * @return array
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't read the page's characters.
         * @throws \Exception if the document is closed.
         */
        public function getChars(): array {}

        /**
         * Index of the page.
         *
         * @return int
         */
        public function getIndex(): int {}

        /**
         * `[width, height]` in points.
         *
         * @return array
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't load the page.
         * @throws \Exception if the document is closed.
         */
        public function getSize(): array {}

        /**
         * Returns the text of the page.
         *
         * @return string
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't read the page's text.
         * @throws \Exception if the document is closed.
         */
        public function getText(): string {}

        /**
         * Checks whether the page is empty.
         *
         * @return bool
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't read the page.
         * @throws \Exception if the document is closed.
         */
        public function isEmpty(): bool {}

        /**
         * Renders the page.
         *
         * @param float $scale
         * @return \PdfBitmap
         * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't render the page.
         * @throws \Exception if the document is closed.
         */
        public function renderToBitmap(float $scale = 1.0): \PdfBitmap {}
    }

    /**
     * `BernardLedit\Pdf\PdfiumException extends \Exception`: `PDFium` can't be loaded, or fails to
     * read, render or write a document.
     */
    class PdfiumException extends \Exception {
        public function __construct() {}
    }

    /**
     * Keeps or removes pages. Negative indexes count from the end.
     *
     * @param string $data
     * @param string $operation
     * @param int $on_min_pages
     * @param array $page_indexes
     * @return array
     * @throws \ValueError if `$operation` isn't `KEEP_ONLY` or `REMOVE`, the PDF has no pages, or no pages would be left.
     * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't be loaded, `$data` isn't a PDF it can read, or it can't build the new document.
     */
    function applyPageOptions(string $data, string $operation, int $on_min_pages, array $page_indexes): array {}

    /**
     * Compresses PDF bytes. Returns the input unchanged if it can't be made smaller.
     *
     * @param string $data
     * @param int $quality
     * @param bool|null $optimize
     * @param float|null $dpi
     * @return string
     * @throws \BernardLedit\Image\ImageException if `$quality` is outside 5-100, or JPEG encoding fails.
     * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't be loaded, `$data` isn't a PDF it can read, or it can't render a page or build the new document.
     */
    function compress(string $data, int $quality = 85, ?bool $optimize = null, ?float $dpi = null): string {}

    /**
     * Builds one new document per group of 0-indexed page numbers.
     * `data` is a PDF or an image (treated as a one-page document).
     *
     * @param string $data
     * @param array $page_indexes
     * @return array
     * @throws \ValueError if `$page_indexes` or one of its groups is empty, or there's no page at one of the indexes.
     * @throws \BernardLedit\Image\ImageException if `$data` is neither a PDF nor an identifiable, decodable image.
     * @throws \BernardLedit\Pdf\PdfiumException if `PDFium` can't be loaded, `$data` is a PDF it can't read, or it can't build the documents.
     */
    function extractFromPageRanges(string $data, array $page_indexes): array {}

    /**
     * Drops every byte before the first `%PDF-` marker.
     *
     * @param string $data
     * @param int $maximum_offset
     * @return string
     * @throws \ValueError if `$maximum_offset` is negative, or there's no marker before it.
     */
    function fixPdf(string $data, int $maximum_offset = 500): string {}
}

namespace {
    class PdfBitmap {
        public function __construct() {}

        /**
         * Debug info.
         *
         * @return array
         */
        public function __debugInfo(): array {}

        /**
         * String rep.
         *
         * @return string
         */
        public function __toString(): string {}

        /**
         * Height in pixels.
         *
         * @return int
         */
        public function getHeight(): int {}

        /**
         * Width in pixels.
         *
         * @return int
         */
        public function getWidth(): int {}

        /**
         * Raw RGBA8 pixel buffer (row-major, 4 bytes per pixel).
         *
         * @return string
         */
        public function toBytes(): string {}

        /**
         * Encode the bitmap as a PNG byte stream.
         *
         * @return string
         * @throws \BernardLedit\Image\ImageException if PNG encoding fails.
         */
        public function toPngBytes(): string {}
    }

    /**
     * Text character representation.
     */
    class TextChar {
        /**
         * Create a new text character.
         *
         * @param string $char
         * @param string $font_name
         * @param float $font_size
         * @param int $font_weight
         * @param array|null $stroke_color
         * @param array|null $fill_color
         * @param int $font_flags
         * @param array|null $bounds
         * @throws \ValueError if `$stroke_color`, `$fill_color` or `$bounds` is given with a length other than 4.
         */
        public function __construct(string $char, string $font_name, float $font_size, int $font_weight, ?array $stroke_color, ?array $fill_color, int $font_flags, ?array $bounds = null) {}

        /**
         * Default debug info.
         *
         * @return array
         */
        public function __debugInfo(): array {}

        /**
         * String rep.
         *
         * @return string
         */
        public function __toString(): string {}
    }
}
