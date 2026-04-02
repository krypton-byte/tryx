"""Pytest configuration for Tryx integration tests."""

from __future__ import annotations

import pytest


@pytest.fixture(scope="session")
def db_path(tmp_path_factory: pytest.TempPathFactory) -> str:
    """Provide a temporary SQLite database path for tests."""
    return str(tmp_path_factory.mktemp("tryx") / "test.db")
