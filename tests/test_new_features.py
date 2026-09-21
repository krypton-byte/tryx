"""Tests for new features: multi-device SQLite store and new types."""

from __future__ import annotations

import asyncio
from pathlib import Path

from tryx.backend import SqliteStore, StoredDeviceSummary
from tryx.client import (
    ContactClient,
    GroupsClient,
    IsOnWhatsAppResult,
    UserInfo,
    UsernameLookupUser,
)


def test_sqlite_store_multidevice(tmp_path: Path) -> None:
    async def run() -> None:
        db_file = str(tmp_path / "devices.db")
        store = SqliteStore(db_file, 0)
        assert store.device_id == 0
        assert store.path == db_file

        dev_1 = await store.create_new_device()
        assert dev_1 > 0
        assert await store.device_exists(dev_1)

        devices = await store.list_devices()
        assert isinstance(devices, list)
        assert any(d.id == dev_1 for d in devices)
        for d in devices:
            assert isinstance(d, StoredDeviceSummary)
            assert isinstance(d.id, int)
            assert isinstance(d.push_name, str)
            assert isinstance(d.linked, bool)

        await store.remove_device(dev_1)
        assert not await store.device_exists(dev_1)

    asyncio.run(run())


def test_type_exports() -> None:
    assert hasattr(UsernameLookupUser, "__doc__")
    assert hasattr(IsOnWhatsAppResult, "__doc__")
    assert hasattr(UserInfo, "__doc__")
    assert hasattr(StoredDeviceSummary, "__doc__")
    assert hasattr(ContactClient, "find_by_username")
    assert hasattr(GroupsClient, "fetch_overviews")
