"""Typed exception hierarchy for Tryx."""


class FailedBuildBot(Exception):
	"""Raised when the bot runtime cannot be initialized."""


class FailedToDecodeProto(Exception):
	"""Raised when protobuf payload decoding fails."""


class EventDispatchError(Exception):
	"""Raised when event callback dispatching fails."""


class PyPayloadBuildError(Exception):
	"""Raised when Python payload conversion into Rust structures fails."""


class UnsupportedBackend(Exception):
	"""Raised when a backend object is not supported by the current runtime."""


class UnsupportedEventType(Exception):
	"""Raised when registering or dispatching an unknown event class."""


# Backward-compatible aliases.
class BuildBotError(FailedBuildBot):
	"""Backward-compatible alias of FailedBuildBot."""


class UnsupportedEventTypeError(UnsupportedEventType):
	"""Backward-compatible alias of UnsupportedEventType."""


class UnsupportedBackendError(UnsupportedBackend):
	"""Backward-compatible alias of UnsupportedBackend."""
