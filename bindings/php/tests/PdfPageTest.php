<?php
declare(strict_types=1);

use PHPUnit\Framework\TestCase;
use BernardLedit\Image\Image;
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

    public function testTextIsEmptyOnBlankPage(): void
    {
        $doc = new PdfDocument(self::loadFileContents('blank_1.pdf'));
        self::assertSame('', $doc->getPage(0)->getText());
    }

    public function testRenderReturnsImage(): void
    {
        $doc = new PdfDocument(self::loadFileContents('blank_1.pdf'));
        $image = $doc->getPage(0)->render();

        self::assertInstanceOf(Image::class, $image);
        [$width, $height] = $image->getSize();
        self::assertGreaterThan(0, $width);
        self::assertGreaterThan(0, $height);
    }

    public function testRenderDefaultScaleIsOne(): void
    {
        $doc = new PdfDocument(self::loadFileContents('blank_1.pdf'));
        $page = $doc->getPage(0);

        $defaultScale = $page->render();
        $explicitOne = $page->render(1.0);

        self::assertSame($defaultScale->getSize(), $explicitOne->getSize());
    }

    public function testRenderScaleAffectsSize(): void
    {
        $doc = new PdfDocument(self::loadFileContents('blank_1.pdf'));
        $page = $doc->getPage(0);

        [$smallWidth, $smallHeight] = $page->render(1.0)->getSize();
        [$largeWidth, $largeHeight] = $page->render(2.0)->getSize();

        self::assertGreaterThanOrEqual($smallWidth * 2 - 1, $largeWidth);
        self::assertGreaterThanOrEqual($smallHeight * 2 - 1, $largeHeight);
    }

    public function testOperationsOnPageFromClosedDocumentThrow(): void
    {
        $doc = new PdfDocument(self::loadFileContents('blank_1.pdf'));
        $page = $doc->getPage(0);
        $doc->close();

        $this->expectException(\Exception::class);
        $page->getText();
    }
}
