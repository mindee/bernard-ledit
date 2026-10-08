<?php

// Stubs for bernard_ledit

namespace BernardLedit\Image {
    class Image {
        public function __construct() {}

        /**
         * Crops a rendered page's image from a given bounding box.
         *
         * @param int $left
         * @param int $top
         * @param int $right
         * @param int $bottom
         * @return \BernardLedit\Image\Image
         * @throws \Exception Returns an error if the cropping operation fails.
         */
        public function crop(int $left, int $top, int $right, int $bottom): \BernardLedit\Image\Image {}

        /**
         * Returns the encoded bytes as a PHP string.
         *
         * @param string $format
         * @param int $quality
         * @param bool $optimize
         * @return string
         * @throws \Exception Returns an error if the encoding operation fails.
         */
        public function encode(string $format, int $quality = 85, bool $optimize = true): string {}

        /**
         * Upper-case format name (`"JPEG"`, `"PNG"`, etc.) or `null`.
         *
         * @return string|null
         * @throws \Exception Returns an error if the format name cannot be converted to a string.
         */
        public function format(): ?string {}

        /**
         * Height.
         *
         * @return int
         */
        public function height(): int {}

        /**
         * Resizes an image using dimensions and a filter.
         *
         * @param int $width
         * @param int $height
         * @param string|null $filter
         * @return \BernardLedit\Image\Image
         * @throws \Exception Returns an error if the resizing operation fails.
         */
        public function resize(int $width, int $height, ?string $filter = null): \BernardLedit\Image\Image {}

        /**
         * `[width, height]`
         *
         * @return array
         */
        public function size(): array {}

        /**
         * Width.
         *
         * @return int
         */
        public function width(): int {}
    }

    /**
     * `BernardLedit\Image\ImageException extends \Exception` Decode and encode failures.
     */
    class ImageException extends \Exception {
        public function __construct() {}
    }

    /**
     * Compresses an image mirroring the reference PIL implementation.
     *
     * @param string $data
     * @param int $quality
     * @param int|null $max_width
     * @param int|null $max_height
     * @return array
     * @throws \Exception Returns an error if the compression operation fails, or if the resulting dimensions
     * @throws \Exception don't fit in a 32-bit integer.
     */
    function compress(string $data, int $quality = 85, ?int $max_width = null, ?int $max_height = null): array {}

    /**
     * `BernardLedit\Image\decode(string $data): Image`
     *
     * @param string $data
     * @return \BernardLedit\Image\Image
     * @throws \Exception Returns an error if the decoding operation fails.
     */
    function decode(string $data): \BernardLedit\Image\Image {}

    /**
     * `BernardLedit\Image\guessFormat(string $data): string`
     *
     * @param string $data
     * @return string
     * @throws \Exception Returns an error if the decoding operation fails.
     */
    function guessFormat(string $data): string {}
}

namespace BernardLedit\Pdf {
    class PdfDocument {
        /**
         * Create a new instance from bytes: `new PdfDocument(string $bytes)`.
         *
         * @param string $data
         * @throws \Exception Returns a PHP Exception if the creation fails.
         */
        public function __construct(string $data) {}

        /**
         * Append text to a page.
         *
         * @param int $page_idx
         * @param array $chars
         * @return void
         * @throws \Exception Returns a PHP Exception if the text append fails.
         */
        public function addText(int $page_idx, array $chars): void {}

        /**
         * Appends a sequence of JPEG pages as pages to the document.
         *
         * @param string $jpeg
         * @return void
         * @throws \Exception Returns a PHP Exception if the append fails.
         */
        public function appendJpegPage(string $jpeg): void {}

        /**
         * Append multiple JPEG pages to the document.
         *
         * @param array $jpegs
         * @return void
         * @throws \Exception Returns a PHP Exception if the append fails.
         */
        public function appendMultipleJpegPages(array $jpegs): void {}

        /**
         * Close the document.
         *
         * @return void
         * @throws \Exception Returns a PHP Exception if the close fails.
         */
        public function close(): void {}

        /**
         * `PdfDocument::create()` empty document (new is reserved).
         *
         * @return \BernardLedit\Pdf\PdfDocument
         * @throws \Exception Returns a PHP Exception if the creation fails.
         */
        public static function create(): \BernardLedit\Pdf\PdfDocument {}

        /**
         * Create an instance from file `PdfDocument::fromFile(string $path)`.
         *
         * @param string $path
         * @return \BernardLedit\Pdf\PdfDocument
         * @throws \Exception Returns a PHP Exception if the creation fails.
         */
        public static function fromFile(string $path): \BernardLedit\Pdf\PdfDocument {}

        /**
         * Validates the index now (opens & drops the page), returns a lightweight handle.
         *
         * @param int $index
         * @return \BernardLedit\Pdf\PdfPage
         * @throws \Exception Returns a PHP Exception if the page cannot be retrieved.
         */
        public function getPage(int $index): \BernardLedit\Pdf\PdfPage {}

        /**
         * Returns true if the document has no content.
         *
         * @return bool
         * @throws \Exception Returns a PHP Exception if the check fails.
         */
        public function hasNoContent(): bool {}

        /**
         * Check if the document has any text.
         *
         * @return bool
         * @throws \Exception Returns a PHP Exception if the check fails.
         */
        public function hasText(): bool {}

        /**
         * Append pages `indexes` of `$src` to `$this`.
         *
         * @param \BernardLedit\Pdf\PdfDocument $src
         * @param array $indexes
         * @return void
         * @throws \Exception Returns a PHP Exception if the import fails.
         */
        public function importPages(\BernardLedit\Pdf\PdfDocument $src, array $indexes): void {}

        /**
         * Returns the page count.
         *
         * @return int
         * @throws \Exception Returns a PHP Exception if the count fails.
         */
        public function pageCount(): int {}

        /**
         * Rasterizes a page from a given index.
         *
         * @param int $index
         * @param int $quality
         * @return string
         * @throws \Exception Returns a PHP Exception if the rasterization fails.
         */
        public function rasterizePage(int $index, int $quality): string {}

        /**
         * Returns the PDF bytes (PHP writes files itself and keeps I/O out of Rust).
         *
         * @return string
         * @throws \Exception Returns a PHP Exception if the save fails.
         */
        public function save(): string {}

        /**
         * Saves to a file.
         *
         * @param string $path
         * @return void
         * @throws \Exception Returns a PHP Exception if the save fails.
         */
        public function saveToFile(string $path): void {}

        /**
         * Retrieves the entire text from a document as a string.
         *
         * @return string
         * @throws \Exception Returns a PHP Exception if the text retrieval fails.
         */
        public function text(): string {}
    }

    class PdfPage {
        public function __construct() {}

        /**
         * Returns the characters on the page.
         *
         * @return array
         * @throws \Exception Returns a PHP Exception if the `PdfDocument` is inaccessible.
         */
        public function chars(): array {}

        /**
         * Index of the page.
         *
         * @return int
         */
        public function index(): int {}

        /**
         * Checks whether the entire document is empty.
         *
         * @return bool
         * @throws \Exception Returns a PHP Exception if the `PdfDocument` is inaccessible.
         */
        public function isEmpty(): bool {}

        /**
         * Renders the page.
         *
         * @param float $scale
         * @return \PhpPdfBitmap
         * @throws \Exception Returns a PHP Exception if the `PdfDocument` is inaccessible.
         */
        public function render(float $scale = 1.0): \PhpPdfBitmap {}

        /**
         * `[width, height]` in points.
         *
         * @return array
         * @throws \Exception Returns a PHP Exception if the `PdfDocument` is closed.
         */
        public function size(): array {}

        /**
         * Returns the text of the page.
         *
         * @return string
         * @throws \Exception Returns a PHP Exception if the `PdfDocument` is inaccessible.
         */
        public function text(): string {}
    }

    class PdfiumException extends \Exception {
        public function __construct() {}
    }
}

namespace {
    class PhpPdfBitmap {
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
        public function height(): int {}

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
         * @throws \Exception Returns a PHP Exception if the conversion fails.
         */
        public function toPngBytes(): string {}

        /**
         * Width in pixels.
         *
         * @return int
         */
        public function width(): int {}
    }

    /**
     * Text character representation.
     */
    class PhpTextChar {
        /**
         * @param string $char
         * @param string $font_name
         * @param float $font_size
         * @param int $font_weight
         * @param array|null $stroke_color
         * @param array|null $fill_color
         * @param int $font_flags
         * @param array|null $bounds
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
