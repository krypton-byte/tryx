"""Utility helpers for building payloads and handling common conversions."""

from .client import (
    ChatStateType,
    CreateGroupOptions,
    GroupParticipantOptions,
    MemberAddMode,
    MemberLinkMode,
    MembershipApprovalMode,
    PollOptionResult,
    PresenceStatus,
    StatusPrivacySetting,
    StatusSendOptions,
)
from .types import JID
from .waproto.whatsapp_pb2 import Message as MessageProto

class NewsletterHelpers:
    """Helpers for newsletter message serialization and builders."""

    @staticmethod
    def parse_message(data: bytes) -> MessageProto: ...
    @staticmethod
    def serialize_message(message: MessageProto) -> bytes: ...
    @staticmethod
    def build_text_message(text: str) -> MessageProto: ...

class GroupsHelpers:
    """Helpers for group invite and option object construction."""

    @staticmethod
    def strip_invite_url(code: str) -> str: ...
    @staticmethod
    def build_participant(
        jid: JID,
        phone_number: JID | None = None,
        privacy: bytes | None = None,
    ) -> GroupParticipantOptions: ...
    @staticmethod
    def build_create_options(
        subject: str,
        participants: list[GroupParticipantOptions] = [],
        member_link_mode: MemberLinkMode | None = MemberLinkMode.AdminLink,
        member_add_mode: MemberAddMode | None = MemberAddMode.AllMemberAdd,
        membership_approval_mode: MembershipApprovalMode
        | None = MembershipApprovalMode.Off,
        ephemeral_expiration: int | None = 0,
        is_parent: bool = False,
        closed: bool = False,
        allow_non_admin_sub_group_creation: bool = False,
        create_general_chat: bool = False,
    ) -> CreateGroupOptions: ...

class StatusHelpers:
    """Helpers for status privacy and send options."""

    @staticmethod
    def build_send_options(
        privacy: StatusPrivacySetting = StatusPrivacySetting.Contacts,
    ) -> StatusSendOptions: ...
    @staticmethod
    def default_privacy() -> StatusPrivacySetting: ...

class ChatstateHelpers:
    """Helpers for constructing chat state enum values."""

    @staticmethod
    def composing() -> ChatStateType: ...
    @staticmethod
    def recording() -> ChatStateType: ...
    @staticmethod
    def paused() -> ChatStateType: ...

class BlockingHelpers:
    """Helpers related to blocklist identity matching."""

    @staticmethod
    def same_user(a: JID, b: JID) -> bool: ...

class PollsHelpers:
    """Helpers for poll vote decryption and aggregation."""

    @staticmethod
    def decrypt_vote(
        enc_payload: bytes,
        enc_iv: bytes,
        message_secret: bytes,
        poll_msg_id: str,
        poll_creator_jid: JID,
        voter_jid: JID,
    ) -> list[bytes]: ...
    @staticmethod
    def aggregate_votes(
        poll_options: list[str],
        votes: list[tuple[JID, bytes, bytes]],
        message_secret: bytes,
        poll_msg_id: str,
        poll_creator_jid: JID,
    ) -> list[PollOptionResult]: ...

class PresenceHelpers:
    """Helpers for default presence values."""

    @staticmethod
    def default_status() -> PresenceStatus: ...
