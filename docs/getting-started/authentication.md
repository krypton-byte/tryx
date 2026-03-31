# Authentication Flow

Tryx follows the WhatsApp multi-device pairing flow. The first run links a session, and later runs reuse stored state.

## Pairing Modes

- QR pairing event: `EvPairingQrCode`
- Numeric pairing event: `EvPairingCode`
- Success event: `EvPairSuccess`
- Failure event: `EvPairError`

## Typical First-Run Sequence

1. Start bot runtime.
2. Wait for `EvPairingQrCode` or `EvPairingCode`.
3. Complete pairing from your WhatsApp mobile app.
4. Receive `EvPairSuccess`.
5. Session credentials are persisted in your backend.

## Persistence

Use a stable backend path:

```python
from tryx.backend import SqliteBackend

backend = SqliteBackend("/srv/tryx/session.db")
```

If the same backend path is reused, you usually do not need to pair again.

## Operational Guidance

- Keep one active session owner for a backend path.
- Avoid deleting backend files unless resetting account link is intentional.
- Back up backend data before infrastructure migration.

## Recovery Signals

- `EvLoggedOut`: account session is no longer valid.
- `EvStreamReplaced`: another login/session replaced your current stream.
- `EvTemporaryBan`: temporary restrictions detected; pause high-volume operations.
