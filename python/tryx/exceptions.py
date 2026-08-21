"""Exception hierarchy for Tryx error handling.

This module re-exports all exception classes raised by the Tryx runtime
and provides backward-compatible aliases for legacy API names.

Exception hierarchy::

    Exception
    ├── FailedBuildClient     — Client initialization failure
    │   ├── BuildBotError     — (legacy alias)
    │   └── FailedBuildBot    — (legacy alias)
    ├── FailedToDecodeProto   — Protobuf decoding failure
    ├── EventDispatchError    — Event callback dispatch failure
    ├── PyPayloadBuildError   — Python-to-Rust payload conversion failure
    ├── UnsupportedBackend    — Incompatible storage backend
    │   └── UnsupportedBackendError — (legacy alias)
    └── UnsupportedEventType  — Unknown event class registration
        └── UnsupportedEventTypeError — (legacy alias)

Example::

    from tryx.exceptions import FailedBuildClient

    try:
        app = Tryx(backend)
    except FailedBuildClient as e:
        print(f"Failed to initialize: {e}")
"""

from ._tryx import exceptions as _exceptions  # type: ignore

for name in dir(_exceptions):  # type: ignore
    obj = getattr(_exceptions, name)  # type: ignore
    if isinstance(obj, type):
        globals()[name] = obj

# Prefer modern names, but gracefully fall back to legacy names when needed.
FailedBuildClient = (
    globals().get("FailedBuildClient")
    or globals().get("FailedBuildBot")
    or globals().get("BuildBotError")
)
UnsupportedEventType = globals().get("UnsupportedEventType") or globals().get(
    "UnsupportedEventTypeError"
)
UnsupportedBackend = globals().get("UnsupportedBackend") or globals().get(
    "UnsupportedBackendError"
)

# Backward-compatible aliases for older Python API names.
if isinstance(FailedBuildClient, type):
    FailedBuildBot = FailedBuildClient  # backward compat
    BuildBotError = FailedBuildClient  # backward compat
    globals()["BuildBotError"] = BuildBotError  # Ensure explicit export

if isinstance(UnsupportedEventType, type):
    UnsupportedEventTypeError = UnsupportedEventType

if isinstance(UnsupportedBackend, type):
    UnsupportedBackendError = UnsupportedBackend

__all__ = sorted(name for name, obj in globals().items() if isinstance(obj, type))
