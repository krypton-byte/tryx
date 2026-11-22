# :material-broadcast: Events API

::: tryx.events.Dispatcher
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.events.EvMessage
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.events.MessageData
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.events.EvConnected
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.events.EvDisconnected
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.events.EvReceipt
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.events.ReceiptType
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.events.ChatPresence
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.events.ChatPresenceMedia
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.events.EvPresence
    options:
      show_root_heading: true
      heading_level: 2

::: tryx.events.EvHistorySync
    options:
      show_root_heading: true
      heading_level: 2

---

This page maps event classes in `tryx.events` to practical handler strategies.

## Dispatcher Contract

`Dispatcher` is used internally by `Tryx` and by `@app.on(EventClass)` registration.
Only event types with at least one registered handler are dispatched across the
Rust-Python boundary (interest filtering). Both `async def` and synchronous `def`
handlers are supported.

```python
@app.on(EvMessage)
async def on_message(client, event): ...
```

### `EvMessage` Shortcut Properties

`EvMessage` exposes lightweight Rust-backed getters so common handlers do not
need to materialize `event.data` or `raw_proto`:

- `event.id`, `event.chat`, `event.sender`, `event.timestamp`
- `event.is_from_me`, `event.is_group`, `event.push_name`
- `event.text`, `event.caption`, `event.media_type`, `event.has_media`

### `EvHistorySync` Lazy Decompression

`EvHistorySync` wraps `LazyHistorySync` and inflates payloads off-GIL only when
`event.raw_bytes()` or `event.proto` is accessed. Metadata (`sync_type`,
`chunk_order`, `progress`, `peer_data_request_session_id`, `decompressed_size`)
is available without inflating.

!!! tip "Handler model"
	Keep handlers small, push expensive work into background tasks, and treat incoming event payloads as typed contracts.

## Event Taxonomy

### Lifecycle

- `EvConnected`
- `EvDisconnected`
- `EvLoggedOut`
- `EvStreamReplaced`
- `EvClientOutDated`
- `EvClientExpirationChanged`
- `EvOfflineSyncPreview`
- `EvOfflineSyncCompleted`
- `EvOfflineSyncInterrupted`
- `EvAppStateSyncFailed`

### Pairing

- `EvPairingQrCode`
- `EvPairingCode`
- `EvPairSuccess`
- `EvPairError`
- `EvQrScannedWithoutMultidevice`

### Messaging

- `EvMessage`
- `EvReceipt`
- `EvServerAck`
- `EvUndecryptableMessage`
- `EvNotification`
- `EvMexNotification`

### Sync Actions

- `EvPinUpdate`
- `EvMuteUpdate`
- `EvArchiveUpdate`
- `EvMarkChatAsReadUpdate`
- `EvDeleteChatUpdate`
- `EvDeleteMessageForMeUpdate`
- `EvStarUpdate`
- `EvContactUpdate`
- `EvLockChatUpdate`
- `EvFavoritesUpdate`
- `EvStatusPrivacyUpdate`
- `EvUnarchiveChatsSettingUpdate`
- `EvDisableLinkPreviewsUpdate`

### Contact, Profile, Presence

- `EvPushNameUpdate`
- `EvSelfPushNameUpdated`
- `EvUserAboutUpdate`
- `EvPictureUpdate`
- `EvPresence`
- `EvChatPresence`
- `EvContactUpdated`
- `EvContactRemoved`
- `EvContactNumberChanged`
- `EvContactSyncRequested`
- `EvIdentityChange`

### Device and Business

- `EvDeviceListUpdate`
- `EvBusinessStatusUpdate`

### Group, Newsletter, and Calls

- `EvJoinedGroup`
- `EvGroupInfoUpdate`
- `EvGroupUpdate`
- `EvNewsletterLiveUpdate`
- `IncomingCallEvent` (from `tryx.client`)

## Event-to-Namespace Mapping

| Event family | Namespace actions usually paired |
| --- | --- |
| Messaging | [Chat Actions](chat-actions.md), [Contact](contact.md), root send methods |
| Group updates | [Groups](groups.md), [Community](community.md) |
| Newsletter updates | [Newsletter](newsletter.md), [Polls](polls.md) |
| Presence updates | [Presence](presence.md), [Chatstate](chatstate.md) |
| Profile updates | [Profile](profile.md), [Privacy](privacy.md) |

## Payload Discipline

=== "Recommended"
	- Read typed fields from `event.data`.
	- Guard optional values (`None`) before usage.
	- Log identity metadata (`chat_jid`, `sender`, `message_id`) for observability.

=== "Avoid"
	- Parsing raw protobuf bytes when typed fields already exist.
	- Long blocking work inside handler coroutine.
	- Assuming strict order between unrelated event classes.

## Example: Safe Event Router

```python
from tryx.events import EvMessage, EvPresence


@app.on(EvMessage)
async def on_message(client, event):
    chat = event.data.message_info.source.chat
    text = event.data.get_text() or ""
    if text == "/ping":
        await client.send_text(chat, "pong", quoted=event)


@app.on(EvPresence)
async def on_presence(client, event):
    # keep side effects minimal; enqueue heavy processing
    pass
```

## Example: Group Subject Change with LID Addressing

When a group subject (name) is changed, the `Subject` action carries the renamer's identity. In LID-addressed groups, the phone-number alias arrives in a separate field:

```python
from tryx.events import EvGroupUpdate, GroupNotificationAction


@app.on(EvGroupUpdate)
async def on_group_update(client, event):
    action = event.data.action

    if isinstance(action, GroupNotificationAction.Subject):
        renamer = action.subject_owner  # JID (may be a LID)
        renamer_pn = action.subject_owner_pn  # phone-number JID (when LID addressing)
        renamer_name = action.subject_owner_username  # username (when enabled)

        # Prefer the phone-number JID for display when available
        display_jid = renamer_pn or renamer
        print(f"Group renamed to {action.subject!r} by {display_jid}")

        # Log all identity fields for audit
        if renamer_pn:
            print(f"  LID: {renamer}, PN: {renamer_pn}")
        if renamer_name:
            print(f"  Username: {renamer_name}")
```

!!! tip "LID vs Phone Number"
	In WhatsApp's LID addressing mode, `subject_owner` may be a `@lid` JID that is not human-readable. The `subject_owner_pn` field provides the corresponding phone-number JID for display. When LID addressing is not active, `subject_owner_pn` is `None` and `subject_owner` already contains the phone-number JID.

## Enum-like Support Types

Common reason/state classes used by event payloads:

- `TempBanReason`
- `ReceiptType`
- `UnavailableType`
- `DecryptFailMode`
- `ChatPresence`, `ChatPresenceMedia`
- `DeviceListUpdateType`
- `BusinessStatusUpdateType`
- `GroupNotificationAction`

!!! warning "Reliability"
	Treat sync events as convergence signals, not anomalies. They are expected in multi-device behavior.
