#!/usr/bin/env python3
"""Namespace bundled JPEG symbols in a copy of a PDFium static archive (ELF or Mach-O)."""

import argparse
from collections import Counter
from pathlib import Path
import subprocess
import sys
import tempfile


PREFIX = "bernard_pdfium_"


def symbols(nm, archives, defined_only=False):
    command = [str(nm), "--extern-only", "--format=posix"]
    if defined_only:
        command.append("--defined-only")
    output = subprocess.check_output(
        command + [str(path) for path in archives], text=True
    )
    result = Counter()
    for line in output.splitlines():
        if not line.strip() or line.endswith(":"):
            continue
        fields = line.split()
        if len(fields) < 2 or len(fields[1]) != 1:
            raise RuntimeError(f"Unexpected llvm-nm output: {line}")
        result[(fields[0], fields[1])] += 1
    return result


def namespace_archive(build_root, output_dir):
    build_root = build_root.resolve()
    output_dir = output_dir.resolve()
    is_windows = sys.platform == "win32"
    exe = ".exe" if is_windows else ""
    lib = "pdfium.lib" if is_windows else "libpdfium.a"
    source = build_root / "staging/lib" / lib
    destination = output_dir / lib
    tools = build_root / "pdfium/third_party/llvm-build/Release+Asserts/bin"
    nm, objcopy, ar = (tools / f"{name}{exe}" for name in ("llvm-nm", "llvm-objcopy", "llvm-ar"))
    jpeg_dir = build_root / "pdfium/out/obj/third_party/libjpeg_turbo"
    jpeg_archives = sorted(jpeg_dir.glob("*.lib" if is_windows else "*.a"))
    for path in (source, nm, objcopy, ar):
        if not path.is_file():
            raise FileNotFoundError(f"Required PDFium build artifact/tool missing: {path}")
    if not jpeg_archives:
        raise RuntimeError(f"No JPEG component archives in {jpeg_dir}")
    if destination == source or (
        destination.exists() and destination.samefile(source)
    ):
        raise ValueError("Output must not overwrite the original PDFium archive")

    original = symbols(nm, [source])
    defined = symbols(nm, [source], defined_only=True)
    jpeg = symbols(nm, jpeg_archives, defined_only=True)
    jpeg_names = {name for name, _ in jpeg}
    names = jpeg_names & {name for name, _ in defined}
    # Mach-O (macOS) prefixes C symbols with "_"; ELF (Linux) doesn't.
    underscore = "_" if "_jpeg_natural_order" in names else ""
    if underscore + "jpeg_natural_order" not in names:
        raise RuntimeError(
            "Expected unmodified jpeg_natural_order definition is missing; "
            "use the original archive and component archives from the same build"
        )
    all_names = {name for name, _ in original}
    # Keep the platform prefix in front, so the C-level name becomes bernard_pdfium_<name> everywhere.
    mapping = {
        name: underscore + PREFIX + name.removeprefix(underscore) for name in sorted(names)
    }
    if set(mapping.values()) & all_names:
        raise RuntimeError("Archive already contains destination-prefixed symbols")
    if any(name.removeprefix(underscore).startswith("FPDF") for name in names):
        raise RuntimeError("JPEG component archives unexpectedly define PDFium API symbols")

    expected = Counter()
    for (name, kind), count in original.items():
        expected[(mapping.get(name, name), kind)] += count

    output_dir.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".jpeg-namespace-", dir=output_dir) as tmp:
        tmp = Path(tmp)
        symbol_map = tmp / "jpeg-symbols.map"
        symbol_map.write_text(
            "".join(f"{old} {new}\n" for old, new in mapping.items()), encoding="ascii"
        )
        candidate = tmp / "libpdfium.a"
        subprocess.run(
            [str(objcopy), f"--redefine-syms={symbol_map}", str(source), str(candidate)],
            check=True,
        )
        subprocess.run([str(ar), "s", str(candidate)], check=True)
        # Check definitions AND cross-member references. Everything outside the map,
        # including FPDF_* and unresolved system-library symbols, must stay unchanged.
        actual = symbols(nm, [candidate])
        if actual != expected:
            raise RuntimeError(
                f"Symbol verification failed: missing={expected - actual}, "
                f"unexpected={actual - expected}"
            )
        if names & {name for name, _ in actual}:
            raise RuntimeError("Original JPEG symbols remain in transformed archive")
        symbol_map.replace(output_dir / "jpeg-symbols.map")
        candidate.replace(destination)
    print(f"Namespaced {len(mapping)} JPEG symbols: {destination}")
    return destination


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build_root", type=Path, help="Built pdfium-binaries checkout")
    parser.add_argument("output_dir", type=Path, help="Separate, writable output directory")
    args = parser.parse_args()
    namespace_archive(args.build_root, args.output_dir)


if __name__ == "__main__":
    main()
