<?php
declare(strict_types=1);

use PHPUnit\Framework\TestCase;
use BernardLedit\Pdf\PdfDocument;
use BernardLedit\Pdf\PdfiumException;

final class PdfDocumentTest extends TestCase
{
    private const FIXTURES = __DIR__ . '/../../../tests/data/file_types';

    private static function pdf(string $name): string
    {
        return file_get_contents(self::FIXTURES . '/pdf/' . $name);
    }

    public function testConstructFromBytes(): void
    {
        $doc = new PdfDocument(self::pdf('blank.pdf'));
        self::assertSame(10, $doc->pageCount());
    }

    public function testFromFile(): void
    {
        $doc = PdfDocument::fromFile(self::FIXTURES . '/pdf/blank_1.pdf');
        self::assertSame(1, $doc->pageCount());
    }

    public function testFromFileNonexistentThrows(): void
    {
        // PdfDocument::fromFile() wraps the underlying std::fs::read() I/O error and maps it
        // through PhpException::default(), i.e. a plain \Exception (no dedicated IO exception
        // class exists in this binding yet).
        $this->expectException(\Exception::class);
        PdfDocument::fromFile('/nonexistent/path/does-not-exist.pdf');
    }

    public function testConstructFromInvalidBytesThrowsPdfiumException(): void
    {
        $this->expectException(PdfiumException::class);
        new PdfDocument('not a pdf');
    }

    public function testCreateEmptyDocument(): void
    {
        $doc = PdfDocument::create();
        self::assertSame(0, $doc->pageCount());
    }

    public function testGetPageReturnsPdfPage(): void
    {
        $doc = new PdfDocument(self::pdf('multipage.pdf'));
        $page = $doc->getPage(0);
        self::assertInstanceOf(\BernardLedit\Pdf\PdfPage::class, $page);
        self::assertSame(0, $page->index());
    }

    public function testGetPageOutOfBoundsThrowsValueError(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        // PageIndexOutOfBounds / PageCountOutOfBounds are mapped to \ValueError (PHP has no
        // IndexError); see bindings/php/src/pdf/error.rs.
        $this->expectException(\ValueError::class);
        $doc->getPage(99);
    }

    public function testImportPagesAppends(): void
    {
        $src = new PdfDocument(self::pdf('multipage.pdf'));
        $dst = new PdfDocument(self::pdf('blank_1.pdf'));
        $initial = $dst->pageCount();

        $dst->importPages($src, [0, 1, 2]);

        self::assertSame($initial + 3, $dst->pageCount());
    }

    public function testImportPagesOutOfBoundsThrowsValueError(): void
    {
        $src = new PdfDocument(self::pdf('blank_1.pdf'));
        $dst = new PdfDocument(self::pdf('blank_1.pdf'));

        $this->expectException(\ValueError::class);
        $dst->importPages($src, [100]);
    }

    public function testImportPagesFromSelfThrowsValueError(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $this->expectException(\ValueError::class);
        $doc->importPages($doc, [0]);
    }

    public function testSaveReturnsReloadablePdfBytes(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $saved = $doc->save();

        self::assertStringStartsWith('%PDF-', $saved);

        $reloaded = new PdfDocument($saved);
        self::assertSame($doc->pageCount(), $reloaded->pageCount());
    }

    public function testSaveToFileWritesReloadablePdf(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $path = tempnam(sys_get_temp_dir(), 'bernard_ledit_pdf_');
        self::assertNotFalse($path);

        try {
            $doc->saveToFile($path);
            $bytes = file_get_contents($path);
            self::assertStringStartsWith('%PDF-', $bytes);

            $reloaded = new PdfDocument($bytes);
            self::assertSame(1, $reloaded->pageCount());
        } finally {
            unlink($path);
        }
    }

    public function testCloseMakesOperationsFail(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $doc->close();

        $this->expectException(\Exception::class);
        $doc->pageCount();
    }

    public function testCloseIsIdempotent(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $doc->close();
        $doc->close();
        self::assertTrue(true); // No exception = pass.
    }

    public function testAppendJpegPage(): void
    {
        $doc = PdfDocument::create();
        $jpeg = file_get_contents(self::FIXTURES . '/receipt.jpg');

        $doc->appendJpegPage($jpeg);

        self::assertSame(1, $doc->pageCount());
    }

    public function testAppendJpegPageInvalidDataThrows(): void
    {
        $doc = PdfDocument::create();
        $this->expectException(PdfiumException::class);
        $doc->appendJpegPage('not a valid jpeg');
    }

    public function testAppendMultipleJpegPages(): void
    {
        $doc = PdfDocument::create();
        $jpeg = file_get_contents(self::FIXTURES . '/receipt.jpg');

        $doc->appendMultipleJpegPages([$jpeg, $jpeg, $jpeg]);

        self::assertSame(3, $doc->pageCount());
    }

    public function testHasTextTrue(): void
    {
        $doc = new PdfDocument(self::pdf('multipage.pdf'));
        self::assertTrue($doc->hasText());
    }

    public function testHasTextFalse(): void
    {
        $doc = new PdfDocument(self::pdf('blank.pdf'));
        self::assertFalse($doc->hasText());
    }

    public function testHasNoContentTrueForEmptyDocument(): void
    {
        $doc = PdfDocument::create();
        self::assertTrue($doc->hasNoContent());
    }

    public function testHasNoContentFalseForMultipage(): void
    {
        $doc = new PdfDocument(self::pdf('multipage.pdf'));
        self::assertFalse($doc->hasNoContent());
    }

    public function testRasterizePageReturnsJpegBytes(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $jpeg = $doc->rasterizePage(0, 85);

        self::assertGreaterThan(0, strlen($jpeg));
        self::assertSame("\xFF\xD8", substr($jpeg, 0, 2));
    }

    public function testRasterizePageOutOfBoundsThrowsValueError(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $this->expectException(\ValueError::class);
        $doc->rasterizePage(99, 85);
    }

    public function testRasterizePageOnClosedDocumentThrows(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $doc->close();

        $this->expectException(\Exception::class);
        $doc->rasterizePage(0, 85);
    }

    public function testAddTextUpdatesTextAndChars(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $char = new \PhpTextChar('A', 'Arial', 12.0, 300, null, null, 0, [0.0, 0.0, 10.0, 10.0]);

        $doc->addText(0, [$char]);

        $page = $doc->getPage(0);
        self::assertSame('A', $page->text());

        $chars = $page->chars();
        self::assertCount(1, $chars);
        self::assertInstanceOf(\PhpTextChar::class, $chars[0]);
        self::assertSame('A', (string) $chars[0]);
        // NOTE: PhpTextChar exposes no typed getters (no width/font_name/font_size accessor
        // methods) — only __toString() (the character itself) and __debugInfo() (stringified
        // debug fields). This is a real feature gap vs. the Python bindings' PyTextChar, which
        // exposes every field via #[pyo3(get)]. See the chars()-returns-structured-data finding
        // in local_test/php-bindings-review.md. We can only assert on the debug-info string here.
        $debug = $chars[0]->__debugInfo();
        self::assertSame('Helvetica', $debug['font_name']);
    }
}
