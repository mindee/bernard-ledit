<?php
declare(strict_types=1);

namespace BernardLedit\Tests;

use BernardLedit\Pdf\PdfDocument;
use PHPUnit\Framework\TestCase;

require_once __DIR__ . '/config.php';

use function BernardLedit\Pdf\applyPageOptions;

final class PageOptionsTest extends TestCase
{
    private const KEEP_ONLY = 'KEEP_ONLY';
    private const REMOVE = 'REMOVE';

    private function loadFileContents(string $name): string
    {
        return file_get_contents(TEST_ROOT_PATH . '/file_types/' . $name);
    }

    public function testApplyValidKeepOperation(): void
    {
        $data =$this->loadFileContents('pdf/multipage.pdf');

        [$doc, $skipped] = applyPageOptions($data, self::KEEP_ONLY, 1, [0, 2]);

        $this->assertInstanceOf(PdfDocument::class,$doc);
        $this->assertIsArray($skipped);
        $this->assertSame(2,$doc->getPageCount());
    }
public function testApplyValidRemoveOperation(): void
    {
        $data =$this->loadFileContents('pdf/multipage.pdf');

        [$doc, $skipped] = applyPageOptions($data, self::REMOVE, 1, [1]);

        $this->assertInstanceOf(PdfDocument::class,$doc);
        $this->assertIsArray($skipped);
        $this->assertSame(11,$doc->getPageCount());
    }

    public function testBypassWhenBelowMinPagesThreshold(): void
    {
        $data =$this->loadFileContents('pdf/multipage.pdf');

        [$doc, $skipped] = applyPageOptions($data, self::REMOVE, 50, [0]);

        $this->assertNull($doc);
        $this->assertSame([],$skipped);
    }

    public function testInvalidOperationThrowsValueError(): void
    {
        $data =$this->loadFileContents('pdf/multipage.pdf');

        $this->expectException(\ValueError::class);$this->expectExceptionMessage('Invalid cut behavior specified: shred_it');
        applyPageOptions($data, 'shred_it', 1, [0]);
    }

    public function testNegativeIndexing(): void
    {
        $data =$this->loadFileContents('pdf/multipage.pdf');

        [$doc, $skipped] = applyPageOptions($data, self::REMOVE, 1, [-1]);

        $this->assertInstanceOf(PdfDocument::class,$doc);
        $this->assertSame(11,$doc->getPageCount());
    }

    public function testGarbagePdfDataThrows(): void
    {
        $this->expectException(\Throwable::class);
        applyPageOptions('not a valid pdf', self::KEEP_ONLY, 1, [0]);
    }
}
