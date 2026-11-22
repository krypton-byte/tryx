"""Unit tests for Tryx event and client initialization."""

from __future__ import annotations

import tryx.events as events
from tryx.backend import SqliteStore
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
        "EvIdentityChange",
        "EvLockChatUpdate",
        "EvFavoritesUpdate",
        "EvStatusPrivacyUpdate",
        "EvUnarchiveChatsSettingUpdate",
        "EvDisableLinkPreviewsUpdate",
        "EvContactRemoved",
        "EvOfflineSyncInterrupted",
        "EvClientExpirationChanged",
        "EvServerAck",
        "EvAppStateSyncFailed",
        "EvMexNotification",
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
        backend = SqliteStore(db)
        app = Tryx(backend)
        client = app.get_client()
        assert client is not None
        assert not client.is_connected()

    def test_client_namespaces_exist(self, tmp_path: object) -> None:
        import pathlib

        db = str(pathlib.Path(str(tmp_path)) / "test2.db")
        backend = SqliteStore(db)
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
        backend = SqliteStore(db)
        app = Tryx(backend)

        @app.on(events.EvMessage)
        async def handler(client: object, event: object) -> None:
            pass

        assert handler is not None

    def test_stateless_decorator_composition_and_sync_api(
        self, tmp_path: object
    ) -> None:
        import pathlib

        db = str(pathlib.Path(str(tmp_path)) / "test4.db")
        backend = SqliteStore(db)
        app = Tryx(backend, signal_cache_max_entries=256)

        d_msg = app.on(events.EvMessage)
        d_conn = app.on(events.EvConnected)

        def on_msg(client: object, event: object) -> None:
            pass

        def on_conn(client: object, event: object) -> None:
            pass

        # Composing decorators out of order must register each to its own event
        d_msg(on_msg)
        d_conn(on_conn)

        client = app.get_client()
        for method in (
            "reply_text",
            "react",
            "download_media_to_file",
            "download_media",
        ):
            assert hasattr(client, method), f"Missing TryxClient.{method}"
        for method in (
            "wait_for_socket_ready",
            "wait_for_session_ready",
            "auto_reconnect_enabled",
            "set_auto_reconnect",
            "reconnect_immediately",
            "pair_with_code",
            "logout",
            "shutdown",
        ):
            assert hasattr(client.advanced, method), f"Missing AdvancedClient.{method}"
        assert hasattr(app, "shutdown")
        assert hasattr(client.polls, "create_quiz")

    def test_voip_call_handle_and_player_spawn_safety(self) -> None:
        from tryx.client import CallHandle
        from tryx.media import AudioPlayer, VideoPlayer

        for attr in (
            "phase",
            "media_connected",
            "is_answered",
            "hangup",
            "hangup_local",
        ):
            assert hasattr(CallHandle, attr), f"Missing CallHandle.{attr}"

        # Instantiating AudioPlayer / VideoPlayer outside an active Tokio task
        # must use pyo3-async-runtimes' managed runtime without panicking.
        audio_player = AudioPlayer()
        audio_player.stop()
        video_player = VideoPlayer(fps=15)
        video_player.stop()
