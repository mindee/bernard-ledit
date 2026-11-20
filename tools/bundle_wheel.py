"""Bundle native libraries in a maturin wheel without renaming them."""

import argparse
import base64
import csv
import hashlib
import io
import re
import sys
import tempfile
import zipfile
from pathlib import Path

from bundle_native import bundle, command


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("wheel", type=Path)
    parser.add_argument("output_directory", type=Path)
    parser.add_argument("heif_prefix", type=Path)
    parser.add_argument("pdfium_prefix", type=Path)
    args = parser.parse_args()
    destination = args.output_directory / args.wheel.name
    if args.wheel.resolve() == destination.resolve():
        parser.error("output wheel must differ from the input wheel")

    with tempfile.TemporaryDirectory() as temporary:
        root = Path(temporary)
        with zipfile.ZipFile(args.wheel) as archive:
            archive.extractall(root)
        package = root / "bernard_ledit"
        extensions = list(package.glob("_bernard_ledit*.so")) + list(
            package.glob("_bernard_ledit*.pyd")
        )
        if len(extensions) != 1:
            raise ValueError(f"expected exactly one extension: {extensions}")
        bundle(extensions[0], args.heif_prefix, args.pdfium_prefix)

        if sys.platform == "darwin":
            tags = re.findall(r"macosx_(\d+)_(\d+)_", args.wheel.name)
            if not tags:
                raise ValueError(f"no macOS platform tag in {args.wheel.name}")
            for binary in (
                extensions[0],
                package / "native/libheif.1.dylib",
                package / "native/libpdfium.dylib",
            ):
                load_commands = command("otool", "-l", str(binary))
                versions = re.findall(
                    r"cmd LC_BUILD_VERSION\s+cmdsize \d+\s+platform \S+\s+"
                    r"minos (\d+)\.(\d+)",
                    load_commands,
                )
                versions += re.findall(
                    r"cmd LC_VERSION_MIN_MACOSX\s+cmdsize \d+\s+"
                    r"version (\d+)\.(\d+)",
                    load_commands,
                )
                if not versions:
                    raise ValueError(f"cannot determine deployment target for {binary}")
                minimum = max((int(major), int(minor)) for major, minor in versions)
                if any((int(major), int(minor)) < minimum for major, minor in tags):
                    raise ValueError(
                        f"{binary.name} requires macOS {minimum[0]}.{minimum[1]}, "
                        f"which is newer than the wheel tag"
                    )

        records = list(root.glob("*.dist-info/RECORD"))
        if len(records) != 1:
            raise ValueError(f"expected exactly one wheel RECORD: {records}")
        record = records[0]
        rows = []
        for path in sorted(root.rglob("*")):
            if not path.is_file() or path == record:
                continue
            contents = path.read_bytes()
            digest = base64.urlsafe_b64encode(hashlib.sha256(contents).digest())
            rows.append(
                (
                    path.relative_to(root).as_posix(),
                    "sha256=" + digest.decode().rstrip("="),
                    str(len(contents)),
                )
            )
        rows.append((record.relative_to(root).as_posix(), "", ""))
        output = io.StringIO(newline="")
        csv.writer(output, lineterminator="\n").writerows(rows)
        record.write_text(output.getvalue(), encoding="utf-8")

        args.output_directory.mkdir(parents=True, exist_ok=True)
        with zipfile.ZipFile(destination, "w", zipfile.ZIP_DEFLATED) as archive:
            for path in sorted(root.rglob("*")):
                if path.is_file():
                    archive.write(path, path.relative_to(root).as_posix())
    print(destination)


if __name__ == "__main__":
    main()
