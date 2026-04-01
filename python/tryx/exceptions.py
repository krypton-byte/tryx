from ._tryx import exceptions as _exceptions  # type: ignore

for name in dir(_exceptions):  # type: ignore
    obj = getattr(_exceptions, name)  # type: ignore
    if isinstance(obj, type):
        globals()[name] = obj

# Prefer modern names, but gracefully fall back to legacy names when needed.
FailedBuildBot = globals().get("FailedBuildBot") or globals().get("BuildBotError")
UnsupportedEventType = globals().get("UnsupportedEventType") or globals().get(
    "UnsupportedEventTypeError"
)
UnsupportedBackend = globals().get("UnsupportedBackend") or globals().get(
    "UnsupportedBackendError"
)

# Backward-compatible aliases for older Python API names.
if isinstance(FailedBuildBot, type):
    BuildBotError = FailedBuildBot

if isinstance(UnsupportedEventType, type):
    UnsupportedEventTypeError = UnsupportedEventType

if isinstance(UnsupportedBackend, type):
    UnsupportedBackendError = UnsupportedBackend

__all__ = sorted(name for name, obj in globals().items() if isinstance(obj, type))
