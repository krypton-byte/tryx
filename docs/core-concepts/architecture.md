# Architecture

## Layered Design

Tryx uses a two-layer model:

1. Rust core layer
2. Python API layer

### Rust Core Layer

The Rust side handles:

- protocol parsing
- transport/runtime state
- heavy event transformations
- media and protobuf conversions

### Python API Layer

The Python side provides:

- ergonomic async API
- namespace-based clients (`contact`, `groups`, `privacy`, etc.)
- typed stubs for IDE and static analysis
- callback registration via decorators

## Why This Design Works

- Performance-sensitive logic stays in Rust.
- Product logic stays simple in Python.
- Event payloads are structured classes, not ad-hoc dicts.

## Data Flow

```mermaid
flowchart LR
    A[WhatsApp Stream] --> B[Rust Runtime]
    B --> C[Event Conversion]
    C --> D[PyO3 Classes]
    D --> E[Python Handler]
    E --> F[TryxClient API Calls]
    F --> B
```

## Module Map

- `src/lib.rs`: submodule registration and class exports
- `src/clients/*`: client methods exposed to Python
- `src/events/*`: event classes and dispatcher
- `src/types.rs`: shared data classes (`JID`, `MessageInfo`, etc.)
- `src/wacore/*`: low-level node and stanza models
- `python/tryx/*.py`: runtime re-export wrappers
- `python/tryx/*.pyi`: typed API contracts

## Practical Implication

You can safely treat Python classes as stable contracts while trusting Rust internals for throughput and protocol-heavy work.
