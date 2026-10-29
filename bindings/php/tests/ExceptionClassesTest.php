<?php
declare(strict_types=1);

namespace BernardLedit\Tests;
use BernardLedit\Image\Image;
use BernardLedit\Image\ImageException;
use BernardLedit\Pdf\PdfDocument;
use BernardLedit\Pdf\PdfiumException;
use BernardLedit\Pdf\TextChar;
use PHPUnit\Framework\Attributes\DataProvider;
use PHPUnit\Framework\TestCase;

use function BernardLedit\Image\compress as compressImage;
use function BernardLedit\Image\decode;
use function BernardLedit\Image\extractFromPolygons;
use function BernardLedit\Image\guessFormat;
use function BernardLedit\Pdf\applyPageOptions;
use function BernardLedit\Pdf\compress as compressPdf;
use function BernardLedit\Pdf\extractFromPageRanges;
use function BernardLedit\Pdf\fixPdf;

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
     * @return iterable<string, array{string, callable, string}>
     */
    public static function argumentOutOfRange(): iterable
    {
        $pdf = self::file('pdf/multipage.pdf');
        $jpeg = self::file('receipt.jpg');
        yield 'page_id' => [
            'BernardLedit\Image\extractFromPolygons',
            fn () => extractFromPolygons($pdf, -1, [self::FULL]),
            'page_id must be between 0 and 65535, got -1',
        ];
        foreach (['0' => 0.0, '-1' => -1.0, 'NaN' => NAN, 'inf' => INF, '5000' => 5000.0] as $shown => $dpi) {
            yield "dpi $shown" => [
                'BernardLedit\Image\extractFromPolygons',
                fn () => extractFromPolygons($pdf, 0, [self::FULL], $dpi),
                "dpi must be a finite number between 1 and 1200, got $shown",
            ];
        }
        yield 'Image\compress quality' => [
            'BernardLedit\Image\compress',
            fn () => compressImage($jpeg, 300),
            'quality must be between 5 and 100, got 300',
        ];
        yield 'max_width' => [
            'BernardLedit\Image\compress',
            fn () => compressImage($jpeg, 85, -10),
            'max_width must be between 1 and 4294967295, got -10',
        ];
        yield 'Image::encode quality' => [
            'BernardLedit\Image\Image::encode',
            fn () => decode($jpeg)->encode('JPEG', 0),
            'quality must be between 1 and 100, got 0',
        ];
        yield 'crop' => [
            'BernardLedit\Image\Image::crop',
            fn () => decode($jpeg)->crop(-1, 0, 10, 10),
            'left must be between 0 and 4294967295, got -1',
        ];
        yield 'resize' => [
            'BernardLedit\Image\Image::resize',
            fn () => decode($jpeg)->resize(0, 10),
            'width must be between 1 and 4294967295, got 0',
        ];
        yield 'importPages' => [
            'BernardLedit\Pdf\PdfDocument::importPages',
            fn () => PdfDocument::create()->importPages(new PdfDocument($pdf), [0, -1]),
            'indexes[1] must be between 0 and 65535, got -1',
        ];
        yield 'getPage' => [
            'BernardLedit\Pdf\PdfDocument::getPage',
            fn () => (new PdfDocument($pdf))->getPage(65536),
            'index must be between 0 and 65535, got 65536',
        ];
        yield 'rasterizePageToJpegBytes quality' => [
            'BernardLedit\Pdf\PdfDocument::rasterizePageToJpegBytes',
            fn () => (new PdfDocument($pdf))->rasterizePageToJpegBytes(0, 101),
            'quality must be between 1 and 100, got 101',
        ];
        yield 'PdfDocument::compress quality' => [
            'BernardLedit\Pdf\PdfDocument::compress',
            fn () => (new PdfDocument($pdf))->compress(1),
            'quality must be between 5 and 100, got 1',
        ];
        yield 'Pdf\compress dpi' => [
            'BernardLedit\Pdf\compress',
            fn () => compressPdf($pdf, 85, null, 0.5),
            'dpi must be a finite number between 1 and 1200, got 0.5',
        ];
        yield 'renderToBitmap scale' => [
            'BernardLedit\Pdf\PdfPage::renderToBitmap',
            fn () => (new PdfDocument($pdf))->getPage(0)->renderToBitmap(0.0),
            'scale must be a finite number between 0.01 and 16, got 0',
        ];
        yield 'on_min_pages' => [
            'BernardLedit\Pdf\applyPageOptions',
            fn () => applyPageOptions($pdf, 'KEEP_ONLY', -1, [0]),
            'on_min_pages must be between 0 and 4294967295, got -1',
        ];
        yield 'extractFromPageRanges' => [
            'BernardLedit\Pdf\extractFromPageRanges',
            fn () => extractFromPageRanges($pdf, [[0], [1, 70000]]),
            'page_indexes[1][1] must be between 0 and 65535, got 70000',
        ];
        yield 'fixPdf' => [
            'BernardLedit\Pdf\fixPdf',
            fn () => fixPdf($pdf, -1),
            'maximum_offset must be at least 0, got -1',
        ];
        yield 'TextChar color' => [
            'BernardLedit\Pdf\TextChar::__construct',
            fn () => new TextChar('A', 'Arial', 12.0, 400, [0, 0, 0, 256], null, 0),
            'stroke_color[3] must be between 0 and 255, got 256',
        ];
    }

    #[DataProvider('argumentOutOfRange')]
    public function testArgumentOutOfRangeIsValueError(string $symbol, callable $call, string $message): void
    {
        self::assertThrowsDeclared($symbol, \ValueError::class, $call, $message);
    }

    public function testExceptionsTakeExceptionArguments(): void
    {
        $previous = new \RuntimeException('cause');
        foreach ([ImageException::class, PdfiumException::class] as $class) {
            $e = new $class('message', 3, $previous);
            self::assertSame('message', $e->getMessage());
            self::assertSame(3, $e->getCode());
            self::assertSame($previous, $e->getPrevious());
        }
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
