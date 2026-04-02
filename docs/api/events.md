# Events API

This page maps event classes in `tryx.events` to practical handler strategies.

## Dispatcher Contract

`Dispatcher` is used internally by `Tryx` and by `@app.on(EventClass)` registration.

```python
@app.on(EvMessage)
async def on_message(client, event):
	...
```

!!! tip "Handler model"
	Keep handlers small, push expensive work into background tasks, and treat incoming event payloads as typed contracts.

## Event Taxonomy

### Lifecycle

- `EvConnected`
- `EvDisconnected`
- `EvLoggedOut`
- `EvStreamReplaced`
- `EvClientOutDated`

### Pairing

- `EvPairingQrCode`
- `EvPairingCode`
- `EvPairSuccess`
- `EvPairError`

### Messaging

- `EvMessage`
- `EvReceipt`
- `EvUndecryptableMessage`
- `EvNotification`

### Sync Actions

- `EvPinUpdate`
- `EvMuteUpdate`
- `EvArchiveUpdate`
- `EvMarkChatAsReadUpdate`
- `EvDeleteChatUpdate`
- `EvDeleteMessageForMeUpdate`
- `EvStarUpdate`
- `EvContactUpdate`

### Contact, Profile, Presence

- `EvPushNameUpdate`
- `EvSelfPushNameUpdated`
- `EvUserAboutUpdate`
- `EvPictureUpdate`
- `EvPresence`
- `EvChatPresence`
- `EvContactUpdated`
- `EvContactNumberChanged`
- `EvContactSyncRequested`

### Device and Business

- `EvDeviceListUpdate`
- `EvBusinessStatusUpdate`

### Group and Newsletter

- `EvJoinedGroup`
- `EvGroupInfoUpdate`
- `EvGroupUpdate`
- `EvNewsletterLiveUpdate`

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
