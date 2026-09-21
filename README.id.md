<div align="center">
<img src="assets/mascot.png" width="20%" alt="Tryx">

# Tryx

[![PyPI version](https://img.shields.io/pypi/v/tryx?color=blue)](https://pypi.org/project/tryx/)
[![Python](https://img.shields.io/pypi/pyversions/tryx.svg)](https://pypi.org/project/tryx/)
[![License](https://img.shields.io/github/license/krypton-byte/tryx)](LICENSE)

</div>
[![Docs](https://img.shields.io/badge/docs-online-brightgreen)](https://krypton-byte.github.io/tryx/)

**Bahasa:** [English](README.md) | [简体中文](README.zh.md) | Bahasa Indonesia

Tryx adalah library otomasi WhatsApp untuk Python yang dibangun dengan Rust dan PyO3. API publiknya dibuat agar nyaman digunakan dari Python, dengan dukungan async, sesi persisten, dan akses ke fitur protokol WhatsApp Web tingkat lanjut.

> Tryx adalah proyek independen. Tidak berafiliasi dengan WhatsApp, Meta, atau produk resmi mereka.

## Apa Itu Tryx?

Tryx membantu pengembang Python membangun tools otomasi WhatsApp tanpa harus menulis Rust secara langsung. Logika protokol berjalan di Rust untuk performa, sedangkan API publiknya tetap nyaman untuk aplikasi Python.

Tryx cocok untuk:

- Otomasi WhatsApp Web dari Python.
- Sesi login persisten dengan SQLite, PostgreSQL, atau MySQL.
- API async untuk bot, worker, dashboard, dan tool internal.
- Akses ke kontak, grup, newsletter, status, privasi, profil, label, komentar, event, dan helper protokol tingkat lanjut.

## Fitur Utama

- **Core Rust, API Python** — logika protokol diimplementasikan dalam Rust dan diekspos melalui PyO3.
- **Async first** — dirancang untuk aplikasi `asyncio`.
- **Sesi persisten** — sesi perangkat dapat digunakan ulang melalui database.
- **API luas** — namespace umum dan advanced sudah diexpose.
- **Type stub Python** — tersedia `.pyi` untuk autocomplete dan type checking yang lebih baik.

## Instalasi

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

## Mulai Cepat

```python
import asyncio

from tryx.backend import SqliteStore
from tryx.client import Tryx, TryxClient
from tryx.events import EvMessage, EvPairingQrCode

backend = SqliteStore("whatsapp.db")
app = Tryx(backend)


@app.on(EvPairingQrCode)
async def on_pairing_qr(_client: TryxClient, event: EvPairingQrCode) -> None:
    print("Scan kode QR ini dengan WhatsApp:")
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

## Namespace Client

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
| `voip` | Manajemen panggilan audio dan video. |

## Storage Backend

Tryx mendukung tiga tipe storage untuk menyimpan sesi WhatsApp:

| Tipe | Backend | Kegunaan |
| --- | --- | --- |
| Built-in | `SqliteStore("whatsapp.db")` | Development lokal dan deployment sederhana. |
| Native FFI | `FfiStoreProtocol` | Store eksternal throughput tinggi seperti PostgreSQL. |
| Pure Python | subclass `StoreBase` | Store async custom seperti Redis, MongoDB, atau DynamoDB. |

SQLite cukup untuk development lokal. Native FFI atau backend Python custom lebih cocok jika sesi perlu dipakai bersama oleh beberapa worker atau environment deployment.

## VoIP: Panggilan Audio dan Video

Tryx menyediakan bridge VoIP berbasis Rust untuk panggilan audio dan video WhatsApp. Protocol, RTP/WebRTC, enkripsi, codec, dan orkestrasi call berjalan di whatsapp-rust; Python menyediakan media melalui adapter source dan sink asynchronous.

### Kontrak Media

| Media | Kontrak |
| --- | --- |
| Audio | Mono signed PCM16 little-endian, 16.000 Hz, 960 sample / 1.920 byte per frame, 60 ms |
| Video | H.264 Annex-B access unit melalui `VideoFrame` |

### Adapter Audio Minimal

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

### Panggilan Audio 1:1

```python
from tryx.types import JID


async def start_audio_call(client, phone_number: str):
    peer = JID(phone_number + "@s.whatsapp.net")
    call = await client.voip.call(peer, Microphone(), Speaker())
    print("panggilan dimulai:", call.call_id)
    call.set_muted(True)
    call.set_muted(False)
    await call.wait_ended()
```

### Panggilan Video

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

### Group Call dan Call Link

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

### AudioPlayer Native

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

## Development

```bash
cargo check
cargo test --lib
uv run pytest -q
uv run maturin develop
uv run maturin build --release
```

## Struktur Project

```text
.
├── libs/whatsapp-rust/   # Submodule crate protokol Rust
├── src/                  # Binding Rust dan PyO3
├── python/tryx/          # Package Python dan type stub
├── examples/             # Contoh script otomasi
├── docs/                 # Situs dokumentasi
└── tests/                # Test suite Python
```

## Link Penting

- Dokumentasi: <https://krypton-byte.github.io/tryx/>
- Contoh: [`examples/`](examples/)
- Type stub Python: [`python/tryx/`](python/tryx/)
- Binding Rust: [`src/`](src/)

## Lisensi

Proyek ini dilisensikan di bawah [MIT License](LICENSE).
