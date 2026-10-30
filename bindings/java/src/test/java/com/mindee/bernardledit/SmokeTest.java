package com.mindee.bernardledit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;
import org.junit.jupiter.api.Test;

/** Calls each exported function once, from Java, through the generated Kotlin bindings. */
class SmokeTest {
  private static final Path FILE_TYPES = Path.of("..", "..", "tests", "data", "file_types");
  private static final List<Point> FULL_PAGE =
      List.of(new Point(0.0, 0.0), new Point(1.0, 0.0), new Point(1.0, 1.0), new Point(0.0, 1.0));

  private static byte[] read(String name) throws IOException {
    return Files.readAllBytes(FILE_TYPES.resolve(name));
  }

  private static byte[] head(byte[] data, int length) {
    return Arrays.copyOf(data, length);
  }

  @Test
  void decodesAndEncodesImages() throws Exception {
    byte[] jpeg = read("receipt.jpg");
    assertEquals("JPEG", BernardLedit.guessFormat(jpeg));
    try (Image image = new Image(jpeg)) {
      assertTrue(image.width() > 0);
      assertEquals("JPEG", image.format());
      byte[] png = image.encode("PNG", 85);
      assertArrayEquals("\u0089PNG".getBytes(StandardCharsets.ISO_8859_1), head(png, 4));
    }
  }

  @Test
  void compressesImages() throws Exception {
    CompressedImage compressed = BernardLedit.compressImage(read("receipt.jpg"), 85, 100L, null);
    assertTrue(compressed.getWidth() <= 100);
  }

  @Test
  void reportsErrorsWithTheirKind() {
    BernardLeditException tooHigh =
        assertThrows(
            BernardLeditException.InvalidArgument.class,
            () -> BernardLedit.compressImage(read("receipt.jpg"), 300, null, null));
    assertEquals("quality must be between 5 and 100, got 300", tooHigh.getMessage());
    assertThrows(
        BernardLeditException.Image.class, () -> BernardLedit.guessFormat(read("receipt.heic")));
  }

  @Test
  void readsAndSplitsPdfs() throws Exception {
    byte[] pdf = read("pdf/multipage.pdf");
    try (PdfDocument doc = new PdfDocument(pdf)) {
      assertEquals(12, doc.pageCount());
    }
    List<PdfDocument> parts =
        BernardLedit.extractFromPageRanges(pdf, List.of(List.of(0), List.of(1, 2)));
    assertEquals(2, parts.size());
    assertEquals(1, parts.get(0).pageCount());
    assertEquals(2, parts.get(1).pageCount());
    List<Image> crops = BernardLedit.extractFromPolygons(pdf, 0, List.of(FULL_PAGE), 72.0);
    assertEquals("JPEG", crops.get(0).format());
  }

  @Test
  void fixesAndCompressesPdfs() throws Exception {
    byte[] pdf = read("pdf/multipage.pdf");
    byte[] junk = "junk".getBytes(StandardCharsets.US_ASCII);
    byte[] prefixed = new byte[junk.length + pdf.length];
    System.arraycopy(junk, 0, prefixed, 0, junk.length);
    System.arraycopy(pdf, 0, prefixed, junk.length, pdf.length);
    assertArrayEquals(head(pdf, 5), head(BernardLedit.fixPdf(prefixed, 500L), 5));
    byte[] scan = read("pdf/not_blank_image_only.pdf");
    assertArrayEquals(head(pdf, 5), head(BernardLedit.compressPdf(scan, 85, 150.0), 5));
  }
}
