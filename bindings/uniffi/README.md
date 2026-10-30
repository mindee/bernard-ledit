# UniFFI bindings: Java, .NET, Node.js and Ruby

One Rust crate, `bernard-ledit-uniffi`, exports bernard-ledit's API with
[UniFFI](https://github.com/mozilla/uniffi-rs). Each language package generates its code from the
compiled library with its own generator, so adding a function here makes it available in all four.

| Package           | Generator                                                                                                        | Runtime it needs                           |
|-------------------|------------------------------------------------------------------------------------------------------------------|--------------------------------------------|
| `bindings/java`   | UniFFI's built-in Kotlin generator (this crate's `uniffi-bindgen` binary), called from Java                      | Java 11+, `kotlin-stdlib`, JNA             |
| `bindings/dotnet` | [uniffi-bindgen-cs](https://github.com/NordSecurity/uniffi-bindgen-cs) `v0.11.1+v0.31.0`                         | .NET 8+ or .NET Framework 4.6.1+           |
| `bindings/nodejs` | [uniffi-bindgen-react-native](https://github.com/jhugman/uniffi-bindgen-react-native) `0.31.0-6` (`ubrn`), N-API | Node.js 20+, `@ubjs/node` and `@ubjs/core` |
| `bindings/ruby`   | UniFFI's built-in Ruby generator (this crate's `uniffi-bindgen` binary)                                          | Ruby 3.2+, the `ffi` gem                   |

PHP and Python keep their hand-written bindings (ext-php-rs and PyO3).

## Versions

The crate pins `uniffi = "=0.31.2"`. uniffi-bindgen-cs v0.11.x and uniffi-bindgen-react-native 0.31.0-x
stop at UniFFI 0.31, and a generator only reads libraries built with its own UniFFI minor version.
To upgrade, bump all of these together:

- `uniffi` in `Cargo.toml`;
- `bindgen_cs_tag` in `bindings/dotnet/justfile`;
- `uniffi-bindgen-react-native`, `@ubjs/node` and `@ubjs/core` in `bindings/nodejs/package.json`.

## Why Java goes through Kotlin

[uniffi-bindgen-java](https://github.com/IronCoreLabs/uniffi-bindgen-java) generates idiomatic Java, but its
bindings use the Foreign Function & Memory API and need Java 22+, while mindee-api-java supports Java 11.
UniFFI's Kotlin bindings use JNA, compile for Java 11, and Java calls them directly:

```java
import com.mindee.bernardledit.BernardLedit;  // the free functions
import com.mindee.bernardledit.PdfDocument;

byte[] fixed = BernardLedit.fixPdf(bytes, 500L);
try (PdfDocument doc = new PdfDocument(fixed)) {
    int pages = doc.pageCount();
}
```

`just java generate` adds `@file:JvmName("BernardLedit")` to the generated file, so the free
functions live in `BernardLedit` instead of `Bernard_leditKt`. Kotlin default arguments aren't visible
from Java, so Java passes every argument. If the SDK moves to Java 22+, uniffi-bindgen-java is a drop-in
replacement for the generation step.

## Writing the API

- **Signed integers only.** Kotlin turns `u8`/`u16`/`u32` into inline classes, whose methods Java
  can't call. Take `i32` or `i64` and narrow with `args.rs`, which also gives every language the same
  `InvalidArgument` messages as the PHP binding (`quality must be between 5 and 100, got 300`).
- **One error type.** Every function returns `BernardLeditError` (`error.rs`), mapped from core's errors
  the same way the PHP binding maps them: `InvalidArgument` (PHP's `\ValueError`), `Image`, `Pdfium`,
  `Other`. Kotlin and C# call it `BernardLeditException`.
- **Document the errors.** Each language's generator copies the Rust doc comments into its bindings.
- **No async, no callbacks.** Kotlin's async support needs `kotlinx-coroutines`, and the Ruby generator
  supports neither.

## PDFium

Every PDF function first binds PDFium: a dynamic build loads the library `PDFIUM_PATH` names (the
packages' `test` recipes set it), and a static build (the `static-pdfium*` features, as for PHP) binds
the linked one. A package that ships its own PDFium calls `bindPdfium(path)` / `bind_pdfium(path)`
first.

## Layout and commands

```text
bindings/uniffi/   the crate, uniffi.toml (per-language names), justfile (build, generate, paths)
bindings/java/     Maven project compiling the generated Kotlin; Java smoke tests (JUnit)
bindings/dotnet/   Mindee.BernardLedit (net8.0, net472) and xUnit smoke tests
bindings/nodejs/   @mindee/bernard-ledit, TypeScript; node:test smoke tests
bindings/ruby/     the bernard_ledit gem; Minitest smoke tests
```

The generated code isn't committed: each package's `just <package> generate` writes it from the debug
library, and `just <package> test` (or `ci`) generates, then runs the smoke tests against the debug
library and the dynamic PDFium. `just uniffi lint` and `just uniffi test` check the crate itself.

## Not done yet

- **Packaging the native library.** Each package needs a release build of `bernard_ledit_uniffi` per
  platform, linked against static PDFium like the PHP releases, and shipped the way its ecosystem
  expects:
    - Java: JNA's resource prefixes in the jar (`linux-x86-64/`, `darwin-aarch64/`, …);
    - .NET: `runtimes/<rid>/native/` in the NuGet package;
    - Node.js: one npm package per platform, with `--lib-package-base` instead of `--lib-colocated`;
    - Ruby: platform gems, with the library next to `bernard_ledit.rb`. The generated `ffi_lib` already
      looks there first.
- **The rest of the API.** This first slice covers what the SDKs use most: images (decode, encode,
  `guessFormat`, `compressImage`, `extractFromPolygons`), PDFs (`PdfDocument`, `compressPdf`,
  `extractFromPageRanges`, `fixPdf`). Page options, text characters, page rendering, crop and resize
  are still PHP- and Python-only.
- **Windows.** The justfiles are bash and CI runs Linux and macOS only.
- **Release workflows** for Maven Central, NuGet, npm and RubyGems.
