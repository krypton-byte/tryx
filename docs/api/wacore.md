# WACore API

`tryx.wacore` exposes lower-level protocol-facing types.

## MediaType

Enum for media upload/download classification.

## Node Tree Types

- `NodeValue`
- `NodeContent`
- `Attrs`
- `Node`

These are useful for advanced debugging or when you need access to protocol node shape in events such as stream errors or notifications.

## Business and Key Metadata

- `KeyIndexInfo`
- `BusinessSubscription`

These appear in device/business sync event payloads.

## When to Use WACore Types

Use these types only when high-level client/event abstractions are not enough.
For normal bot logic, prefer typed event payload objects and client namespace methods.
