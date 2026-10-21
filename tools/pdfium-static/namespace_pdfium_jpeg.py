#!/usr/bin/env python3
"""Prefix the libjpeg-turbo symbols bundled in a static PDFium library.

PDFium and mozjpeg both contain libjpeg-turbo, so linking both into one binary clashes on symbols
such as jpeg_* and jsimd_*. This writes a copy of PDFium's archive in which its libjpeg-turbo
symbols are renamed to bernard_pdfium_<name>, then checks that nothing else changed.

ELF and Mach-O archives are rewritten with llvm-objcopy. COFF archives (Windows) are rewritten by
this script, because llvm-objcopy drops the symbol table of NASM-assembled COFF objects.
"""

import argparse
from collections import Counter
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile

PREFIX = "bernard_pdfium_"
MARKER_SYMBOL = "jpeg_natural_order"
IS_WINDOWS = sys.platform == "win32"

AR_MAGIC = b"!<arch>\n"
AR_HEADER_SIZE = 60
AR_INDEX_MEMBERS = (b"/", b"//", b"/SYM64/")
COFF_MACHINES = (0x8664, 0xAA64, 0x14C)
COFF_BIGOBJ_CLASSID = bytes.fromhex("c7a1bad1eebaa94baf20faf66aa4dcb8")


def find_tool(name, bundled_dir):
    """Return PDFium's bundled LLVM tool `name`, or the one on PATH."""
    bundled = bundled_dir / (f"{name}.exe" if IS_WINDOWS else name)
    if bundled.is_file():
        return bundled
    on_path = shutil.which(name)
    if on_path:
        return Path(on_path)
    raise FileNotFoundError(f"{name} not found in {bundled_dir} or on PATH")


def read_symbols(nm, archives, defined_only=False):
    """Count the (name, type) pairs of the external symbols in `archives`."""
    command = [str(nm), "--extern-only", "--format=posix"]
    if defined_only:
        command.append("--defined-only")
    output = subprocess.check_output(command + [str(path) for path in archives], text=True)
    counts = Counter()
    for line in output.splitlines():
        if not line.strip() or line.endswith(":"):
            continue
        fields = line.split()
        if len(fields) < 2 or len(fields[1]) != 1:
            raise RuntimeError(f"Unexpected llvm-nm output: {line}")
        counts[(fields[0], fields[1])] += 1
    return counts


def locate_inputs(build_root):
    """Return the PDFium archive and libjpeg-turbo's component archives of a pdfium-binaries build.

    pdfium-binaries doesn't stage static Windows builds, so there the archive comes from the build
    output. The component archives tell which symbols belong to libjpeg-turbo.
    """
    if IS_WINDOWS:
        source = build_root / "pdfium/out/obj/pdfium.lib"
    else:
        source = build_root / "staging/lib/libpdfium.a"
    if not source.is_file():
        raise FileNotFoundError(f"PDFium archive not found: {source}")
    jpeg_dir = build_root / "pdfium/out/obj/third_party/libjpeg_turbo"
    jpeg_archives = sorted(jpeg_dir.glob("*.lib" if IS_WINDOWS else "*.a"))
    if not jpeg_archives:
        raise FileNotFoundError(f"No libjpeg-turbo archives in {jpeg_dir}")
    return source, jpeg_archives


def build_symbol_map(nm, source, jpeg_archives, original):
    """Map each libjpeg-turbo symbol defined in `source` to its prefixed name.

    Mach-O symbols start with an underscore, which stays in front so that the C name becomes
    bernard_pdfium_<name> on every platform.
    """
    defined = {name for name, _ in read_symbols(nm, [source], defined_only=True)}
    jpeg = {name for name, _ in read_symbols(nm, jpeg_archives, defined_only=True)}
    names = jpeg & defined
    underscore = "_" if "_" + MARKER_SYMBOL in names else ""
    if underscore + MARKER_SYMBOL not in names:
        raise RuntimeError(
            f"{MARKER_SYMBOL} is not defined in {source}: use the original archive "
            "and the component archives of the same build"
        )
    if any(name.removeprefix(underscore).startswith("FPDF") for name in names):
        raise RuntimeError("The libjpeg-turbo archives unexpectedly define PDFium API symbols")
    mapping = {
        name: underscore + PREFIX + name.removeprefix(underscore) for name in sorted(names)
    }
    if set(mapping.values()) & {name for name, _ in original}:
        raise RuntimeError(f"{source} already contains {PREFIX}* symbols")
    return mapping


def coff_symbol_table(obj):
    """Return (offset, symbol count, record size) of a COFF object's symbol table, or None."""
    if obj[:4] == b"\x00\x00\xff\xff":
        is_bigobj = (
                len(obj) >= 56
                and struct.unpack_from("<H", obj, 4)[0] >= 2
                and obj[12:28] == COFF_BIGOBJ_CLASSID
        )
        if not is_bigobj:
            return None
        offset, count = struct.unpack_from("<II", obj, 48)
        return offset, count, 20
    if len(obj) >= 20 and struct.unpack_from("<H", obj, 0)[0] in COFF_MACHINES:
        offset, count = struct.unpack_from("<II", obj, 8)
        return offset, count, 18
    return None


def coff_symbol_name(obj, record, string_table):
    """Return the name of the symbol record at `record`; long names live in the string table."""
    raw = obj[record : record + 8]
    if raw[:4] == b"\0\0\0\0":
        start = string_table + struct.unpack_from("<I", raw, 4)[0]
        return obj[start : obj.index(b"\0", start)].decode("latin-1")
    return raw.rstrip(b"\0").decode("latin-1")


def rename_coff_symbols(obj, mapping):
    """Rename the mapped symbols of one COFF object; other archive members are returned unchanged.

    New names are appended to the string table and the symbol records point at them, so no
    section, relocation or symbol index moves.
    """
    table = coff_symbol_table(obj)
    if table is None or 0 in table[:2]:
        return obj
    offset, count, record_size = table
    string_table = offset + count * record_size
    string_table_size = max(struct.unpack_from("<I", obj, string_table)[0], 4)
    if string_table + string_table_size != len(obj):
        raise RuntimeError("Unexpected data after a COFF string table; refusing to rewrite it")

    out = bytearray(obj[:string_table])
    strings = bytearray(obj[string_table + 4 : string_table + string_table_size])
    index = 0
    while index < count:
        record = offset + index * record_size
        new_name = mapping.get(coff_symbol_name(obj, record, string_table))
        if new_name is not None:
            struct.pack_into("<II", out, record, 0, 4 + len(strings))
            strings += new_name.encode("ascii") + b"\0"
        index += 1 + obj[record + record_size - 1]
    out += struct.pack("<I", 4 + len(strings)) + strings
    return bytes(out)


def rename_in_coff_archive(source, destination, mapping):
    """Copy an ar archive, renaming the mapped symbols in every COFF member.

    The index members are copied unchanged, so they go stale until the archive is rebuilt.
    """
    data = source.read_bytes()
    if not data.startswith(AR_MAGIC):
        raise RuntimeError(f"{source} is not an ar archive")
    out = bytearray(AR_MAGIC)
    position = len(AR_MAGIC)
    while position < len(data):
        header = data[position : position + AR_HEADER_SIZE]
        if len(header) < AR_HEADER_SIZE or header[58:60] != b"`\n":
            raise RuntimeError(f"Bad archive member header at offset {position}")
        name = header[:16].rstrip(b" ")
        size = int(header[48:58].decode().strip())
        body = data[position + AR_HEADER_SIZE : position + AR_HEADER_SIZE + size]
        position += AR_HEADER_SIZE + size + size % 2
        if name not in AR_INDEX_MEMBERS and not name.startswith(b"/<"):
            body = rename_coff_symbols(body, mapping)
        out += header[:48] + f"{len(body):<10}".encode() + b"`\n" + body
        if len(body) % 2:
            out += b"\n"
    destination.write_bytes(out)


def rewrite_with_objcopy(llvm_bin, source, destination, mapping, workdir):
    """Rename the mapped symbols of an ELF or Mach-O archive, then rebuild its index."""
    symbol_map = workdir / "jpeg-symbols.map"
    symbol_map.write_text(
        "".join(f"{old} {new}\n" for old, new in mapping.items()), encoding="ascii"
    )
    objcopy = find_tool("llvm-objcopy", llvm_bin)
    subprocess.run(
        [str(objcopy), f"--redefine-syms={symbol_map}", str(source), str(destination)],
        check=True,
    )
    subprocess.run([str(find_tool("llvm-ar", llvm_bin)), "s", str(destination)], check=True)


def rewrite_coff(llvm_bin, source, destination, mapping, workdir):
    """Rename the mapped symbols of a COFF archive, then rebuild it with llvm-lib.

    llvm-lib writes both index members in the layout link.exe expects; some link.exe versions
    reject an index regenerated by llvm-ar (LNK1127).
    """
    renamed = workdir / "renamed.lib"
    rename_in_coff_archive(source, renamed, mapping)
    llvm_lib = find_tool("llvm-lib", llvm_bin)
    subprocess.run([str(llvm_lib), "/NOLOGO", f"/OUT:{destination}", str(renamed)], check=True)


def verify(nm, candidate, original, mapping):
    """Check that only the mapped symbols changed, in definitions and references alike."""
    expected = Counter()
    for (name, kind), count in original.items():
        expected[(mapping.get(name, name), kind)] += count
    actual = read_symbols(nm, [candidate])
    if actual != expected:
        raise RuntimeError(
            f"Symbol verification failed: missing={expected - actual}, "
            f"unexpected={actual - expected}"
        )


def namespace_archive(build_root, output_dir):
    """Write the namespaced copy of the PDFium archive of `build_root` into `output_dir`."""
    build_root = build_root.resolve()
    output_dir = output_dir.resolve()
    source, jpeg_archives = locate_inputs(build_root)
    destination = output_dir / source.name
    if destination == source:
        raise ValueError("The output directory must not be the directory of the original archive")

    llvm_bin = build_root / "pdfium/third_party/llvm-build/Release+Asserts/bin"
    nm = find_tool("llvm-nm", llvm_bin)
    original = read_symbols(nm, [source])
    mapping = build_symbol_map(nm, source, jpeg_archives, original)

    output_dir.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix=".jpeg-namespace-", dir=output_dir) as workdir:
        workdir = Path(workdir)
        candidate = workdir / source.name
        rewrite = rewrite_coff if IS_WINDOWS else rewrite_with_objcopy
        rewrite(llvm_bin, source, candidate, mapping, workdir)
        verify(nm, candidate, original, mapping)
        candidate.replace(destination)
    print(f"Namespaced {len(mapping)} libjpeg-turbo symbols: {destination}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("build_root", type=Path, help="pdfium-binaries checkout after a build")
    parser.add_argument("output_dir", type=Path, help="directory for the namespaced archive")
    args = parser.parse_args()
    namespace_archive(args.build_root, args.output_dir)


if __name__ == "__main__":
    main()
