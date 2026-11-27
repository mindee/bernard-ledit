"""Shared fixtures for Python binding tests."""

from pathlib import Path

import pytest


@pytest.fixture(scope="session")
def test_data_dir() -> Path:
    """Path to the workspace-level shared test data directory."""
    return Path(__file__).resolve().parents[4] / "tests" / "data"
