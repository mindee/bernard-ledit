"""Place the pinned native libraries next to a built extension."""

import argparse
import ctypes
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path


def command(*args: str) -> str:
    return subprocess.run(args, check=True, text=True, stdout=subprocess.PIPE).stdout


def bundle(
    extension: Path, heif: Path, pdfium: Path, *, windows_flat: bool = False
) -> None:
    expected_pdfium = (
        (Path(__file__).resolve().parent.parent / "core" / "pdfium-release")
        .read_text(encoding="utf-8")
        .strip()
    )
    version = (pdfium / "VERSION").read_text(encoding="utf-8")
    builds = re.findall(r"^BUILD=(\d+)$", version, re.MULTILINE)
    if builds != [expected_pdfium.rsplit("/", 1)[-1]]:
        raise ValueError(f"expected PDFium {expected_pdfium}, found {builds}")

    if sys.platform == "darwin":
        heif_name, pdfium_name = "libheif.1.dylib", "libpdfium.dylib"
        heif_path = heif / "lib" / heif_name
        pdfium_path = pdfium / "lib" / pdfium_name
    elif sys.platform.startswith("linux"):
        heif_name, pdfium_name = "libheif.so.1", "libpdfium.so"
        heif_path = heif / "lib" / heif_name
        pdfium_path = pdfium / "lib" / pdfium_name
    elif sys.platform == "win32":
        candidates = list((heif / "bin").glob("*heif*.dll"))
        if len(candidates) != 1:
            raise ValueError(f"expected exactly one libheif DLL in {heif / 'bin'}")
        heif_path = candidates[0]
        heif_name, pdfium_name = heif_path.name, "pdfium.dll"
        pdfium_path = pdfium / "bin" / pdfium_name
    else:
        raise ValueError(f"unsupported platform: {sys.platform}")

    if sys.platform == "win32":
        dll_directory = os.add_dll_directory(str(heif_path.parent.resolve()))
    library = ctypes.CDLL(str(heif_path.resolve()))
    library.heif_get_version.restype = ctypes.c_char_p
    version = library.heif_get_version()
    if version != b"1.21.1":
        raise ValueError(f"expected libheif 1.21.1, found {version!r}")

    if sys.platform == "darwin":
        heif_dependencies = "\n".join(
            command("otool", "-L", str(heif_path)).splitlines()[1:]
        )
    elif sys.platform.startswith("linux"):
        heif_dependencies = "\n".join(
            line
            for line in command("readelf", "-d", str(heif_path)).splitlines()
            if "(NEEDED)" in line
        )
    else:
        inspector = shutil.which("llvm-readobj")
        if inspector:
            output = command(inspector, "--coff-imports", str(heif_path))
            heif_dependencies = "\n".join(
                line for line in output.splitlines() if re.match(r"\s*Name:", line)
            )
        else:
            inspector = shutil.which("dumpbin")
            if not inspector:
                raise RuntimeError("llvm-readobj or dumpbin is required on Windows")
            output = command(inspector, "/DEPENDENTS", str(heif_path))
            heif_dependencies = "\n".join(
                line
                for line in output.splitlines()
                if re.fullmatch(r"\s*[\w.-]+\.dll\s*", line, re.IGNORECASE)
            )
        if not heif_dependencies:
            raise ValueError(f"could not inspect imports of {heif_path}")
    if re.search(
        r"de265|avcodec|avutil|swscale",
        heif_dependencies,
        re.IGNORECASE,
    ):
        raise ValueError("libheif must not link to an HEVC decoder")

    if sys.platform == "win32" and windows_flat:
        native = extension.parent
    else:
        native = extension.parent / "native"
        native.mkdir(parents=True)
    shutil.copy2(heif_path, native / heif_name)
    shutil.copy2(pdfium_path, native / pdfium_name)
    licenses = native / "licenses"
    licenses.mkdir()
    shutil.copyfile(heif / "libheif-LICENSE", licenses / "libheif.txt")
    shutil.copyfile(pdfium / "LICENSE", licenses / "pdfium.txt")
    shutil.copytree(pdfium / "licenses", licenses / "pdfium")

    if sys.platform == "darwin":
        bundled_heif = native / heif_name
        command("install_name_tool", "-id", f"@rpath/{heif_name}", str(bundled_heif))
        dependencies = command("otool", "-L", str(extension)).splitlines()[1:]
        changes: list[str] = []
        for name in (heif_name, pdfium_name):
            matches = [
                line.strip().split(" (", 1)[0]
                for line in dependencies
                if Path(line.strip().split(" (", 1)[0]).name == name
            ]
            if len(matches) != 1:
                raise ValueError(f"expected one dependency named {name}: {matches}")
            changes.extend(("-change", matches[0], f"@loader_path/native/{name}"))
        load_commands = command("otool", "-l", str(extension))
        rpaths = re.findall(
            r"cmd LC_RPATH\s+cmdsize \d+\s+path (.+?) \(offset \d+\)",
            load_commands,
        )
        for rpath in rpaths:
            if rpath.startswith("/"):
                changes.extend(("-delete_rpath", rpath))
        if "@loader_path/native" not in rpaths:
            changes.extend(("-add_rpath", "@loader_path/native"))
        command("install_name_tool", *changes, str(extension))
        command("codesign", "--force", "--sign", "-", str(bundled_heif))
        command("codesign", "--force", "--sign", "-", str(extension))
    elif sys.platform.startswith("linux"):
        dependencies = command(
            "patchelf", "--print-needed", str(extension)
        ).splitlines()
        for name in (heif_name, pdfium_name):
            if dependencies.count(name) != 1:
                raise ValueError(f"expected dependency {name}: {dependencies}")
        command("patchelf", "--set-rpath", "$ORIGIN/native", str(extension))
    if sys.platform == "win32":
        dll_directory.close()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("extension", type=Path)
    parser.add_argument("heif_prefix", type=Path)
    parser.add_argument("pdfium_prefix", type=Path)
    parser.add_argument("--windows-flat", action="store_true")
    args = parser.parse_args()
    bundle(
        args.extension,
        args.heif_prefix,
        args.pdfium_prefix,
        windows_flat=args.windows_flat,
    )


if __name__ == "__main__":
    main()
