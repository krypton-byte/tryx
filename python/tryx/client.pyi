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
        """Send a pre-built protobuf message directly.

        Use this for raw protocol-level sends when the helper methods
        (``send_text``, ``send_photo``, etc.) are not sufficient.

        Args:
            to: Recipient JID.
            message: Fully constructed protobuf Message.

        Returns:
            SendResult with the server-assigned message ID.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            from tryx.waproto.whatsapp_pb2 import Message

            msg = Message(conversation="Hello from raw proto!")
            result = await client.send_message(
                to=JID('123', 's.whatsapp.net'),
                message=msg,
            )
        """
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
        """Send an image with an optional caption.

        The image is uploaded to WhatsApp servers before the message is sent.
        If *mimetype* is ``None``, it is auto-detected from the raw bytes
        (defaults to ``image/jpeg`` on failure).

        Args:
            to: Recipient JID.
            photo_data: Raw image bytes.
            mimetype: MIME type (auto-detected if None).
            caption: Optional image caption.
            quoted: Optional message to quote-reply.

        Returns:
            SendResult with the server-assigned message ID.

        Raises:
            RuntimeError: If the client is not running or upload fails.

        Example::

            with open('photo.jpg', 'rb') as f:
                photo_data = f.read()
            result = await client.send_photo(
                to=JID('123', 's.whatsapp.net'),
                photo_data=photo_data,
                caption='Check this out!',
            )
        """
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
        """Send a document/file with an optional caption.

        The document is uploaded to WhatsApp servers before the message is sent.
        If *mimetype* is ``None``, it is auto-detected from the raw bytes
        (defaults to ``application/octet-stream`` on failure).

        Args:
            to: Recipient JID.
            document_data: Raw document bytes.
            mimetype: MIME type (auto-detected if None).
            file_name: Display file name on the recipient side.
            caption: Optional document caption.
            quoted: Optional message to quote-reply.

        Returns:
            SendResult with the server-assigned message ID.

        Raises:
            RuntimeError: If the client is not running or upload fails.

        Example::

            with open('report.pdf', 'rb') as f:
                doc_data = f.read()
            result = await client.send_document(
                to=JID('123', 's.whatsapp.net'),
                document_data=doc_data,
                file_name='report.pdf',
                caption='Monthly report',
            )
        """
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

        The audio is uploaded to WhatsApp servers before the message is sent.
        If *mimetype* is ``None``, it is auto-detected from the raw bytes
        (defaults to ``audio/ogg; codecs=opus`` on failure).

        Args:
            to: Recipient JID.
            audio_data: Raw audio bytes.
            mimetype: MIME type (auto-detected if None).
            ptt: If True, send as push-to-talk voice message.
            seconds: Duration in seconds (used for voice messages).
            quoted: Optional message to quote-reply.

        Returns:
            SendResult with the server-assigned message ID.

        Raises:
            RuntimeError: If the client is not running or upload fails.

        Example::

            with open('voice.ogg', 'rb') as f:
                audio_data = f.read()
            result = await client.send_audio(
                to=JID('123', 's.whatsapp.net'),
                audio_data=audio_data,
                ptt=True,
                seconds=5,
            )
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
        """Send a video with an optional caption.

        The video is uploaded to WhatsApp servers before the message is sent.
        If *mimetype* is ``None``, it is auto-detected from the raw bytes
        (defaults to ``video/mp4`` on failure).

        Args:
            to: Recipient JID.
            video_data: Raw video bytes.
            mimetype: MIME type (auto-detected if None).
            caption: Optional video caption.
            seconds: Duration in seconds.
            gif_playback: If True, display as a GIF on the recipient side.
            quoted: Optional message to quote-reply.

        Returns:
            SendResult with the server-assigned message ID.

        Raises:
            RuntimeError: If the client is not running or upload fails.

        Example::

            with open('clip.mp4', 'rb') as f:
                video_data = f.read()
            result = await client.send_video(
                to=JID('123', 's.whatsapp.net'),
                video_data=video_data,
                caption='Funny clip',
                seconds=15,
            )
        """
        ...
    async def send_gif(
        self,
        to: JID,
        gif_data: bytes,
        caption: str | None = None,
        seconds: int | None = None,
        quoted: EvMessage | None = None,
    ) -> SendResult:
        """Send a GIF (sent as video with gif_playback=True).

        Convenience wrapper around :meth:`send_video` that sets
        ``gif_playback=True`` and ``mimetype='video/mp4'``.

        Args:
            to: Recipient JID.
            gif_data: Raw GIF/video bytes.
            caption: Optional caption.
            seconds: Duration in seconds.
            quoted: Optional message to quote-reply.

        Returns:
            SendResult with the server-assigned message ID.

        Raises:
            RuntimeError: If the client is not running or upload fails.

        Example::

            with open('animation.gif', 'rb') as f:
                gif_data = f.read()
            result = await client.send_gif(
                to=JID('123', 's.whatsapp.net'),
                gif_data=gif_data,
                caption='Look at this!',
            )
        """
        ...
    async def send_sticker(
        self,
        to: JID,
        sticker_data: bytes,
        is_animated: bool = False,
        quoted: EvMessage | None = None,
    ) -> SendResult:
        """Send a sticker (static WEBP or animated).

        The sticker is uploaded to WhatsApp servers before the message is sent.
        MIME type is auto-detected from the raw bytes (defaults to
        ``image/webp`` on failure).

        Args:
            to: Recipient JID.
            sticker_data: Raw sticker bytes.
            is_animated: If True, mark as an animated sticker.
            quoted: Optional message to quote-reply.

        Returns:
            SendResult with the server-assigned message ID.

        Raises:
            RuntimeError: If the client is not running or upload fails.

        Example::

            with open('sticker.webp', 'rb') as f:
                sticker_data = f.read()
            result = await client.send_sticker(
                to=JID('123', 's.whatsapp.net'),
                sticker_data=sticker_data,
            )
        """
        ...
    async def request_media_reupload(
        self,
        message_id: str,
        chat_jid: JID,
        media_key: bytes,
        is_from_me: bool = False,
        participant: JID | None = None,
    ) -> MediaReuploadResult:
        """Request WhatsApp to re-upload expired media for re-download.

        When a media direct path has expired, this method asks WhatsApp
        servers to re-upload the media so it can be downloaded again.

        Args:
            message_id: Server-assigned message ID.
            chat_jid: JID of the chat containing the message.
            media_key: The media encryption key bytes.
            is_from_me: Whether the message was sent by the local user.
            participant: Optional participant JID (for group messages).

        Returns:
            MediaReuploadResult with the new direct path and status.

        Raises:
            ValueError: If *media_key* is empty.
            RuntimeError: If the client is not running.

        Example::

            result = await client.request_media_reupload(
                message_id='3EB0ABC123',
                chat_jid=JID('123', 's.whatsapp.net'),
                media_key=media_key_bytes,
                is_from_me=True,
            )
            if result.status == 'ok':
                data = await client.download_media(message.image_message)
        """
        ...

class CallHandle:
    """Handle for an active voice/video call."""

    call_id: str
    peer: JID

    def is_muted(self) -> bool:
        """Return True if the local microphone is muted."""
        ...

    def set_muted(self, muted: bool) -> None:
        """Mute or unmute the local microphone.

        Args:
            muted: ``True`` to mute, ``False`` to unmute.

        Example::

            handle.set_muted(True)
        """
        ...

    async def hangup(self) -> None:
        """End the call.

        Raises:
            RuntimeError: If the call has already ended.

        Example::

            await handle.hangup()
        """
        ...

    async def wait_ended(self) -> None:
        """Block until the call is ended (by either party).

        Raises:
            RuntimeError: If the call handle is invalid.

        Example::

            await handle.wait_ended()
            print('Call finished')
        """
        ...

    async def start_video(
        self, video_source: VideoSource, video_sink: VideoSink
    ) -> None:
        """Start sending and receiving video in the call.

        Args:
            video_source: Source providing outgoing video frames.
            video_sink: Sink receiving incoming video frames.

        Raises:
            RuntimeError: If the call has ended or video adapter is missing.
        """
        ...

    async def stop_video(self) -> None:
        """Stop sending video (audio continues).

        Raises:
            RuntimeError: If the call has ended.
        """
        ...

    async def invite_participant(self, target: JID) -> None:
        """Invite a participant to a group call.

        Args:
            target: JID of the participant to invite.

        Raises:
            RuntimeError: If the call has ended.

        Example::

            await handle.invite_participant(JID('5599800001', 's.whatsapp.net'))
        """
        ...

    async def ring_participant(self, target: JID) -> None:
        """Ring a specific participant in a group call.

        Args:
            target: JID of the participant to ring.

        Raises:
            RuntimeError: If the call has ended.
        """
        ...

    async def start_screen_share(self, screen_share_id: int | None = None) -> None:
        """Start sharing screen content.

        Args:
            screen_share_id: Optional screen share identifier.

        Raises:
            RuntimeError: If the call has ended.
        """
        ...

    async def stop_screen_share(self) -> None:
        """Stop screen sharing.

        Raises:
            RuntimeError: If the call has ended.
        """
        ...

    async def set_approval_required(self, enabled: bool) -> None:
        """Toggle whether new participants need admin approval.

        Args:
            enabled: ``True`` to require approval, ``False`` to disable.

        Raises:
            RuntimeError: If the call has ended.
        """
        ...

    async def admit_waiting_user(self, target: JID) -> None:
        """Admit a user waiting in the lobby.

        Args:
            target: JID of the user to admit.

        Raises:
            RuntimeError: If the call has ended.
        """
        ...

    async def deny_waiting_user(self, target: JID) -> None:
        """Deny a user waiting in the lobby.

        Args:
            target: JID of the user to deny.

        Raises:
            RuntimeError: If the call has ended.
        """
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

        Args:
            audio_source: Source providing outgoing audio frames.
            audio_sink: Sink receiving incoming audio frames.

        Returns:
            CallHandle for controlling the call.

        Raises:
            RuntimeError: If the call was already consumed.

        Example::

            handle = await event.accept(audio_source, audio_sink)
            await handle.wait_ended()
        """
        ...

    async def reject(self) -> None:
        """Reject the incoming call.

        Raises:
            RuntimeError: If the call was already consumed.

        Example::

            await event.reject()
        """
        ...

class VoipClient:
    """VoIP client for making and receiving voice/video calls."""

    async def call(
        self, peer: JID, audio_source: AudioSource, audio_sink: AudioSink
    ) -> CallHandle:
        """Start a 1:1 voice call.

        Args:
            peer: JID of the callee.
            audio_source: Source providing outgoing audio frames.
            audio_sink: Sink receiving incoming audio frames.

        Returns:
            CallHandle for controlling the call.

        Raises:
            RuntimeError: If the call fails to connect.

        Example::

            handle = await client.voip.call(peer, audio_source, audio_sink)
            await handle.wait_ended()
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
        """Start a group voice/video call with multiple participants.

        Args:
            peers: JIDs of the participants to call.
            audio_source: Source providing outgoing audio frames.
            audio_sink: Sink receiving incoming audio frames.
            video_source: Optional source providing outgoing video frames.
            video_sink: Optional sink receiving incoming video frames.

        Returns:
            CallHandle for controlling the call.

        Raises:
            RuntimeError: If the call fails to connect.
        """
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
        """Join a call via invite link.

        Args:
            token_or_url: Call link token or full URL.
            media: Media type to join with (e.g. ``'audio'``, ``'video'``).
            audio_source: Source providing outgoing audio frames.
            audio_sink: Sink receiving incoming audio frames.
            video_source: Optional source providing outgoing video frames.
            video_sink: Optional sink receiving incoming video frames.

        Returns:
            CallHandle for controlling the call.

        Raises:
            RuntimeError: If the call link is invalid or join fails.
        """
        ...

    async def video_call(
        self,
        peer: JID,
        audio_source: AudioSource,
        audio_sink: AudioSink,
        video_source: VideoSource,
        video_sink: VideoSink,
    ) -> CallHandle:
        """Start a 1:1 video call with camera and microphone.

        Args:
            peer: JID of the callee.
            audio_source: Source providing outgoing audio frames.
            audio_sink: Sink receiving incoming audio frames.
            video_source: Source providing outgoing video frames.
            video_sink: Sink receiving incoming video frames.

        Returns:
            CallHandle for controlling the call.

        Raises:
            RuntimeError: If the call fails to connect.

        Example::

            handle = await client.voip.video_call(
                peer, audio_source, audio_sink,
                video_source, video_sink,
            )
            await handle.start_video(video_source, video_sink)
        """
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
        """Block until the WebSocket connection is established.

        Args:
            timeout_seconds: Maximum seconds to wait before timing out.

        Raises:
            RuntimeError: If the timeout is reached.
        """
        ...

    async def wait_for_connected(self, timeout_seconds: float) -> None:
        """Block until the client is fully connected.

        Args:
            timeout_seconds: Maximum seconds to wait before timing out.

        Raises:
            RuntimeError: If the timeout is reached.

        Example::

            await client.advanced.wait_for_connected(30.0)
        """
        ...

    async def wait_for_startup_sync(self, timeout_seconds: float) -> None:
        """Block until the initial history sync completes.

        Args:
            timeout_seconds: Maximum seconds to wait before timing out.

        Raises:
            RuntimeError: If the timeout is reached.
        """
        ...

    async def flush_pending_signal_state(self) -> None:
        """Flush any pending Signal protocol state to the store.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.advanced.flush_pending_signal_state()
        """
        ...

    async def send_raw_bytes(self, plaintext: bytes) -> None:
        """Send raw encrypted bytes directly over the socket.

        Args:
            plaintext: Pre-encrypted bytes to send.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def send_node(self, node: Node) -> None:
        """Send a protocol node directly over the socket.

        Args:
            node: Protocol node to send.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    def set_force_active_delivery_receipts(self, active: bool) -> None:
        """Toggle forced active delivery receipts.

        Args:
            active: ``True`` to force active delivery receipts.
        """
        ...

class LabelsClient:
    """WhatsApp label app-state operations.

    All methods raise ``RuntimeError`` if the client is not running.
    """

    async def create_label(self, label_id: str, name: str, color: int) -> None:
        """Create a new label with the given name and color.

        Args:
            label_id: Unique label identifier.
            name: Display name for the label.
            color: Label color value.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.labels.create_label('L1', 'Urgent', 0xFF0000)
        """
        ...

    async def delete_label(self, label_id: str) -> None:
        """Delete a label by ID.

        Args:
            label_id: Unique label identifier.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def add_chat_label(self, jid: JID, label_id: str) -> None:
        """Attach a label to a chat.

        Args:
            jid: Chat JID to label.
            label_id: Label identifier to attach.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def remove_chat_label(self, jid: JID, label_id: str) -> None:
        """Remove a label from a chat.

        Args:
            jid: Chat JID.
            label_id: Label identifier to remove.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

class CommentsClient:
    """Channel/comment operations anchored to a received parent message.

    All methods raise ``RuntimeError`` if the client is not running.
    """

    async def send_text(self, parent: EvMessage, text: str) -> str:
        """Reply to a channel message with text.

        Args:
            parent: The parent message to reply to.
            text: Comment text body.

        Returns:
            Server-assigned message ID of the comment.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            msg_id = await client.comments.send_text(event, 'Great post!')
        """
        ...

    async def send_message(self, parent: EvMessage, message: MessageProto) -> str:
        """Reply to a channel message with a protobuf message.

        Args:
            parent: The parent message to reply to.
            message: Fully constructed protobuf Message.

        Returns:
            Server-assigned message ID of the comment.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

class EventResponse:
    """RSVP response for WhatsApp events."""

    Going: EventResponse
    NotGoing: EventResponse
    Maybe: EventResponse

class EventsClient:
    """WhatsApp event creation and RSVP operations.

    All methods raise ``RuntimeError`` if the client is not running.
    """

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
        """Create a WhatsApp event in a chat.

        Args:
            chat_jid: Chat where the event is created.
            name: Event name/title.
            start_time: Optional start time as Unix timestamp.
            end_time: Optional end time as Unix timestamp.
            description: Optional event description.
            join_link: Optional join link for the event.
            is_scheduled_call: Whether the event is a scheduled call.
            extra_guests_allowed: Whether extra guests are allowed.

        Returns:
            Dict with event creation details.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            import time
            result = await client.events.create(
                chat_jid=JID('123', 's.whatsapp.net'),
                name='Team standup',
                start_time=int(time.time()) + 3600,
                description='Daily standup meeting',
            )
        """
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
        """RSVP to a WhatsApp event.

        Args:
            chat_jid: Chat containing the event.
            event_message_id: Message ID of the event.
            event_creator_jid: JID of the event creator.
            message_secret: Event message secret bytes.
            response: RSVP response (Going, NotGoing, Maybe).
            extra_guest_count: Optional number of extra guests.

        Returns:
            Server-assigned message ID of the RSVP response.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

class ContactClient:
    """Contact and profile lookup operations.

    All methods raise ``RuntimeError`` if the client is not running.
    """

    async def get_info(self, phones: list[str]) -> list[ContactInfo]:
        """Look up contact info by phone numbers.

        Args:
            phones: List of phone number strings (e.g. ``['5599800001']``).

        Returns:
            List of ContactInfo entries for each phone number.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            results = await client.contact.get_info(['5599800001'])
            for info in results:
                print(info.jid, info.is_registered)
        """
        ...

    async def get_user_info(self, jid: JID) -> dict[JID, UserInfo]:
        """Get detailed user profile info by JID.

        Args:
            jid: Target user JID.

        Returns:
            Dict mapping JIDs to their UserInfo profiles.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            info_map = await client.contact.get_user_info(sender_jid)
            user_info = next(iter(info_map.values()), None)
            if user_info:
                print(user_info.status)
        """
        ...

    async def get_profile_picture(self, jid: JID, preview: bool) -> ProfilePicture:
        """Fetch the profile picture metadata for a JID.

        Args:
            jid: Target user or group JID.
            preview: If True, return the small preview version.

        Returns:
            ProfilePicture with URL, direct_path, and hash.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            pic = await client.contact.get_profile_picture(sender_jid, False)
            if pic.url:
                data = await download_bytes(pic.url)
        """
        ...

    async def is_on_whatsapp(self, jid: list[JID]) -> list[IsOnWhatsAppResult]:
        """Check which JIDs are registered on WhatsApp.

        Args:
            jid: List of JIDs to check.

        Returns:
            List of IsOnWhatsAppResult with registration status.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            results = await client.contact.is_on_whatsapp([
                JID('5599800001', 's.whatsapp.net'),
            ])
            for r in results:
                print(r.jid, r.is_registered)
        """
        ...

class ChatActionsClient:
    """Chat-level actions such as archive, pin, mute, and reactions.

    All async methods raise ``RuntimeError`` if the client is not running.
    """

    @staticmethod
    def build_message_key(
        id: str,
        remote_jid: JID,
        from_me: bool,
        participant: JID | None = None,
    ) -> MessageKey:
        """Build a protobuf MessageKey from its components.

        Args:
            id: Message ID.
            remote_jid: Chat JID.
            from_me: Whether the message was sent by the local user.
            participant: Optional participant JID (for group chats).

        Returns:
            Constructed MessageKey proto.

        Example::

            key = ChatActionsClient.build_message_key(
                id='3EB0ABC123',
                remote_jid=JID('123', 's.whatsapp.net'),
                from_me=True,
            )
        """
        ...

    @staticmethod
    def build_message_range(
        last_message_timestamp: int,
        last_system_message_timestamp: int | None,
        messages: list[tuple[MessageKey, int]],
    ) -> SyncActionValue.SyncActionMessageRange:
        """Build a SyncActionMessageRange for sync operations.

        Args:
            last_message_timestamp: Timestamp of the last message.
            last_system_message_timestamp: Timestamp of the last system
                message, or ``None``.
            messages: List of ``(MessageKey, timestamp)`` tuples.

        Returns:
            Constructed SyncActionMessageRange proto.

        Example::

            key = ChatActionsClient.build_message_key(
                '3EB0', JID('123', 's.whatsapp.net'), True,
            )
            msg_range = ChatActionsClient.build_message_range(
                last_message_timestamp=1700000000,
                last_system_message_timestamp=None,
                messages=[(key, 1700000000)],
            )
        """
        ...

    async def archive_chat(
        self,
        jid: JID,
        message_range: SyncActionValue.SyncActionMessageRange | None = None,
    ) -> None:
        """Archive a chat.

        Args:
            jid: Chat JID to archive.
            message_range: Optional sync message range.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.chat_actions.archive_chat(JID('123', 's.whatsapp.net'))
        """
        ...

    async def unarchive_chat(
        self,
        jid: JID,
        message_range: SyncActionValue.SyncActionMessageRange | None = None,
    ) -> None:
        """Unarchive a chat.

        Args:
            jid: Chat JID to unarchive.
            message_range: Optional sync message range.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def pin_chat(self, jid: JID) -> None:
        """Pin a chat to the top of the list.

        Args:
            jid: Chat JID to pin.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.chat_actions.pin_chat(JID('123', 's.whatsapp.net'))
        """
        ...

    async def unpin_chat(self, jid: JID) -> None:
        """Unpin a chat.

        Args:
            jid: Chat JID to unpin.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def mute_chat(self, jid: JID) -> None:
        """Mute a chat indefinitely.

        Args:
            jid: Chat JID to mute.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.chat_actions.mute_chat(JID('123', 's.whatsapp.net'))
        """
        ...

    async def mute_chat_until(self, jid: JID, mute_end_timestamp_ms: int) -> None:
        """Mute a chat until the given Unix timestamp (ms).

        Args:
            jid: Chat JID to mute.
            mute_end_timestamp_ms: Expiration timestamp in milliseconds.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            import time
            await client.chat_actions.mute_chat_until(
                JID('123', 's.whatsapp.net'),
                int(time.time() * 1000) + 3600000,
            )
        """
        ...

    async def unmute_chat(self, jid: JID) -> None:
        """Unmute a chat.

        Args:
            jid: Chat JID to unmute.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def star_message(
        self,
        chat_jid: JID,
        participant_jid: JID | None,
        message_id: str,
        from_me: bool,
    ) -> None:
        """Star a message in a chat.

        Args:
            chat_jid: Chat JID.
            participant_jid: Participant JID (for group chats), or None.
            message_id: Message ID to star.
            from_me: Whether the message was sent by the local user.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def unstar_message(
        self,
        chat_jid: JID,
        participant_jid: JID | None,
        message_id: str,
        from_me: bool,
    ) -> None:
        """Unstar a message in a chat.

        Args:
            chat_jid: Chat JID.
            participant_jid: Participant JID (for group chats), or None.
            message_id: Message ID to unstar.
            from_me: Whether the message was sent by the local user.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def mark_chat_as_read(
        self,
        jid: JID,
        read: bool,
        message_range: SyncActionValue.SyncActionMessageRange | None = None,
    ) -> None:
        """Mark a chat as read or unread.

        Args:
            jid: Chat JID.
            read: ``True`` to mark as read, ``False`` to mark as unread.
            message_range: Optional sync message range.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.chat_actions.mark_chat_as_read(jid, True)
        """
        ...

    async def delete_chat(
        self,
        jid: JID,
        delete_media: bool,
        message_range: SyncActionValue.SyncActionMessageRange | None = None,
    ) -> None:
        """Delete an entire chat (with optional media deletion).

        Args:
            jid: Chat JID to delete.
            delete_media: If True, also delete media from storage.
            message_range: Optional sync message range.

        Raises:
            RuntimeError: If the client is not running.
        """
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
        """Delete a single message for the local user only.

        Args:
            chat_jid: Chat JID.
            participant_jid: Participant JID (for group chats), or None.
            message_id: Message ID to delete.
            from_me: Whether the message was sent by the local user.
            delete_media: If True, also delete the media file.
            message_timestamp: Optional message timestamp for sync.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.chat_actions.delete_message_for_me(
                chat_jid, participant_jid, msg_id, True, False,
            )
        """
        ...

    async def clear_chat(
        self,
        jid: JID,
        delete_starred: bool,
        delete_media: bool,
        message_range: SyncActionValue.SyncActionMessageRange | None = None,
    ) -> None:
        """Clear all messages in a chat.

        Args:
            jid: Chat JID to clear.
            delete_starred: If True, also delete starred messages.
            delete_media: If True, also delete media files.
            message_range: Optional sync message range.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def save_contact(
        self,
        jid: JID,
        full_name: str | None = None,
        first_name: str | None = None,
        save_on_primary_addressbook: bool = False,
    ) -> None:
        """Save a contact to the address book.

        Args:
            jid: Contact JID.
            full_name: Optional full display name.
            first_name: Optional first name.
            save_on_primary_addressbook: If True, save to the primary
                address book.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def edit_message(
        self,
        chat_jid: JID,
        original_id: str,
        new_message: MessageProto,
    ) -> str:
        """Edit a previously sent message.

        Args:
            chat_jid: Chat JID.
            original_id: ID of the message to edit.
            new_message: Replacement protobuf message.

        Returns:
            Server-assigned message ID of the edit.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def revoke_message(
        self,
        chat_jid: JID,
        message_id: str,
        original_sender: JID | None = None,
    ) -> None:
        """Revoke (delete for everyone) a sent message.

        Args:
            chat_jid: Chat JID.
            message_id: Message ID to revoke.
            original_sender: Optional sender JID (for group messages).

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.chat_actions.revoke_message(
                chat_jid, '3EB0ABC123',
            )
        """
        ...

    async def react_message(
        self,
        chat_jid: JID,
        message_id: str,
        reaction: str,
        from_me: bool = False,
        participant_jid: JID | None = None,
    ) -> str:
        """Add or remove an emoji reaction to a message.

        Pass an empty string for *reaction* to remove an existing reaction.

        Args:
            chat_jid: Chat JID.
            message_id: Message ID to react to.
            reaction: Emoji string, or empty string to remove.
            from_me: Whether the message was sent by the local user.
            participant_jid: Optional participant JID (for group chats).

        Returns:
            Server-assigned message ID of the reaction.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.chat_actions.react_message(
                chat_jid, msg_id, '✅', from_me=True,
            )
        """
        ...

class GroupType:
    """Classification of a WhatsApp group (default, community, linked)."""

    Default: GroupType
    Community: GroupType
    LinkedSubgroup: GroupType
    LinkedAnnouncementGroup: GroupType
    LinkedGeneralGroup: GroupType

class CreateCommunityOptions:
    """Options for creating a new WhatsApp community."""

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
            allow_non_admin_sub_group_creation: Allow non-admins to create
                subgroups.
            create_general_chat: Auto-create a general chat.
        """
        ...

class CreateCommunityResult:
    """Result returned after creating a community."""

    gid: JID
    metadata: GroupMetadata

class CommunitySubgroup:
    """Subgroup entry within a WhatsApp community."""

    id: JID
    subject: str
    participant_count: int | None
    is_default_sub_group: bool
    is_general_chat: bool

class LinkSubgroupsResult:
    """Result of linking subgroups to a community."""

    linked_jids: list[JID]
    failed_groups: list[tuple[JID, int]]

class UnlinkSubgroupsResult:
    """Result of unlinking subgroups from a community."""

    unlinked_jids: list[JID]
    failed_groups: list[tuple[JID, int]]

class GroupParticipant:
    """Participant entry within a WhatsApp group."""

    jid: JID
    phone_number: JID | None
    is_admin: bool

class GroupMetadata:
    """Full metadata for a WhatsApp group."""

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
    """WhatsApp community creation and subgroup management.

    All async methods raise ``RuntimeError`` if the client is not running.
    """

    @staticmethod
    def classify_group(metadata: GroupMetadata) -> GroupType:
        """Classify a group as Default, Community, LinkedSubgroup, etc.

        Args:
            metadata: Group metadata to classify.

        Returns:
            GroupType classification.

        Example::

            group_type = CommunityClient.classify_group(metadata)
            if group_type == GroupType.Community:
                print('This is a community')
        """
        ...

    async def create(self, options: CreateCommunityOptions) -> CreateCommunityResult:
        """Create a new community with the given options.

        Args:
            options: Community creation configuration.

        Returns:
            CreateCommunityResult with group ID and metadata.

        Example::

            from tryx.client import CreateCommunityOptions
            opts = CreateCommunityOptions(name='My Community')
            result = await client.community.create(opts)
            print(result.gid)
        """
        ...

    async def deactivate(self, community_jid: JID) -> None:
        """Deactivate (archive) a community.

        Args:
            community_jid: JID of the community to deactivate.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def link_subgroups(
        self,
        community_jid: JID,
        subgroup_jids: list[JID],
    ) -> LinkSubgroupsResult:
        """Link existing groups as subgroups of a community.

        Args:
            community_jid: JID of the community.
            subgroup_jids: JIDs of groups to link.

        Returns:
            LinkSubgroupsResult with linked and failed group JIDs.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            result = await client.community.link_subgroups(
                community_jid,
                [JID('123', 'g.us'), JID('456', 'g.us')],
            )
            print(result.linked_jids)
        """
        ...

    async def unlink_subgroups(
        self,
        community_jid: JID,
        subgroup_jids: list[JID],
        remove_orphan_members: bool,
    ) -> UnlinkSubgroupsResult:
        """Unlink subgroups from a community.

        Args:
            community_jid: JID of the community.
            subgroup_jids: JIDs of subgroups to unlink.
            remove_orphan_members: If True, remove members who are no
                longer in any linked group.

        Returns:
            UnlinkSubgroupsResult with unlinked and failed group JIDs.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def get_subgroups(self, community_jid: JID) -> list[CommunitySubgroup]:
        """List all subgroups in a community.

        Args:
            community_jid: JID of the community.

        Returns:
            List of CommunitySubgroup entries.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            groups = await client.community.get_subgroups(community_jid)
            for g in groups:
                print(g.subject, g.participant_count)
        """
        ...

    async def get_subgroup_participant_counts(
        self,
        community_jid: JID,
    ) -> list[tuple[JID, int]]:
        """Get participant counts for each subgroup.

        Args:
            community_jid: JID of the community.

        Returns:
            List of ``(subgroup_jid, participant_count)`` tuples.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def query_linked_group(
        self,
        community_jid: JID,
        subgroup_jid: JID,
    ) -> GroupMetadata:
        """Query metadata for a specific linked subgroup.

        Args:
            community_jid: JID of the community.
            subgroup_jid: JID of the subgroup to query.

        Returns:
            GroupMetadata for the linked subgroup.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def join_subgroup(
        self,
        community_jid: JID,
        subgroup_jid: JID,
    ) -> GroupMetadata:
        """Join a subgroup within a community.

        Args:
            community_jid: JID of the community.
            subgroup_jid: JID of the subgroup to join.

        Returns:
            GroupMetadata after joining.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def get_linked_groups_participants(
        self,
        community_jid: JID,
    ) -> list[GroupParticipant]:
        """Get all participants across linked groups.

        Args:
            community_jid: JID of the community.

        Returns:
            List of GroupParticipant entries from all linked groups.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

class NewsletterVerification:
    """Verification status of a WhatsApp newsletter."""

    Verified: NewsletterVerification
    Unverified: NewsletterVerification

class NewsletterState:
    """Operational state of a WhatsApp newsletter."""

    Active: NewsletterState
    Suspended: NewsletterState
    Geosuspended: NewsletterState

class NewsletterRole:
    """Role of a user within a WhatsApp newsletter."""

    Owner: NewsletterRole
    Admin: NewsletterRole
    Subscriber: NewsletterRole
    Guest: NewsletterRole

class NewsletterReactionCount:
    """Reaction tally for a newsletter message."""

    code: str
    count: int

class NewsletterMetadata:
    """Metadata for a WhatsApp newsletter channel."""

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
    """A message within a WhatsApp newsletter."""

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
    """WhatsApp newsletter channel operations.

    All async methods raise ``RuntimeError`` if the client is not running.
    """

    async def list_subscribed(self) -> list[NewsletterMetadata]:
        """List all newsletters the account is subscribed to.

        Returns:
            List of NewsletterMetadata for each subscribed newsletter.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            newsletters = await client.newsletter.list_subscribed()
            for nl in newsletters:
                print(nl.name, nl.subscriber_count)
        """
        ...

    async def get_admin_info(self, jid: JID) -> NewsletterAdminInfo:
        """Get admin information for a newsletter.

        Args:
            jid: Newsletter JID.

        Returns:
            NewsletterAdminInfo with admin count and profiles.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def get_followers(self, jid: JID, count: int) -> list[NewsletterFollower]:
        """Get the follower list for a newsletter.

        Args:
            jid: Newsletter JID.
            count: Maximum number of followers to return.

        Returns:
            List of NewsletterFollower entries.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def get_metadata(self, jid: JID) -> NewsletterMetadata:
        """Get metadata for a newsletter by JID.

        Args:
            jid: Newsletter JID.

        Returns:
            NewsletterMetadata with name, subscriber count, etc.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            metadata = await client.newsletter.get_metadata(newsletter_jid)
            print(metadata.name, metadata.subscriber_count)
        """
        ...

    async def get_metadata_by_invite(self, invite_code: str) -> NewsletterMetadata:
        """Get metadata for a newsletter by invite code.

        Args:
            invite_code: Newsletter invite code.

        Returns:
            NewsletterMetadata for the newsletter.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            metadata = await client.newsletter.get_metadata_by_invite(invite_code)
            await client.newsletter.join(metadata.jid)
        """
        ...

    async def create(
        self,
        name: str,
        description: str | None = None,
    ) -> NewsletterMetadata:
        """Create a new newsletter channel.

        Args:
            name: Newsletter display name.
            description: Optional description.

        Returns:
            NewsletterMetadata of the created newsletter.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            metadata = await client.newsletter.create(
                name='My Newsletter',
                description='Weekly updates',
            )
            print(metadata.jid)
        """
        ...

    async def join(self, jid: JID) -> NewsletterMetadata:
        """Join a newsletter by JID.

        Args:
            jid: Newsletter JID to join.

        Returns:
            NewsletterMetadata after joining.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def leave(self, jid: JID) -> None:
        """Leave a newsletter.

        Args:
            jid: Newsletter JID to leave.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def update(
        self,
        jid: JID,
        name: str | None = None,
        description: str | None = None,
    ) -> NewsletterMetadata:
        """Update newsletter name or description.

        Args:
            jid: Newsletter JID.
            name: New display name, or None to keep current.
            description: New description, or None to keep current.

        Returns:
            Updated NewsletterMetadata.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def subscribe_live_updates(self, jid: JID) -> int:
        """Subscribe to live updates for a newsletter.

        Args:
            jid: Newsletter JID.

        Returns:
            Ticket ID for the live update stream.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def send_message(self, jid: JID, message: MessageProto) -> str:
        """Send a protobuf message to a newsletter.

        Args:
            jid: Newsletter JID.
            message: Fully constructed protobuf Message.

        Returns:
            Server-assigned message ID.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            from tryx.waproto.whatsapp_pb2 import Message
            msg = Message(conversation='Hello from newsletter!')
            msg_id = await client.newsletter.send_message(newsletter_jid, msg)
        """
        ...

    async def send_reaction(self, jid: JID, server_id: int, reaction: str) -> None:
        """React to a newsletter message.

        Args:
            jid: Newsletter JID.
            server_id: Server-assigned message ID to react to.
            reaction: Emoji reaction string.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def set_follower_mute(self, jid: JID, muted: bool) -> None:
        """Mute or unmute newsletter notifications as a follower.

        Args:
            jid: Newsletter JID.
            muted: ``True`` to mute, ``False`` to unmute.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def set_admin_mute(self, jid: JID, muted: bool) -> None:
        """Mute or unmute newsletter notifications as an admin.

        Args:
            jid: Newsletter JID.
            muted: ``True`` to mute, ``False`` to unmute.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def edit_message(
        self,
        jid: JID,
        message_id: str,
        message: MessageProto,
    ) -> None:
        """Edit a previously sent newsletter message.

        Args:
            jid: Newsletter JID.
            message_id: Message ID to edit.
            message: Replacement protobuf message.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def revoke_message(self, jid: JID, message_id: str) -> None:
        """Revoke a newsletter message.

        Args:
            jid: Newsletter JID.
            message_id: Message ID to revoke.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def get_messages(
        self,
        jid: JID,
        count: int,
        before: int | None = None,
    ) -> list[NewsletterMessage]:
        """Fetch recent messages from a newsletter.

        Args:
            jid: Newsletter JID.
            count: Maximum number of messages to return.
            before: Optional server_id to paginate before.

        Returns:
            List of NewsletterMessage entries.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            messages = await client.newsletter.get_messages(
                newsletter_jid, count=10,
            )
            for msg in messages:
                print(msg.server_id, msg.message_type)
        """
        ...

class MemberLinkMode:
    """Controls who can share the group invite link."""

    AdminLink: MemberLinkMode
    AllMemberLink: MemberLinkMode

class MemberAddMode:
    """Controls who can add new members to the group."""

    AdminAdd: MemberAddMode
    AllMemberAdd: MemberAddMode

class MembershipApprovalMode:
    """Controls whether new members require admin approval."""

    Off: MembershipApprovalMode
    On: MembershipApprovalMode

class GroupParticipantOptions:
    """Options for adding a participant to a group."""

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
    """Options for creating a new WhatsApp group."""

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
    """Result returned after creating a group."""

    gid: JID
    metadata: GroupMetadata

class JoinGroupResult:
    """Result returned after joining a group."""

    jid: JID
    pending_approval: bool

class ParticipantChangeResponse:
    """Response for a single participant add/remove/promote/demote operation."""

    jid: JID
    status: str | None
    error: str | None

class MembershipRequest:
    """Pending membership request for a group."""

    jid: JID
    request_time: int | None

class GroupInfo:
    """Basic group information returned by query_info."""

    participants: list[JID]
    addressing_mode: str
    lid_to_pn_map: list[tuple[str, JID]]

class GroupsClient:
    """WhatsApp group management operations.

    All async methods raise ``RuntimeError`` if the client is not running.
    """

    async def query_info(self, jid: JID) -> GroupInfo:
        """Query basic group info (participants, addressing mode).

        Args:
            jid: Group JID.

        Returns:
            GroupInfo with participants and addressing mode.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            info = await client.groups.query_info(group_jid)
            print(info.participants)
        """
        ...

    async def get_participating(self) -> dict[str, GroupMetadata]:
        """Get all groups the account is participating in.

        Returns:
            Dict mapping group JID strings to GroupMetadata.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            groups = await client.groups.get_participating()
            for jid_str, metadata in groups.items():
                print(metadata.subject)
        """
        ...

    async def get_metadata(self, jid: JID) -> GroupMetadata:
        """Get full group metadata including participants.

        Args:
            jid: Group JID.

        Returns:
            GroupMetadata with full participant list.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            metadata = await client.groups.get_metadata(group_jid)
            for p in metadata.participants:
                print(p.jid, p.is_admin)
        """
        ...

    async def create_group(self, options: CreateGroupOptions) -> CreateGroupResult:
        """Create a new group with the given options.

        Args:
            options: Group creation configuration.

        Returns:
            CreateGroupResult with group ID and metadata.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            from tryx.client import CreateGroupOptions
            opts = CreateGroupOptions(subject='Engineering Room')
            result = await client.groups.create_group(opts)
            print(result.gid)
        """
        ...

    async def set_subject(self, jid: JID, subject: str) -> None:
        """Change the group name (subject).

        Args:
            jid: Group JID.
            subject: New group name.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def set_description(
        self,
        jid: JID,
        description: str | None = None,
        prev: str | None = None,
    ) -> None:
        """Set or update the group description.

        Args:
            jid: Group JID.
            description: New description, or None to clear.
            prev: Expected previous description for optimistic concurrency.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def leave(self, jid: JID) -> None:
        """Leave a group.

        Args:
            jid: Group JID to leave.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.groups.leave(group_jid)
        """
        ...

    async def add_participants(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]:
        """Add participants to a group.

        Args:
            jid: Group JID.
            participants: List of participant JIDs to add.

        Returns:
            List of ParticipantChangeResponse for each participant.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            responses = await client.groups.add_participants(
                group_jid,
                [JID('5599800001', 's.whatsapp.net')],
            )
            for r in responses:
                print(r.jid, r.status)
        """
        ...

    async def remove_participants(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]:
        """Remove participants from a group.

        Args:
            jid: Group JID.
            participants: List of participant JIDs to remove.

        Returns:
            List of ParticipantChangeResponse for each participant.
        """
        ...

    async def promote_participants(self, jid: JID, participants: list[JID]) -> None:
        """Promote participants to group admin.

        Args:
            jid: Group JID.
            participants: List of participant JIDs to promote.
        """
        ...

    async def demote_participants(self, jid: JID, participants: list[JID]) -> None:
        """Demote admins to regular participants.

        Args:
            jid: Group JID.
            participants: List of admin JIDs to demote.
        """
        ...

    async def get_invite_link(self, jid: JID, reset: bool) -> str:
        """Get (or reset) the group invite link.

        Args:
            jid: Group JID.
            reset: If True, generate a new invite link.

        Returns:
            Group invite link string.
        """
        ...

    async def set_locked(self, jid: JID, locked: bool) -> None:
        """Lock or unlock group info changes to admins only.

        Args:
            jid: Group JID.
            locked: ``True`` to lock, ``False`` to unlock.
        """
        ...

    async def set_announce(self, jid: JID, announce: bool) -> None:
        """Set whether only admins can send messages.

        Args:
            jid: Group JID.
            announce: ``True`` for admin-only messaging.
        """
        ...

    async def set_ephemeral(self, jid: JID, expiration: int) -> None:
        """Set disappearing message timer (seconds, 0 to disable).

        Args:
            jid: Group JID.
            expiration: Timer in seconds (0 to disable).
        """
        ...

    async def set_membership_approval(
        self,
        jid: JID,
        mode: MembershipApprovalMode,
    ) -> None:
        """Set membership approval mode (On/Off).

        Args:
            jid: Group JID.
            mode: MembershipApprovalMode.On or .Off.
        """
        ...

    async def join_with_invite_code(self, code: str) -> JoinGroupResult:
        """Join a group using an invite code.

        Args:
            code: Invite code string.

        Returns:
            JoinGroupResult with group JID and pending approval status.
        """
        ...

    async def join_with_invite_v4(
        self,
        group_jid: JID,
        code: str,
        expiration: int,
        admin_jid: JID,
    ) -> JoinGroupResult:
        """Join a group using a v4 invite link components.

        Args:
            group_jid: Group JID.
            code: Invite code.
            expiration: Invite expiration timestamp.
            admin_jid: Admin JID that generated the invite.

        Returns:
            JoinGroupResult with group JID and pending approval status.
        """
        ...

    async def get_invite_info(self, code: str) -> GroupMetadata:
        """Preview group metadata from an invite code.

        Args:
            code: Invite code string.

        Returns:
            GroupMetadata for the group.
        """
        ...

    async def get_membership_requests(self, jid: JID) -> list[MembershipRequest]:
        """List pending membership requests.

        Args:
            jid: Group JID.

        Returns:
            List of MembershipRequest entries.
        """
        ...

    async def approve_membership_requests(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]:
        """Approve pending membership requests.

        Args:
            jid: Group JID.
            participants: List of participant JIDs to approve.

        Returns:
            List of ParticipantChangeResponse for each participant.
        """
        ...

    async def reject_membership_requests(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]:
        """Reject pending membership requests.

        Args:
            jid: Group JID.
            participants: List of participant JIDs to reject.

        Returns:
            List of ParticipantChangeResponse for each participant.
        """
        ...

    async def set_member_add_mode(self, jid: JID, mode: MemberAddMode) -> None:
        """Set who can add members (admin-only or all).

        Args:
            jid: Group JID.
            mode: MemberAddMode.AdminAdd or .AllMemberAdd.
        """
        ...

    async def set_no_frequently_forwarded(
        self,
        jid: JID,
        restrict: bool,
    ) -> None:
        """Toggle the frequently-forwarded restriction.

        Args:
            jid: Group JID.
            restrict: ``True`` to restrict forwarding.
        """
        ...

    async def set_allow_admin_reports(self, jid: JID, allow: bool) -> None:
        """Toggle whether admins can send reports.

        Args:
            jid: Group JID.
            allow: ``True`` to allow admin reports.
        """
        ...

    async def set_group_history(self, jid: JID, enabled: bool) -> None:
        """Toggle group history visibility for new members.

        Args:
            jid: Group JID.
            enabled: ``True`` to show history to new members.
        """
        ...

    async def set_member_link_mode(self, jid: JID, mode: MemberLinkMode) -> None:
        """Set the member link mode (admin or all members).

        Args:
            jid: Group JID.
            mode: MemberLinkMode.AdminLink or .AllMemberLink.
        """
        ...

    async def set_limit_sharing(self, jid: JID, enabled: bool) -> None:
        """Toggle the limit-sharing restriction.

        Args:
            jid: Group JID.
            enabled: ``True`` to limit sharing.
        """
        ...

    async def cancel_membership_requests(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]:
        """Cancel pending membership requests (self).

        Args:
            jid: Group JID.
            participants: List of participant JIDs whose requests to cancel.

        Returns:
            List of ParticipantChangeResponse for each participant.
        """
        ...

    async def revoke_request_code(
        self,
        jid: JID,
        participants: list[JID],
    ) -> list[ParticipantChangeResponse]:
        """Revoke membership request codes for participants.

        Args:
            jid: Group JID.
            participants: List of participant JIDs.

        Returns:
            List of ParticipantChangeResponse for each participant.
        """
        ...

    async def acknowledge(self, jid: JID) -> None:
        """Acknowledge group info (mark as seen).

        Args:
            jid: Group JID.
        """
        ...

    async def set_profile_picture(self, jid: JID, image_data: bytes) -> str:
        """Set the group profile picture from raw image bytes.

        Args:
            jid: Group JID.
            image_data: Raw image bytes.

        Returns:
            Server-assigned picture ID.
        """
        ...

    async def remove_profile_picture(self, jid: JID) -> str:
        """Remove the group profile picture.

        Args:
            jid: Group JID.

        Returns:
            Server-assigned picture ID.
        """
        ...

    async def update_member_label(self, jid: JID, label: str) -> None:
        """Update the label for a group member.

        Args:
            jid: Group JID.
            label: New label string.
        """
        ...

class StatusPrivacySetting:
    """Privacy setting for WhatsApp status updates."""

    Contacts: StatusPrivacySetting
    AllowList: StatusPrivacySetting
    DenyList: StatusPrivacySetting

class StatusSendOptions:
    """Options for sending a WhatsApp status update."""

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
    """WhatsApp status (story) posting operations.

    All async methods raise ``RuntimeError`` if the client is not running.
    """

    async def send_text(
        self,
        text: str,
        background_argb: int,
        font: int,
        recipients: list[JID],
        options: StatusSendOptions | None = None,
    ) -> str:
        """Send a text status update.

        Args:
            text: Status text body.
            background_argb: Background color as ARGB integer.
            font: Font style identifier.
            recipients: List of recipient JIDs.
            options: Optional status send options.

        Returns:
            Server-assigned message ID.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            msg_id = await client.status.send_text(
                text='Hello world!',
                background_argb=0xFF1F9D86,
                font=1,
                recipients=[JID('status', 'status')],
            )
        """
        ...

    async def send_image(
        self,
        upload: UploadResponse,
        thumbnail: bytes,
        recipients: list[JID],
        caption: str | None = None,
        options: StatusSendOptions | None = None,
    ) -> str:
        """Send an image status update.

        Args:
            upload: Upload metadata from a prior upload call.
            thumbnail: Thumbnail image bytes.
            recipients: List of recipient JIDs.
            caption: Optional image caption.
            options: Optional status send options.

        Returns:
            Server-assigned message ID.
        """
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
        """Send a video status update.

        Args:
            upload: Upload metadata from a prior upload call.
            thumbnail: Thumbnail image bytes.
            duration_seconds: Video duration in seconds.
            recipients: List of recipient JIDs.
            caption: Optional video caption.
            options: Optional status send options.

        Returns:
            Server-assigned message ID.
        """
        ...

    async def send_raw(
        self,
        message: MessageProto,
        recipients: list[JID],
        options: StatusSendOptions | None = None,
    ) -> str:
        """Send a raw protobuf message as a status update.

        Args:
            message: Fully constructed protobuf Message.
            recipients: List of recipient JIDs.
            options: Optional status send options.

        Returns:
            Server-assigned message ID.
        """
        ...

    async def revoke(
        self,
        message_id: str,
        recipients: list[JID],
        options: StatusSendOptions | None = None,
    ) -> str:
        """Revoke a status update by message ID.

        Args:
            message_id: Status message ID to revoke.
            recipients: List of recipient JIDs.
            options: Optional status send options.

        Returns:
            Server-assigned message ID of the revocation.
        """
        ...

    @staticmethod
    def default_privacy() -> StatusPrivacySetting:
        """Return the default status privacy setting.

        Returns:
            StatusPrivacySetting.Contacts.
        """
        ...

class ChatStateType:
    """Chat state indicator type (composing, recording, paused)."""

    Composing: ChatStateType
    Recording: ChatStateType
    Paused: ChatStateType

class BlocklistEntry:
    """A single entry in the user's blocklist."""

    jid: JID
    timestamp: int | None

class PollOptionResult:
    """Aggregated result for a single poll option."""

    name: str
    voters: list[str]

class PresenceStatus:
    """Online presence status (available or unavailable)."""

    Available: PresenceStatus
    Unavailable: PresenceStatus

class PrivacyCategory:
    """Category of a WhatsApp privacy setting."""

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
    """Value for a WhatsApp privacy setting."""

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
    """Action to apply to a disallowed list entry."""

    Add: DisallowedListAction
    Remove: DisallowedListAction

class PrivacySetting:
    """A privacy category-value pair."""

    category: PrivacyCategory
    value: PrivacyValue

class DisallowedListUserEntry:
    """A single entry in a privacy disallowed list."""

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
    """Update payload for a privacy disallowed list."""

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
    """Chat state indicators (typing, recording, paused).

    All methods raise ``RuntimeError`` if the client is not running.
    """

    async def send(self, to: JID, state: ChatStateType) -> None:
        """Send a chat state (composing, recording, or paused).

        Args:
            to: Chat JID.
            state: ChatStateType indicator to send.
        """
        ...

    async def send_composing(self, to: JID) -> None:
        """Send typing indicator to a chat.

        Args:
            to: Chat JID.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.chatstate.send_composing(chat_jid)
            await asyncio.sleep(2)
            await client.chatstate.send_paused(chat_jid)
        """
        ...

    async def send_recording(self, to: JID) -> None:
        """Send recording indicator to a chat.

        Args:
            to: Chat JID.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def send_paused(self, to: JID) -> None:
        """Send paused indicator to a chat.

        Args:
            to: Chat JID.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

class BlockingClient:
    """Block and unblock WhatsApp contacts.

    All methods raise ``RuntimeError`` if the client is not running.
    """

    async def block(self, jid: JID) -> None:
        """Block a JID.

        Args:
            jid: JID to block.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.blocking.block(sender_jid)
        """
        ...

    async def unblock(self, jid: JID) -> None:
        """Unblock a JID.

        Args:
            jid: JID to unblock.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def get_blocklist(self) -> list[BlocklistEntry]:
        """Return the list of blocked JIDs.

        Returns:
            List of BlocklistEntry with JID and timestamp.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            blocklist = await client.blocking.get_blocklist()
            for entry in blocklist:
                print(entry.jid)
        """
        ...

    async def is_blocked(self, jid: JID) -> bool:
        """Return True if the JID is blocked.

        Args:
            jid: JID to check.

        Returns:
            ``True`` if the JID is in the blocklist.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

class ProfileClient:
    """Account profile management (name, status, picture).

    All methods raise ``RuntimeError`` if the client is not running.
    """

    async def set_push_name(self, name: str) -> None:
        """Update the account display name.

        Args:
            name: New display name.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.profile.set_push_name('My Bot')
        """
        ...

    async def set_status_text(self, text: str) -> None:
        """Update the account about/status text.

        Args:
            text: New about/status text.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

    async def set_profile_picture(self, image_data: bytes) -> str:
        """Set the profile picture from raw image bytes.

        Args:
            image_data: Raw image bytes.

        Returns:
            Server-assigned picture ID.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            with open('avatar.jpg', 'rb') as f:
                pic_id = await client.profile.set_profile_picture(f.read())
        """
        ...

    async def remove_profile_picture(self) -> str:
        """Remove the profile picture.

        Returns:
            Server-assigned picture ID.

        Raises:
            RuntimeError: If the client is not running.
        """
        ...

class PrivacyClient:
    """WhatsApp privacy settings management.

    All methods raise ``RuntimeError`` if the client is not running.
    """

    async def fetch_settings(self) -> list[PrivacySetting]:
        """Fetch all current privacy settings.

        Returns:
            List of PrivacySetting entries.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            rows = await client.privacy.fetch_settings()
            for row in rows:
                print(row.category, row.value)
        """
        ...

    async def set_setting(
        self,
        category: PrivacyCategory,
        value: PrivacyValue,
    ) -> str | None:
        """Set a privacy category to a specific value.

        Args:
            category: Privacy category to update.
            value: New privacy value.

        Returns:
            Server status string, or None on success.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.privacy.set_setting(
                PrivacyCategory.Status,
                PrivacyValue.Contacts,
            )
        """
        ...

    async def set_disallowed_list(
        self,
        category: PrivacyCategory,
        update: DisallowedListUpdate,
    ) -> str | None:
        """Update the disallowed list for a privacy category.

        Args:
            category: Privacy category to update.
            update: DisallowedListUpdate with entries to add/remove.

        Returns:
            Server status string, or None on success.
        """
        ...

    async def set_default_disappearing_mode(self, duration_seconds: int) -> None:
        """Set the default disappearing message duration (seconds).

        Args:
            duration_seconds: Duration in seconds (0 to disable).
        """
        ...

class PollsClient:
    """Poll creation, voting, and decryption operations.

    All async methods raise ``RuntimeError`` if the client is not running.
    """

    async def create(
        self,
        to: JID,
        name: str,
        options: list[str],
        selectable_count: int,
    ) -> tuple[str, bytes]:
        """Create a poll in a chat.

        Args:
            to: Chat JID.
            name: Poll question/name.
            options: List of option strings.
            selectable_count: Number of options a voter can select.

        Returns:
            ``(message_id, poll_enc_key)`` tuple.

        Raises:
            ValueError: If *options* is empty or *selectable_count* is
                invalid.
            RuntimeError: If the client is not running.

        Example::

            msg_id, secret = await client.polls.create(
                to=JID('123', 's.whatsapp.net'),
                name='Favorite color?',
                options=['Red', 'Blue', 'Green'],
                selectable_count=1,
            )
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
        """Cast a vote on a poll message.

        Args:
            chat_jid: Chat JID containing the poll.
            poll_msg_id: Poll message ID.
            poll_creator_jid: JID of the poll creator.
            message_secret: Poll message secret bytes.
            option_names: Selected option names.

        Returns:
            Server-assigned message ID of the vote.
        """
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
        """Decrypt a single poll vote without LID/PN fallback.

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
        """Aggregate multiple poll votes into per-option results.

        Args:
            poll_options: List of option name strings.
            votes: List of ``(voter_jid, enc_payload, enc_iv)`` tuples.
            message_secret: Poll message secret key.
            poll_msg_id: Poll message ID.
            poll_creator_jid: Poll creator JID.

        Returns:
            List of PollOptionResult with name and voters.
        """
        ...

class PresenceClient:
    """Online presence status management.

    All methods raise ``RuntimeError`` if the client is not running.
    """

    async def set(self, status: PresenceStatus) -> None:
        """Set presence status (Available or Unavailable).

        Args:
            status: PresenceStatus.Available or .Unavailable.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.presence.set(PresenceStatus.Available)
        """
        ...

    async def set_available(self) -> None:
        """Set presence to Available."""
        ...

    async def set_unavailable(self) -> None:
        """Set presence to Unavailable."""
        ...

    async def subscribe(self, jid: JID) -> None:
        """Subscribe to presence updates for a JID.

        Args:
            jid: JID to subscribe to.

        Raises:
            RuntimeError: If the client is not running.

        Example::

            await client.presence.subscribe(target_jid)
        """
        ...

    async def unsubscribe(self, jid: JID) -> None:
        """Unsubscribe from presence updates for a JID.

        Args:
            jid: JID to unsubscribe from.
        """
        ...
