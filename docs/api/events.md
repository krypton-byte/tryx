# Events API

This page summarizes the event classes exported by `tryx.events`.

## Dispatcher

`Dispatcher` is the internal callback registry used by `Tryx`.

- `on(EventClass)` selects a target event class.
- calling the returned object with a function registers a callback.

## Lifecycle Events

- `EvConnected`
- `EvDisconnected`
- `EvLoggedOut`
- `EvStreamReplaced`
- `EvClientOutDated`

## Pairing Events

- `EvPairingQrCode`
- `EvPairingCode`
- `EvPairSuccess`
- `EvPairError`

## Messaging Events

- `EvMessage`
- `EvReceipt`
- `EvUndecryptableMessage`
- `EvNotification`

## Sync Action Events

- `EvPinUpdate`
- `EvMuteUpdate`
- `EvArchiveUpdate`
- `EvMarkChatAsReadUpdate`
- `EvDeleteChatUpdate`
- `EvDeleteMessageForMeUpdate`
- `EvStarUpdate`
- `EvContactUpdate`

## Contact/Profile/Presence Events

- `EvPushNameUpdate`
- `EvSelfPushNameUpdated`
- `EvUserAboutUpdate`
- `EvPictureUpdate`
- `EvPresence`
- `EvChatPresence`
- `EvContactUpdated`
- `EvContactNumberChanged`
- `EvContactSyncRequested`

## Device and Business Events

- `EvDeviceListUpdate`
- `EvBusinessStatusUpdate`

## Group and Newsletter Events

- `EvJoinedGroup`
- `EvGroupInfoUpdate`
- `EvGroupUpdate`
- `EvNewsletterLiveUpdate`

## Data Shape Guidance

Most event classes expose `data` with a dedicated payload class.
Use explicit payload attributes instead of parsing raw protobuf bytes whenever possible.

## Enum-Like Event Types

Tryx also exports reason/state classes used in event payloads, such as:

- `TempBanReason`
- `ReceiptType`
- `UnavailableType`
- `DecryptFailMode`
- `ChatPresence` and `ChatPresenceMedia`
- `DeviceListUpdateType`
- `BusinessStatusUpdateType`
- `GroupNotificationAction` (opaque variant object)

## Recommendation

Treat event classes as contracts. If your logic depends on a field, type it explicitly and guard optional values.
