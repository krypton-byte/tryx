"""Type stubs for the compiled Rust extension ``_tryx``.

These stubs allow ``griffe`` (used by ``mkdocstrings``) to resolve imports
without needing the compiled ``.so``/``.pyd`` binary.  The actual runtime
types come from the compiled extension; this file exists solely for
documentation generation and static analysis.
"""

from typing import Any

# ── Sub-module stubs ──────────────────────────────────────────────
# Each sub-module mirrors the ``_tryx.<name>`` namespace that the
# Rust PyO3 build exports.  We only need enough structure for griffe
# to resolve ``from ._tryx import <submodule>``; the real types are
# defined in the sibling ``.pyi`` files (client.pyi, types.pyi, etc.).

class _backend:
    BackendBase: Any
    SqliteStore: Any
    FfiStore: Any
    PythonStore: Any

class _client:
    Tryx: Any
    TryxClient: Any
    AdvancedClient: Any
    AudioPlayer: Any
    VideoPlayer: Any

class _events: ...

class _exceptions:
    FailedBuildClient: type[Exception]
    FailedBuildBot: type[Exception]
    BuildBotError: type[Exception]
    FailedToDecodeProto: type[Exception]
    EventDispatchError: type[Exception]
    PyPayloadBuildError: type[Exception]
    UnsupportedBackend: type[Exception]
    UnsupportedBackendError: type[Exception]
    UnsupportedEventType: type[Exception]
    UnsupportedEventTypeError: type[Exception]

class _helpers: ...

class _types:
    DeviceSentMeta: Any
    JID: Any
    MediaReuploadResult: Any
    MessageInfo: Any
    MessageSource: Any
    MsgBotInfo: Any
    MsgMetaInfo: Any
    ProfilePicture: Any
    SendResult: Any
    UploadResponse: Any

class _wacore: ...
