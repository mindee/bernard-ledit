<?php
declare(strict_types=1);

use PHPUnit\Framework\TestCase;
use BernardLedit\Pdf\PdfDocument;

final class PdfPageTest extends TestCase
{
    private const FIXTURES = __DIR__ . '/../../../tests/data/file_types';

    private static function pdf(string $name): string
    {
        return file_get_contents(self::FIXTURES . '/pdf/' . $name);
    }

    public function testIndexMatchesRequestedPage(): void
    {
        $doc = new PdfDocument(self::pdf('multipage.pdf'));
        self::assertSame(2, $doc->getPage(2)->index());
    }

    public function testSizeReturnsPositiveWidthAndHeight(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        [$width, $height] = $doc->getPage(0)->size();

        self::assertGreaterThan(0, $width);
        self::assertGreaterThan(0, $height);
    }

    public function testIsEmptyTrueForBlankPage(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        self::assertTrue($doc->getPage(0)->isEmpty());
    }

    public function testIsEmptyFalseForMultipage(): void
    {
        $doc = new PdfDocument(self::pdf('multipage.pdf'));
        self::assertFalse($doc->getPage(0)->isEmpty());
    }

    public function testTextAndCharsAreEmptyOnBlankPage(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $page = $doc->getPage(0);

        self::assertSame('', $page->text());
        self::assertSame([], $page->chars());
    }

    public function testRenderReturnsBitmap(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $bitmap = $doc->getPage(0)->render();

        self::assertInstanceOf(\PdfBitmap::class, $bitmap);
        self::assertGreaterThan(0, $bitmap->width());
        self::assertGreaterThan(0, $bitmap->height());
    }

    public function testRenderDefaultScaleIsOne(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $page = $doc->getPage(0);

        $defaultScale = $page->render();
        $explicitOne = $page->render(1.0);

        self::assertSame($defaultScale->width(), $explicitOne->width());
        self::assertSame($defaultScale->height(), $explicitOne->height());
    }

    public function testRenderScaleAffectsSize(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $page = $doc->getPage(0);

        $small = $page->render(1.0);
        $large = $page->render(2.0);

        self::assertGreaterThanOrEqual($small->width() * 2 - 1, $large->width());
        self::assertGreaterThanOrEqual($small->height() * 2 - 1, $large->height());
    }

    public function testOperationsOnPageFromClosedDocumentThrow(): void
    {
        $doc = new PdfDocument(self::pdf('blank_1.pdf'));
        $page = $doc->getPage(0);
        $doc->close();

        $this->expectException(\Exception::class);
        $page->size();
    }
}
