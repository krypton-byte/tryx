from ._tryx import exceptions as _exceptions  # type: ignore

for name in dir(_exceptions):  # type: ignore
    obj = getattr(_exceptions, name)  # type: ignore
    if isinstance(obj, type):
        globals()[name] = obj

# Backward-compatible aliases for older Python API names.
BuildBotError = FailedBuildBot
UnsupportedEventTypeError = UnsupportedEventType
UnsupportedBackendError = UnsupportedBackend

__all__ = sorted(name for name, obj in globals().items() if isinstance(obj, type))
