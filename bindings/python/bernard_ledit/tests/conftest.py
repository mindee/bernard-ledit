"""Pytest bootstrap for local extension-module tests."""

import importlib
import importlib.machinery
import shutil
import subprocess
import sys
import sysconfig
from pathlib import Path


def pytest_configure():
    if _can_import_extension():
        return

    project_root = Path(__file__).resolve().parents[3]

    if _run_maturin_develop(project_root):
        importlib.invalidate_caches()
        if _can_import_extension():
            return

    _build_extension_with_cargo(project_root)
    importlib.invalidate_caches()


def _can_import_extension() -> bool:
    try:
        importlib.import_module("bernard_ledit._bernard_ledit")
    except ModuleNotFoundError:
        return False
    return True


def _run_maturin_develop(project_root: Path) -> bool:
    try:
        subprocess.run(
            [sys.executable, "-m", "maturin", "develop", "--quiet"],
            cwd=project_root,
            check=True,
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        return False
    return True


def _build_extension_with_cargo(project_root: Path) -> None:
    workspace_root = project_root.parents[1]
    subprocess.run(
        ["cargo", "build", "-p", "bernard-ledit-python"],
        cwd=workspace_root,
        check=True,
    )

    built_extension = _built_extension_path(workspace_root)
    package_extension = _package_extension_path(project_root)
    shutil.copy2(built_extension, package_extension)


def _built_extension_path(workspace_root: Path) -> Path:
    target_dir = workspace_root / "target" / "debug"
    candidates = [
        target_dir / "libbernard_ledit_python.so",
        target_dir / "libbernard_ledit_python.dylib",
        target_dir / "bernard_ledit_python.dll",
    ]

    for candidate in candidates:
        if candidate.exists():
            return candidate

    raise RuntimeError("could not find compiled bernard-ledit-python extension")


def _package_extension_path(project_root: Path) -> Path:
    suffixes = importlib.machinery.EXTENSION_SUFFIXES
    suffix = next(
        (suffix for suffix in suffixes if suffix.startswith(".abi3")),
        sysconfig.get_config_var("EXT_SUFFIX") or suffixes[0],
    )
    return project_root / "python" / "bernard_ledit" / f"_bernard_ledit{suffix}"
