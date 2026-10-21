#!/usr/bin/env python3
"""Namespace bundled JPEG symbols in a copy of a PDFium static archive (ELF, Mach-O or COFF)."""

import argparse
from collections import Counter
from pathlib import Path
import os
import shutil
import subprocess
import struct
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

# GUID identifying the COFF "bigobj" format, as stored in the file (little-endian).
BIGOBJ_CLASSID = bytes.fromhex("c7a1bad1eebaa94baf20faf66aa4dcb8")


def coff_symbol_table(obj):
    """(symbol table offset, symbol count, record size) for a COFF object, or None."""
    if len(obj) >= 56 and obj[:4] == b"\x00\x00\xff\xff":
        version = struct.unpack_from("<H", obj, 4)[0]
        if version >= 2 and obj[12:28] == BIGOBJ_CLASSID:
            pointer, count = struct.unpack_from("<II", obj, 48)
            return pointer, count, 20
        return None  # import object or unknown header
    if len(obj) >= 20 and struct.unpack_from("<H", obj, 0)[0] in (0x8664, 0xAA64, 0x14C):
        pointer, count = struct.unpack_from("<II", obj, 8)
        return pointer, count, 18
    return None


def rename_coff_symbols(obj, mapping):
    """Rename symbols in one COFF object. New names go at the end of the string table,
    so no code, section or relocation moves. Returns (new bytes, number renamed)."""
    table = coff_symbol_table(obj)
    if table is None:
        return obj, 0
    pointer, count, record = table
    if pointer == 0 or count == 0:
        return obj, 0
    strtab = pointer + count * record
    strsize = max(struct.unpack_from("<I", obj, strtab)[0], 4)
    if strtab + strsize != len(obj):
        raise RuntimeError("Unexpected data after the COFF string table; refusing to rewrite")

    out = bytearray(obj[:strtab])
    strings = bytearray(obj[strtab + 4 : strtab + strsize])
    renamed = 0
    index = 0
    while index < count:
        offset = pointer + index * record
        raw = obj[offset : offset + 8]
        if raw[:4] == b"\0\0\0\0":
            start = strtab + struct.unpack_from("<I", raw, 4)[0]
            name = obj[start : obj.index(b"\0", start)].decode("latin-1")
        else:
            name = raw.rstrip(b"\0").decode("latin-1")
        new = mapping.get(name)
        if new is not None:
            struct.pack_into("<II", out, offset, 0, 4 + len(strings))  # string table offset
            strings += new.encode("ascii") + b"\0"
            renamed += 1
        index += 1 + obj[offset + record - 1]  # skip auxiliary records
    out += struct.pack("<I", 4 + len(strings)) + strings
    return bytes(out), renamed


def rename_in_archive(source, destination, mapping):
    """Rewrite every COFF object in an ar archive. The symbol index is left stale
    on purpose; the caller rebuilds the archive with llvm-lib afterwards."""
    data = source.read_bytes()
    if not data.startswith(b"!<arch>\n"):
        raise RuntimeError(f"{source} is not an ar archive")
    out = bytearray(b"!<arch>\n")
    position, total = 8, 0
    while position < len(data):
        header = data[position : position + 60]
        if len(header) < 60 or header[58:60] != b"`\n":
            raise RuntimeError(f"Bad archive member header at offset {position}")
        name = header[:16].rstrip(b" ")
        size = int(header[48:58].decode().strip())
        body = data[position + 60 : position + 60 + size]
        position += 60 + size + (size & 1)
        # "/" = symbol index, "//" = long names, "/<...>" and "/SYM64/" = other special members.
        if not (name in (b"/", b"//", b"/SYM64/") or name.startswith(b"/<")):
            body, renamed = rename_coff_symbols(body, mapping)
            total += renamed
        out += header[:48] + f"{len(body):<10}".encode() + b"`\n" + body
        if len(body) & 1:
            out += b"\n"
    destination.write_bytes(out)
    return total

def namespace_archive(build_root, output_dir):
    build_root = build_root.resolve()
    output_dir = output_dir.resolve()
    is_windows = sys.platform == "win32"
    exe = ".exe" if is_windows else ""

    staging = build_root / "staging/lib"
    if is_windows:
        # pdfium-binaries doesn't stage static Windows builds; take the archive from the build output.
        candidates = [*staging.glob("*pdfium*.lib"), build_root / "pdfium/out/obj/pdfium.lib"]
    else:
        candidates = [staging / "libpdfium.a"]
    matches = [p for p in candidates if p.is_file()]
    if not matches:
        found = sorted(p.name for p in staging.iterdir()) if staging.is_dir() else []
        raise FileNotFoundError(
            f"No PDFium archive found; tried {[str(p) for p in candidates]}; staging/lib has: {found}"
        )
    source = matches[0]
    destination = output_dir / ("pdfium.lib" if is_windows else "libpdfium.a")

    bundled = build_root / "pdfium/third_party/llvm-build/Release+Asserts/bin"

    def find_tool(name):
        """PDFium's bundled LLVM tool if present, else one on PATH (LLVM_BIN overrides both)."""
        override = os.environ.get("LLVM_BIN")
        for candidate in ([Path(override) / f"{name}{exe}"] if override else []) + [bundled / f"{name}{exe}"]:
            if candidate.is_file():
                return candidate
        on_path = shutil.which(name)
        if on_path:
            return Path(on_path)
        raise FileNotFoundError(f"Required LLVM tool not found: {name} (looked in {bundled} and PATH)")

    jpeg_dir = build_root / "pdfium/out/obj/third_party/libjpeg_turbo"
    # Include libjpeg12/16 and the separate SIMD C/assembly archives.
    jpeg_archives = sorted(jpeg_dir.glob("*.lib" if is_windows else "*.a"))
    nm = find_tool("llvm-nm")
    ar = None if is_windows else find_tool("llvm-ar")
    objcopy = None if is_windows else find_tool("llvm-objcopy")
    lib_tool = find_tool("llvm-lib") if is_windows else None
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
        if is_windows:
            # llvm-objcopy drops the whole symbol table of NASM-assembled COFF objects,
            # so rename in place instead.
            rename_in_archive(source, candidate, mapping)
        else:
            subprocess.run(
                [str(objcopy), f"--redefine-syms={symbol_map}", str(source), str(candidate)],
                check=True,
            )
        if is_windows:
            # Rebuild the archive in Microsoft's format (both linker members), which link.exe
            # requires; llvm-ar's regenerated index isn't accepted by every link.exe version.
            renamed_lib = tmp / "renamed.lib"
            candidate.replace(renamed_lib)
            subprocess.run([str(lib_tool), "/NOLOGO", f"/OUT:{candidate}", str(renamed_lib)], check=True)
        else:
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
