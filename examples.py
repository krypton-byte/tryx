import asyncio
from tryx.backend import SqliteBackend
from tryx.client import TryxClient, Tryx  # , Test, K
from tryx.events import EvMessage
from tryx.waproto.whatsapp_pb2 import Message as msg

DB_PATH = "whatsapp.db"

# t = Test()
# @t.on(K)
# async def handle_event(client: TryxClient, event: int) -> None:
#     print("Handling event with data:", event)

backend = SqliteBackend(DB_PATH)
client = Tryx(backend)


@client.on(EvMessage)
async def on_message(client: TryxClient, event: EvMessage) -> None:
    info = event.message_info
    source = info.source
    sender = source.sender

    text = event.get_text() or event.caption or "<non-text message>"
    sender_jid = f"{sender.user}@{sender.server}"
    chat = source.chat
    print(event.raw_proto)
    print(f"[{info.id}] {sender_jid}: {text}")
    print("text:", event.get_text())
    print("client:", client)
    print("chat:", chat, dir(chat))
    await client.send_message(chat, msg(conversation="Hello!"))
    await asyncio.sleep(1)
    await client.send_message(chat, msg(conversation="This is a follow-up message."))
    await asyncio.sleep(4)
    await client.send_message(
        chat, msg(conversation="This is a message after a delay.")
    )


async def main() -> None:
    await client.run()


if __name__ == "__main__":
    loop = asyncio.new_event_loop()
    loop.run_until_complete(main())
