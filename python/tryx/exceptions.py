class BuildBotError(Exception):
    """Raised when the bot fails to build properly."""

    pass


class FailedToDecodeProtoError(Exception):
    """Raised when the bot fails to decode a protocol buffer."""

    pass


class UnsupportedEventTypeError(Exception):
    """Raised when an unsupported event type is encountered."""

    pass


class UnsupportedBackendError(Exception):
    """Raised when an unsupported backend is encountered."""

    pass
