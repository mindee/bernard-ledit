<?php
declare(strict_types=1);

namespace BernardLedit\Tests;

use BernardLedit\Image\Image;
use BernardLedit\Image\ImageException;
use BernardLedit\Pdf\PdfDocument;
use PHPUnit\Framework\Attributes\DataProvider;
use PHPUnit\Framework\TestCase;

use function BernardLedit\Image\compress as compressImage;
use function BernardLedit\Image\decode;
use function BernardLedit\Image\extractFromPolygons;
use function BernardLedit\Image\guessFormat;
use function BernardLedit\Pdf\compress as compressPdf;
use function BernardLedit\Pdf\extractFromPageRanges;

require_once __DIR__ . '/config.php';
require_once __DIR__ . '/DeclaredThrows.php';

/**
 * One class per failure, whichever function reports it, and declared in the stubs.
 */
final class ExceptionClassesTest extends TestCase
{
    use DeclaredThrows;

    private const FULL = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    private static function file(string $name): string
    {
        return file_get_contents(TEST_ROOT_PATH . '/file_types/' . $name);
    }

    /**
     * @return iterable<string, array{string, callable}>
     */
    public static function unidentifiableImage(): iterable
    {
        $heic = self::file('receipt.heic');
        yield 'guessFormat' => ['BernardLedit\Image\guessFormat', fn () => guessFormat($heic)];
        yield 'decode' => ['BernardLedit\Image\decode', fn () => decode($heic)];
        yield 'new Image' => ['BernardLedit\Image\Image::__construct', fn () => new Image($heic)];
        yield 'Image\compress' => ['BernardLedit\Image\compress', fn () => compressImage($heic)];
        yield 'extractFromPolygons' => [
            'BernardLedit\Image\extractFromPolygons',
            fn () => extractFromPolygons($heic, 0, [self::FULL]),
        ];
        yield 'extractFromPageRanges' => [
            'BernardLedit\Pdf\extractFromPageRanges',
            fn () => extractFromPageRanges($heic, [[0]]),
        ];
        yield 'appendJpegPage' => [
            'BernardLedit\Pdf\PdfDocument::appendJpegPage',
            fn () => PdfDocument::create()->appendJpegPage($heic),
        ];
    }

    #[DataProvider('unidentifiableImage')]
    public function testUnidentifiableImageIsImageException(string $symbol, callable $call): void
    {
        self::assertThrowsDeclared($symbol, ImageException::class, $call);
    }

    /**
     * @return iterable<string, array{string, callable, string}>
     */
    public static function pageOutOfRange(): iterable
    {
        $pdf = self::file('pdf/multipage.pdf');
        $twelvePages = 'Page index 40 out of bounds (document has 12 pages)';
        yield 'extractFromPolygons on an image' => [
            'BernardLedit\Image\extractFromPolygons',
            fn () => extractFromPolygons(self::file('receipt.jpg'), 1, [self::FULL]),
            'Page index 1 out of bounds (an image has 1 page)',
        ];
        yield 'extractFromPolygons on a PDF' => [
            'BernardLedit\Image\extractFromPolygons',
            fn () => extractFromPolygons($pdf, 40, [self::FULL]),
            $twelvePages,
        ];
        yield 'extractFromPageRanges' => [
            'BernardLedit\Pdf\extractFromPageRanges',
            fn () => extractFromPageRanges($pdf, [[40]]),
            $twelvePages,
        ];
        yield 'getPage' => [
            'BernardLedit\Pdf\PdfDocument::getPage',
            fn () => (new PdfDocument($pdf))->getPage(40),
            $twelvePages,
        ];
        yield 'rasterizePageToJpegBytes' => [
            'BernardLedit\Pdf\PdfDocument::rasterizePageToJpegBytes',
            fn () => (new PdfDocument($pdf))->rasterizePageToJpegBytes(40, 85),
            $twelvePages,
        ];
        yield 'addText' => [
            'BernardLedit\Pdf\PdfDocument::addText',
            fn () => (new PdfDocument($pdf))->addText(40, []),
            $twelvePages,
        ];
        yield 'importPages' => [
            'BernardLedit\Pdf\PdfDocument::importPages',
            fn () => PdfDocument::create()->importPages(new PdfDocument($pdf), [40]),
            $twelvePages,
        ];
    }

    #[DataProvider('pageOutOfRange')]
    public function testPageOutOfRangeIsValueError(string $symbol, callable $call, string $message): void
    {
        self::assertThrowsDeclared($symbol, \ValueError::class, $call, $message);
    }

    /**
     * @return iterable<string, array{string, callable}>
     */
    public static function badQuality(): iterable
    {
        $scan = self::file('pdf/not_blank_image_only.pdf');
        $jpeg = self::file('receipt.jpg');
        yield 'Image::encode' => [
            'BernardLedit\Image\Image::encode',
            fn () => decode($jpeg)->encode('JPEG', 0),
        ];
        yield 'Image\compress' => ['BernardLedit\Image\compress', fn () => compressImage($jpeg, 1)];
        yield 'Pdf\compress' => ['BernardLedit\Pdf\compress', fn () => compressPdf($scan, 1)];
        yield 'PdfDocument::compress' => [
            'BernardLedit\Pdf\PdfDocument::compress',
            fn () => (new PdfDocument($scan))->compress(1),
        ];
        yield 'rasterizePageToJpegBytes' => [
            'BernardLedit\Pdf\PdfDocument::rasterizePageToJpegBytes',
            fn () => (new PdfDocument($scan))->rasterizePageToJpegBytes(0, 101),
        ];
    }

    #[DataProvider('badQuality')]
    public function testBadQualityIsImageException(string $symbol, callable $call): void
    {
        self::assertThrowsDeclared($symbol, ImageException::class, $call);
    }

    public function testClosedDocumentIsException(): void
    {
        $doc = new PdfDocument(self::file('pdf/blank_1.pdf'));
        $doc->close();
        $doc->close();
        self::assertThrowsDeclared(
            'BernardLedit\Pdf\PdfDocument::getPageCount',
            \Exception::class,
            fn () => $doc->getPageCount(),
            'PdfDocument is closed'
        );
    }

    public function testDeclaredClassesExist(): void
    {
        $tokens = \PhpToken::tokenize(file_get_contents(__DIR__ . '/../bernard_ledit.stubs.php'));
        $checked = 0;
        foreach ($tokens as $token) {
            if ($token->is(T_DOC_COMMENT) && preg_match_all('/@throws\s+\\\\?(\S+)/', $token->text, $m)) {
                foreach ($m[1] as $class) {
                    self::assertTrue(is_a($class, \Throwable::class, true), "@throws $class isn't a Throwable");
                    $checked++;
                }
            }
        }
        self::assertGreaterThan(0, $checked);
    }
}
