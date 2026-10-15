<?php
declare(strict_types=1);

namespace BernardLedit\Tests;

use BernardLedit\Pdf\PdfDocument;
use PHPUnit\Framework\TestCase;

require_once __DIR__ . '/config.php';

use function BernardLedit\Pdf\extractFromPageRanges;

final class ExtractDocumentsTest extends TestCase
{
    private function data(string $name): string
    {
        return file_get_contents(TEST_ROOT_PATH . '/file_types/' . $name);
    }

    public function testGroupPageCounts(): void
    {
        $docs = extractFromPageRanges($this->data('pdf/multipage.pdf'), [[0], [1, 2]]);
        $this->assertCount(2, $docs);
        $this->assertSame(1, $docs[0]->pageCount());
        $this->assertSame(2, $docs[1]->pageCount());
    }

    public function testOutputRoundtrips(): void
    {
        [$doc] = extractFromPageRanges($this->data('pdf/multipage.pdf'), [[0, 1]]);
        $bytes = $doc->save();
        $this->assertStringStartsWith('%PDF-', $bytes);
        $this->assertSame(2, (new PdfDocument($bytes))->pageCount());
    }

    public function testExtractedDocumentHasText(): void
    {
        [$doc] = extractFromPageRanges($this->data('pdf/multipage.pdf'), [[2]]);
        $this->assertTrue($doc->hasText());
    }

    public function testFromImageIsOnePageDocument(): void
    {
        foreach (['receipt.jpg', 'receipt.png'] as $file) {
            [$doc] = extractFromPageRanges($this->data($file), [[0]]);
            $this->assertSame(1, $doc->pageCount(), $file);
        }
    }

    public function testNoGroupsThrows(): void
    {
        $this->expectException(\ValueError::class);
        extractFromPageRanges($this->data('pdf/multipage.pdf'), []);
    }

    public function testEmptyGroupThrows(): void
    {
        $this->expectException(\ValueError::class);
        extractFromPageRanges($this->data('pdf/multipage.pdf'), [[0], []]);
    }

    public function testIndexOutOfRangeThrows(): void
    {
        $this->expectException(\Throwable::class); // PageIndexOutOfBounds goes through map_pdf_err
        extractFromPageRanges($this->data('pdf/multipage.pdf'), [[12]]);
    }

    public function testGarbageThrows(): void
    {
        $this->expectException(\Throwable::class);
        extractFromPageRanges('garbage', [[0]]);
    }
}