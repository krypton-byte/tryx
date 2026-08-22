# Tryx

[![PyPI version](https://img.shields.io/pypi/v/tryx?color=blue)](https://pypi.org/project/tryx/)
[![Python](https://img.shields.io/pypi/pyversions/tryx.svg)](https://pypi.org/project/tryx/)
[![License](https://img.shields.io/github/license/krypton-byte/tryx)](LICENSE)
[![Docs](https://img.shields.io/badge/docs-online-brightgreen)](https://krypton-byte.github.io/tryx/)

**语言：** [English](README.md) | 简体中文 | [Bahasa Indonesia](README.id.md)

Tryx 是一个基于 Rust 和 PyO3 的 Python WhatsApp 自动化库。底层协议能力来自 `whatsapp-rust` crate，Python 侧提供更易用的异步 API，适合构建机器人、后台任务、内部工具和自动化服务。

> Tryx 是一个独立项目，与 WhatsApp、Meta 或其官方产品无关。

## Tryx 是什么？

Tryx 帮助 Python 开发者构建 WhatsApp 自动化工具，无需直接编写 Rust 代码。核心协议逻辑在 Rust 中运行以保证性能，而公共接口则保持对 Python 应用的友好性。

Tryx 适合以下场景：

- 使用 Python 操作 WhatsApp Web。
- 通过 SQLite、PostgreSQL 或 MySQL 持久化登录会话。
- 在 `asyncio` 项目中构建机器人或自动化流程。
- 使用联系人、群组、频道、状态、隐私、资料、标签、评论、事件和高级协议能力。

## 主要特点

- **Rust 核心，Python 调用** — 协议逻辑在 Rust 中运行，Python 通过 PyO3 调用。
- **异步优先** — 适合 `asyncio` 应用。
- **会话持久化** — 可以复用已登录的 WhatsApp 设备会话。
- **API 覆盖面更广** — 暴露常用功能和高级功能命名空间。
- **类型提示支持** — 提供 `.pyi` 文件，提升编辑器补全体验。

## 安装

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

## 快速开始

```python
import asyncio

from tryx.backend import SqliteStore
from tryx.client import Tryx, TryxClient
from tryx.events import EvMessage, EvPairingQrCode

backend = SqliteStore("whatsapp.db")
app = Tryx(backend)


@app.on(EvPairingQrCode)
async def on_pairing_qr(_client: TryxClient, event: EvPairingQrCode) -> None:
    print("请用 WhatsApp 扫描此二维码：")
    print(event.code)


@app.on(EvMessage)
async def on_message(client: TryxClient, event: EvMessage) -> None:
    text = (event.data.get_text() or "").strip()
    if text.lower() == "ping":
        await client.send_text(
            event.data.message_info.source.chat, "pong", quoted=event
        )


async def main() -> None:
    await app.run()


asyncio.run(main())
```

## Client 命名空间

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
| `voip` | 音频和视频通话管理。 |

## 存储后端

Tryx 支持三种会话存储方式：

| 类型 | 后端 | 使用场景 |
| --- | --- | --- |
| 内置 | `SqliteStore("whatsapp.db")` | 本地开发和简单部署。 |
| Native FFI | `FfiStoreProtocol` | 高吞吐外部存储，例如 PostgreSQL。 |
| 纯 Python | `StoreBase` 子类 | 自定义异步存储，例如 Redis、MongoDB 或 DynamoDB。 |

本地开发通常使用 SQLite 即可。如果多个 worker 或部署环境需要共享会话状态，建议使用 Native FFI 或自定义 Python 后端。

## VoIP：音频和视频通话

Tryx 提供基于 Rust 的 WhatsApp 音频和视频通话桥接层。协议处理、RTP/WebRTC、加密、编解码器和通话编排由 whatsapp-rust 执行；Python 通过异步 source 和 sink adapter 提供媒体数据。

### 媒体契约

| 媒体 | 契约 |
| --- | --- |
| 音频 | 单声道 signed PCM16 little-endian，16,000 Hz，每帧 960 个 sample / 1,920 字节，60 ms |
| 视频 | 通过 `VideoFrame` 表示的 H.264 Annex-B access unit |

### 最小音频 Adapter

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

### 一对一音频通话

```python
from tryx.types import JID


async def start_audio_call(client, phone_number: str):
    peer = JID(phone_number + "@s.whatsapp.net")
    call = await client.voip.call(peer, Microphone(), Speaker())
    print("通话已开始:", call.call_id)
    call.set_muted(True)
    call.set_muted(False)
    await call.wait_ended()
```

### 视频通话

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

### 群组通话和 Call Link

```python
call = await client.voip.group_call(
    peers=[peer_a, peer_b],
    audio_source=Microphone(),
    audio_sink=Speaker(),
)
await call.invite_participant(peer_c)
await call.ring_participant(peer_c)

linked = await client.voip.join_call_link(
    "https://call.whatsapp.com/your-token",
    "audio",
    Microphone(),
    Speaker(),
)
```

### Rust 原生 AudioPlayer

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

## 开发命令

```bash
cargo check
cargo test --lib
uv run pytest -q
uv run maturin develop
uv run maturin build --release
```

## 项目结构

```text
.
├── libs/whatsapp-rust/   # Rust 协议 crate 子模块
├── src/                  # Rust 和 PyO3 绑定
├── python/tryx/          # Python 包和类型存根
├── examples/             # 示例自动化脚本
├── docs/                 # 文档站点
└── tests/                # Python 测试套件
```

## 相关链接

- 文档：<https://krypton-byte.github.io/tryx/>
- 示例：[`examples/`](examples/)
- Python 类型文件：[`python/tryx/`](python/tryx/)
- Rust 绑定代码：[`src/`](src/)

## 许可证

本项目基于 [MIT 许可证](LICENSE) 授权。
