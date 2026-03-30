"""Command bot example for Tryx.

Commands:
- ping      -> reply pong
- pp        -> download sender profile picture and send it back
- pushname  -> show sender push name from incoming message metadata
- bio       -> fetch sender about/bio using contact user info API
- help      -> show available commands
"""

from __future__ import annotations

import asyncio
import os
from urllib.request import urlopen

from tryx.backend import SqliteBackend
from tryx.client import Tryx, TryxClient
from tryx.events import EvMessage, EvPushNameUpdate, EvUserAboutUpdate

DB_PATH = os.getenv("TRYX_DB_PATH", "whatsapp.db")


def jid_to_text(jid: object) -> str:
    user = getattr(jid, "user", "")
    server = getattr(jid, "server", "")
    return f"{user}@{server}"


async def download_bytes(url: str) -> bytes:
    def _download() -> bytes:
        with urlopen(url, timeout=20) as response:
            return response.read()

    return await asyncio.to_thread(_download)


backend = SqliteBackend(DB_PATH)
bot = Tryx(backend)


@bot.on(EvPushNameUpdate)
async def on_push_name_update(_client: TryxClient, event: EvPushNameUpdate) -> None:
    data = event.data
    print(
        "[pushname-update]",
        jid_to_text(data.jid),
        "=>",
        repr(data.new_push_name),
    )


@bot.on(EvUserAboutUpdate)
async def on_user_about_update(_client: TryxClient, event: EvUserAboutUpdate) -> None:
    data = event.data
    print("[bio-update]", jid_to_text(data.jid), "=>", repr(data.status))


@bot.on(EvMessage)
async def on_message(client: TryxClient, event: EvMessage) -> None:
    data = event.data
    info = data.message_info
    source = info.source

    chat_jid = source.chat
    sender_jid = source.sender

    text = (data.get_text() or data.caption or "").strip().lower()
    if not text:
        return

    print(f"[message] from={jid_to_text(sender_jid)} chat={jid_to_text(chat_jid)} text={text!r}")

    if text in {"help", "menu"}:
        await client.send_text(
            chat_jid,
            "Perintah: ping | pp | pushname | bio",
            quoted=event,
        )
        return

    if text == "ping":
        await client.send_text(chat_jid, "pong", quoted=event)
        return

    if text == "pushname":
        push_name = info.push_name.strip() if info.push_name else "(kosong)"
        await client.send_text(chat_jid, f"Pushname: {push_name}", quoted=event)
        return

    if text == "bio":
        try:
            info_map = await client.contact.get_user_info(sender_jid)
        except Exception as exc:
            await client.send_text(chat_jid, f"Gagal ambil bio: {exc}", quoted=event)
            return

        user_info = next(iter(info_map.values()), None)
        status_text = "(tidak ada)" if user_info is None or not user_info.status else user_info.status
        await client.send_text(chat_jid, f"Bio: {status_text}", quoted=event)
        return

    if text == "pp":
        try:
            profile_picture = await client.contact.get_profile_picture(sender_jid, False)
        except Exception as exc:
            await client.send_text(chat_jid, f"Gagal ambil profile picture: {exc}", quoted=event)
            return

        if not profile_picture.url:
            await client.send_text(chat_jid, "Profile picture tidak tersedia.", quoted=event)
            return

        try:
            photo_data = await download_bytes(profile_picture.url)
        except Exception as exc:
            await client.send_text(chat_jid, f"Gagal download profile picture: {exc}", quoted=event)
            return

        await client.send_photo(
            chat_jid,
            photo_data,
            caption=f"Profile picture dari {jid_to_text(sender_jid)}",
            quoted=event,
        )


async def main() -> None:
    print(f"Starting command bot with DB: {DB_PATH}")
    await bot.run()


if __name__ == "__main__":
    asyncio.run(main())
