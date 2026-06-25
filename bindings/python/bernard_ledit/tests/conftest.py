"""Pytest bootstrap for local extension-module tests."""

import ctypes
import importlib
import importlib.machinery
import logging
import os
import shutil
import subprocess
import sys
import sysconfig
from pathlib import Path

import pytest

logging.basicConfig(level=logging.INFO, format="%(levelname)s: %(message)s")
logger = logging.getLogger(__name__)


@pytest.fixture(scope="session")
def test_data_dir() -> Path:
    """Path to the workspace-level shared test data directory."""
    return Path(__file__).resolve().parents[4] / "tests" / "data"


def pytest_configure():
    conftest_path = Path(__file__).resolve()
    workspace_root = conftest_path.parents[4]
    python_pkg_dir = conftest_path.parents[1]
    target_dir = workspace_root / "target" / "debug"
    py_ext = sysconfig.get_config_var("EXT_SUFFIX") or (
        ".pyd" if sys.platform == "win32" else ".so"
    )
    expected_so = python_pkg_dir / f"_bernard_ledit{py_ext}"

    if not expected_so.exists():
        logger.info("🦀 Extension not found. Building with Cargo...")
        subprocess.run(
            ["cargo", "build", "-p", "bernard-ledit-python"],
            cwd=workspace_root,
            check=True,
        )
        if sys.platform == "win32":
            cargo_built_name = "bernard_ledit_python"
            cargo_ext = ".dll"
        elif sys.platform == "darwin":
            cargo_built_name = "libbernard_ledit_python"
            cargo_ext = ".dylib"
        else:
            cargo_built_name = "libbernard_ledit_python"
            cargo_ext = ".so"

        built_so = target_dir / f"{cargo_built_name}{cargo_ext}"

        if built_so.exists():
            shutil.copy2(built_so, expected_so)
            logger.info(f"Copied extension to {expected_so}")
        else:
            raise RuntimeError(f"Build succeeded but couldn't find {built_so}")

    _load_pdfium_globally(workspace_root)
    importlib.invalidate_caches()


def _can_import_extension() -> bool:
    try:
        importlib.import_module("bernard_ledit._bernard_ledit")
    except (ModuleNotFoundError, ImportError) as e:
        logger.warning(f"Raised while importing extension: {e}.")
        return False
    return True


def _load_pdfium_globally(workspace_root):
    cache_dir = workspace_root / "core" / ".pdfium_cache"
    lib_pattern = "**/libpdfium.*" if sys.platform != "win32" else "**/pdfium.dll"
    libs = list(cache_dir.glob(lib_pattern))

    if libs:
        lib_path = str(libs[0].resolve())
        os.environ.setdefault("PDFIUM_PATH", lib_path)
        try:
            ctypes.CDLL(lib_path, mode=ctypes.RTLD_GLOBAL)
        except Exception as e:
            logger.warning(f"Failed to pre-load PDFium: {e}")
    else:
        logger.warning(f"PDFium library not found in {cache_dir}")


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
