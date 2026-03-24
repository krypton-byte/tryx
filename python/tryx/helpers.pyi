from .waproto.whatsapp_pb2 import Message as MessageProto
from .client import (
    BlocklistEntry,
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


class NewsletterHelpers:
    @staticmethod
    def parse_message(data: bytes) -> MessageProto: ...
    @staticmethod
    def serialize_message(message: MessageProto) -> bytes: ...
    @staticmethod
    def build_text_message(text: str) -> MessageProto: ...


class GroupsHelpers:
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
        membership_approval_mode: MembershipApprovalMode | None = MembershipApprovalMode.Off,
        ephemeral_expiration: int | None = 0,
        is_parent: bool = False,
        closed: bool = False,
        allow_non_admin_sub_group_creation: bool = False,
        create_general_chat: bool = False,
    ) -> CreateGroupOptions: ...


class StatusHelpers:
    @staticmethod
    def build_send_options(
        privacy: StatusPrivacySetting = StatusPrivacySetting.Contacts,
    ) -> StatusSendOptions: ...
    @staticmethod
    def default_privacy() -> StatusPrivacySetting: ...


class ChatstateHelpers:
    @staticmethod
    def composing() -> ChatStateType: ...
    @staticmethod
    def recording() -> ChatStateType: ...
    @staticmethod
    def paused() -> ChatStateType: ...


class BlockingHelpers:
    @staticmethod
    def same_user(a: JID, b: JID) -> bool: ...


class PollsHelpers:
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
    @staticmethod
    def default_status() -> PresenceStatus: ...
