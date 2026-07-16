# Tryx

[![PyPI version](https://img.shields.io/pypi/v/tryx?color=blue)](https://pypi.org/project/tryx/)
[![Python](https://img.shields.io/pypi/pyversions/tryx.svg)](https://pypi.org/project/tryx/)
[![License](https://img.shields.io/github/license/krypton-byte/tryx)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-online-brightgreen)](http://krypton-byte.tech/tryx/)

**Language:** [English](#english) | [简体中文](#简体中文) | [Bahasa Indonesia](#bahasa-indonesia)

Tryx is a Python automation library powered by Rust and PyO3. It provides a Python-friendly API on top of the `whatsapp-rust` crate, with async support, persistent sessions, and access to advanced WhatsApp Web protocol features.

> Tryx is an independent project. It is not affiliated with WhatsApp, Meta, or their official products.

---

## English

### What Is Tryx?

Tryx helps Python developers build WhatsApp automation tools without writing Rust directly. The core protocol logic runs in Rust for performance, while the public interface stays ergonomic for Python applications.

Use Tryx when you need:

- WhatsApp Web automation from Python.
- Persistent login sessions with SQLite, PostgreSQL, or MySQL.
- Async APIs for bots, workers, dashboards, and internal tools.
- Access to contacts, groups, newsletters, status, privacy, profile, labels, comments, events, and lower-level protocol helpers.

### Highlights

- **Rust core, Python API** - protocol logic is implemented in Rust and exposed through PyO3.
- **Async first** - designed for `asyncio` applications.
- **Persistent sessions** - reuse device sessions through a database-backed store.
- **Broad API surface** - exposes common and advanced client namespaces.
- **Typed Python stubs** - includes `.pyi` files for better editor support.

### Installation

```bash
pip install tryx
```

For local development:

```bash
git clone https://github.com/krypton-byte/tryx.git
cd tryx
git submodule update --init --recursive
uv sync --group dev
uv run maturin develop
```

### Quick Start

```python
import asyncio

from tryx.backend import SqliteStore
from tryx.client import Tryx, TryxClient
from tryx.events import EvMessage, EvPairingQrCode

backend = SqliteStore("whatsapp.db")
app = Tryx(backend)


@app.on(EvPairingQrCode)
async def on_pairing_qr(_client: TryxClient, event: EvPairingQrCode) -> None:
    print("Scan this QR code with WhatsApp:")
    print(event.code)


@app.on(EvMessage)
async def on_message(client: TryxClient, event: EvMessage) -> None:
    text = (event.data.get_text() or "").strip()
    if text.lower() == "ping":
        await client.send_text(event.data.message_info.source.chat, "pong", quoted=event)


async def main() -> None:
    await app.run()


asyncio.run(main())
```

### Runtime Client Namespaces

The runtime `TryxClient` object exposes several namespaces so the API stays organized:

| Namespace | Purpose |
| --- | --- |
| `contact` | Contact lookup and contact-related helpers. |
| `chat_actions` | Chat utilities such as mute, archive, pin, mark read, clear chat, and save contact. |
| `groups` | Group creation, metadata, members, invites, membership requests, and admin settings. |
| `community` | Community-related operations. |
| `newsletter` | Newsletter info, messages, mute settings, edits, and revoke operations. |
| `status` | Status posting, lookup, and related helpers. |
| `chatstate` | Typing, recording, paused, and presence-style chat state updates. |
| `blocking` | Block and unblock contacts. |
| `polls` | Poll creation and voting helpers. |
| `presence` | Presence subscriptions and availability updates. |
| `privacy` | Privacy settings and related controls. |
| `profile` | Profile name, status text, and picture operations. |
| `labels` | Label management for chats and messages. |
| `comments` | Comment-related protocol helpers. |
| `events` | Event response helpers and event-oriented operations. |
| `advanced` | Lifecycle waits, diagnostics, and lower-level protocol escape hatches. |

### Storage Backends

Tryx supports three storage tiers:

| Tier | Backend | Use case |
| --- | --- | --- |
| Built-in | `SqliteStore("whatsapp.db")` | Local development and simple deployments. |
| Native FFI | `FfiStoreProtocol` | High-throughput external stores such as PostgreSQL. |
| Pure Python | `StoreBase` subclass | Custom async stores such as Redis, MongoDB, or DynamoDB. |

SQLite is usually enough for local development. Native FFI or a custom Python backend is better when several workers or deployment environments need shared session state.

### Development

Useful commands:

```bash
cargo check
cargo test --lib
env UV_CACHE_DIR=/tmp/uv-cache uv run pytest -q
uv run maturin develop
uv run maturin build --release
```

Project layout:

```text
.
├── libs/whatsapp-rust/   # Rust protocol crate submodule
├── src/                  # Rust and PyO3 bindings
├── python/tryx/          # Python package and type stubs
├── examples/             # Example automation scripts
├── docs/                 # Documentation site
└── tests/                # Python test suite
```

### Links

- Documentation: <http://krypton-byte.tech/tryx/>
- Examples: [`examples/`](examples/)
- Python type stubs: [`python/tryx/`](python/tryx/)
- Rust bindings: [`src/`](src/)

---

## 简体中文

### Tryx 是什么？

Tryx 是一个基于 Rust 和 PyO3 的 Python WhatsApp 自动化库。底层协议能力来自 `whatsapp-rust` crate，Python 侧提供更易用的异步 API，适合构建机器人、后台任务、内部工具和自动化服务。

Tryx 适合以下场景：

- 使用 Python 操作 WhatsApp Web。
- 通过 SQLite、PostgreSQL 或 MySQL 持久化登录会话。
- 在 `asyncio` 项目中构建机器人或自动化流程。
- 使用联系人、群组、频道、状态、隐私、资料、标签、评论、事件和高级协议能力。

### 主要特点

- **Rust 核心，Python 调用** - 协议逻辑在 Rust 中运行，Python 通过 PyO3 调用。
- **异步优先** - 适合 `asyncio` 应用。
- **会话持久化** - 可以复用已登录的 WhatsApp 设备会话。
- **API 覆盖面更广** - 暴露常用功能和高级功能命名空间。
- **类型提示支持** - 提供 `.pyi` 文件，提升编辑器补全体验。

### 安装

```bash
pip install tryx
```

本地开发：

```bash
git clone https://github.com/krypton-byte/tryx.git
cd tryx
git submodule update --init --recursive
uv sync --group dev
uv run maturin develop
```

### 快速开始

```python
import asyncio

from tryx.backend import SqliteStore
from tryx.client import Tryx, TryxClient
from tryx.events import EvMessage, EvPairingQrCode

backend = SqliteStore("whatsapp.db")
app = Tryx(backend)


@app.on(EvPairingQrCode)
async def on_pairing_qr(_client: TryxClient, event: EvPairingQrCode) -> None:
    print("Scan this QR code with WhatsApp:")
    print(event.code)


@app.on(EvMessage)
async def on_message(client: TryxClient, event: EvMessage) -> None:
    text = (event.data.get_text() or "").strip()
    if text.lower() == "ping":
        await client.send_text(event.data.message_info.source.chat, "pong", quoted=event)


async def main() -> None:
    await app.run()


asyncio.run(main())
```

### Client 命名空间

运行时 `TryxClient` 对象按功能拆分为多个命名空间：

| 命名空间 | 用途 |
| --- | --- |
| `contact` | 联系人查询和联系人相关工具。 |
| `chat_actions` | 聊天操作，例如静音、归档、置顶、标记已读、清空聊天和保存联系人。 |
| `groups` | 群组创建、信息、成员、邀请、入群请求和管理员设置。 |
| `community` | 社区相关操作。 |
| `newsletter` | 频道信息、消息、静音设置、编辑和撤回。 |
| `status` | 状态发布、查询和相关工具。 |
| `chatstate` | 输入中、录音中、暂停等聊天状态。 |
| `blocking` | 拉黑和取消拉黑联系人。 |
| `polls` | 投票创建和投票操作。 |
| `presence` | 在线状态订阅和可用性更新。 |
| `privacy` | 隐私设置和控制。 |
| `profile` | 资料名称、状态文本和头像操作。 |
| `labels` | 聊天和消息标签管理。 |
| `comments` | 评论相关协议工具。 |
| `events` | 事件响应和事件相关操作。 |
| `advanced` | 生命周期等待、诊断和更底层的协议能力。 |

### 存储后端

Tryx 支持三种会话存储方式：

| 类型 | 后端 | 使用场景 |
| --- | --- | --- |
| 内置 | `SqliteStore("whatsapp.db")` | 本地开发和简单部署。 |
| Native FFI | `FfiStoreProtocol` | 高吞吐外部存储，例如 PostgreSQL。 |
| 纯 Python | `StoreBase` 子类 | 自定义异步存储，例如 Redis、MongoDB 或 DynamoDB。 |

本地开发通常使用 SQLite 即可。如果多个 worker 或部署环境需要共享会话状态，建议使用 Native FFI 或自定义 Python 后端。

### 开发命令

```bash
cargo check
cargo test --lib
env UV_CACHE_DIR=/tmp/uv-cache uv run pytest -q
uv run maturin develop
uv run maturin build --release
```

### 相关链接

- 文档：<http://krypton-byte.tech/tryx/>
- 示例：[`examples/`](examples/)
- Python 类型文件：[`python/tryx/`](python/tryx/)
- Rust 绑定代码：[`src/`](src/)

---

## Bahasa Indonesia

### Apa Itu Tryx?

Tryx adalah library otomasi WhatsApp untuk Python yang dibangun dengan Rust dan PyO3. Logika protokol berjalan di Rust melalui `whatsapp-rust`, sedangkan API publiknya dibuat agar nyaman digunakan dari Python.

Tryx cocok untuk:

- Membuat otomasi WhatsApp Web dari Python.
- Menyimpan sesi login dengan SQLite, PostgreSQL, atau MySQL.
- Membangun bot, worker, dashboard, dan tool internal berbasis `asyncio`.
- Mengakses fitur kontak, grup, newsletter, status, privasi, profil, label, komentar, event, dan helper protokol tingkat lanjut.

### Fitur Utama

- **Core Rust, API Python** - performa dan logika protokol ditangani Rust, pemakaian tetap sederhana dari Python.
- **Async first** - dirancang untuk aplikasi `asyncio`.
- **Sesi persisten** - sesi WhatsApp dapat dipakai ulang melalui database.
- **API luas** - namespace umum dan advanced sudah diexpose.
- **Type stub Python** - tersedia `.pyi` untuk autocomplete dan type checking yang lebih baik.

### Instalasi

```bash
pip install tryx
```

Untuk development lokal:

```bash
git clone https://github.com/krypton-byte/tryx.git
cd tryx
git submodule update --init --recursive
uv sync --group dev
uv run maturin develop
```

### Mulai Cepat

```python
import asyncio

from tryx.backend import SqliteStore
from tryx.client import Tryx, TryxClient
from tryx.events import EvMessage, EvPairingQrCode

backend = SqliteStore("whatsapp.db")
app = Tryx(backend)


@app.on(EvPairingQrCode)
async def on_pairing_qr(_client: TryxClient, event: EvPairingQrCode) -> None:
    print("Scan this QR code with WhatsApp:")
    print(event.code)


@app.on(EvMessage)
async def on_message(client: TryxClient, event: EvMessage) -> None:
    text = (event.data.get_text() or "").strip()
    if text.lower() == "ping":
        await client.send_text(event.data.message_info.source.chat, "pong", quoted=event)


async def main() -> None:
    await app.run()


asyncio.run(main())
```

### Namespace Client

Objek runtime `TryxClient` dibagi menjadi beberapa namespace agar API lebih rapi:

| Namespace | Fungsi |
| --- | --- |
| `contact` | Helper untuk kontak dan pencarian kontak. |
| `chat_actions` | Operasi chat seperti mute, archive, pin, mark read, clear chat, dan save contact. |
| `groups` | Pembuatan grup, metadata, member, invite, request membership, dan pengaturan admin. |
| `community` | Operasi terkait komunitas. |
| `newsletter` | Info newsletter, pesan, mute, edit, dan revoke. |
| `status` | Posting status, lookup status, dan helper terkait status. |
| `chatstate` | State chat seperti typing, recording, dan paused. |
| `blocking` | Block dan unblock kontak. |
| `polls` | Membuat poll dan mengirim vote. |
| `presence` | Subscribe presence dan update availability. |
| `privacy` | Pengaturan privasi. |
| `profile` | Nama profil, status teks, dan foto profil. |
| `labels` | Manajemen label untuk chat dan pesan. |
| `comments` | Helper protokol terkait komentar. |
| `events` | Helper response dan operasi berbasis event. |
| `advanced` | Wait lifecycle, diagnostik, dan akses protokol tingkat lanjut. |

### Storage Backend

Tryx mendukung tiga tipe storage untuk menyimpan sesi WhatsApp:

| Tipe | Backend | Kegunaan |
| --- | --- | --- |
| Built-in | `SqliteStore("whatsapp.db")` | Development lokal dan deployment sederhana. |
| Native FFI | `FfiStoreProtocol` | Store eksternal throughput tinggi seperti PostgreSQL. |
| Pure Python | subclass `StoreBase` | Store async custom seperti Redis, MongoDB, atau DynamoDB. |

SQLite cukup untuk development lokal. Native FFI atau backend Python custom lebih cocok jika sesi perlu dipakai bersama oleh beberapa worker atau environment deployment.

### Development

Command yang umum dipakai:

```bash
cargo check
cargo test --lib
env UV_CACHE_DIR=/tmp/uv-cache uv run pytest -q
uv run maturin develop
uv run maturin build --release
```

Struktur project:

```text
.
├── libs/whatsapp-rust/   # Submodule crate protokol Rust
├── src/                  # Binding Rust dan PyO3
├── python/tryx/          # Package Python dan type stub
├── examples/             # Contoh script otomasi
├── docs/                 # Situs dokumentasi
└── tests/                # Test suite Python
```

### Link Penting

- Dokumentasi: <http://krypton-byte.tech/tryx/>
- Contoh: [`examples/`](examples/)
- Type stub Python: [`python/tryx/`](python/tryx/)
- Binding Rust: [`src/`](src/)

---

## License

This project is licensed under the terms of the [MIT License](LICENSE).
