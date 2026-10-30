// Calls each exported function once through the generated TypeScript bindings.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  BernardLeditError,
  compressImage,
  compressPdf,
  extractFromPageRanges,
  extractFromPolygons,
  fixPdf,
  guessFormat,
  Image,
  PdfDocument,
} from "../generated/index.ts";

const FILE_TYPES = new URL("../../../tests/data/file_types/", import.meta.url);
const FULL_PAGE = [
  { x: 0, y: 0 },
  { x: 1, y: 0 },
  { x: 1, y: 1 },
  { x: 0, y: 1 },
];

function read(name: string): ArrayBuffer {
  const bytes = readFileSync(new URL(name, FILE_TYPES));
  return bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength);
}

function head(data: ArrayBuffer, length: number): string {
  return Buffer.from(data.slice(0, length)).toString("latin1");
}

test("decodes and encodes images", () => {
  const jpeg = read("receipt.jpg");
  assert.equal(guessFormat(jpeg), "JPEG");
  const image = new Image(jpeg);
  assert.ok(image.width() > 0n);
  assert.equal(image.format(), "JPEG");
  assert.equal(head(image.encode("PNG", 85), 4), "\x89PNG");
});

test("compresses images", () => {
  const compressed = compressImage(read("receipt.jpg"), 85, 100n, undefined);
  assert.ok(compressed.width <= 100n);
});

test("reports errors with their kind", () => {
  assert.throws(
    () => compressImage(read("receipt.jpg"), 300, undefined, undefined),
    (e: unknown) =>
      BernardLeditError.InvalidArgument.instanceOf(e) &&
      e.message.includes("quality must be between 5 and 100, got 300"),
  );
  assert.throws(() => guessFormat(read("receipt.heic")), BernardLeditError.Image.instanceOf);
});

test("reads and splits PDFs", () => {
  const pdf = read("pdf/multipage.pdf");
  assert.equal(new PdfDocument(pdf).pageCount(), 12);
  const parts = extractFromPageRanges(pdf, [[0], [1, 2]]);
  assert.deepEqual(
    parts.map((part) => part.pageCount()),
    [1, 2],
  );
  const [crop] = extractFromPolygons(pdf, 0, [FULL_PAGE], 72);
  assert.equal(crop.format(), "JPEG");
});

test("fixes and compresses PDFs", () => {
  const pdf = read("pdf/multipage.pdf");
  const prefixed = Buffer.concat([Buffer.from("junk"), Buffer.from(pdf)]);
  assert.equal(head(fixPdf(prefixed.buffer.slice(prefixed.byteOffset), 500n), 5), "%PDF-");
  assert.equal(head(compressPdf(read("pdf/not_blank_image_only.pdf"), 85, 150), 5), "%PDF-");
});
