"""Core data objects used across client APIs and event payloads.

This module re-exports the fundamental data types that appear throughout
the Tryx API — in event payloads, client method signatures, and helper
constructors.

Key types:

- :class:`JID` — WhatsApp JID (user + server pair).
- :class:`MessageSource` — Origin and routing metadata for a message.
- :class:`MessageInfo` — Normalized metadata for a received or sent message.
- :class:`SendResult` — Result of a successful message send operation.
- :class:`ProfilePicture` — Profile picture download result.
- :class:`UploadResponse` — Media upload response from WhatsApp servers.

Example::

    from tryx.types import JID

    jid = JID("1234567890", "s.whatsapp.net")
    print(jid.user, jid.server)
"""

from ._tryx import types as _types  # type: ignore

DeviceSentMeta = _types.DeviceSentMeta  # type: ignore[attr-defined]
JID = _types.JID  # type: ignore[attr-defined]
MediaReuploadResult = _types.MediaReuploadResult  # type: ignore[attr-defined]
MessageInfo = _types.MessageInfo  # type: ignore[attr-defined]
MessageSource = _types.MessageSource  # type: ignore[attr-defined]
MsgBotInfo = _types.MsgBotInfo  # type: ignore[attr-defined]
MsgMetaInfo = _types.MsgMetaInfo  # type: ignore[attr-defined]
ProfilePicture = _types.ProfilePicture  # type: ignore[attr-defined]
SendResult = _types.SendResult  # type: ignore[attr-defined]
UploadResponse = _types.UploadResponse  # type: ignore[attr-defined]

__all__ = [
    "DeviceSentMeta",
    "JID",
    "MediaReuploadResult",
    "MessageInfo",
    "MessageSource",
    "MsgBotInfo",
    "MsgMetaInfo",
    "ProfilePicture",
    "SendResult",
    "UploadResponse",
]
