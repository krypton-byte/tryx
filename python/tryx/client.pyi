"""High-level client API surface for Tryx Python bindings."""

from typing import Any, Awaitable, Callable, Type, TypeVar

from .backend import BackendBase
from .events import EvMessage
from .types import JID, MediaReuploadResult, ProfilePicture, SendResult, UploadResponse
from .wacore import MediaType
from .waproto.whatsapp_pb2 import Message as MessageProto
from .waproto.whatsapp_pb2 import MessageKey, SyncActionValue

EventT = TypeVar("EventT")


class IsOnWhatsAppResult:
    """Result entry for WhatsApp registration lookup."""
    jid: JID
    is_registered: bool


class UserInfo:
    """Basic profile metadata for a single user."""
    jid: JID
    lid: JID | None
    status: str | None
    picture_id: str | None
    is_business: bool


class ContactInfo:
    """Resolved contact information from phone lookups."""
    jid: JID
    lid: JID | None
    is_registered: bool
    is_business: bool
    status: str | None
    picture_id: int | None


DownloadableMedia = (
    MessageProto.ImageMessage
    | MessageProto.VideoMessage
    | MessageProto.AudioMessage
    | MessageProto.DocumentMessage
    | MessageProto.StickerMessage
)


class Tryx:
    """Main bot runtime controller.

    Use this class to register handlers and start the connection lifecycle.
    """
    handlers: Any

    def __init__(self, backend: BackendBase) -> None: ...
    def get_client(self) -> TryxClient: ...
    def on(
        self, event_type: Type[EventT]
    ) -> Callable[[Callable[..., Awaitable[Any]]], Callable[..., Awaitable[Any]]]: ...
    def run(self) -> Awaitable[None]: ...
    def run_blocking(self) -> None: ...


class TryxClient:
    """Connected client facade for messaging and feature namespaces."""
    contact: ContactClient
    chat_actions: ChatActionsClient
    community: CommunityClient
    newsletter: NewsletterClient
    groups: GroupsClient
    status: StatusClient
    chatstate: ChatstateClient
    blocking: BlockingClient
    polls: PollsClient
    presence: PresenceClient
    privacy: PrivacyClient
    profile: ProfileClient

    def is_connected(self) -> bool: ...
    async def download_media(self, message: DownloadableMedia) -> bytes: ...
    async def upload_file(self, path: str, media_type: MediaType) -> UploadResponse: ...
    async def upload(self, data: bytes, media_type: MediaType) -> UploadResponse: ...
    async def send_message(self, to: JID, message: MessageProto) -> SendResult: ...
    async def send_text(self, to: JID, text: str, quoted: EvMessage | None = None) -> SendResult: ...
    async def send_photo(
        self,
        to: JID,
        photo_data: bytes,
        caption: str | None = None,
        quoted: EvMessage | None = None,
    ) -> SendResult: ...
    async def send_document(
        self,
        to: JID,
        document_data: bytes,
        mimetype: str,
        file_name: str | None = None,
        caption: str | None = None,
        quoted: EvMessage | None = None,
    ) -> SendResult: ...
    async def send_audio(
        self,
        to: JID,
        audio_data: bytes,
        mimetype: str | None = None,
        ptt: bool = False,
        seconds: int | None = None,
        quoted: EvMessage | None = None,
    ) -> SendResult: ...
    async def send_video(
        self,
        to: JID,
        video_data: bytes,
        mimetype: str | None = None,
        caption: str | None = None,
        seconds: int | None = None,
        gif_playback: bool = False,
        quoted: EvMessage | None = None,
    ) -> SendResult: ...
    async def send_gif(
        self,
        to: JID,
        gif_data: bytes,
        caption: str | None = None,
        seconds: int | None = None,
        quoted: EvMessage | None = None,
    ) -> SendResult: ...
    async def send_sticker(
        self,
        to: JID,
        sticker_data: bytes,
        is_animated: bool = False,
        quoted: EvMessage | None = None,
    ) -> SendResult: ...
    async def request_media_reupload(
        self,
        message_id: str,
        chat_jid: JID,
        media_key: bytes,
        is_from_me: bool = False,
        participant: JID | None = None,
    ) -> MediaReuploadResult: ...


class ContactClient:
    """Contact and profile lookup operations."""
    async def get_info(self, phones: list[str]) -> list[ContactInfo]: ...
    async def get_user_info(self, jid: JID) -> dict[JID, UserInfo]: ...
    async def get_profile_picture(self, jid: JID, preview: bool) -> ProfilePicture: ...
    async def is_on_whatsapp(self, jid: list[JID]) -> list[IsOnWhatsAppResult]: ...


class ChatActionsClient:
    """Chat-level actions such as archive, pin, mute, and reactions."""
    @staticmethod
    def build_message_key(
        id: str,
        remote_jid: JID,
        from_me: bool,
        participant: JID | None = None,
    ) -> MessageKey: ...
    @staticmethod
    def build_message_range(
        last_message_timestamp: int,
        last_system_message_timestamp: int | None,
        messages: list[tuple[MessageKey, int]],
    ) -> SyncActionValue.SyncActionMessageRange: ...
    async def archive_chat(
        self,
        jid: JID,
        message_range: SyncActionValue.SyncActionMessageRange | None = None,
    ) -> None: ...
    async def unarchive_chat(
        self,
        jid: JID,
        message_range: SyncActionValue.SyncActionMessageRange | None = None,
    ) -> None: ...
    async def pin_chat(self, jid: JID) -> None: ...
    async def unpin_chat(self, jid: JID) -> None: ...
    async def mute_chat(self, jid: JID) -> None: ...
    async def mute_chat_until(self, jid: JID, mute_end_timestamp_ms: int) -> None: ...
    async def unmute_chat(self, jid: JID) -> None: ...
    async def star_message(
        self,
        chat_jid: JID,
        participant_jid: JID | None,
        message_id: str,
        from_me: bool,
    ) -> None: ...
    async def unstar_message(
        self,
        chat_jid: JID,
        participant_jid: JID | None,
        message_id: str,
        from_me: bool,
    ) -> None: ...
    async def mark_chat_as_read(
        self,
        jid: JID,
        read: bool,
        message_range: SyncActionValue.SyncActionMessageRange | None = None,
    ) -> None: ...
    async def delete_chat(
        self,
        jid: JID,
        delete_media: bool,
        message_range: SyncActionValue.SyncActionMessageRange | None = None,
    ) -> None: ...
    async def delete_message_for_me(
        self,
        chat_jid: JID,
        participant_jid: JID | None,
        message_id: str,
        from_me: bool,
        delete_media: bool,
        message_timestamp: int | None = None,
    ) -> None: ...
    async def edit_message(
        self,
        chat_jid: JID,
        original_id: str,
        new_message: MessageProto,
    ) -> str: ...
    async def revoke_message(
        self,
        chat_jid: JID,
        message_id: str,
        original_sender: JID | None = None,
    ) -> None: ...
    async def react_message(
        self,
        chat_jid: JID,
        message_id: str,
        reaction: str,
        from_me: bool = False,
        participant_jid: JID | None = None,
    ) -> str: ...


class GroupType:
    """Type stub for GroupType."""
    Default: GroupType
    Community: GroupType
    LinkedSubgroup: GroupType
    LinkedAnnouncementGroup: GroupType
    LinkedGeneralGroup: GroupType


class CreateCommunityOptions:
    """Type stub for CreateCommunityOptions."""
    name: str
    description: str | None
    closed: bool
    allow_non_admin_sub_group_creation: bool
    create_general_chat: bool

    def __init__(
        self,
        name: str,
        description: str | None = None,
        closed: bool = False,
        allow_non_admin_sub_group_creation: bool = False,
        create_general_chat: bool = True,
    ) -> None: ...


class CreateCommunityResult:
    """Type stub for CreateCommunityResult."""
    gid: JID


class CommunitySubgroup:
    """Type stub for CommunitySubgroup."""
    id: JID
    subject: str
    participant_count: int | None
    is_default_sub_group: bool
    is_general_chat: bool


class LinkSubgroupsResult:
    """Type stub for LinkSubgroupsResult."""
    linked_jids: list[JID]
    failed_groups: list[tuple[JID, int]]


class UnlinkSubgroupsResult:
    """Type stub for UnlinkSubgroupsResult."""
    unlinked_jids: list[JID]
    failed_groups: list[tuple[JID, int]]


class GroupParticipant:
    """Type stub for GroupParticipant."""
    jid: JID
    phone_number: JID | None
    is_admin: bool


class GroupMetadata:
    """Type stub for GroupMetadata."""
    id: JID
    subject: str
    participants: list[GroupParticipant]
    addressing_mode: str
    creator: JID | None
    creation_time: int | None
    subject_time: int | None
    subject_owner: JID | None
    description: str | None
    description_id: str | None
    is_locked: bool
    is_announcement: bool
    ephemeral_expiration: int
    membership_approval: bool
    member_add_mode: str | None
    member_link_mode: str | None
    size: int | None
    is_parent_group: bool
    parent_group_jid: JID | None
    is_default_sub_group: bool
    is_general_chat: bool
    allow_non_admin_sub_group_creation: bool
    group_type: GroupType


class CommunityClient:
    """Type stub for CommunityClient."""
    @staticmethod
    def classify_group(metadata: GroupMetadata) -> GroupType: ...
    async def create(self, options: CreateCommunityOptions) -> CreateCommunityResult: ...
    async def deactivate(self, community_jid: JID) -> None: ...
    async def link_subgroups(
        self,
        community_jid: JID,
        subgroup_jids: list[JID],
    ) -> LinkSubgroupsResult: ...
    async def unlink_subgroups(
        self,
        community_jid: JID,
        subgroup_jids: list[JID],
        remove_orphan_members: bool,
    ) -> UnlinkSubgroupsResult: ...
    async def get_subgroups(self, community_jid: JID) -> list[CommunitySubgroup]: ...
    async def get_subgroup_participant_counts(
        self,
        community_jid: JID,
    ) -> list[tuple[JID, int]]: ...
    async def query_linked_group(
        self,
        community_jid: JID,
        subgroup_jid: JID,
    ) -> GroupMetadata: ...
    async def join_subgroup(
        self,
        community_jid: JID,
        subgroup_jid: JID,
    ) -> GroupMetadata: ...
    async def get_linked_groups_participants(
        self,
        community_jid: JID,
    ) -> list[GroupParticipant]: ...


class NewsletterVerification:
    """Type stub for NewsletterVerification."""
    Verified: NewsletterVerification
    Unverified: NewsletterVerification


class NewsletterState:
    """Type stub for NewsletterState."""
    Active: NewsletterState
    Suspended: NewsletterState
    Geosuspended: NewsletterState


class NewsletterRole:
    """Type stub for NewsletterRole."""
    Owner: NewsletterRole
    Admin: NewsletterRole
    Subscriber: NewsletterRole
    Guest: NewsletterRole


class NewsletterReactionCount:
    """Type stub for NewsletterReactionCount."""
    code: str
    count: int


class NewsletterMetadata:
    """Type stub for NewsletterMetadata."""
    jid: JID
    name: str
    description: str | None
    subscriber_count: int
    verification: NewsletterVerification
    state: NewsletterState
    picture_url: str | None
    preview_url: str | None
    invite_code: str | None
    role: NewsletterRole | None
    creation_time: int | None


class NewsletterMessage:
    """Type stub for NewsletterMessage."""
    server_id: int
    timestamp: int
    message_type: str
    is_sender: bool
    reactions: list[NewsletterReactionCount]
    message: MessageProto | None


class NewsletterClient:
    """Type stub for NewsletterClient."""
    async def list_subscribed(self) -> list[NewsletterMetadata]: ...
    async def get_metadata(self, jid: JID) -> NewsletterMetadata: ...
    async def get_metadata_by_invite(self, invite_code: str) -> NewsletterMetadata: ...
    async def create(
        self,
        name: str,
        description: str | None = None,
    ) -> NewsletterMetadata: ...
    async def join(self, jid: JID) -> NewsletterMetadata: ...
    async def leave(self, jid: JID) -> None: ...
    async def update(
        self,
        jid: JID,
        name: str | None = None,
        description: str | None = None,
    ) -> NewsletterMetadata: ...
    async def subscribe_live_updates(self, jid: JID) -> int: ...
    async def send_message(self, jid: JID, message: MessageProto) -> str: ...
    async def send_reaction(self, jid: JID, server_id: int, reaction: str) -> None: ...
    async def get_messages(
        self,
        jid: JID,
        count: int,
        before: int | None = None,
    ) -> list[NewsletterMessage]: ...


class MemberLinkMode:
    """Type stub for MemberLinkMode."""
    AdminLink: MemberLinkMode
    AllMemberLink: MemberLinkMode


class MemberAddMode:
    """Type stub for MemberAddMode."""
    AdminAdd: MemberAddMode
    AllMemberAdd: MemberAddMode


class MembershipApprovalMode:
    """Type stub for MembershipApprovalMode."""
    Off: MembershipApprovalMode
    On: MembershipApprovalMode


class GroupParticipantOptions:
    """Type stub for GroupParticipantOptions."""
    jid: JID
    phone_number: JID | None
    privacy: bytes | None

    def __init__(
        self,
        jid: JID,
        phone_number: JID | None = None,
        privacy: bytes | None = None,
    ) -> None: ...


class CreateGroupOptions:
    """Type stub for CreateGroupOptions."""
    subject: str
    participants: list[GroupParticipantOptions]
    member_link_mode: MemberLinkMode | None
    member_add_mode: MemberAddMode | None
    membership_approval_mode: MembershipApprovalMode | None
    ephemeral_expiration: int | None
    is_parent: bool
    closed: bool
    allow_non_admin_sub_group_creation: bool
    create_general_chat: bool

    def __init__(
        self,
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
    ) -> None: ...


class CreateGroupResult:
    """Type stub for CreateGroupResult."""
    gid: JID


class JoinGroupResult:
    """Type stub for JoinGroupResult."""
    jid: JID
    pending_approval: bool


class ParticipantChangeResponse:
    """Type stub for ParticipantChangeResponse."""
    jid: JID
    status: str | None
    error: str | None


class MembershipRequest:
    """Type stub for MembershipRequest."""
    jid: JID
    request_time: int | None


class GroupInfo:
    """Type stub for GroupInfo."""
    participants: list[JID]
    addressing_mode: str
    lid_to_pn_map: list[tuple[str, JID]]


class GroupsClient:
    """Type stub for GroupsClient."""
    async def query_info(self, jid: JID) -> GroupInfo: ...
    async def get_participating(self) -> dict[str, GroupMetadata]: ...
    async def get_metadata(self, jid: JID) -> GroupMetadata: ...
    async def create_group(self, options: CreateGroupOptions) -> CreateGroupResult: ...
    async def set_subject(self, jid: JID, subject: str) -> None: ...
    async def set_description(
        self,
        jid: JID,
        description: str | None = None,
        prev: str | None = None,
    ) -> None: ...
    async def leave(self, jid: JID) -> None: ...
    async def add_participants(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]: ...
    async def remove_participants(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]: ...
    async def promote_participants(self, jid: JID, participants: list[JID]) -> None: ...
    async def demote_participants(self, jid: JID, participants: list[JID]) -> None: ...
    async def get_invite_link(self, jid: JID, reset: bool) -> str: ...
    async def set_locked(self, jid: JID, locked: bool) -> None: ...
    async def set_announce(self, jid: JID, announce: bool) -> None: ...
    async def set_ephemeral(self, jid: JID, expiration: int) -> None: ...
    async def set_membership_approval(
        self,
        jid: JID,
        mode: MembershipApprovalMode,
    ) -> None: ...
    async def join_with_invite_code(self, code: str) -> JoinGroupResult: ...
    async def join_with_invite_v4(
        self,
        group_jid: JID,
        code: str,
        expiration: int,
        admin_jid: JID,
    ) -> JoinGroupResult: ...
    async def get_invite_info(self, code: str) -> GroupMetadata: ...
    async def get_membership_requests(self, jid: JID) -> list[MembershipRequest]: ...
    async def approve_membership_requests(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]: ...
    async def reject_membership_requests(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]: ...
    async def set_member_add_mode(self, jid: JID, mode: MemberAddMode) -> None: ...


class StatusPrivacySetting:
    """Type stub for StatusPrivacySetting."""
    Contacts: StatusPrivacySetting
    AllowList: StatusPrivacySetting
    DenyList: StatusPrivacySetting


class StatusSendOptions:
    """Type stub for StatusSendOptions."""
    privacy: StatusPrivacySetting

    def __init__(
        self,
        privacy: StatusPrivacySetting = StatusPrivacySetting.Contacts,
    ) -> None: ...


class StatusClient:
    """Type stub for StatusClient."""
    async def send_text(
        self,
        text: str,
        background_argb: int,
        font: int,
        recipients: list[JID],
        options: StatusSendOptions | None = None,
    ) -> str: ...
    async def send_image(
        self,
        upload: UploadResponse,
        thumbnail: bytes,
        recipients: list[JID],
        caption: str | None = None,
        options: StatusSendOptions | None = None,
    ) -> str: ...
    async def send_video(
        self,
        upload: UploadResponse,
        thumbnail: bytes,
        duration_seconds: int,
        recipients: list[JID],
        caption: str | None = None,
        options: StatusSendOptions | None = None,
    ) -> str: ...
    async def send_raw(
        self,
        message: MessageProto,
        recipients: list[JID],
        options: StatusSendOptions | None = None,
    ) -> str: ...
    async def revoke(
        self,
        message_id: str,
        recipients: list[JID],
        options: StatusSendOptions | None = None,
    ) -> str: ...
    @staticmethod
    def default_privacy() -> StatusPrivacySetting: ...


class ChatStateType:
    """Type stub for ChatStateType."""
    Composing: ChatStateType
    Recording: ChatStateType
    Paused: ChatStateType


class BlocklistEntry:
    """Type stub for BlocklistEntry."""
    jid: JID
    timestamp: int | None


class PollOptionResult:
    """Type stub for PollOptionResult."""
    name: str
    voters: list[str]


class PresenceStatus:
    """Type stub for PresenceStatus."""
    Available: PresenceStatus
    Unavailable: PresenceStatus


class PrivacyCategory:
    """Type stub for PrivacyCategory."""
    Last: PrivacyCategory
    Online: PrivacyCategory
    Profile: PrivacyCategory
    Status: PrivacyCategory
    GroupAdd: PrivacyCategory
    ReadReceipts: PrivacyCategory
    CallAdd: PrivacyCategory
    Messages: PrivacyCategory
    DefenseMode: PrivacyCategory
    Other: PrivacyCategory


class PrivacyValue:
    """Type stub for PrivacyValue."""
    All: PrivacyValue
    Contacts: PrivacyValue
    None_: PrivacyValue
    ContactBlacklist: PrivacyValue
    MatchLastSeen: PrivacyValue
    Known: PrivacyValue
    Off: PrivacyValue
    OnStandard: PrivacyValue
    Other: PrivacyValue


class DisallowedListAction:
    """Type stub for DisallowedListAction."""
    Add: DisallowedListAction
    Remove: DisallowedListAction


class PrivacySetting:
    """Type stub for PrivacySetting."""
    category: PrivacyCategory
    value: PrivacyValue


class DisallowedListUserEntry:
    """Type stub for DisallowedListUserEntry."""
    action: DisallowedListAction
    jid: JID
    pn_jid: JID | None

    def __init__(
        self,
        action: DisallowedListAction,
        jid: JID,
        pn_jid: JID | None = None,
    ) -> None: ...


class DisallowedListUpdate:
    """Type stub for DisallowedListUpdate."""
    dhash: str
    users: list[DisallowedListUserEntry]

    def __init__(
        self,
        dhash: str,
        users: list[DisallowedListUserEntry] = [],
    ) -> None: ...


class ChatstateClient:
    """Type stub for ChatstateClient."""
    async def send(self, to: JID, state: ChatStateType) -> None: ...
    async def send_composing(self, to: JID) -> None: ...
    async def send_recording(self, to: JID) -> None: ...
    async def send_paused(self, to: JID) -> None: ...


class BlockingClient:
    """Type stub for BlockingClient."""
    async def block(self, jid: JID) -> None: ...
    async def unblock(self, jid: JID) -> None: ...
    async def get_blocklist(self) -> list[BlocklistEntry]: ...
    async def is_blocked(self, jid: JID) -> bool: ...


class ProfileClient:
    """Type stub for ProfileClient."""
    async def set_push_name(self, name: str) -> None: ...
    async def set_status_text(self, text: str) -> None: ...
    async def set_profile_picture(self, image_data: bytes) -> str: ...
    async def remove_profile_picture(self) -> str: ...


class PrivacyClient:
    """Type stub for PrivacyClient."""
    async def fetch_settings(self) -> list[PrivacySetting]: ...
    async def set_setting(
        self,
        category: PrivacyCategory,
        value: PrivacyValue,
    ) -> str | None: ...
    async def set_disallowed_list(
        self,
        category: PrivacyCategory,
        update: DisallowedListUpdate,
    ) -> str | None: ...
    async def set_default_disappearing_mode(self, duration_seconds: int) -> None: ...


class PollsClient:
    """Type stub for PollsClient."""
    async def create(
        self,
        to: JID,
        name: str,
        options: list[str],
        selectable_count: int,
    ) -> tuple[str, bytes]: ...
    async def vote(
        self,
        chat_jid: JID,
        poll_msg_id: str,
        poll_creator_jid: JID,
        message_secret: bytes,
        option_names: list[str],
    ) -> str: ...
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


class PresenceClient:
    """Type stub for PresenceClient."""
    async def set(self, status: PresenceStatus) -> None: ...
    async def set_available(self) -> None: ...
    async def set_unavailable(self) -> None: ...
    async def subscribe(self, jid: JID) -> None: ...
    async def unsubscribe(self, jid: JID) -> None: ...

