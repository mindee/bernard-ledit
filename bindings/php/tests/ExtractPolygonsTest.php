<?php
declare(strict_types=1);

namespace BernardLedit\Tests;

use PHPUnit\Framework\TestCase;

require_once __DIR__ . '/config.php';

use function BernardLedit\Image\extractFromPolygons;

final class ExtractPolygonsTest extends TestCase
{
    private const FULL = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    private const QUARTER = [[0.0, 0.0], [0.5, 0.0], [0.5, 0.5], [0.0, 0.5]];

    private function loadFileContents($name): string
    {
        return file_get_contents(TEST_ROOT_PATH . '/file_types/' . $name);
    }

    public function testFullPagePolygonKeepsSize(): void
    {
        $bytes = $this->loadFileContents('receipt.jpg');
        [$img] = extractFromPolygons($bytes, 0, [self::FULL]);
        $source = new \BernardLedit\Image\Image($bytes);
        $this->assertSame($source->getWidth(), $img->getWidth());
        $this->assertSame($source->getHeight(), $img->getHeight());
        $this->assertSame('JPEG', $img->getFormat());
    }

    public function testQuarterPolygonIsHalfSize(): void
    {
        $bytes = $this->loadFileContents('receipt.jpg');
        $source = new \BernardLedit\Image\Image($bytes);
        [$img] = extractFromPolygons($bytes, 0, [self::QUARTER]);
        $this->assertEqualsWithDelta($source->getWidth() / 2, $img->getWidth(), 1);
        $this->assertEqualsWithDelta($source->getHeight() / 2, $img->getHeight(), 1);
    }

    public function testOneImagePerPolygon(): void
    {
        $images = extractFromPolygons($this->loadFileContents('receipt.jpg'), 0, [self::FULL, self::QUARTER]);
        $this->assertCount(2, $images);
    }

    public function testFromPdfIsJpeg(): void
    {
        $pdf = $this->loadFileContents('pdf/multipage.pdf');
        [$img] = extractFromPolygons($pdf, 0, [self::FULL]);
        $this->assertSame('JPEG', $img->getFormat());
        $this->assertGreaterThan(0, $img->getWidth());
    }

    public function testImagePageOutOfRangeThrows(): void
    {
        $this->expectException(\Throwable::class);
        extractFromPolygons($this->loadFileContents('receipt.jpg'), 1, [self::FULL]);
    }

    public function testEmptyPolygonThrows(): void
    {
        $this->expectException(\Throwable::class);
        extractFromPolygons($this->loadFileContents('receipt.jpg'), 0, [[]]);
    }

    public function testMalformedPointThrows(): void
    {
        $this->expectException(\Throwable::class);
        extractFromPolygons($this->loadFileContents('receipt.jpg'), 0, [[[0.0, 0.0, 0.0]]]);
    }

    public function testGarbageThrows(): void
    {
        $this->expectException(\Throwable::class);
        extractFromPolygons('not an image', 0, [self::FULL]);
    }
}