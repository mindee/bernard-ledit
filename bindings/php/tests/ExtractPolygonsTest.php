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

    private function data(string $name): string
    {
        return file_get_contents(TEST_ROOT_PATH . '/file_types/' . $name);
    }

    public function testFullPagePolygonKeepsSize(): void
    {
        $bytes = $this->data('receipt.jpg');
        [$img] = extractFromPolygons($bytes, 0, [self::FULL]);
        $source = new \BernardLedit\Image\Image($bytes);
        $this->assertSame($source->width(), $img->width());
        $this->assertSame($source->height(), $img->height());
        $this->assertSame('JPEG', $img->format());
    }

    public function testQuarterPolygonIsHalfSize(): void
    {
        $bytes = $this->data('receipt.jpg');
        $source = new \BernardLedit\Image\Image($bytes);
        [$img] = extractFromPolygons($bytes, 0, [self::QUARTER]);
        $this->assertEqualsWithDelta($source->width() / 2, $img->width(), 1);
        $this->assertEqualsWithDelta($source->height() / 2, $img->height(), 1);
    }

    public function testOneImagePerPolygon(): void
    {
        $images = extractFromPolygons($this->data('receipt.jpg'), 0, [self::FULL, self::QUARTER]);
        $this->assertCount(2, $images);
    }

    public function testFromPdfIsJpeg(): void
    {
        $pdf = $this->data('pdf/multipage.pdf');
        [$img] = extractFromPolygons($pdf, 0, [self::FULL]);
        $this->assertSame('JPEG', $img->format());
        $this->assertGreaterThan(0, $img->width());
    }

    public function testImagePageOutOfRangeThrows(): void
    {
        $this->expectException(\Throwable::class);
        extractFromPolygons($this->data('receipt.jpg'), 1, [self::FULL]);
    }

    public function testEmptyPolygonThrows(): void
    {
        $this->expectException(\Throwable::class);
        extractFromPolygons($this->data('receipt.jpg'), 0, [[]]);
    }

    public function testMalformedPointThrows(): void
    {
        $this->expectException(\Throwable::class);
        extractFromPolygons($this->data('receipt.jpg'), 0, [[[0.0, 0.0, 0.0]]]);
    }

    public function testGarbageThrows(): void
    {
        $this->expectException(\Throwable::class);
        extractFromPolygons('not an image', 0, [self::FULL]);
    }
}