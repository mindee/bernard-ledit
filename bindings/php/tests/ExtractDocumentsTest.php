<?php
declare(strict_types=1);

namespace BernardLedit\Tests;

use BernardLedit\Pdf\PdfDocument;
use PHPUnit\Framework\TestCase;

require_once __DIR__ . '/config.php';

use function BernardLedit\Pdf\extractFromPageRanges;

final class ExtractDocumentsTest extends TestCase
{
    private function loadFileContents(string $name): string
    {
        return file_get_contents(TEST_ROOT_PATH . '/file_types/' . $name);
    }

    public function testGroupPageCounts(): void
    {
        $docs = extractFromPageRanges($this->loadFileContents('pdf/multipage.pdf'), [[0], [1, 2]]);
        $this->assertCount(2, $docs);
        $this->assertSame(1, $docs[0]->getPageCount());
        $this->assertSame(2, $docs[1]->getPageCount());
    }

    public function testOutputRoundtrips(): void
    {
        [$doc] = extractFromPageRanges($this->loadFileContents('pdf/multipage.pdf'), [[0, 1]]);
        $bytes = $doc->asBytes();
        $this->assertStringStartsWith('%PDF-', $bytes);
        $this->assertSame(2, (new PdfDocument($bytes))->getPageCount());
    }

    public function testExtractedDocumentHasText(): void
    {
        [$doc] = extractFromPageRanges($this->loadFileContents('pdf/multipage.pdf'), [[2]]);
        $this->assertTrue($doc->hasText());
    }

    public function testFromImageIsOnePageDocument(): void
    {
        foreach (['receipt.jpg', 'receipt.png'] as $file) {
            [$doc] = extractFromPageRanges($this->loadFileContents($file), [[0]]);
            $this->assertSame(1, $doc->getPageCount(), $file);
        }
    }

    public function testNoGroupsThrows(): void
    {
        $this->expectException(\ValueError::class);
        extractFromPageRanges($this->loadFileContents('pdf/multipage.pdf'), []);
    }

    public function testEmptyGroupThrows(): void
    {
        $this->expectException(\ValueError::class);
        extractFromPageRanges($this->loadFileContents('pdf/multipage.pdf'), [[0], []]);
    }

    public function testIndexOutOfRangeThrows(): void
    {
        $this->expectException(\ValueError::class);
        extractFromPageRanges($this->loadFileContents('pdf/multipage.pdf'), [[12]]);
    }

    public function testGarbageThrows(): void
    {
        $this->expectException(\Throwable::class);
        extractFromPageRanges('garbage', [[0]]);
    }
}