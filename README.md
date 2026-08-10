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

### VoIP: Audio and Video Calls

Tryx provides a Rust-backed bridge for WhatsApp audio and video calls. After the client is connected, use client.voip. Protocol handling, RTP/WebRTC, encryption, codecs, and call orchestration run in whatsapp-rust; Python supplies media through asynchronous source and sink adapters.

#### Features

- One-to-one audio calls through voip.call().
- One-to-one video calls through voip.video_call().
- Group calls with optional video through voip.group_call().
- Call-link joining through voip.join_call_link() with audio or video media.
- Hangup, wait, mute/unmute, video, participant, approval, and screen-sharing controls.
- Native Rust playback for WAV, MP3, OGG, Vorbis, and PCM audio files.
- Native FFmpeg playback producing H.264 Annex-B video access units.
- Python adapters for microphones, speakers, cameras, codecs, TTS, DSP, and custom pipelines.

#### Media Contract

| Media | Kontrak |
| --- | --- |
| Audio | Mono signed PCM16 little-endian, 16,000 Hz, 960 samples / 1,920 bytes per frame, 60 ms |
| Video | H.264 Annex-B access unit represented by VideoFrame |

AudioSource.frames() must yield exactly 1,920 bytes per frame. AudioSink.write() receives the same format. VideoFrame carries data, timestamp_us, duration_us, keyframe, optional dimensions, and orientation.

#### Minimal Audio Adapters

```python
from collections.abc import AsyncIterator
from tryx.media import AudioSink, AudioSource, validate_audio_frame
class Microphone(AudioSource):
    async def frames(self) -> AsyncIterator[bytes]:
        while True:
            frame = await read_microphone_frame()
            yield validate_audio_frame(frame)
class Speaker(AudioSink):
    async def write(self, frame: bytes) -> None:
        validate_audio_frame(frame)
        await play_speaker_frame(frame)
```

Replace the two backend functions with PyAudio, sounddevice, ALSA, CoreAudio, or another device library.

#### One-to-One Audio Call

```python
from tryx.types import JID
async def start_audio_call(client, phone_number: str):
    peer = JID(phone_number + "@s.whatsapp.net")
    call = await client.voip.call(peer, Microphone(), Speaker())
    print("call started:", call.call_id)
    call.set_muted(True)
    call.set_muted(False)
    await call.wait_ended()
```
Use await call.hangup() to end a call explicitly. Keep the CallHandle alive and await wait_ended() for deterministic cleanup.

#### Video Call
```python
from tryx.media import VideoPlayer
async def start_video_call(client, peer, video_sink):
    video_source = VideoPlayer(fps=15)
    video_source.play("sample.mp4")
    call = await client.voip.video_call(
        peer, Microphone(), Speaker(), video_source, video_sink
    )
    await call.wait_ended()
    video_source.stop()
```
VideoPlayer accepts 1–60 FPS and defaults to 15. FFmpeg must be in PATH; missing files and missing FFmpeg produce explicit errors.

#### Group Calls and Call Links
```python
call = await client.voip.group_call(
    peers=[peer_a, peer_b],
    audio_source=Microphone(),
    audio_sink=Speaker(),
)
await call.invite_participant(peer_c)
await call.ring_participant(peer_c)
await call.set_approval_required(True)
linked = await client.voip.join_call_link(
    "https://call.whatsapp.com/your-token",
    "audio",
    Microphone(),
    Speaker(),
)
```
group_call() accepts optional video source/sink pairs. join_call_link() accepts a token or URL and media audio or video.

#### Incoming Calls

IncomingCallEvent exposes call_id, peer, is_video, accept(audio_source, audio_sink), and reject(). Accept or reject it only once because the invitation is consumed.
```python
@app.on(EvIncomingCall)
async def on_incoming_call(_client, event):
    if event.is_video:
        await event.reject()
        return
    call = await event.accept(Microphone(), Speaker())
    await call.wait_ended()
```
Use the incoming-call event exported by the installed package version.

#### Native AudioPlayer

AudioPlayer decodes files in Rust and normalizes them to mono PCM16 at 16 kHz.
```python
from tryx.media import AudioPlayer
player = AudioPlayer(buffer_frames=3)
player.play("intro.mp3", mode="replace")
call = await client.voip.call(peer, player, Speaker())
player.pause()
player.resume()
player.enqueue("next.wav")
player.skip()
player.clear_queue()
player.stop()
```
Modes are replace, queue, and interrupt. buffer_frames defaults to 3 and is capped at 30. States are idle, playing, and paused; non-16 kHz audio uses linear interpolation.

#### Internal Data Flow and Backpressure

    Python source/sink -> PyO3 bridge + bounded async channel
                        -> whatsapp-rust VoIP facade
                        -> WaCore call engine -> RTP/SRTP/WebRTC
                        -> WhatsApp call network

Inbound media follows the reverse path: transport decrypts and decodes, the bridge creates PCM16 or VideoFrame, and the Python sink receives it. Bounded channels prevent unbounded queues; audio defaults to three frames, about 180 ms before network and codec overhead. Slow sinks cause asynchronous backpressure instead of CPU spin-waiting.

Audio commands wait for the manager result and propagate errors to Python. VideoPlayer.stop() cancels its FFmpeg task, which kills and reaps the child process instead of leaving an orphan.

#### Lifecycle and Troubleshooting

- Start calls only after Tryx is connected; use one native player per active call.
- Stop players when cancelling a call and release them after wait_ended().
- Send only mono PCM16 16 kHz frames of exactly 1,920 bytes.
- Video sources must emit H.264 Annex-B, not an MP4 container.
- For invalid audio sizes, chunk or resample into 960 samples.
- For high latency, use buffer_frames=2 or 3.
- Install FFmpeg and verify it with ffmpeg -version when video playback fails.

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

### VoIP：音频和视频通话

Tryx 提供基于 Rust 的 WhatsApp 音频和视频通话桥接层。客户端连接后，可以通过 client.voip 使用 API。协议处理、RTP/WebRTC、加密、编解码器和通话编排由 whatsapp-rust 执行；Python 通过异步 source 和 sink adapter 提供媒体数据。

#### 功能

- 使用 voip.call() 发起一对一音频通话。
- 使用 voip.video_call() 发起一对一视频通话。
- 使用 voip.group_call() 发起带可选视频的群组通话。
- 使用 voip.join_call_link() 加入音频或视频 call link。
- 支持挂断、等待结束、静音/取消静音、开始/停止视频。
- 支持邀请、响铃、参与者审批，以及允许或拒绝等待中的用户。
- 支持 start_screen_share() 和 stop_screen_share() 屏幕共享。
- Rust 原生播放 WAV、MP3、OGG、Vorbis 和 PCM 音频文件。
- 通过 FFmpeg 原生播放视频并输出 H.264 Annex-B access unit。
- 支持用于麦克风、扬声器、摄像头、编解码器、TTS、DSP 和自定义媒体管线的 Python adapter。

#### 媒体契约

| 媒体 | 契约 |
| --- | --- |
| 音频 | 单声道 signed PCM16 little-endian，16,000 Hz，每帧 960 个 sample / 1,920 字节，60 ms |
| 视频 | 通过 VideoFrame 表示的 H.264 Annex-B access unit |

AudioSource.frames() 必须每次产生恰好 1,920 字节。AudioSink.write() 接收相同的 PCM 格式。VideoFrame 包含 data、timestamp_us、duration_us、keyframe、可选的 width 和 height，以及 orientation。

#### 最小音频 Adapter
```python
from collections.abc import AsyncIterator
from tryx.media import AudioSink, AudioSource, validate_audio_frame
class Microphone(AudioSource):
    async def frames(self) -> AsyncIterator[bytes]:
        while True:
            frame = await read_microphone_frame()
            yield validate_audio_frame(frame)
class Speaker(AudioSink):
    async def write(self, frame: bytes) -> None:
        validate_audio_frame(frame)
        await play_speaker_frame(frame)
```
请将两个设备函数替换为 PyAudio、sounddevice、ALSA、CoreAudio 或其他音频设备库的实现。

#### 一对一音频通话

```python
from tryx.types import JID
async def start_audio_call(client, phone_number: str):
    peer = JID(phone_number + "@s.whatsapp.net")
    call = await client.voip.call(peer, Microphone(), Speaker())
    print("call started:", call.call_id)
    call.set_muted(True)
    call.set_muted(False)
    await call.wait_ended()
```
使用 await call.hangup() 主动结束通话。通话期间请保持 CallHandle，并等待 wait_ended()，以便媒体资源能够稳定清理。

#### 视频通话
```python
from tryx.media import VideoPlayer
async def start_video_call(client, peer, video_sink):
    video_source = VideoPlayer(fps=15)
    video_source.play("sample.mp4")
    call = await client.voip.video_call(
        peer, Microphone(), Speaker(), video_source, video_sink
    )
    await call.wait_ended()
    video_source.stop()
```
VideoPlayer 支持 1–60 FPS，默认值为 15。系统必须在 PATH 中提供 FFmpeg；文件不存在或 FFmpeg 不可用时，play() 会返回明确错误。

#### 群组通话和 Call Link
```python
call = await client.voip.group_call(
    peers=[peer_a, peer_b],
    audio_source=Microphone(),
    audio_sink=Speaker(),
)
await call.invite_participant(peer_c)
await call.ring_participant(peer_c)
await call.set_approval_required(True)
linked = await client.voip.join_call_link(
    "https://call.whatsapp.com/your-token",
    "audio",
    Microphone(),
    Speaker(),
)
```
group_call() 可以接收可选的 video source/sink。join_call_link() 可以接收 token 或 URL，并使用 media="audio" 或 media="video"。

#### 来电

IncomingCallEvent 提供 call_id、peer、is_video、accept(audio_source, audio_sink) 和 reject()。一个 event 只能 accept 或 reject 一次，因为底层来电邀请在操作后会被消费。
```python
@app.on(EvIncomingCall)
async def on_incoming_call(_client, event):
    if event.is_video:
        await event.reject()
        return
    call = await event.accept(Microphone(), Speaker())
    await call.wait_ended()
```
请使用当前安装版本导出的来电 event 名称。

#### Rust 原生 AudioPlayer

AudioPlayer 在 Rust 中解码文件，并将其标准化为 16 kHz 单声道 PCM16。
```python
from tryx.media import AudioPlayer
player = AudioPlayer(buffer_frames=3)
player.play("intro.mp3", mode="replace")
call = await client.voip.call(peer, player, Speaker())
player.pause()
player.resume()
player.enqueue("next.wav")
player.skip()
player.clear_queue()
player.stop()
```
播放模式为 replace、queue 和 interrupt。buffer_frames 默认 3，最大限制为 30。播放器状态为 idle、playing 和 paused。非 16 kHz 音频使用线性插值转换。

#### 内部数据流和背压

    Python source/sink -> PyO3 bridge + bounded async channel
                        -> whatsapp-rust VoIP facade
                        -> WaCore call engine -> RTP/SRTP/WebRTC
                        -> WhatsApp call network

接收媒体沿反方向流动：transport 解密并解码，bridge 将数据转换为 PCM16 或 VideoFrame，再交给 Python sink。bounded channel 防止队列无限增长；音频默认缓存 3 帧，即在网络和 codec 开销之前约 180 ms。sink 较慢时 producer 会异步等待，而不是消耗 CPU 自旋等待。

AudioPlayer command 会等待 manager 的结果，并将错误传递给 Python。VideoPlayer.stop() 会取消 FFmpeg task，由该 task kill 并回收 child process，避免留下孤儿进程。

#### 生命周期和故障排查

- 只有在 Tryx 连接成功后才开始通话；每个活动通话使用一个 native player。
- 取消通话时停止 player，并在 wait_ended() 后释放资源。
- 只能发送恰好 1,920 字节的单声道 PCM16 16 kHz 音频帧。
- 视频 source 必须输出 H.264 Annex-B，不能直接输出 MP4 container。
- 音频尺寸错误时，将音频切分或重采样为 960 samples。
- 延迟过高时使用 buffer_frames=2 或 3。
- 视频播放失败时安装 FFmpeg，并使用 ffmpeg -version 验证 PATH。

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

### VoIP: Panggilan Audio dan Video

Tryx menyediakan bridge VoIP berbasis Rust untuk panggilan audio dan video WhatsApp. Setelah client tersambung, API tersedia melalui client.voip. Protocol, RTP/WebRTC, enkripsi, codec, dan orkestrasi call berjalan di whatsapp-rust; Python menyediakan media melalui adapter source dan sink asynchronous.

#### Fitur

- Panggilan audio 1:1 melalui voip.call().
- Panggilan video 1:1 melalui voip.video_call().
- Group call dengan video opsional melalui voip.group_call().
- Bergabung ke call link melalui voip.join_call_link() dengan media audio atau video.
- Kontrol hangup, wait, mute/unmute, video, peserta, approval, dan screen sharing.
- Playback file audio native Rust untuk WAV, MP3, OGG, Vorbis, dan PCM.
- Playback video native melalui FFmpeg dengan output H.264 Annex-B access unit.
- Adapter Python untuk microphone, speaker, camera, codec, TTS, DSP, dan pipeline media custom.

#### Kontrak Media

| Media | Kontrak |
| --- | --- |
| Audio | Mono signed PCM16 little-endian, 16.000 Hz, 960 sample / 1.920 byte per frame, 60 ms |
| Video | H.264 Annex-B access unit melalui VideoFrame |

AudioSource.frames() harus menghasilkan async iterator dengan tepat 1.920 byte untuk setiap frame. AudioSink.write() menerima format PCM yang sama. VideoFrame berisi data, timestamp_us, duration_us, keyframe, dimensi opsional, dan orientation.

#### Adapter Audio Minimal
```python
    from collections.abc import AsyncIterator
    from tryx.media import AudioSink, AudioSource, validate_audio_frame

    class Microphone(AudioSource):
        async def frames(self) -> AsyncIterator[bytes]:
            while True:
                frame = await read_microphone_frame()
                yield validate_audio_frame(frame)

    class Speaker(AudioSink):
        async def write(self, frame: bytes) -> None:
            validate_audio_frame(frame)
            await play_speaker_frame(frame)
```
Ganti fungsi perangkat tersebut dengan implementasi PyAudio, sounddevice, ALSA, CoreAudio, atau library perangkat lain.

#### Panggilan Audio 1:1
```python
    from tryx.types import JID

    async def start_audio_call(client, phone_number: str):
        peer = JID(phone_number + "@s.whatsapp.net")
        call = await client.voip.call(peer, Microphone(), Speaker())
        print("call started:", call.call_id)
        call.set_muted(True)
        call.set_muted(False)
        await call.wait_ended()
```
Gunakan await call.hangup() untuk mengakhiri call secara eksplisit. Pertahankan CallHandle selama call aktif dan tunggu wait_ended() agar cleanup media berjalan deterministik.

#### Panggilan Video
```python
    from tryx.media import VideoPlayer

    async def start_video_call(client, peer, video_sink):
        video_source = VideoPlayer(fps=15)
        video_source.play("sample.mp4")
        call = await client.voip.video_call(
            peer, Microphone(), Speaker(), video_source, video_sink
        )
        await call.wait_ended()
        video_source.stop()
```
VideoPlayer mendukung 1–60 FPS dengan default 15. FFmpeg harus tersedia di PATH. File yang tidak ditemukan atau FFmpeg yang tidak tersedia akan menghasilkan error yang jelas.

#### Group Call dan Call Link
```python
    call = await client.voip.group_call(
        peers=[peer_a, peer_b],
        audio_source=Microphone(),
        audio_sink=Speaker(),
    )
    await call.invite_participant(peer_c)
    await call.ring_participant(peer_c)
    await call.set_approval_required(True)

    linked = await client.voip.join_call_link(
        "https://call.whatsapp.com/your-token",
        "audio",
        Microphone(),
        Speaker(),
    )
```
group_call() menerima video source dan sink secara opsional. join_call_link() menerima token atau URL dengan media audio atau video.

#### Panggilan Masuk

IncomingCallEvent menyediakan call_id, peer, is_video, accept(audio_source, audio_sink), dan reject(). Event hanya boleh di-accept atau reject satu kali karena invitation akan dikonsumsi.
```python
    @app.on(EvIncomingCall)
    async def on_incoming_call(_client, event):
        if event.is_video:
            await event.reject()
            return
        call = await event.accept(Microphone(), Speaker())
        await call.wait_ended()
```
Gunakan nama event incoming-call yang diexport oleh versi package yang terpasang.

#### AudioPlayer Native

AudioPlayer mendecode file di Rust dan menormalisasikannya menjadi mono PCM16 16 kHz.
```python
    from tryx.media import AudioPlayer

    player = AudioPlayer(buffer_frames=3)
    player.play("intro.mp3", mode="replace")
    call = await client.voip.call(peer, player, Speaker())
    player.pause()
    player.resume()
    player.enqueue("next.wav")
    player.skip()
    player.clear_queue()
    player.stop()
```
Mode playback adalah replace, queue, dan interrupt. buffer_frames default 3 dan dibatasi maksimal 30 frame. State player adalah idle, playing, atau paused. Audio non-16 kHz diproses dengan interpolasi linear.

#### Alur Internal dan Backpressure

    Python source/sink -> PyO3 bridge + bounded async channel
                        -> whatsapp-rust VoIP facade
                        -> WaCore call engine -> RTP/SRTP/WebRTC
                        -> WhatsApp call network

Media masuk berjalan terbalik: transport mendekripsi dan mendecode, bridge mengubahnya menjadi PCM16 atau VideoFrame, kemudian sink Python menerimanya. Bounded channel mencegah queue tumbuh tanpa batas. Buffer audio default 3 frame, sekitar 180 ms sebelum overhead jaringan dan codec. Jika sink lambat, producer menunggu secara asynchronous tanpa spin-wait CPU.

Command AudioPlayer menunggu hasil manager dan meneruskan error ke Python. VideoPlayer.stop() membatalkan task FFmpeg; task tersebut membunuh dan mereap child process agar tidak meninggalkan proses yatim.

#### Lifecycle dan Troubleshooting

- Mulai call setelah Tryx tersambung dan gunakan satu native player untuk setiap call aktif.
- Hentikan player saat call dibatalkan dan lepaskan setelah wait_ended().
- Kirim hanya frame mono PCM16 16 kHz dengan ukuran tepat 1.920 byte.
- Video source harus menghasilkan H.264 Annex-B, bukan MP4 container.
- Jika ukuran audio salah, lakukan chunk atau resample menjadi 960 sample.
- Jika latency tinggi, gunakan buffer_frames=2 atau 3.
- Jika video gagal, instal FFmpeg dan verifikasi dengan ffmpeg -version.

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
