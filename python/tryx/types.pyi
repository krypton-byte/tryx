"""Core data objects used across client APIs and event payloads."""

from datetime import datetime
from typing import Literal

from .waproto.whatsapp_pb2 import VerifiedNameCertificate

class JID:
    """Represents a WhatsApp JID (user + server)."""

    user: str
    server: str

    def __init__(self, user: str, server: str) -> None:
        """Create a JID from user and server parts."""
        ...

class MessageSource:
    """Describes where a message came from and who it targets."""

    sender: JID
    chat: JID
    is_from_me: bool
    is_group: bool
    addressing_mode: Literal["pn", "lid"] | None
    sender_alt: JID | None
    recipient_alt: JID | None
    broadcast_list_owner: JID | None
    recipient: JID | None

class MsgBotInfo:
    """Bot edit metadata attached to a message."""

    @property
    def edit_type(self) -> Literal["First", "Inner", "Last"] | None:
        """
        Return the edit type (``'First'``, ``'Inner'``, ``'Last'``), or ``None``.

        Returns:
            Edit type string or ``None``.
        """
        ...
    @property
    def edit_target_id(self) -> str | None:
        """
        Return the edit target message ID, or ``None``.

        Returns:
            Target message ID string or ``None``.
        """
        ...
    @property
    def edit_sender_timestamp(self) -> int | None:
        """
        Return the edit sender timestamp, or ``None``.

        Returns:
            Timestamp integer or ``None``.
        """
        ...

class MsgMetaInfo:
    """Additional metadata attached to a message payload."""

    @property
    def target_id(self) -> str | None:
        """
        Return the target message ID, or ``None``.

        Returns:
            Target message ID string or ``None``.
        """
        ...
    @property
    def target_sender(self) -> JID | None:
        """
        Return the target sender JID, or ``None``.

        Returns:
            Target sender JID or ``None``.
        """
        ...
    @property
    def target_chat(self) -> JID | None:
        """
        Return the target chat JID, or ``None``.

        Returns:
            Target chat JID or ``None``.
        """
        ...
    @property
    def deprecated_lid_session(self) -> bool | None:
        """
        Return deprecated LID session flag, or ``None``.

        Returns:
            Boolean or ``None``.
        """
        ...
    @property
    def thread_message_id(self) -> str | None:
        """
        Return the thread message ID, or ``None``.

        Returns:
            Thread message ID string or ``None``.
        """
        ...
    @property
    def thread_message_sender_jid(self) -> JID | None:
        """
        Return the thread message sender JID, or ``None``.

        Returns:
            Sender JID or ``None``.
        """
        ...
    @property
    def content_type(self) -> str | None:
        """
        Return the content type string, or ``None``.

        Returns:
            Content type string or ``None``.
        """
        ...
    @property
    def appdata(self) -> str | None:
        """
        Return the app data string, or ``None``.

        Returns:
            App data string or ``None``.
        """
        ...
    @property
    def reporting_tag(self) -> bytes | None:
        """
        Return the reporting tag bytes, or ``None``.

        Returns:
            Raw bytes or ``None``.
        """
        ...
    @property
    def reporting_token(self) -> bytes | None:
        """
        Return the reporting token bytes, or ``None``.

        Returns:
            Raw bytes or ``None``.
        """
        ...
    @property
    def reporting_token_version(self) -> int | None:
        """
        Return the reporting token version, or ``None``.

        Returns:
            Version integer or ``None``.
        """
        ...

class DeviceSentMeta:
    """Metadata used for device-sent message synchronization."""

    @property
    def destination_jid(self) -> str:
        """
        Return the destination JID string.

        Returns:
            Destination JID string.
        """
        ...
    @property
    def phash(self) -> str:
        """
        Return the phone hash string.

        Returns:
            Phone hash string.
        """
        ...

class MessageInfo:
    """Normalized metadata for a received or sent message."""

    id: str
    type: str
    push_name: str

    @property
    def source(self) -> MessageSource:
        """
        Return the message source (sender, chat, is_from_me).

        Returns:
            MessageSource with sender and chat JIDs.
        """
        ...
    @property
    def multicast(self) -> bool:
        """
        Return ``True`` if the message was sent to multiple recipients.

        Returns:
            Boolean flag.
        """
        ...
    @property
    def server_id(self) -> int:
        """
        Return the server-assigned message ID.

        Returns:
            Server ID integer.
        """
        ...
    @property
    def timestamp(self) -> datetime:
        """
        Return the message timestamp.

        Returns:
            datetime of the message.
        """
        ...
    @property
    def media_type(self) -> str:
        """
        Return the media type string (e.g. ``'image'``, ``'video'``).

        Returns:
            Media type string.
        """
        ...
    @property
    def edit(self) -> str:
        """
        Return the edit status string, or ``None``.

        Returns:
            Edit string or ``None``.
        """
        ...
    @property
    def bot_info(self) -> MsgBotInfo | None:
        """
        Return bot edit metadata, or ``None``.

        Returns:
            MsgBotInfo or ``None``.
        """
        ...
    @property
    def meta_info(self) -> MsgMetaInfo:
        """
        Return additional message metadata.

        Returns:
            MsgMetaInfo with target_id, content_type, etc.
        """
        ...
    @property
    def verified_name(self) -> VerifiedNameCertificate | None:
        """
        Return the verified name certificate, or ``None``.

        Returns:
            VerifiedNameCertificate or ``None``.
        """
        ...
    @property
    def device_sent_meta(self) -> DeviceSentMeta | None:
        """
        Return device-sent metadata, or ``None``.

        Returns:
            DeviceSentMeta or ``None``.
        """
        ...
    @property
    def category(self) -> str:
        """
        Return the message category string.

        Returns:
            Category string (e.g. ``'message'``).
        """
        ...
    @property
    def ephemeral_expiration(self) -> int | None:
        """
        Return the ephemeral expiration timer in seconds, or ``None``.

        Returns:
            Expiration seconds or ``None``.
        """
        ...
    @property
    def is_offline(self) -> bool:
        """
        Return ``True`` if the message was sent while offline.

        Returns:
            Boolean flag.
        """
        ...
    @property
    def unavailable_request_id(self) -> str | None:
        """
        Return the unavailable request ID, or ``None``.

        Returns:
            Request ID string or ``None``.
        """
        ...
    @property
    def server_timestamp_us(self) -> int | None:
        """
        Return the server timestamp in microseconds, or ``None``.

        Returns:
            Microsecond timestamp or ``None``.
        """
        ...
    @property
    def verified_level(self) -> str | None:
        """
        Return the verified level string, or ``None``.

        Returns:
            Level string or ``None``.
        """
        ...
    @property
    def verified_name_serial(self) -> int | None:
        """
        Return the verified name serial number, or ``None``.

        Returns:
            Serial integer or ``None``.
        """
        ...
    @property
    def peer_recipient_pn(self) -> JID | None:
        """
        Return the peer recipient phone JID, or ``None``.

        Returns:
            JID or ``None``.
        """
        ...
    @property
    def bcl_participants(self) -> list[JID]:
        """
        Return the broadcast list participant JIDs.

        Returns:
            List of JID objects.
        """
        ...

class UploadResponse:
    """Result of a media upload call."""

    @property
    def url(self) -> str:
        """
        Return the upload URL.

        Returns:
            URL string.
        """
        ...
    @property
    def direct_path(self) -> str:
        """
        Return the direct download path.

        Returns:
            Direct path string.
        """
        ...
    @property
    def media_key(self) -> bytes:
        """
        Return the media encryption key.

        Returns:
            Media key bytes.
        """
        ...
    @property
    def file_enc_sha256(self) -> bytes:
        """
        Return the encrypted file SHA-256 hash.

        Returns:
            SHA-256 hash bytes.
        """
        ...
    @property
    def file_sha256(self) -> bytes:
        """
        Return the plaintext file SHA-256 hash.

        Returns:
            SHA-256 hash bytes.
        """
        ...
    @property
    def file_length(self) -> int:
        """
        Return the file size in bytes.

        Returns:
            File length integer.
        """
        ...
    @property
    def media_key_timestamp(self) -> int:
        """
        Return the media key timestamp.

        Returns:
            Timestamp integer.
        """
        ...
    @property
    def streaming_sidecar(self) -> bytes | None:
        """
        Return the streaming sidecar bytes, or ``None``.

        Returns:
            Sidecar bytes or ``None``.
        """
        ...

class SendResult:
    """Result metadata for send operations."""

    @property
    def message_id(self) -> str:
        """
        Return the server-assigned message ID.

        Returns:
            Message ID string.
        """
        ...
    @property
    def to(self) -> JID:
        """
        Return the recipient JID.

        Returns:
            Recipient JID.
        """
        ...

class MediaReuploadResult:
    """Result of media reupload request."""

    @property
    def status(self) -> str:
        """
        Return the reupload status string.

        Returns:
            Status string (e.g. ``'ok'``).
        """
        ...
    @property
    def direct_path(self) -> str | None:
        """
        Return the new direct path, or ``None``.

        Returns:
            Direct path string or ``None``.
        """
        ...

class ProfilePicture:
    """Metadata about a user's profile picture."""

    id: str
    url: str
    direct_path: str | None
    hash: str | None
