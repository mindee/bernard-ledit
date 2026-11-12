<?php
declare(strict_types=1);

namespace BernardLedit\Tests;

use PHPUnit\Framework\TestCase;

require_once __DIR__ . '/config.php';

final class CropPolygonTest extends TestCase
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
        $img = $source->cropPolygon(self::FULL);
        $this->assertSame($source->getWidth(), $img->getWidth());
        $this->assertSame($source->getHeight(), $img->getHeight());
        $this->assertSame('JPEG', $img->getFormat());
    }

    public function testQuarterPolygonIsHalfSize(): void
    {
        $bytes = $this->loadFileContents('receipt.jpg');
        $source = new \BernardLedit\Image\Image($bytes);
        $img = $source->cropPolygon(self::QUARTER);
        $this->assertEqualsWithDelta($source->getWidth() / 2, $img->getWidth(), 1);
        $this->assertEqualsWithDelta($source->getHeight() / 2, $img->getHeight(), 1);
    }

    public function testCanCropMultiplePolygonsIndependently(): void
    {
        $image = new \BernardLedit\Image\Image($this->loadFileContents('receipt.jpg'));
        $crops = [$image->cropPolygon(self::FULL), $image->cropPolygon(self::QUARTER)];
        $this->assertCount(2, $crops);
    }

    public function testEmptyPolygonThrows(): void
    {
        $this->expectException(\ValueError::class);
        $image = new \BernardLedit\Image\Image($this->loadFileContents('receipt.jpg'));
        $image->cropPolygon([]);
    }

    public function testMalformedPointThrows(): void
    {
        $this->expectException(\ValueError::class);
        $image = new \BernardLedit\Image\Image($this->loadFileContents('receipt.jpg'));
        $image->cropPolygon([[0.0, 0.0, 0.0]]);
    }
}