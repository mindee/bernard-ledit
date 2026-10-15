<?php
declare(strict_types=1);

namespace BernardLedit\Tests;

use PHPUnit\Framework\TestCase;

require_once __DIR__ . '/config.php';

use function BernardLedit\Pdf\fixPdf;

final class FixPdfTest extends TestCase
{
    private function data(string $name): string
    {
        return file_get_contents(TEST_ROOT_PATH . '/file_types/' . $name);
    }

    public function testReturnsInputUnchangedWhenMarkerIsFirst(): void
    {
        $data = '%PDF-1.4 rest of file';
        $this->assertEquals('%PDF-1.4 rest of file', (string)fixPdf($data));
    }

    public function testStripsBytesBeforeMarker(): void
    {
        $data = "junk\r\n%PDF-1.4 rest of file";
        $this->assertEquals('%PDF-1.4 rest of file', (string)fixPdf($data));
    }

    public function testUsesFirstMarkerWhenSeveralExist(): void
    {
        $data = 'ab%PDF-1 cd%PDF-2';
        $this->assertEquals('%PDF-1 cd%PDF-2', (string)fixPdf($data));
    }

    public function testMarkerRespectsMaximumOffset(): void
    {
        $data = str_repeat('x', 10) . '%PDF-1.4';
        $this->assertEquals('%PDF-1.4', (string)fixPdf($data, 11));
    }

    public function testThrowsWhenMarkerTooFar(): void
    {
        $data = str_repeat('x', 10) . '%PDF-1.4';

        $this->expectException(\ValueError::class);
        $this->expectExceptionMessage("PDF couldn't be fixed");
        fixPdf($data, 10);
    }

    public function testThrowsWhenMissingMarker(): void
    {
        $this->expectException(\ValueError::class);
        $this->expectExceptionMessage("Provided stream isn't a valid PDF-like object");
        fixPdf('just some random text', 500);
    }

    public function testEmptyAndTinyInputsThrow(): void
    {
        $this->expectException(\ValueError::class);
        fixPdf('');
    }

    public function testIntegrationWithRealDocument(): void
    {
        $cleanPdf = $this->data('pdf/multipage.pdf');
        $garbageData = "GARBAGE-HEADER\r\n" . $cleanPdf;

        $fixedBytes = fixPdf($garbageData);

        $this->assertTrue(strlen((string)$fixedBytes) > 0);
        $this->assertEquals($cleanPdf, (string)$fixedBytes);
    }
}