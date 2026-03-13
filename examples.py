from pathlib import Path
import asyncio
import segno
import asyncio
from tryx import Message, PairingQrCode, SqliteBackend, Tryx
from tryx.waproto.whatsapp_pb2 import Message as msg

DB_PATH = "whatsapp.db"


backend = SqliteBackend(DB_PATH)
client = Tryx(backend)

@client.on(Message)
async def on_message(client, event: Message) -> None:
    info = event.message_info
    source = info.source
    sender = source.sender

    text = event.get_text() or event.caption or "<non-text message>"
    sender_jid = f"{sender.user}@{sender.server}"
    chat = source.chat

    print(f"[{info.id}] {sender_jid}: {text}")
    print("text:", event.get_text())
    print("client:", client)
    print("chat:", chat)
    b = await client.send_message(chat, msg(conversation="Hello from Tryx!"))
    await asyncio.sleep(1)
    await client.send_message(chat, msg(conversation="This is a follow-up message."))
    await asyncio.sleep(4)
    await client.send_message(chat, msg(conversation="This is a message after a delay."))

async def main() -> None:
    await client.run()

if __name__ == "__main__":
    loop = asyncio.new_event_loop()
    loop.run_until_complete(main())