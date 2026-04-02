"""Unit tests for Tryx event and client initialization."""

from __future__ import annotations

import tryx.events as events
from tryx.backend import SqliteBackend
from tryx.client import Tryx


class TestEventClassesExist:
    """Verify that all expected event classes are importable."""

    EVENT_NAMES = [
        "EvConnected",
        "EvDisconnected",
        "EvLoggedOut",
        "EvPairSuccess",
        "EvPairError",
        "EvPairingQrCode",
        "EvPairingCode",
        "EvMessage",
        "EvReceipt",
        "EvUndecryptableMessage",
        "EvNotification",
        "EvChatPresence",
        "EvPresence",
        "EvPictureUpdate",
        "EvUserAboutUpdate",
        "EvJoinedGroup",
        "EvGroupInfoUpdate",
        "EvGroupUpdate",
        "EvContactUpdate",
        "EvPushNameUpdate",
        "EvSelfPushNameUpdated",
        "EvPinUpdate",
        "EvMuteUpdate",
        "EvArchiveUpdate",
        "EvMarkChatAsReadUpdate",
        "EvHistorySync",
        "EvOfflineSyncPreview",
        "EvOfflineSyncCompleted",
        "EvDeviceListUpdate",
        "EvBusinessStatusUpdate",
        "EvStreamReplaced",
        "EvTemporaryBan",
        "EvConnectFailure",
        "EvStreamError",
        "EvContactNumberChanged",
        "EvDisappearingModeChanged",
        "EvContactSyncRequested",
        "EvContactUpdated",
        "EvStarUpdate",
        "EvNewsletterLiveUpdate",
        "EvDeleteChatUpdate",
        "EvDeleteMessageForMeUpdate",
    ]

    def test_all_event_classes_importable(self) -> None:
        for name in self.EVENT_NAMES:
            assert hasattr(events, name), f"Missing event class: {name}"
            cls = getattr(events, name)
            assert isinstance(cls, type), f"{name} is not a class"


class TestTryxInitialization:
    """Verify that Tryx can be initialised with a SQLite backend."""

    def test_basic_init(self, tmp_path: object) -> None:
        import pathlib

        db = str(pathlib.Path(str(tmp_path)) / "test.db")
        backend = SqliteBackend(db)
        app = Tryx(backend)
        client = app.get_client()
        assert client is not None
        assert not client.is_connected()

    def test_client_namespaces_exist(self, tmp_path: object) -> None:
        import pathlib

        db = str(pathlib.Path(str(tmp_path)) / "test2.db")
        backend = SqliteBackend(db)
        app = Tryx(backend)
        client = app.get_client()
        for ns in [
            "contact",
            "chat_actions",
            "community",
            "newsletter",
            "groups",
            "status",
            "chatstate",
            "blocking",
            "polls",
            "presence",
            "privacy",
            "profile",
        ]:
            assert hasattr(client, ns), f"Missing namespace: {ns}"

    def test_handler_registration(self, tmp_path: object) -> None:
        import pathlib

        db = str(pathlib.Path(str(tmp_path)) / "test3.db")
        backend = SqliteBackend(db)
        app = Tryx(backend)

        @app.on(events.EvMessage)
        async def handler(client: object, event: object) -> None:
            pass

        assert handler is not None
