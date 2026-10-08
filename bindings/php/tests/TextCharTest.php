<?php
declare(strict_types=1);

use PHPUnit\Framework\TestCase;

final class TextCharTest extends TestCase
{
    public function testConstructWithMinimalArguments(): void
    {
        $char = new \PhpTextChar('A', 'Arial', 12.0, 400, null, null, 0);

        self::assertSame('A', (string) $char);
    }

    public function testConstructWithAllArguments(): void
    {
        $char = new \PhpTextChar(
            'B',
            'Helvetica',
            14.5,
            700,
            [255, 0, 0, 255],
            [0, 255, 0, 255],
            32,
            [1.0, 2.0, 3.0, 4.0],
        );

        self::assertSame('B', (string) $char);
    }

    public function testToStringReturnsOnlyTheCharacter(): void
    {
        $char = new \PhpTextChar('Z', 'Arial', 10.0, 400, null, null, 0);
        self::assertSame('Z', (string) $char);
    }

    public function testMultiCharStringTruncatesToFirstCharacter(): void
    {
        $char = new \PhpTextChar('AB', 'Arial', 10.0, 400, null, null, 0);
        self::assertSame('A', (string) $char);
    }

    public function testEmptyStringDefaultsToSpace(): void
    {
        $char = new \PhpTextChar('', 'Arial', 10.0, 400, null, null, 0);
        self::assertSame(' ', (string) $char);
    }

    public function testDebugInfoContainsAllFields(): void
    {
        $char = new \PhpTextChar(
            'A',
            'Arial',
            12.0,
            400,
            [1, 2, 3, 4],
            null,
            0,
            [0.0, 0.0, 10.0, 10.0],
        );

        $debug = $char->__debugInfo();

        self::assertSame('A', $debug['char']);
        self::assertSame('Arial', $debug['font_name']);
        self::assertSame('12', $debug['font_size']);
        self::assertSame('400', $debug['font_weight']);
        self::assertSame('0', $debug['font_flags']);
        self::assertSame('null', $debug['fill_color']);
        self::assertStringContainsString('1', $debug['stroke_color']);
    }

    public function testNoTypedGettersAreExposed(): void
    {
        $char = new \PhpTextChar('A', 'Arial', 12.0, 400, null, null, 0);

        self::assertFalse(method_exists($char, 'getFontName'));
        self::assertFalse(method_exists($char, 'getFontSize'));
        self::assertFalse(property_exists($char, 'font_name'));
    }
}
