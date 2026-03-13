from pathlib import Path
import asyncio
import segno
import asyncio
from tryx import Message, PairingQrCode, SqliteBackend, Tryx


DB_PATH = "whatsapp.db"


backend = SqliteBackend(DB_PATH)
client = Tryx(backend)

@client.on(Message)
async def on_message(_, event: Message) -> None:
	info = event.message_info
	source = info.source
	sender = source.sender

	text = event.get_text() or event.caption or "<non-text message>"
	sender_jid = f"{sender.user}@{sender.server}"

	print(f"[{info.id}] {sender_jid}: {text}")

async def main() -> None:
    await client.run()

if __name__ == "__main__":
    loop = asyncio.new_event_loop()
    loop.run_until_complete(main())