<?php
declare(strict_types=1);

use PHPUnit\Framework\TestCase;
use BernardLedit\Pdf\PdfDocument;

require_once __DIR__ . '/config.php';

final class PdfPageTest extends TestCase
{
    private const FIXTURES = TEST_ROOT_PATH . '/file_types';

    private static function loadFileContents(string $name): string
    {
        return file_get_contents(self::FIXTURES . '/pdf/' . $name);
    }

    public function testIndexMatchesRequestedPage(): void
    {
        $doc = new PdfDocument(self::loadFileContents('multipage.pdf'));
        self::assertSame(2, $doc->getPage(2)->getIndex());
    }

    public function testSizeReturnsPositiveWidthAndHeight(): void
    {
        $doc = new PdfDocument(self::loadFileContents('blank_1.pdf'));
        [$width, $height] = $doc->getPage(0)->getSize();

        self::assertGreaterThan(0, $width);
        self::assertGreaterThan(0, $height);
    }

    public function testIsEmptyTrueForBlankPage(): void
    {
        $doc = new PdfDocument(self::loadFileContents('blank_1.pdf'));
        self::assertTrue($doc->getPage(0)->isEmpty());
    }

    public function testIsEmptyFalseForMultipage(): void
    {
        $doc = new PdfDocument(self::loadFileContents('multipage.pdf'));
        self::assertFalse($doc->getPage(0)->isEmpty());
    }

    public function testTextAndCharsAreEmptyOnBlankPage(): void
    {
        $doc = new PdfDocument(self::loadFileContents('blank_1.pdf'));
        $page = $doc->getPage(0);

        self::assertSame('', $page->getText());
        self::assertSame([], $page->getChars());
    }

    public function testRenderReturnsBitmap(): void
    {
        $doc = new PdfDocument(self::loadFileContents('blank_1.pdf'));
        $bitmap = $doc->getPage(0)->renderToBitmap();

        self::assertInstanceOf(\BernardLedit\Pdf\PdfBitmap::class, $bitmap);
        self::assertGreaterThan(0, $bitmap->getWidth());
        self::assertGreaterThan(0, $bitmap->getHeight());
    }

    public function testRenderDefaultScaleIsOne(): void
    {
        $doc = new PdfDocument(self::loadFileContents('blank_1.pdf'));
        $page = $doc->getPage(0);

        $defaultScale = $page->renderToBitmap();
        $explicitOne = $page->renderToBitmap(1.0);

        self::assertSame($defaultScale->getWidth(), $explicitOne->getWidth());
        self::assertSame($defaultScale->getHeight(), $explicitOne->getHeight());
    }

    public function testRenderScaleAffectsSize(): void
    {
        $doc = new PdfDocument(self::loadFileContents('blank_1.pdf'));
        $page = $doc->getPage(0);

        $small = $page->renderToBitmap(1.0);
        $large = $page->renderToBitmap(2.0);

        self::assertGreaterThanOrEqual($small->getWidth() * 2 - 1, $large->getWidth());
        self::assertGreaterThanOrEqual($small->getHeight() * 2 - 1, $large->getHeight());
    }

    public function testOperationsOnPageFromClosedDocumentThrow(): void
    {
        $doc = new PdfDocument(self::loadFileContents('blank_1.pdf'));
        $page = $doc->getPage(0);
        $doc->close();

        $this->expectException(\Exception::class);
        $page->getSize();
    }
}
