"""Media Bot — demonstrates sending various media types.

Commands:
  photo    -> send a sample photo from URL
  document -> send a sample PDF document
  audio    -> send a sample audio clip
  video    -> send a sample video clip
  sticker  -> send a sample sticker
  gif      -> send a sample GIF
"""

import asyncio
import os
from urllib.request import urlopen

from tryx.backend import SqliteStore
from tryx.client import Tryx, TryxClient
from tryx.events import EvMessage, EvPairingQrCode

DB_PATH = os.getenv("TRYX_DB_PATH", "whatsapp.db")

# ── Sample media URLs (public domain) ────────────────────────────────────────

SAMPLE_MEDIA = {
    "photo": {
        "url": "https://samplelib.com/lib/preview/png/sample-boat-400x300.png",
        "name": "sample-boat.png",
    },
    "document": {
        "url": "https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf",
        "name": "sample-document.pdf",
    },
    "audio": {
        "url": "https://samplelib.com/lib/preview/mp3/sample-3s.mp3",
        "name": "sample-audio.mp3",
    },
    "video": {
        "url": "https://samplelib.com/lib/preview/mp4/sample-5s.mp4",
        "name": "sample-video.mp4",
    },
}


def jid_to_text(jid: object) -> str:
    """Format a JID object as user@server string."""
    user = getattr(jid, "user", "")
    server = getattr(jid, "server", "")
    return f"{user}@{server}"


async def download_bytes(url: str) -> bytes:
    """Download bytes from an HTTPS URL. Rejects non-HTTPS for safety."""
    if not url.startswith("https://"):
        raise ValueError(f"Refusing to download from non-HTTPS URL: {url}")

    def _download() -> bytes:
        with urlopen(url, timeout=30) as response:  # noqa: S310
            return response.read()

    return await asyncio.to_thread(_download)


# ── Setup ────────────────────────────────────────────────────────────────────

backend = SqliteStore(DB_PATH)
app = Tryx(backend)


@app.on(EvPairingQrCode)
async def on_pairing_qr(_client: TryxClient, event: EvPairingQrCode) -> None:
    """Display the QR code for initial device pairing."""
    print("=" * 40)
    print("Scan this QR code with WhatsApp:")
    print(event.code)
    print("=" * 40)


@app.on(EvMessage)
async def on_message(client: TryxClient, event: EvMessage) -> None:
    """Handle incoming media requests."""
    data = event.data
    source = data.message_info.source
    chat_jid = source.chat
    sender_jid = source.sender
    text = (data.get_text() or "").strip().lower()

    sender = jid_to_text(sender_jid)
    chat = jid_to_text(chat_jid)
    print(f"[message] from={sender} chat={chat} text={text!r}")

    if not text:
        return

    # ── /photo ───────────────────────────────────────────────────────────
    if text == "photo":
        try:
            await client.chatstate.send_composing(chat_jid)
            photo_data = await download_bytes(SAMPLE_MEDIA["photo"]["url"])
            result = await client.send_photo(
                chat_jid, photo_data, caption="Sample photo from public domain"
            )
            print(f"[photo] sent: {result.message_id}")
        except Exception as exc:
            msg = f"Failed to send photo: {exc}"
            await client.send_text(chat_jid, msg, quoted=event)

    # ── /document ────────────────────────────────────────────────────────
    elif text == "document":
        try:
            await client.chatstate.send_composing(chat_jid)
            doc_data = await download_bytes(SAMPLE_MEDIA["document"]["url"])
            result = await client.send_document(
                chat_jid,
                doc_data,
                file_name="sample.pdf",
                caption="Sample PDF document",
            )
            print(f"[document] sent: {result.message_id}")
        except Exception as exc:
            msg = f"Failed to send document: {exc}"
            await client.send_text(chat_jid, msg, quoted=event)

    # ── /audio ───────────────────────────────────────────────────────────
    elif text == "audio":
        try:
            await client.chatstate.send_composing(chat_jid)
            audio_data = await download_bytes(SAMPLE_MEDIA["audio"]["url"])
            result = await client.send_audio(chat_jid, audio_data)
            print(f"[audio] sent: {result.message_id}")
        except Exception as exc:
            msg = f"Failed to send audio: {exc}"
            await client.send_text(chat_jid, msg, quoted=event)

    # ── /video ───────────────────────────────────────────────────────────
    elif text == "video":
        try:
            await client.chatstate.send_composing(chat_jid)
            video_data = await download_bytes(SAMPLE_MEDIA["video"]["url"])
            result = await client.send_video(
                chat_jid, video_data, caption="Sample video clip"
            )
            print(f"[video] sent: {result.message_id}")
        except Exception as exc:
            msg = f"Failed to send video: {exc}"
            await client.send_text(chat_jid, msg, quoted=event)

    # ── /help ────────────────────────────────────────────────────────────
    elif text == "help":
        help_text = (
            "*Media Bot Commands*\n\n"
            "• photo — send a sample PNG image\n"
            "• document — send a sample PDF\n"
            "• audio — send a sample MP3 clip\n"
            "• video — send a sample MP4 video\n"
            "• help — show this message"
        )
        await client.send_text(chat_jid, help_text, quoted=event)


# ── Entry point ──────────────────────────────────────────────────────────────


async def main() -> None:
    print(f"Starting media bot with DB: {DB_PATH}")
    await app.run()


if __name__ == "__main__":
    asyncio.run(main())
