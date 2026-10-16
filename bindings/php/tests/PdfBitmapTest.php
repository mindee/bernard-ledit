<?php
declare(strict_types=1);

use PHPUnit\Framework\TestCase;
use BernardLedit\Pdf\PdfDocument;

require_once __DIR__ . '/config.php';

final class PdfBitmapTest extends TestCase
{
    private const FIXTURES = TEST_ROOT_PATH . '/file_types';

    private static function renderBitmap(): \PdfBitmap
    {
        $doc = new PdfDocument(file_get_contents(self::FIXTURES . '/pdf/blank_1.pdf'));
        return $doc->getPage(0)->renderToBitmap();
    }

    public function testToBytesMatchesDimensions(): void
    {
        $bitmap = self::renderBitmap();
        $raw = $bitmap->toBytes();

        self::assertSame($bitmap->getWidth() * $bitmap->getHeight() * 4, strlen($raw));
    }

    public function testToPngBytesHasPngMagicNumber(): void
    {
        $bitmap = self::renderBitmap();
        $png = $bitmap->toPngBytes();

        self::assertSame("\x89PNG\r\n\x1a\n", substr($png, 0, 8));
    }

    public function testToStringContainsDimensions(): void
    {
        $bitmap = self::renderBitmap();
        $text = (string) $bitmap;

        self::assertStringContainsString((string) $bitmap->getWidth(), $text);
        self::assertStringContainsString((string) $bitmap->getHeight(), $text);
    }

    public function testDebugInfoContainsWidthAndHeight(): void
    {
        $bitmap = self::renderBitmap();
        $debug = $bitmap->__debugInfo();

        self::assertSame((string) $bitmap->getWidth(), $debug['width']);
        self::assertSame((string) $bitmap->getHeight(), $debug['height']);
    }
}
