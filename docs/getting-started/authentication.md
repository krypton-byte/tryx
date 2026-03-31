# Authentication Flow

Tryx follows the WhatsApp multi-device pairing flow. The first run links a session, and later runs reuse stored state.

!!! note
	Authentication stability is mostly a storage and ownership problem. Treat backend/session files as critical runtime state.

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

## Event-level Interpretation

| Event | Meaning | Operator action |
| --- | --- | --- |
| `EvPairingQrCode` | QR challenge issued | scan with mobile app |
| `EvPairingCode` | code-based pairing challenge | enter code in paired device flow |
| `EvPairSuccess` | session linked and persisted | continue normal operations |
| `EvPairError` | pairing rejected/failed | inspect logs and retry pairing |

## Persistence

Use a stable backend path:

```python
from tryx.backend import SqliteBackend

backend = SqliteBackend("/srv/tryx/session.db")
```

If the same backend path is reused, you usually do not need to pair again.

!!! warning "Single writer rule"
	Avoid multiple runtime instances writing to the same backend path unless you explicitly control ownership.

## Operational Guidance

- Keep one active session owner for a backend path.
- Avoid deleting backend files unless resetting account link is intentional.
- Back up backend data before infrastructure migration.

## Recovery Signals

- `EvLoggedOut`: account session is no longer valid.
- `EvStreamReplaced`: another login/session replaced your current stream.
- `EvTemporaryBan`: temporary restrictions detected; pause high-volume operations.

## Recovery Playbook

1. if `EvLoggedOut`: re-pair and rotate session artifacts.
2. if `EvStreamReplaced`: check if another deployment is using the same account/backend.
3. if temporary-ban signal: stop automation burst traffic and re-enable gradually.

## Related Docs

- [Deployment Guide](../operations/deployment.md)
- [Troubleshooting](../operations/troubleshooting.md)
