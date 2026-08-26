---
icon: material/home
---

<div align="center">
<img src="assets/mascot.png" width="15%" alt="Tryx">
</div>

# Tryx

**Rust-powered Python SDK for event-driven WhatsApp automation.**

Tryx pairs a Rust runtime core with a typed Python API. Protocol handling,
media processing, and Signal encryption run at native speed in Rust — your
application logic stays in clean async Python.

<div class="tryx-pill-row">
  <span class="tryx-pill">Async-first</span>
  <span class="tryx-pill">PyO3 Native</span>
  <span class="tryx-pill">Typed Stubs (.pyi)</span>
  <span class="tryx-pill">Event-Driven</span>
  <span class="tryx-pill">Signal E2E</span>
  <span class="tryx-pill">Pluggable Storage</span>
</div>

---

## What You Can Build

<div class="tryx-grid">
  <div class="tryx-card">
    <h3>Messaging</h3>
    <p>Send text, photo, audio, document, video, GIF, and stickers through a clean async API with full type safety.</p>
  </div>
  <div class="tryx-card">
    <h3>Realtime Events</h3>
    <p>Subscribe to messages, contact updates, sync actions, and lifecycle changes with typed event classes and decorator-based handlers.</p>
  </div>
  <div class="tryx-card">
    <h3>Namespace Clients</h3>
    <p>Dedicated clients for contacts, groups, newsletters, status, privacy, polls, presence, communities, and chat actions.</p>
  </div>
  <div class="tryx-card">
    <h3>Pluggable Storage</h3>
    <p>SQLite (built-in), FFI shared libraries (Postgres, Redis), or pure Python backends — all with the same Signal protocol store API.</p>
  </div>
</div>

---

## Quick Start

```python
from tryx.client import Tryx
from tryx.backend import SqliteStore

app = Tryx(SqliteStore("whatsapp.db"))
client = app.get_client()


@app.on(EvMessage)
async def on_message(client, event):
    text = event.data.get_text()
    chat = event.data.message_info.source.chat
    if text:
        await client.send_text(chat, f"Echo: {text}")


app.run_blocking()
```

<div class="tryx-link-grid" markdown>

[**Installation**](getting-started/installation.md){ .md-link }
Set up your development environment

[**Quick Start**](getting-started/quickstart.md){ .md-link }
Build your first bot in 5 minutes

[**Authentication**](getting-started/authentication.md){ .md-link }
Pair your phone with QR or code

[**Architecture**](core-concepts/architecture.md){ .md-link }
Understand the Rust + Python layers

</div>

---

## Architecture

```
┌──────────────────────────────────────────────────────┐
│                 Python Application                   │
│          (handlers, business logic, bots)            │
├──────────────────────────────────────────────────────┤
│                 Python API Layer                     │
│     TryxClient, namespace clients, @app.on()        │
│         (async methods, typed stubs)                │
├──────────────────────────────────────────────────────┤
│                 PyO3 Bridge                          │
│     GIL management, type conversion, error marshal  │
├──────────────────────────────────────────────────────┤
│                 Rust Core                            │
│   Protocol parsing, Signal crypto, media, transport │
│         (Noise, protobuf, tokio)                    │
└──────────────────────────────────────────────────────┘
```

---

## Client Namespaces

| Namespace | Purpose |
|-----------|---------|
| [`contact`](api/contact.md) | Profile lookup, phone number checks, profile pictures |
| [`chat_actions`](api/chat-actions.md) | Archive, pin, mute, star, edit/revoke, reactions |
| [`groups`](api/groups.md) | Group lifecycle, membership, invite links, admin actions |
| [`community`](api/community.md) | Community creation, subgroup management |
| [`newsletter`](api/newsletter.md) | Channel subscription, posting, follower management |
| [`status`](api/status.md) | Status updates with privacy controls |
| [`chatstate`](api/chatstate.md) | Typing and recording indicators |
| [`blocking`](api/blocking.md) | Block/unblock and blocklist queries |
| [`polls`](api/polls.md) | Encrypted poll creation and vote aggregation |
| [`presence`](api/presence.md) | Online/offline presence and subscriptions |
| [`privacy`](api/privacy.md) | Privacy settings and disallowed lists |
| [`profile`](api/profile.md) | Push name, status text, profile picture |

---

## Reading Paths

=== "New to Tryx"

    1. [Installation](getting-started/installation.md) — set up your environment
    2. [Quick Start](getting-started/quickstart.md) — build your first bot
    3. [Authentication](getting-started/authentication.md) — pair your phone
    4. [Client API](api/client.md) — explore the namespace clients

=== "Building Features"

    1. [Client API](api/client.md) — find the right client for your task
    2. [Events API](api/events.md) — subscribe to WhatsApp events
    3. [Types API](api/types.md) — understand the data model
    4. [Tutorials](tutorials/command-bot.md) — implementation patterns

=== "Going to Production"

    1. [Deployment](operations/deployment.md) — production setup
    2. [Reliability](operations/reliability.md) — error handling and recovery
    3. [Performance](operations/performance.md) — optimization tips
    4. [Security](operations/security.md) — security considerations

---

## Project Structure

```
tryx/
├── src/                        # Rust source (PyO3 bindings)
│   ├── lib.rs                  # Module registration
│   ├── clients/                # Client implementations
│   │   ├── tryx.rs             # Root TryxClient methods
│   │   ├── groups.rs           # Groups namespace
│   │   ├── newsletter.rs       # Newsletter namespace
│   │   └── ...
│   ├── events/                 # Event dispatcher & types
│   │   ├── dispatcher.rs       # Handler routing
│   │   └── types/              # Event class definitions
│   ├── types.rs                # Shared data classes
│   └── backend/                # Storage backend bridge
├── python/tryx/                # Python package
│   ├── client.pyi              # Client type stubs
│   ├── events.pyi              # Event type stubs
│   ├── types.pyi               # Shared type stubs
│   ├── backend.pyi             # Backend type stubs
│   └── waproto/                # Protobuf definitions
├── docs/                       # This documentation
└── examples/                   # Usage examples
```

---

## Example Projects

| Example | Description |
|---------|-------------|
| [Basic Bot](https://github.com/krypton-byte/tryx/tree/dev/examples/basic_bot.py) | Echo bot with ping/help/time commands |
| [Media Bot](https://github.com/krypton-byte/tryx/tree/dev/examples/media_bot.py) | Photo, document, audio, and video sending |
| [Group Bot](https://github.com/krypton-byte/tryx/tree/dev/examples/group_bot.py) | Group admin operations |

---

## Links

<div class="tryx-link-grid" markdown>

[**PyPI Package**](https://pypi.org/project/tryx/){ .md-link }
Install with `pip install tryx`

[**GitHub**](https://github.com/krypton-byte/tryx){ .md-link }
Source code, issues, and releases

[**Type Stubs**](api/client.md){ .md-link }
Full `.pyi` API reference

[**Tutorials**](tutorials/command-bot.md){ .md-link }
Step-by-step bot guides

</div>
