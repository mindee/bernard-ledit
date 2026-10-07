<?php
declare(strict_types=1);

use PHPUnit\Framework\TestCase;
use function BernardLedit\Image\decode;
use function BernardLedit\Image\guessFormat;
use function BernardLedit\Image\compress;
use BernardLedit\Image\Image;
use BernardLedit\Image\ImageException;

final class ImageTest extends TestCase
{
    private const FIXTURES = __DIR__ . '/../../../tests/data/file_types';

    private static function fixture(string $name): string
    {
        return file_get_contents(self::FIXTURES . '/' . $name);
    }

    /** PNG byte stream of exact dimensions, derived from receipt.png (resize + re-encode). */
    private static function makePng(int $width, int $height): string
    {
        $base = decode(self::fixture('receipt.png'));
        return $base->resize($width, $height)->encode('PNG');
    }

    public function testDecodeAndSize(): void
    {
        $img = decode(self::fixture('receipt.jpg'));
        self::assertInstanceOf(Image::class, $img);
        self::assertSame('JPEG', $img->format());
        [$w, $h] = $img->size();
        self::assertGreaterThan(0, $w);
        self::assertSame($w, $img->width());
        self::assertSame($h, $img->height());
    }

    public function testDecodePng(): void
    {
        $img = decode(self::fixture('receipt.png'));
        self::assertSame('PNG', $img->format());
    }

    public function testBinaryRoundTripIsNotUtf8Mangled(): void
    {
        $bytes = self::fixture('receipt.jpg');
        $out = decode($bytes)->encode('JPEG', 90);
        self::assertSame("\xFF\xD8", substr($out, 0, 2));
        self::assertSame('JPEG', guessFormat($out));
    }

    public function testInvalidDataThrowsImageException(): void
    {
        // Garbage bytes fail format sniffing inside the underlying `image` crate, which the
        // bindings map to the custom ImageException (not \ValueError — that's reserved for
        // structurally-valid-but-rejected inputs like UnknownFormat/InvalidCrop/etc.;
        // see bindings/php/src/image/error.rs).
        $this->expectException(ImageException::class);
        decode('not an image');
    }

    public function testGuessFormatJpeg(): void
    {
        self::assertSame('JPEG', guessFormat(self::fixture('receipt.jpg')));
    }

    public function testGuessFormatPng(): void
    {
        self::assertSame('PNG', guessFormat(self::fixture('receipt.png')));
    }

    public function testGuessFormatTiff(): void
    {
        self::assertSame('TIFF', guessFormat(self::fixture('receipt.tiff')));
    }

    public function testGuessFormatGarbageThrowsValueError(): void
    {
        $this->expectException(\ValueError::class);
        guessFormat("\x00\x01\x02\x03");
    }

    public function testSizeMatchesRequestedDimensions(): void
    {
        $img = decode(self::makePng(30, 20));
        self::assertSame([30, 20], $img->size());
    }

    public function testCropValidBox(): void
    {
        $img = decode(self::makePng(100, 80));
        $cropped = $img->crop(10, 20, 60, 70);
        self::assertSame([50, 50], $cropped->size());
    }

    public function testCropOutOfBoundsThrowsValueError(): void
    {
        $img = decode(self::makePng(30, 30));
        $this->expectException(\ValueError::class);
        $img->crop(0, 0, 31, 10);
    }

    public function testCropInvertedBoxThrowsValueError(): void
    {
        $img = decode(self::makePng(30, 30));
        $this->expectException(\ValueError::class);
        $img->crop(20, 5, 10, 25);
    }

    public function testResizeExact(): void
    {
        $img = decode(self::makePng(40, 40));
        self::assertSame([80, 20], $img->resize(80, 20)->size());
    }

    public function testResizeDefaultFilterIsAccepted(): void
    {
        $img = decode(self::makePng(40, 40));
        self::assertSame([10, 10], $img->resize(10, 10)->size());
    }

    public function testResizeNamedFilters(): void
    {
        $img = decode(self::makePng(40, 40));
        foreach (['nearest', 'triangle', 'catmullrom', 'bicubic', 'gaussian', 'lanczos'] as $name) {
            self::assertSame([20, 20], $img->resize(20, 20, $name)->size());
        }
    }

    public function testResizeUnknownFilterThrowsValueError(): void
    {
        $img = decode(self::makePng(40, 40));
        $this->expectException(\ValueError::class);
        $img->resize(20, 20, 'sinc');
    }

    public function testResizeZeroDimensionThrowsValueError(): void
    {
        $img = decode(self::makePng(40, 40));
        $this->expectException(\ValueError::class);
        $img->resize(0, 20);
    }

    public function testEncodeJpegMagic(): void
    {
        $img = decode(self::makePng(16, 16));
        $out = $img->encode('JPEG');
        self::assertSame("\xFF\xD8\xFF", substr($out, 0, 3));
    }

    public function testEncodePngMagic(): void
    {
        $img = decode(self::makePng(16, 16));
        $out = $img->encode('PNG');
        self::assertSame("\x89PNG\r\n\x1a\n", substr($out, 0, 8));
    }

    public function testEncodeQualityArgument(): void
    {
        $img = decode(self::makePng(64, 64));
        $out = $img->encode('JPEG', 50);
        self::assertSame("\xFF\xD8\xFF", substr($out, 0, 3));
    }

    public function testEncodeOptimizeReducesJpegSize(): void
    {
        $img = decode(self::makePng(256, 256));
        $unoptimized = $img->encode('JPEG', 85, false);
        $optimized = $img->encode('JPEG', 85, true);
        self::assertLessThanOrEqual(strlen($unoptimized), strlen($optimized));
    }

    public function testEncodeUnknownFormatThrowsValueError(): void
    {
        $img = decode(self::makePng(16, 16));
        $this->expectException(\ValueError::class);
        $img->encode('NOPE');
    }

    public function testEncodeRoundtripReadable(): void
    {
        $img = decode(self::makePng(24, 12));
        $jpeg = $img->encode('JPEG', 90);
        $reloaded = decode($jpeg);
        self::assertSame([24, 12], $reloaded->size());
        self::assertSame('JPEG', $reloaded->format());
    }

    public function testCompressReturnsJpegAndDimensions(): void
    {
        [$jpeg, $w, $h] = compress(self::makePng(200, 100), 85, 100, 100);
        self::assertSame("\xFF\xD8\xFF", substr($jpeg, 0, 3));
        self::assertSame(100, $w);
        self::assertSame(50, $h);
    }

    public function testCompressDoesNotUpscale(): void
    {
        [, $w, $h] = compress(self::makePng(30, 30), 85, 500, 500);
        self::assertSame(30, $w);
        self::assertSame(30, $h);
    }

    public function testCompressNoBoundsKeepsDimensions(): void
    {
        [, $w, $h] = compress(self::makePng(64, 48));
        self::assertSame(64, $w);
        self::assertSame(48, $h);
    }

    public function testCompressOutputIsJpegBytes(): void
    {
        [$jpeg] = compress(self::makePng(40, 40));
        self::assertSame("\xFF\xD8\xFF", substr($jpeg, 0, 3));
    }

    public function testCompressGarbageThrowsImageException(): void
    {
        $this->expectException(ImageException::class);
        compress("\x00\x01");
    }

    public function testExtensionNameIsWhatTheSdkChecks(): void
    {
        self::assertTrue(extension_loaded('bernard_ledit'));
        self::assertNotFalse(phpversion('bernard_ledit'));
    }
}
