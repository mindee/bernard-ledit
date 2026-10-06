<?php
declare(strict_types=1);

use PHPUnit\Framework\TestCase;
use function BernardLedit\Image\decode;
use function BernardLedit\Image\guessFormat;
use BernardLedit\Image\ImageException;

final class ImageTest extends TestCase
{
    private const FIXTURES = __DIR__ . '/../../../tests/data/file_types';

    public function testDecodeAndSize(): void
    {
        $bytes = file_get_contents(self::FIXTURES . '/receipt.jpg');
        $img = decode($bytes);
        self::assertSame('JPEG', $img->format());
        [$w, $h] = $img->size();
        self::assertGreaterThan(0, $w);
        self::assertSame($w, $img->width());
    }

    public function testBinaryRoundTripIsNotUtf8Mangled(): void
    {
        $bytes = file_get_contents(self::FIXTURES . '/receipt.jpg');
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

    public function testExtensionNameIsWhatTheSdkChecks(): void
    {
        self::assertTrue(extension_loaded('bernard_ledit'));
        self::assertNotFalse(phpversion('bernard_ledit'));
    }
}
