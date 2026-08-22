"""High-level client API surface for Tryx Python bindings."""

from typing import Awaitable, Callable, TypeVar

from .backend import BackendBase, FfiStoreProtocol, StoreBase
from .events import Dispatcher, EvMessage
from .media import (
    AudioPlayer as AudioPlayer,
)
from .media import (
    AudioSink,
    AudioSource,
    VideoSink,
    VideoSource,
)
from .media import (
    VideoPlayer as VideoPlayer,
)
from .types import JID, MediaReuploadResult, ProfilePicture, SendResult, UploadResponse
from .wacore import MediaType, Node
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
    """Main automation runtime controller.

    Use this class to register event handlers and start the connection lifecycle.

    Accepts any of the 3 storage backend tiers:

    - ``SqliteStore`` — built-in SQLite backend
    - ``FfiStoreProtocol`` — native FFI backend (e.g. tryx-store-postgres)
    - ``StoreBase`` subclass — pure Python custom backend
    """

    handlers: Dispatcher

    def __init__(self, backend: BackendBase | FfiStoreProtocol | StoreBase) -> None:
        """Create a Tryx runtime with the given storage backend.

        Args:
            backend: Storage backend (SqliteStore, FfiStoreProtocol, or StoreBase).

        Example::

            from tryx.backend import SqliteStore
            from tryx.client import Tryx

            app = Tryx(SqliteStore('session.db'))
        """
        ...
    def get_client(self) -> TryxClient:
        """Return the connected client facade.

        Raises:
            RuntimeError: If the client is not yet running.

        Example::

            client = app.get_client()
            await client.send_text(to=JID('123', 's.whatsapp.net'), text='hi')
        """
        ...
    def on(
        self, event_type: type[EventT]
    ) -> Callable[
        [Callable[[TryxClient, EventT], Awaitable[None]]],
        Callable[[TryxClient, EventT], Awaitable[None]],
    ]:
        """Decorator to register an async event handler.

        Example::

            @app.on(EvMessage)
            async def handler(client: TryxClient, event: EvMessage) -> None:
                text = event.data.get_text()
                chat = event.data.message_info.source.chat
                await client.send_text(to=chat, text=text)
        """
        ...
    def run(self) -> Awaitable[None]:
        """Start the client in async mode.

        Example::

            asyncio.run(app.run())
        """
        ...
    def run_blocking(self) -> None:
        """Start the client and block until it exits.

        Example::

            app.run_blocking()
        """
        ...

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
    advanced: AdvancedClient
    labels: LabelsClient
    comments: CommentsClient
    events: EventsClient
    voip: VoipClient

    def is_connected(self) -> bool:
        """Return True if the underlying WebSocket is connected."""
        ...
    async def download_media(self, message: DownloadableMedia) -> bytes:
        """Download media content from a WhatsApp message.

        Accepts any protobuf media type (Image, Video, Audio, Document, Sticker).

        Returns:
            Raw media bytes.

        Example::

            data = await client.download_media(event.data.raw_proto.image_message)
        """
        ...
    async def upload_file(self, path: str, media_type: MediaType) -> UploadResponse:
        """Upload a file from disk to WhatsApp servers.

        Args:
            path: Filesystem path to the file.
            media_type: Category of media (MediaType.Image, etc.).

        Returns:
            Upload metadata including URL, media key, and hashes.

        Example::

            resp = await client.upload_file('photo.jpg', MediaType.Image)
        """
        ...
    async def upload(self, data: bytes, media_type: MediaType) -> UploadResponse:
        """Upload raw bytes to WhatsApp servers.

        Args:
            data: Raw file content.
            media_type: Category of media.

        Returns:
            Upload metadata.
        """
        ...
    async def send_message(self, to: JID, message: MessageProto) -> SendResult:
        """Send a pre-built protobuf message."""
        ...
    async def send_text(
        self, to: JID, text: str, quoted: EvMessage | None = None
    ) -> SendResult:
        """Send a plain text message.

        Args:
            to: Recipient JID.
            text: Message body.
            quoted: Optional message to quote-reply.

        Returns:
            SendResult with the server-assigned message ID.

        Example::

            result = await client.send_text(
                to=JID('123', 's.whatsapp.net'),
                text='Hello!',
            )
            print(result.message_id)
        """
        ...
    async def send_photo(
        self,
        to: JID,
        photo_data: bytes,
        mimetype: str | None = None,
        caption: str | None = None,
        quoted: EvMessage | None = None,
    ) -> SendResult:
        """Send an image with an optional caption."""
        ...
    async def send_document(
        self,
        to: JID,
        document_data: bytes,
        mimetype: str | None = None,
        file_name: str | None = None,
        caption: str | None = None,
        quoted: EvMessage | None = None,
    ) -> SendResult:
        """Send a document/file with an optional caption."""
        ...
    async def send_audio(
        self,
        to: JID,
        audio_data: bytes,
        mimetype: str | None = None,
        ptt: bool = False,
        seconds: int | None = None,
        quoted: EvMessage | None = None,
    ) -> SendResult:
        """Send an audio clip.

        Args:
            to: Recipient JID.
            audio_data: Raw audio bytes.
            mimetype: MIME type (auto-detected if None).
            ptt: If True, send as push-to-talk voice message.
            seconds: Duration in seconds (used for voice messages).
            quoted: Optional message to quote-reply.
        """
        ...
    async def send_video(
        self,
        to: JID,
        video_data: bytes,
        mimetype: str | None = None,
        caption: str | None = None,
        seconds: int | None = None,
        gif_playback: bool = False,
        quoted: EvMessage | None = None,
    ) -> SendResult:
        """Send a video with an optional caption."""
        ...
    async def send_gif(
        self,
        to: JID,
        gif_data: bytes,
        caption: str | None = None,
        seconds: int | None = None,
        quoted: EvMessage | None = None,
    ) -> SendResult:
        """Send a GIF (sent as video with gif_playback=True)."""
        ...
    async def send_sticker(
        self,
        to: JID,
        sticker_data: bytes,
        is_animated: bool = False,
        quoted: EvMessage | None = None,
    ) -> SendResult:
        """Send a sticker (static WEBP or animated)."""
        ...
    async def request_media_reupload(
        self,
        message_id: str,
        chat_jid: JID,
        media_key: bytes,
        is_from_me: bool = False,
        participant: JID | None = None,
    ) -> MediaReuploadResult:
        """Request WhatsApp to re-upload expired media for re-download."""
        ...

class CallHandle:
    """Handle for an active voice/video call."""

    call_id: str
    peer: JID
    def is_muted(self) -> bool:
        """Return True if the local microphone is muted."""
        ...
    def set_muted(self, muted: bool) -> None:
        """Mute or unmute the local microphone."""
        ...
    async def hangup(self) -> None:
        """End the call."""
        ...
    async def wait_ended(self) -> None:
        """Block until the call is ended (by either party)."""
        ...
    async def start_video(
        self, video_source: VideoSource, video_sink: VideoSink
    ) -> None:
        """Start sending and receiving video in the call."""
        ...
    async def stop_video(self) -> None:
        """Stop sending video (audio continues)."""
        ...
    async def invite_participant(self, target: JID) -> None:
        """Invite a participant to a group call."""
        ...
    async def ring_participant(self, target: JID) -> None:
        """Ring a specific participant in a group call."""
        ...
    async def start_screen_share(self, screen_share_id: int | None = None) -> None:
        """Start sharing screen content."""
        ...
    async def stop_screen_share(self) -> None:
        """Stop screen sharing."""
        ...
    async def set_approval_required(self, enabled: bool) -> None:
        """Toggle whether new participants need admin approval."""
        ...
    async def admit_waiting_user(self, target: JID) -> None:
        """Admit a user waiting in the lobby."""
        ...
    async def deny_waiting_user(self, target: JID) -> None:
        """Deny a user waiting in the lobby."""
        ...

class IncomingCallEvent:
    """Event emitted when an incoming call is received."""

    call_id: str
    peer: JID
    is_video: bool
    async def accept(
        self, audio_source: AudioSource, audio_sink: AudioSink
    ) -> CallHandle:
        """Accept the incoming call and return a handle for control.

        Returns:
            CallHandle for controlling the call.
        """
        ...
    async def reject(self) -> None:
        """Reject the incoming call."""
        ...

class VoipClient:
    """VoIP client for making and receiving voice/video calls."""

    async def call(
        self, peer: JID, audio_source: AudioSource, audio_sink: AudioSink
    ) -> CallHandle:
        """Start a 1:1 voice call.

        Returns:
            CallHandle for controlling the call.
        """
        ...
    async def group_call(
        self,
        peers: list[JID],
        audio_source: AudioSource,
        audio_sink: AudioSink,
        video_source: VideoSource | None = None,
        video_sink: VideoSink | None = None,
    ) -> CallHandle:
        """Start a group voice/video call with multiple participants."""
        ...
    async def join_call_link(
        self,
        token_or_url: str,
        media: str,
        audio_source: AudioSource,
        audio_sink: AudioSink,
        video_source: VideoSource | None = None,
        video_sink: VideoSink | None = None,
    ) -> CallHandle:
        """Join a call via invite link."""
        ...
    async def video_call(
        self,
        peer: JID,
        audio_source: AudioSource,
        audio_sink: AudioSink,
        video_source: VideoSource,
        video_sink: VideoSink,
    ) -> CallHandle:
        """Start a 1:1 video call with camera and microphone."""
        ...

class AdvancedClient:
    """Advanced diagnostics, lifecycle waits, and raw protocol escape hatches."""
    def is_logged_in(self) -> bool:
        """Return True if the session is authenticated."""
        ...
    def get_push_name(self) -> str:
        """Return the current push name (display name)."""
        ...
    def get_pn(self) -> JID | None:
        """Return the phone-number JID, or None if not linked."""
        ...
    def get_lid(self) -> JID | None:
        """Return the LID (linked ID) JID, or None if not linked."""
        ...
    def stats(self) -> dict[str, int]:
        """Return internal counters (handlers, messages, etc.)."""
        ...
    async def memory_report_text(self) -> str:
        """Return a human-readable memory usage report."""
        ...
    async def resource_report_text(self) -> str:
        """Return a human-readable resource usage report."""
        ...
    async def wait_for_socket(self, timeout_seconds: float) -> None:
        """Block until the WebSocket connection is established."""
        ...
    async def wait_for_connected(self, timeout_seconds: float) -> None:
        """Block until the client is fully connected."""
        ...
    async def wait_for_startup_sync(self, timeout_seconds: float) -> None:
        """Block until the initial history sync completes."""
        ...
    async def flush_pending_signal_state(self) -> None:
        """Flush any pending Signal protocol state to the store."""
        ...
    async def send_raw_bytes(self, plaintext: bytes) -> None:
        """Send raw encrypted bytes directly over the socket."""
        ...
    async def send_node(self, node: Node) -> None:
        """Send a protocol node directly over the socket."""
        ...
    def set_force_active_delivery_receipts(self, active: bool) -> None:
        """Toggle forced active delivery receipts."""
        ...

class LabelsClient:
    """WhatsApp label app-state operations."""
    async def create_label(self, label_id: str, name: str, color: int) -> None:
        """Create a new label with the given name and color."""
        ...
    async def delete_label(self, label_id: str) -> None:
        """Delete a label by ID."""
        ...
    async def add_chat_label(self, jid: JID, label_id: str) -> None:
        """Attach a label to a chat."""
        ...
    async def remove_chat_label(self, jid: JID, label_id: str) -> None:
        """Remove a label from a chat."""
        ...

class CommentsClient:
    """Channel/comment operations anchored to a received parent message."""
    async def send_text(self, parent: EvMessage, text: str) -> str:
        """Reply to a channel message with text."""
        ...
    async def send_message(self, parent: EvMessage, message: MessageProto) -> str:
        """Reply to a channel message with a protobuf message."""
        ...

class EventResponse:
    """RSVP response for WhatsApp events."""

    Going: EventResponse
    NotGoing: EventResponse
    Maybe: EventResponse

class EventsClient:
    """WhatsApp event creation and RSVP operations."""
    async def create(
        self,
        chat_jid: JID,
        name: str,
        start_time: int | None = None,
        end_time: int | None = None,
        description: str | None = None,
        join_link: str | None = None,
        is_scheduled_call: bool | None = None,
        extra_guests_allowed: bool | None = None,
    ) -> dict[str, object]:
        """Create a WhatsApp event in a chat."""
        ...
    async def respond(
        self,
        chat_jid: JID,
        event_message_id: str,
        event_creator_jid: JID,
        message_secret: bytes,
        response: EventResponse,
        extra_guest_count: int | None = None,
    ) -> str:
        """RSVP to a WhatsApp event."""
        ...

class ContactClient:
    """Contact and profile lookup operations."""
    async def get_info(self, phones: list[str]) -> list[ContactInfo]:
        """Look up contact info by phone numbers."""
        ...
    async def get_user_info(self, jid: JID) -> dict[JID, UserInfo]:
        """Get detailed user profile info by JID."""
        ...
    async def get_profile_picture(self, jid: JID, preview: bool) -> ProfilePicture:
        """Fetch the profile picture metadata for a JID."""
        ...
    async def is_on_whatsapp(self, jid: list[JID]) -> list[IsOnWhatsAppResult]:
        """Check which JIDs are registered on WhatsApp."""
        ...

class ChatActionsClient:
    """Chat-level actions such as archive, pin, mute, and reactions."""
    @staticmethod
    def build_message_key(
        id: str,
        remote_jid: JID,
        from_me: bool,
        participant: JID | None = None,
    ) -> MessageKey:
        """Build a protobuf MessageKey from its components."""
        ...
    @staticmethod
    def build_message_range(
        last_message_timestamp: int,
        last_system_message_timestamp: int | None,
        messages: list[tuple[MessageKey, int]],
    ) -> SyncActionValue.SyncActionMessageRange:
        """Build a SyncActionMessageRange for sync operations."""
        ...
    async def archive_chat(
        self,
        jid: JID,
        message_range: SyncActionValue.SyncActionMessageRange | None = None,
    ) -> None:
        """Archive a chat."""
        ...
    async def unarchive_chat(
        self,
        jid: JID,
        message_range: SyncActionValue.SyncActionMessageRange | None = None,
    ) -> None:
        """Unarchive a chat."""
        ...
    async def pin_chat(self, jid: JID) -> None:
        """Pin a chat to the top of the list."""
        ...
    async def unpin_chat(self, jid: JID) -> None:
        """Unpin a chat."""
        ...
    async def mute_chat(self, jid: JID) -> None:
        """Mute a chat indefinitely."""
        ...
    async def mute_chat_until(self, jid: JID, mute_end_timestamp_ms: int) -> None:
        """Mute a chat until the given Unix timestamp (ms)."""
        ...
    async def unmute_chat(self, jid: JID) -> None:
        """Unmute a chat."""
        ...
    async def star_message(
        self,
        chat_jid: JID,
        participant_jid: JID | None,
        message_id: str,
        from_me: bool,
    ) -> None:
        """Star a message in a chat."""
        ...
    async def unstar_message(
        self,
        chat_jid: JID,
        participant_jid: JID | None,
        message_id: str,
        from_me: bool,
    ) -> None:
        """Unstar a message in a chat."""
        ...
    async def mark_chat_as_read(
        self,
        jid: JID,
        read: bool,
        message_range: SyncActionValue.SyncActionMessageRange | None = None,
    ) -> None:
        """Mark a chat as read or unread."""
        ...
    async def delete_chat(
        self,
        jid: JID,
        delete_media: bool,
        message_range: SyncActionValue.SyncActionMessageRange | None = None,
    ) -> None:
        """Delete an entire chat (with optional media deletion)."""
        ...
    async def delete_message_for_me(
        self,
        chat_jid: JID,
        participant_jid: JID | None,
        message_id: str,
        from_me: bool,
        delete_media: bool,
        message_timestamp: int | None = None,
    ) -> None:
        """Delete a single message for the local user only."""
        ...
    async def clear_chat(
        self,
        jid: JID,
        delete_starred: bool,
        delete_media: bool,
        message_range: SyncActionValue.SyncActionMessageRange | None = None,
    ) -> None:
        """Clear all messages in a chat."""
        ...
    async def save_contact(
        self,
        jid: JID,
        full_name: str | None = None,
        first_name: str | None = None,
        save_on_primary_addressbook: bool = False,
    ) -> None:
        """Save a contact to the address book."""
        ...
    async def edit_message(
        self,
        chat_jid: JID,
        original_id: str,
        new_message: MessageProto,
    ) -> str:
        """Edit a previously sent message."""
        ...
    async def revoke_message(
        self,
        chat_jid: JID,
        message_id: str,
        original_sender: JID | None = None,
    ) -> None:
        """Revoke (delete for everyone) a sent message."""
        ...
    async def react_message(
        self,
        chat_jid: JID,
        message_id: str,
        reaction: str,
        from_me: bool = False,
        participant_jid: JID | None = None,
    ) -> str:
        """Add or remove an emoji reaction to a message."""
        ...

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
    ) -> None:
        """Create options for a new community.

        Args:
            name: Community name.
            description: Optional description.
            closed: If True, only admins can add subgroups.
            allow_non_admin_sub_group_creation: Allow non-admins to create subgroups.
            create_general_chat: Auto-create a general chat.
        """
        ...

class CreateCommunityResult:
    """Type stub for CreateCommunityResult."""

    gid: JID
    metadata: GroupMetadata

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
    def classify_group(metadata: GroupMetadata) -> GroupType:
        """Classify a group as Default, Community, LinkedSubgroup, etc."""
        ...
    async def create(self, options: CreateCommunityOptions) -> CreateCommunityResult:
        """Create a new community with the given options."""
        ...
    async def deactivate(self, community_jid: JID) -> None:
        """Deactivate (archive) a community."""
        ...
    async def link_subgroups(
        self,
        community_jid: JID,
        subgroup_jids: list[JID],
    ) -> LinkSubgroupsResult:
        """Link existing groups as subgroups of a community."""
        ...
    async def unlink_subgroups(
        self,
        community_jid: JID,
        subgroup_jids: list[JID],
        remove_orphan_members: bool,
    ) -> UnlinkSubgroupsResult:
        """Unlink subgroups from a community."""
        ...
    async def get_subgroups(self, community_jid: JID) -> list[CommunitySubgroup]:
        """List all subgroups in a community."""
        ...
    async def get_subgroup_participant_counts(
        self,
        community_jid: JID,
    ) -> list[tuple[JID, int]]:
        """Get participant counts for each subgroup."""
        ...
    async def query_linked_group(
        self,
        community_jid: JID,
        subgroup_jid: JID,
    ) -> GroupMetadata:
        """Query metadata for a specific linked subgroup."""
        ...
    async def join_subgroup(
        self,
        community_jid: JID,
        subgroup_jid: JID,
    ) -> GroupMetadata:
        """Join a subgroup within a community."""
        ...
    async def get_linked_groups_participants(
        self,
        community_jid: JID,
    ) -> list[GroupParticipant]:
        """Get all participants across linked groups."""
        ...

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

class NewsletterAdminProfile:
    """Admin profile information for a newsletter."""

    id: str | None
    name: str
    picture_id: str | None
    picture_direct_path: str | None

class NewsletterAdminInfo:
    """Administrative information for a newsletter."""

    admin_count: int | None
    admin_profile: NewsletterAdminProfile | None
    admin_profiles_enabled: bool | None

class NewsletterFollower:
    """Follower entry for a newsletter."""

    jid: JID
    phone_jid: JID | None
    display_name: str | None
    username: str | None
    role: NewsletterRole | None
    follow_time: int | None
    admin_profile: NewsletterAdminProfile | None

class NewsletterClient:
    """Type stub for NewsletterClient."""
    async def list_subscribed(self) -> list[NewsletterMetadata]:
        """List all newsletters the account is subscribed to."""
        ...
    async def get_admin_info(self, jid: JID) -> NewsletterAdminInfo:
        """Get admin information for a newsletter."""
        ...
    async def get_followers(self, jid: JID, count: int) -> list[NewsletterFollower]:
        """Get the follower list for a newsletter."""
        ...
    async def get_metadata(self, jid: JID) -> NewsletterMetadata:
        """Get metadata for a newsletter by JID."""
        ...
    async def get_metadata_by_invite(self, invite_code: str) -> NewsletterMetadata:
        """Get metadata for a newsletter by invite code."""
        ...
    async def create(
        self,
        name: str,
        description: str | None = None,
    ) -> NewsletterMetadata:
        """Create a new newsletter channel."""
        ...
    async def join(self, jid: JID) -> NewsletterMetadata:
        """Join a newsletter by JID."""
        ...
    async def leave(self, jid: JID) -> None:
        """Leave a newsletter."""
        ...
    async def update(
        self,
        jid: JID,
        name: str | None = None,
        description: str | None = None,
    ) -> NewsletterMetadata:
        """Update newsletter name or description."""
        ...
    async def subscribe_live_updates(self, jid: JID) -> int:
        """Subscribe to live updates for a newsletter.

        Returns:
            Ticket ID for the live update stream.
        """
        ...
    async def send_message(self, jid: JID, message: MessageProto) -> str:
        """Send a protobuf message to a newsletter."""
        ...
    async def send_reaction(self, jid: JID, server_id: int, reaction: str) -> None:
        """React to a newsletter message."""
        ...
    async def set_follower_mute(self, jid: JID, muted: bool) -> None:
        """Mute or unmute newsletter notifications as a follower."""
        ...
    async def set_admin_mute(self, jid: JID, muted: bool) -> None:
        """Mute or unmute newsletter notifications as an admin."""
        ...
    async def edit_message(
        self,
        jid: JID,
        message_id: str,
        message: MessageProto,
    ) -> None:
        """Edit a previously sent newsletter message."""
        ...
    async def revoke_message(self, jid: JID, message_id: str) -> None:
        """Revoke a newsletter message."""
        ...
    async def get_messages(
        self,
        jid: JID,
        count: int,
        before: int | None = None,
    ) -> list[NewsletterMessage]:
        """Fetch recent messages from a newsletter."""
        ...

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
    ) -> None:
        """Create a group participant entry.

        Args:
            jid: Participant JID.
            phone_number: Optional phone number JID.
            privacy: Optional privacy bytes.
        """
        ...

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
        membership_approval_mode: MembershipApprovalMode
        | None = MembershipApprovalMode.Off,
        ephemeral_expiration: int | None = 0,
        is_parent: bool = False,
        closed: bool = False,
        allow_non_admin_sub_group_creation: bool = False,
        create_general_chat: bool = False,
    ) -> None:
        """Create options for a new group.

        Args:
            subject: Group name.
            participants: List of initial participants.
            member_link_mode: Who can share the group link.
            member_add_mode: Who can add members.
            membership_approval_mode: Require approval for new members.
            ephemeral_expiration: Disappearing message timer in seconds (0 = off).
            is_parent: If True, create as a parent group.
            closed: If True, only admins can edit group info.
            allow_non_admin_sub_group_creation: Allow non-admins to create subgroups.
            create_general_chat: Auto-create a general chat.
        """
        ...

class CreateGroupResult:
    """Type stub for CreateGroupResult."""

    gid: JID
    metadata: GroupMetadata

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
    async def query_info(self, jid: JID) -> GroupInfo:
        """Query basic group info (participants, addressing mode)."""
        ...
    async def get_participating(self) -> dict[str, GroupMetadata]:
        """Get all groups the account is participating in."""
        ...
    async def get_metadata(self, jid: JID) -> GroupMetadata:
        """Get full group metadata including participants."""
        ...
    async def create_group(self, options: CreateGroupOptions) -> CreateGroupResult:
        """Create a new group with the given options."""
        ...
    async def set_subject(self, jid: JID, subject: str) -> None:
        """Change the group name (subject)."""
        ...
    async def set_description(
        self,
        jid: JID,
        description: str | None = None,
        prev: str | None = None,
    ) -> None:
        """Set or update the group description."""
        ...
    async def leave(self, jid: JID) -> None:
        """Leave a group."""
        ...
    async def add_participants(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]:
        """Add participants to a group."""
        ...
    async def remove_participants(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]:
        """Remove participants from a group."""
        ...
    async def promote_participants(self, jid: JID, participants: list[JID]) -> None:
        """Promote participants to group admin."""
        ...
    async def demote_participants(self, jid: JID, participants: list[JID]) -> None:
        """Demote admins to regular participants."""
        ...
    async def get_invite_link(self, jid: JID, reset: bool) -> str:
        """Get (or reset) the group invite link."""
        ...
    async def set_locked(self, jid: JID, locked: bool) -> None:
        """Lock or unlock group info changes to admins only."""
        ...
    async def set_announce(self, jid: JID, announce: bool) -> None:
        """Set whether only admins can send messages."""
        ...
    async def set_ephemeral(self, jid: JID, expiration: int) -> None:
        """Set disappearing message timer (seconds, 0 to disable)."""
        ...
    async def set_membership_approval(
        self,
        jid: JID,
        mode: MembershipApprovalMode,
    ) -> None:
        """Set membership approval mode (On/Off)."""
        ...
    async def join_with_invite_code(self, code: str) -> JoinGroupResult:
        """Join a group using an invite code."""
        ...
    async def join_with_invite_v4(
        self,
        group_jid: JID,
        code: str,
        expiration: int,
        admin_jid: JID,
    ) -> JoinGroupResult:
        """Join a group using a v4 invite link components."""
        ...
    async def get_invite_info(self, code: str) -> GroupMetadata:
        """Preview group metadata from an invite code."""
        ...
    async def get_membership_requests(self, jid: JID) -> list[MembershipRequest]:
        """List pending membership requests."""
        ...
    async def approve_membership_requests(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]:
        """Approve pending membership requests."""
        ...
    async def reject_membership_requests(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]:
        """Reject pending membership requests."""
        ...
    async def set_member_add_mode(self, jid: JID, mode: MemberAddMode) -> None:
        """Set who can add members (admin-only or all)."""
        ...
    async def set_no_frequently_forwarded(
        self,
        jid: JID,
        restrict: bool,
    ) -> None:
        """Toggle the frequently-forwarded restriction."""
        ...
    async def set_allow_admin_reports(self, jid: JID, allow: bool) -> None:
        """Toggle whether admins can send reports."""
        ...
    async def set_group_history(self, jid: JID, enabled: bool) -> None:
        """Toggle group history visibility for new members."""
        ...
    async def set_member_link_mode(self, jid: JID, mode: MemberLinkMode) -> None:
        """Set the member link mode (admin or all members)."""
        ...
    async def set_limit_sharing(self, jid: JID, enabled: bool) -> None:
        """Toggle the limit-sharing restriction."""
        ...
    async def cancel_membership_requests(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]:
        """Cancel pending membership requests (self)."""
        ...
    async def revoke_request_code(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]:
        """Revoke membership request codes for participants."""
        ...
    async def acknowledge(self, jid: JID) -> None:
        """Acknowledge group info (mark as seen)."""
        ...
    async def set_profile_picture(self, jid: JID, image_data: bytes) -> str:
        """Set the group profile picture from raw image bytes."""
        ...
    async def remove_profile_picture(self, jid: JID) -> str:
        """Remove the group profile picture."""
        ...
    async def update_member_label(self, jid: JID, label: str) -> None:
        """Update the label for a group member."""
        ...

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
    ) -> None:
        """Create status send options.

        Args:
            privacy: Privacy setting (Contacts, AllowList, DenyList).
        """
        ...

class StatusClient:
    """Type stub for StatusClient."""
    async def send_text(
        self,
        text: str,
        background_argb: int,
        font: int,
        recipients: list[JID],
        options: StatusSendOptions | None = None,
    ) -> str:
        """Send a text status update."""
        ...
    async def send_image(
        self,
        upload: UploadResponse,
        thumbnail: bytes,
        recipients: list[JID],
        caption: str | None = None,
        options: StatusSendOptions | None = None,
    ) -> str:
        """Send an image status update."""
        ...
    async def send_video(
        self,
        upload: UploadResponse,
        thumbnail: bytes,
        duration_seconds: int,
        recipients: list[JID],
        caption: str | None = None,
        options: StatusSendOptions | None = None,
    ) -> str:
        """Send a video status update."""
        ...
    async def send_raw(
        self,
        message: MessageProto,
        recipients: list[JID],
        options: StatusSendOptions | None = None,
    ) -> str:
        """Send a raw protobuf message as a status update."""
        ...
    async def revoke(
        self,
        message_id: str,
        recipients: list[JID],
        options: StatusSendOptions | None = None,
    ) -> str:
        """Revoke a status update by message ID."""
        ...
    @staticmethod
    def default_privacy() -> StatusPrivacySetting:
        """Return the default status privacy setting."""
        ...

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
    ) -> None:
        """Create a disallowed list entry.

        Args:
            action: Add or Remove.
            jid: Target JID.
            pn_jid: Optional phone-number JID.
        """
        ...

class DisallowedListUpdate:
    """Type stub for DisallowedListUpdate."""

    dhash: str
    users: list[DisallowedListUserEntry]

    def __init__(
        self,
        dhash: str,
        users: list[DisallowedListUserEntry] = [],
    ) -> None:
        """Create a disallowed list update.

        Args:
            dhash: Current list hash.
            users: List of user entries to add/remove.
        """
        ...

class ChatstateClient:
    """Type stub for ChatstateClient."""
    async def send(self, to: JID, state: ChatStateType) -> None:
        """Send a chat state (composing, recording, or paused)."""
        ...
    async def send_composing(self, to: JID) -> None:
        """Send typing indicator to a chat."""
        ...
    async def send_recording(self, to: JID) -> None:
        """Send recording indicator to a chat."""
        ...
    async def send_paused(self, to: JID) -> None:
        """Send paused indicator to a chat."""
        ...

class BlockingClient:
    """Type stub for BlockingClient."""
    async def block(self, jid: JID) -> None:
        """Block a JID."""
        ...
    async def unblock(self, jid: JID) -> None:
        """Unblock a JID."""
        ...
    async def get_blocklist(self) -> list[BlocklistEntry]:
        """Return the list of blocked JIDs."""
        ...
    async def is_blocked(self, jid: JID) -> bool:
        """Return True if the JID is blocked."""
        ...

class ProfileClient:
    """Type stub for ProfileClient."""
    async def set_push_name(self, name: str) -> None:
        """Update the account display name."""
        ...
    async def set_status_text(self, text: str) -> None:
        """Update the account about/status text."""
        ...
    async def set_profile_picture(self, image_data: bytes) -> str:
        """Set the profile picture from raw image bytes."""
        ...
    async def remove_profile_picture(self) -> str:
        """Remove the profile picture."""
        ...

class PrivacyClient:
    """Type stub for PrivacyClient."""
    async def fetch_settings(self) -> list[PrivacySetting]:
        """Fetch all current privacy settings."""
        ...
    async def set_setting(
        self,
        category: PrivacyCategory,
        value: PrivacyValue,
    ) -> str | None:
        """Set a privacy category to a specific value."""
        ...
    async def set_disallowed_list(
        self,
        category: PrivacyCategory,
        update: DisallowedListUpdate,
    ) -> str | None:
        """Update the disallowed list for a privacy category."""
        ...
    async def set_default_disappearing_mode(self, duration_seconds: int) -> None:
        """Set the default disappearing message duration (seconds)."""
        ...

class PollsClient:
    """Type stub for PollsClient."""
    async def create(
        self,
        to: JID,
        name: str,
        options: list[str],
        selectable_count: int,
    ) -> tuple[str, bytes]:
        """Create a poll in a chat.

        Returns:
            (message_id, poll_enc_key) tuple.
        """
        ...
    async def vote(
        self,
        chat_jid: JID,
        poll_msg_id: str,
        poll_creator_jid: JID,
        message_secret: bytes,
        option_names: list[str],
    ) -> str:
        """Cast a vote on a poll message."""
        ...
    @staticmethod
    def decrypt_vote(
        enc_payload: bytes,
        enc_iv: bytes,
        message_secret: bytes,
        poll_msg_id: str,
        poll_creator_jid: JID,
        voter_jid: JID,
    ) -> list[bytes]:
        """Decrypt a single poll vote without LID/PN fallback."""
        ...
    @staticmethod
    def aggregate_votes(
        poll_options: list[str],
        votes: list[tuple[JID, bytes, bytes]],
        message_secret: bytes,
        poll_msg_id: str,
        poll_creator_jid: JID,
    ) -> list[PollOptionResult]:
        """Aggregate multiple poll votes into per-option results."""
        ...

class PresenceClient:
    """Type stub for PresenceClient."""
    async def set(self, status: PresenceStatus) -> None:
        """Set presence status (Available or Unavailable)."""
        ...
    async def set_available(self) -> None:
        """Set presence to Available."""
        ...
    async def set_unavailable(self) -> None:
        """Set presence to Unavailable."""
        ...
    async def subscribe(self, jid: JID) -> None:
        """Subscribe to presence updates for a JID."""
        ...
    async def unsubscribe(self, jid: JID) -> None:
        """Unsubscribe from presence updates for a JID."""
        ...
