"""Group Management Bot — demonstrates group admin operations.

Commands (admin only):
  info        -> show group metadata
  pin         -> pin the group
  unpin       -> unpin the group
  announce    -> toggle announcement-only mode
  lock        -> toggle group info lock
  ephemeral N -> set disappearing messages timer (seconds)
  members     -> list group members
  help        -> show available commands
"""

import asyncio
import os

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
    """Handle group admin commands."""
    data = event.data
    info = data.message_info
    source = info.source
    chat_jid = source.chat
    sender_jid = source.sender
    text = (data.get_text() or "").strip()

    sender = jid_to_text(sender_jid)
    chat = jid_to_text(chat_jid)
    print(f"[message] from={sender} chat={chat} text={text!r}")

    if not text or not source.is_group:
        return

    cmd_parts = text.lower().split()
    cmd = cmd_parts[0] if cmd_parts else ""

    # ── /info ────────────────────────────────────────────────────────────
    if cmd == "info":
        try:
            metadata = await client.groups.get_metadata(chat_jid)
            lines = [
                "*Group Info*",
                f"• Name: {metadata.subject}",
                f"• Members: {metadata.size or len(metadata.participants)}",
                f"• Locked: {'Yes' if metadata.is_locked else 'No'}",
                f"• Announce: {'Yes' if metadata.is_announcement else 'No'}",
                f"• Ephemeral: {metadata.ephemeral_expiration}s",
            ]
            if metadata.description:
                lines.append(f"• Description: {metadata.description}")
            await client.send_text(chat_jid, "\n".join(lines), quoted=event)
        except Exception as exc:
            await client.send_text(chat_jid, f"Error: {exc}", quoted=event)

    # ── /pin ─────────────────────────────────────────────────────────────
    elif cmd == "pin":
        await client.chat_actions.pin_chat(chat_jid)
        await client.send_text(chat_jid, "Chat pinned", quoted=event)

    # ── /unpin ───────────────────────────────────────────────────────────
    elif cmd == "unpin":
        await client.chat_actions.unpin_chat(chat_jid)
        await client.send_text(chat_jid, "Chat unpinned", quoted=event)

    # ── /announce ────────────────────────────────────────────────────────
    elif cmd == "announce":
        try:
            metadata = await client.groups.get_metadata(chat_jid)
            new_value = not metadata.is_announcement
            await client.groups.set_announce(chat_jid, new_value)
            state = "enabled" if new_value else "disabled"
            await client.send_text(chat_jid, f"Announcement mode {state}", quoted=event)
        except Exception as exc:
            await client.send_text(chat_jid, f"Error: {exc}", quoted=event)

    # ── /lock ────────────────────────────────────────────────────────────
    elif cmd == "lock":
        try:
            metadata = await client.groups.get_metadata(chat_jid)
            new_value = not metadata.is_locked
            await client.groups.set_locked(chat_jid, new_value)
            state = "locked" if new_value else "unlocked"
            await client.send_text(chat_jid, f"Group info {state}", quoted=event)
        except Exception as exc:
            await client.send_text(chat_jid, f"Error: {exc}", quoted=event)

    # ── /ephemeral N ─────────────────────────────────────────────────────
    elif cmd == "ephemeral":
        if len(cmd_parts) < 2:
            msg = "Usage: /ephemeral <seconds>\n0 = off"
            await client.send_text(chat_jid, msg, quoted=event)
            return
        try:
            seconds = int(cmd_parts[1])
            await client.groups.set_ephemeral(chat_jid, seconds)
            if seconds == 0:
                msg = "Disabling disappearing messages"
                await client.send_text(chat_jid, msg, quoted=event)
            else:
                msg = f"Disappearing messages set to {seconds}s"
                await client.send_text(chat_jid, msg, quoted=event)
        except ValueError:
            await client.send_text(chat_jid, "Invalid number", quoted=event)

    # ── /members ─────────────────────────────────────────────────────────
    elif cmd == "members":
        try:
            metadata = await client.groups.get_metadata(chat_jid)
            lines = [f"*Members ({len(metadata.participants)})*"]
            for p in metadata.participants[:30]:  # limit to 30
                role = " 👑" if p.is_admin else ""
                lines.append(f"• {jid_to_text(p.jid)}{role}")
            if len(metadata.participants) > 30:
                lines.append(f"• ... and {len(metadata.participants) - 30} more")
            await client.send_text(chat_jid, "\n".join(lines), quoted=event)
        except Exception as exc:
            await client.send_text(chat_jid, f"Error: {exc}", quoted=event)

    # ── /help ────────────────────────────────────────────────────────────
    elif cmd == "help":
        help_text = (
            "*Group Admin Commands*\n\n"
            "• info — show group metadata\n"
            "• pin — pin the group\n"
            "• unpin — unpin the group\n"
            "• announce — toggle announcement-only mode\n"
            "• lock — toggle group info lock\n"
            "• ephemeral N — set disappearing messages timer\n"
            "• members — list group members\n"
            "• help — show this message"
        )
        await client.send_text(chat_jid, help_text, quoted=event)


# ── Entry point ──────────────────────────────────────────────────────────────


async def main() -> None:
    print(f"Starting group bot with DB: {DB_PATH}")
    await app.run()


if __name__ == "__main__":
    asyncio.run(main())
