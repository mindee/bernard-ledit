<?php
// Appended to the generated stubs by generate.sh: see src/exceptions.rs.

namespace BernardLedit\Image {
    /**
     * Input that isn't an identifiable or decodable image, and encoding failures.
     */
    class ImageException extends \Exception {}
}

namespace BernardLedit\Pdf {
    /**
     * `PDFium` can't be loaded, or fails to read, render or write a document.
     */
    class PdfiumException extends \Exception {}
}
