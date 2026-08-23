"""Basic Echo Bot — replies to every text message.

Commands:
  ping     -> reply pong
  help     -> show available commands
  time     -> show current server time
  info     -> show sender info
"""

import asyncio
import os
from datetime import datetime, timezone

from tryx.backend import SqliteStore
from tryx.client import Tryx, TryxClient
from tryx.events import EvMessage, EvPairingQrCode

DB_PATH = os.getenv("TRYX_DB_PATH", "whatsapp.db")


def jid_to_text(jid: object) -> str:
    """Format a JID object as user@server string."""
    user = getattr(jid, "user", "")
    server = getattr(jid, "server", "")
    return f"{user}@{server}"


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
    """Handle incoming messages and dispatch commands."""
    data = event.data
    info = data.message_info
    source = info.source
    chat_jid = source.chat
    sender_jid = source.sender
    text = (data.get_text() or "").strip()

    sender = jid_to_text(sender_jid)
    chat = jid_to_text(chat_jid)
    print(f"[message] from={sender} chat={chat} text={text!r}")

    if not text:
        return

    cmd = text.lower()

    # ── /ping ────────────────────────────────────────────────────────────
    if cmd == "ping":
        await client.chatstate.send_composing(chat_jid)
        await asyncio.sleep(1)  # Simulate processing
        await client.send_text(chat_jid, "pong", quoted=event)
        await client.chatstate.send_paused(chat_jid)

    # ── /help ────────────────────────────────────────────────────────────
    elif cmd == "help":
        help_text = (
            "*Available Commands*\n\n"
            "• ping — check if bot is alive\n"
            "• help — show this message\n"
            "• time — show current time\n"
            "• info — show your info"
        )
        await client.send_text(chat_jid, help_text, quoted=event)

    # ── /time ────────────────────────────────────────────────────────────
    elif cmd == "time":
        now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")
        await client.send_text(chat_jid, f"Current time: {now}", quoted=event)

    # ── /info ────────────────────────────────────────────────────────────
    elif cmd == "info":
        info_lines = [
            "*Your Info*",
            f"• JID: {jid_to_text(sender_jid)}",
            f"• Chat: {jid_to_text(chat_jid)}",
            f"• Push name: {info.push_name or '(none)'}",
        ]
        await client.send_text(chat_jid, "\n".join(info_lines), quoted=event)


# ── Entry point ──────────────────────────────────────────────────────────────


async def main() -> None:
    print(f"Starting basic bot with DB: {DB_PATH}")
    await app.run()


if __name__ == "__main__":
    asyncio.run(main())
