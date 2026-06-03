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
    def edit_type(self) -> Literal["First", "Inner", "Last"] | None: ...
    @property
    def edit_target_id(self) -> str | None: ...
    @property
    def edit_sender_timestamp(self) -> int | None: ...

class MsgMetaInfo:
    """Additional metadata attached to a message payload."""

    @property
    def target_id(self) -> str | None: ...
    @property
    def target_sender(self) -> JID | None: ...
    @property
    def target_chat(self) -> JID | None: ...
    @property
    def deprecated_lid_session(self) -> bool | None: ...
    @property
    def thread_message_id(self) -> str | None: ...
    @property
    def thread_message_sender_jid(self) -> JID | None: ...
    @property
    def content_type(self) -> str | None: ...
    @property
    def appdata(self) -> str | None: ...
    @property
    def reporting_tag(self) -> bytes | None: ...
    @property
    def reporting_token(self) -> bytes | None: ...
    @property
    def reporting_token_version(self) -> int | None: ...

class DeviceSentMeta:
    """Metadata used for device-sent message synchronization."""

    @property
    def destination_jid(self) -> str: ...
    @property
    def phash(self) -> str: ...

class MessageInfo:
    """Normalized metadata for a received or sent message."""

    id: str
    type: str
    push_name: str

    @property
    def source(self) -> MessageSource: ...
    @property
    def multicast(self) -> bool: ...
    @property
    def server_id(self) -> int: ...
    @property
    def timestamp(self) -> datetime: ...
    @property
    def media_type(self) -> str: ...
    @property
    def edit(self) -> str: ...
    @property
    def bot_info(self) -> MsgBotInfo | None: ...
    @property
    def meta_info(self) -> MsgMetaInfo: ...
    @property
    def verified_name(self) -> VerifiedNameCertificate | None: ...
    @property
    def device_sent_meta(self) -> DeviceSentMeta | None: ...
    @property
    def category(self) -> str: ...
    @property
    def ephemeral_expiration(self) -> int | None: ...
    @property
    def is_offline(self) -> bool: ...
    @property
    def unavailable_request_id(self) -> str | None: ...
    @property
    def server_timestamp_us(self) -> int | None: ...
    @property
    def verified_level(self) -> str | None: ...
    @property
    def verified_name_serial(self) -> int | None: ...
    @property
    def peer_recipient_pn(self) -> JID | None: ...
    @property
    def bcl_participants(self) -> list[JID]: ...

class UploadResponse:
    """Result of a media upload call."""

    @property
    def url(self) -> str: ...
    @property
    def direct_path(self) -> str: ...
    @property
    def media_key(self) -> bytes: ...
    @property
    def file_enc_sha256(self) -> bytes: ...
    @property
    def file_sha256(self) -> bytes: ...
    @property
    def file_length(self) -> int: ...
    @property
    def media_key_timestamp(self) -> int: ...
    @property
    def streaming_sidecar(self) -> bytes | None: ...

class SendResult:
    """Result metadata for send operations."""

    @property
    def message_id(self) -> str: ...
    @property
    def to(self) -> JID: ...

class MediaReuploadResult:
    """Result of media reupload request."""

    @property
    def status(self) -> str: ...
    @property
    def direct_path(self) -> str | None: ...

class ProfilePicture:
    """Metadata about a user's profile picture."""

    id: str
    url: str
    direct_path: str | None
    hash: str | None
