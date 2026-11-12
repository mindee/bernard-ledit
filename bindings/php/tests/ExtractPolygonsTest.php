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
        $source = new \BernardLedit\Image\Image($bytes);
        [$img] = extractFromPolygons($source, [self::FULL]);
        $this->assertSame($source->getWidth(), $img->getWidth());
        $this->assertSame($source->getHeight(), $img->getHeight());
        $this->assertSame('JPEG', $img->getFormat());
    }

    public function testQuarterPolygonIsHalfSize(): void
    {
        $bytes = $this->loadFileContents('receipt.jpg');
        $source = new \BernardLedit\Image\Image($bytes);
        [$img] = extractFromPolygons($source, [self::QUARTER]);
        $this->assertEqualsWithDelta($source->getWidth() / 2, $img->getWidth(), 1);
        $this->assertEqualsWithDelta($source->getHeight() / 2, $img->getHeight(), 1);
    }

    public function testOneImagePerPolygon(): void
    {
        $image = new \BernardLedit\Image\Image($this->loadFileContents('receipt.jpg'));
        $images = extractFromPolygons($image, [self::FULL, self::QUARTER]);
        $this->assertCount(2, $images);
    }

    public function testEmptyPolygonThrows(): void
    {
        $this->expectException(\ValueError::class);
        $image = new \BernardLedit\Image\Image($this->loadFileContents('receipt.jpg'));
        extractFromPolygons($image, [[]]);
    }

    public function testMalformedPointThrows(): void
    {
        $this->expectException(\ValueError::class);
        $image = new \BernardLedit\Image\Image($this->loadFileContents('receipt.jpg'));
        extractFromPolygons($image, [[[0.0, 0.0, 0.0]]]);
    }

    public function testRequiresImage(): void
    {
        $this->expectException(\TypeError::class);
        extractFromPolygons($this->loadFileContents('receipt.jpg'), [self::FULL]);
    }
}