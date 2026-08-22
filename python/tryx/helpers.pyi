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
    def parse_message(data: bytes) -> MessageProto:
        """
        Deserialize protobuf bytes into a Message proto.

        Args:
            data: Raw protobuf bytes.

        Returns:
            Parsed Message proto.
        """
        ...
    @staticmethod
    def serialize_message(message: MessageProto) -> bytes:
        """
        Serialize a Message proto into protobuf bytes.

        Args:
            message: Message proto to serialize.

        Returns:
            Serialized bytes.
        """
        ...
    @staticmethod
    def build_text_message(text: str) -> MessageProto:
        """
        Build a text-only Message proto.

        Args:
            text: Message body text.

        Returns:
            Message proto with conversation set.
        """
        ...

class GroupsHelpers:
    """Helpers for group invite and option object construction."""

    @staticmethod
    def strip_invite_url(code: str) -> str:
        """
        Strip the WhatsApp invite URL prefix from a code.

        Args:
            code: Full invite URL or raw code.

        Returns:
            Stripped invite code.
        """
        ...
    @staticmethod
    def build_participant(
        jid: JID,
        phone_number: JID | None = None,
        privacy: bytes | None = None,
    ) -> GroupParticipantOptions:
        """
        Build a GroupParticipantOptions object.

        Args:
            jid: Participant JID.
            phone_number: Optional phone number JID.
            privacy: Optional privacy bytes.

        Returns:
            GroupParticipantOptions instance.
        """
        ...
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
    ) -> CreateGroupOptions:
        """
        Build a CreateGroupOptions object.

        Args:
            subject: Group name.
            participants: List of initial participants.
            member_link_mode: Who can share the group link.
            member_add_mode: Who can add members.
            membership_approval_mode: Require approval for new members.
            ephemeral_expiration: Disappearing message timer in seconds.
            is_parent: If ``True``, create as a parent group.
            closed: If ``True``, only admins can edit group info.
            allow_non_admin_sub_group_creation: Allow non-admins to create subgroups.
            create_general_chat: Auto-create a general chat.

        Returns:
            CreateGroupOptions instance.
        """
        ...

class StatusHelpers:
    """Helpers for status privacy and send options."""

    @staticmethod
    def build_send_options(
        privacy: StatusPrivacySetting = StatusPrivacySetting.Contacts,
    ) -> StatusSendOptions:
        """
        Build a StatusSendOptions object.

        Args:
            privacy: Privacy setting (Contacts, AllowList, DenyList).

        Returns:
            StatusSendOptions instance.
        """
        ...
    @staticmethod
    def default_privacy() -> StatusPrivacySetting:
        """
        Return the default status privacy setting.

        Returns:
            StatusPrivacySetting.Contacts.
        """
        ...

class ChatstateHelpers:
    """Helpers for constructing chat state enum values."""

    @staticmethod
    def composing() -> ChatStateType:
        """
        Return ChatStateType.Composing.

        Returns:
            ChatStateType for typing indicator.
        """
        ...
    @staticmethod
    def recording() -> ChatStateType:
        """
        Return ChatStateType.Recording.

        Returns:
            ChatStateType for recording indicator.
        """
        ...
    @staticmethod
    def paused() -> ChatStateType:
        """
        Return ChatStateType.Paused.

        Returns:
            ChatStateType for paused indicator.
        """
        ...

class BlockingHelpers:
    """Helpers related to blocklist identity matching."""

    @staticmethod
    def same_user(a: JID, b: JID) -> bool:
        """
        Check if two JIDs belong to the same user (ignoring device).

        Args:
            a: First JID.
            b: Second JID.

        Returns:
            ``True`` if both JIDs share the same user part.
        """
        ...

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
    ) -> list[bytes]:
        """
        Decrypt a single poll vote without LID/PN fallback.

        Args:
            enc_payload: Encrypted vote payload.
            enc_iv: Encrypted initialization vector.
            message_secret: Poll message secret key.
            poll_msg_id: Poll message ID.
            poll_creator_jid: Poll creator JID.
            voter_jid: Voter JID.

        Returns:
            List of selected option name hashes.
        """
        ...
    @staticmethod
    def aggregate_votes(
        poll_options: list[str],
        votes: list[tuple[JID, bytes, bytes]],
        message_secret: bytes,
        poll_msg_id: str,
        poll_creator_jid: JID,
    ) -> list[PollOptionResult]:
        """
        Aggregate multiple poll votes into per-option results.

        Args:
            poll_options: List of option name strings.
            votes: List of (voter_jid, enc_payload, enc_iv) tuples.
            message_secret: Poll message secret key.
            poll_msg_id: Poll message ID.
            poll_creator_jid: Poll creator JID.

        Returns:
            List of PollOptionResult with name and voters.
        """
        ...

class PresenceHelpers:
    """Helpers for default presence values."""

    @staticmethod
    def default_status() -> PresenceStatus:
        """
        Return the default presence status.

        Returns:
            PresenceStatus.Available.
        """
        ...
