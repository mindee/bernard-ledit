// Calls each exported function once through the generated C# bindings.

using System;
using System.IO;
using System.Linq;
using System.Text;
using Mindee.BernardLedit;
using Xunit;

public class SmokeTest
{
    private static readonly Point[] FullPage =
    {
        new Point(0.0, 0.0), new Point(1.0, 0.0), new Point(1.0, 1.0), new Point(0.0, 1.0),
    };

    private static byte[] Read(string name)
    {
        var dir = new DirectoryInfo(AppContext.BaseDirectory);
        while (dir != null && !Directory.Exists(Path.Combine(dir.FullName, "tests", "data", "file_types")))
        {
            dir = dir.Parent;
        }
        if (dir == null)
        {
            throw new DirectoryNotFoundException("tests/data/file_types not found above the test assembly");
        }
        return File.ReadAllBytes(Path.Combine(dir.FullName, "tests", "data", "file_types", name));
    }

    private static string Head(byte[] data, int length) =>
        Encoding.Latin1.GetString(data, 0, length);

    [Fact]
    public void DecodesAndEncodesImages()
    {
        var jpeg = Read("receipt.jpg");
        Assert.Equal("JPEG", BernardLedit.GuessFormat(jpeg));
        using var image = new Image(jpeg);
        Assert.True(image.Width() > 0);
        Assert.Equal("JPEG", image.Format());
        Assert.Equal("\u0089PNG", Head(image.Encode("PNG", 85), 4));
    }

    [Fact]
    public void CompressesImages()
    {
        var compressed = BernardLedit.CompressImage(Read("receipt.jpg"), 85, 100, null);
        Assert.True(compressed.Width <= 100);
    }

    [Fact]
    public void ReportsErrorsWithTheirKind()
    {
        var tooHigh = Assert.Throws<BernardLeditException.InvalidArgument>(
            () => BernardLedit.CompressImage(Read("receipt.jpg"), 300, null, null));
        Assert.Contains("quality must be between 5 and 100, got 300", tooHigh.Message);
        Assert.Throws<BernardLeditException.Image>(() => BernardLedit.GuessFormat(Read("receipt.heic")));
    }

    [Fact]
    public void ReadsAndSplitsPdfs()
    {
        var pdf = Read("pdf/multipage.pdf");
        using (var doc = new PdfDocument(pdf))
        {
            Assert.Equal(12, doc.PageCount());
        }
        var parts = BernardLedit.ExtractFromPageRanges(pdf, new[] { new[] { 0 }, new[] { 1, 2 } });
        Assert.Equal(new[] { 1, 2 }, parts.Select(part => part.PageCount()));
        var crops = BernardLedit.ExtractFromPolygons(pdf, 0, new[] { FullPage }, 72.0);
        Assert.Equal("JPEG", crops[0].Format());
    }

    [Fact]
    public void FixesAndCompressesPdfs()
    {
        var pdf = Read("pdf/multipage.pdf");
        var prefixed = Encoding.ASCII.GetBytes("junk").Concat(pdf).ToArray();
        Assert.Equal("%PDF-", Head(BernardLedit.FixPdf(prefixed, 500), 5));
        Assert.Equal("%PDF-", Head(BernardLedit.CompressPdf(Read("pdf/not_blank_image_only.pdf"), 85, 150.0), 5));
    }
}
