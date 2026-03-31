# Troubleshooting

## Bot Does Not Connect

Checklist:

1. verify network access
2. verify backend path is writable
3. inspect `EvConnectFailure` and `EvStreamError`
4. confirm account is still paired

## Frequent Reconnects

Possible causes:

- unstable network
- another active session replacing stream (`EvStreamReplaced`)
- backend corruption or stale state

## Message Send Fails

- confirm `JID` target format
- ensure account is connected
- verify protobuf payload correctness
- inspect returned exception type (`PyPayloadBuildError`, `EventDispatchError`)

## Media Download Fails

- check whether media URL/direct path expired
- request reupload and retry
- verify correct media subtype object passed to `download_media`

## Sync Events Are Confusing

Sync events are normal in WhatsApp multi-device behavior.
Treat events like `EvArchiveUpdate`, `EvMarkChatAsReadUpdate`, `EvDeleteChatUpdate` as state convergence signals.
