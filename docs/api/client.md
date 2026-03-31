# Client API

`TryxClient` is the main runtime client object passed to event handlers.

## Root Methods

- `is_connected() -> bool`
- `download_media(message) -> bytes`
- `upload_file(path, media_type) -> UploadResponse`
- `upload(data, media_type) -> UploadResponse`
- `send_message(to, message) -> SendResult`
- `send_text(...) -> SendResult`
- `send_photo(...) -> SendResult`
- `send_document(...) -> SendResult`
- `send_audio(...) -> SendResult`
- `send_video(...) -> SendResult`
- `send_gif(...) -> SendResult`
- `send_sticker(...) -> SendResult`
- `request_media_reupload(...) -> MediaReuploadResult`

## Namespace Clients

| Namespace | Purpose |
| --- | --- |
| `client.contact` | Contact lookup and profile picture info |
| `client.chat_actions` | Archive, pin, mute, mark-read, edit/revoke/react |
| `client.community` | Community and linked subgroup operations |
| `client.newsletter` | Newsletter metadata, messaging, subscription |
| `client.groups` | Group management lifecycle |
| `client.status` | Status posting and privacy |
| `client.chatstate` | Typing/recording/paused state |
| `client.blocking` | Blocklist operations |
| `client.polls` | Poll creation and vote cryptography helpers |
| `client.presence` | Presence updates and subscriptions |
| `client.privacy` | Privacy setting changes |
| `client.profile` | Push name, bio, and profile picture updates |

## ContactClient

- `get_info(phones)`
- `get_user_info(jid)`
- `get_profile_picture(jid, preview)`
- `is_on_whatsapp(jids)`

## ChatActionsClient

Notable operations:

- archive/unarchive chat
- pin/unpin chat
- mute/unmute chat
- mark chat as read
- delete chat
- delete message for me
- edit, revoke, and react to messages

Builder helpers:

- `build_message_key(...)`
- `build_message_range(...)`

## Groups and Community

`GroupsClient` handles conventional group lifecycle.
`CommunityClient` focuses on linked subgroup topology and community-specific operations.

## Newsletter

`NewsletterClient` supports:

- listing subscriptions
- metadata lookup
- joining/leaving
- sending messages and reactions
- polling historical messages

## Privacy and Profile

Use `PrivacyClient` for settings and disallowed lists.
Use `ProfileClient` for account-facing text and picture changes.

## Operational Tip

For production handlers, use namespace methods from the event callback and avoid storing stale references across reconnect boundaries.
