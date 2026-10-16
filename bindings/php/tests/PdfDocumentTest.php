<?php
declare(strict_types=1);

use PHPUnit\Framework\TestCase;
use BernardLedit\Pdf\PdfDocument;
use BernardLedit\Pdf\PdfiumException;

require_once __DIR__ . '/config.php';

final class PdfDocumentTest extends TestCase
{
    private const FIXTURES = TEST_ROOT_PATH . '/file_types';

    private static function pdf(string $name): string
    {
        return file_get_contents(self::FIXTURES . '/pdf/' . $name);
    }

    public function testConstructFromBytes(): void
    {
        $doc = new PdfDocument(self::pdf('blank.pdf'));
        self::assertSame(10, $doc->getPageCount());
    }

    public function testFromFile(): void
    {
        $doc = new PdfDocument(file_get_contents(self::FIXTURES . '/pdf/blank_1.pdf'));
        self::assertSame(1, $doc->getPageCount());
    }

    public function testConstructFromInvalidBytesThrowsPdfiumException(): void
    {
        $this->expectException(PdfiumException::class);
        new PdfDocument('not a pdf');
    }

    public function testCreateEmptyDocument(): void
    {
        $doc = PdfDocument::create();
        self::assertSame(0, $doc->getPageCount());
    }

    public function testGetPageReturnsPdfPage(): void
    {
        $doc = new PdfDocument(self::pdf('multipage.pdf'));
        $page = $doc->getPage(0);
        self::assertInstanceOf(\BernardLedit\Pdf\PdfPage::class, $page);
        self::assertSame(0, $page->getIndex());
    }

    public function testGetPageOutOfBoundsThrowsPdfiumException(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $this->expectException(PdfiumException::class);
        $doc->getPage(99);
    }

    public function testImportPagesAppends(): void
    {
        $src = new PdfDocument(self::pdf('multipage.pdf'));
        $dst = new PdfDocument(self::pdf('blank_1.pdf'));
        $initial = $dst->getPageCount();

        $dst->importPages($src, [0, 1, 2]);

        self::assertSame($initial + 3, $dst->getPageCount());
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
        $saved = $doc->asBytes();

        self::assertStringStartsWith('%PDF-', $saved);

        $reloaded = new PdfDocument($saved);
        self::assertSame($doc->getPageCount(), $reloaded->getPageCount());
    }

    public function testSaveToFileWritesReloadablePdf(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $path = tempnam(sys_get_temp_dir(), 'bernard_ledit_pdf_');
        self::assertNotFalse($path);

        try {
            file_put_contents($path, $doc->asBytes());
            $bytes = file_get_contents($path);
            self::assertStringStartsWith('%PDF-', $bytes);

            $reloaded = new PdfDocument($bytes);
            self::assertSame(1, $reloaded->getPageCount());
        } finally {
            unlink($path);
        }
    }

    public function testCloseMakesOperationsFail(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $doc->close();

        $this->expectException(\Exception::class);
        $doc->getPageCount();
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

        self::assertSame(1, $doc->getPageCount());
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

        self::assertSame(3, $doc->getPageCount());
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
        $jpeg = $doc->rasterizePageToJpegBytes(0, 85);

        self::assertGreaterThan(0, strlen($jpeg));
        self::assertSame("\xFF\xD8", substr($jpeg, 0, 2));
    }

    public function testRasterizePageOutOfBoundsThrowsPdfiumException(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $this->expectException(PdfiumException::class);
        $doc->rasterizePageToJpegBytes(99, 85);
    }

    public function testRasterizePageOnClosedDocumentThrows(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $doc->close();

        $this->expectException(\Exception::class);
        $doc->rasterizePageToJpegBytes(0, 85);
    }

    public function testAddTextUpdatesTextAndChars(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $char = new \TextChar('A', 'Arial', 12.0, 300, null, null, 0, [0.0, 0.0, 10.0, 10.0]);

        $doc->addText(0, [$char]);

        $page = $doc->getPage(0);
        self::assertSame('A', $page->getText());

        $chars = $page->getChars();
        self::assertCount(1, $chars);
        self::assertInstanceOf(\TextChar::class, $chars[0]);
        self::assertSame('A', (string) $chars[0]);
        $debug = $chars[0]->__debugInfo();
        self::assertSame('Helvetica', $debug['font_name']);
    }
}
