<?php
declare(strict_types=1);

use PHPUnit\Framework\TestCase;
use BernardLedit\Pdf\PdfDocument;

final class PdfBitmapTest extends TestCase
{
    private const FIXTURES = __DIR__ . '/../../../tests/data/file_types';

    private static function renderedBitmap(): \PdfBitmap
    {
        $doc = new PdfDocument(file_get_contents(self::FIXTURES . '/pdf/blank_1.pdf'));
        return $doc->getPage(0)->render();
    }

    public function testToBytesMatchesDimensions(): void
    {
        $bitmap = self::renderedBitmap();
        $raw = $bitmap->toBytes();

        self::assertSame($bitmap->width() * $bitmap->height() * 4, strlen($raw));
    }

    public function testToPngBytesHasPngMagicNumber(): void
    {
        $bitmap = self::renderedBitmap();
        $png = $bitmap->toPngBytes();

        self::assertSame("\x89PNG\r\n\x1a\n", substr($png, 0, 8));
    }

    public function testToStringContainsDimensions(): void
    {
        $bitmap = self::renderedBitmap();
        $text = (string) $bitmap;

        self::assertStringContainsString((string) $bitmap->width(), $text);
        self::assertStringContainsString((string) $bitmap->height(), $text);
    }

    public function testDebugInfoContainsWidthAndHeight(): void
    {
        $bitmap = self::renderedBitmap();
        $debug = $bitmap->__debugInfo();

        self::assertSame((string) $bitmap->width(), $debug['width']);
        self::assertSame((string) $bitmap->height(), $debug['height']);
    }
}
