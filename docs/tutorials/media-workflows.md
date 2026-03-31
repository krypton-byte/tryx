# Tutorial: Media Workflows

This tutorial covers send, download, and recovery flows for media-heavy bots.

## Step 1: Upload and Send

```python
from tryx.wacore import MediaType

upload = await client.upload_file("image.jpg", MediaType.Image)
await client.send_photo(chat_jid, photo_data=image_bytes, caption="report")
```

!!! tip
    Use `upload_file` when media already exists on disk. Use `upload` for in-memory transformed bytes.

## Step 2: Download Media

```python
if message := event.data.raw_proto:
    if message.image_message:
        blob = await client.download_media(message.image_message)
        with open("downloaded.jpg", "wb") as fp:
            fp.write(blob)
```

## Step 3: Reupload Recovery

When direct media path is stale:

1. collect metadata: `message_id`, `chat_jid`, `media_key`, optional participant
2. call `request_media_reupload(...)`
3. retry download

```python
result = await client.request_media_reupload(
    message_id=message_id,
    chat_jid=chat_jid,
    media_key=media_key,
    is_from_me=False,
    participant=participant_jid,
)
```

## Step 4: Production-safe Retry Pattern

```python
async def download_with_retry(client, media_node, retries=3):
    last_exc = None
    for _ in range(retries):
        try:
            return await client.download_media(media_node)
        except Exception as exc:
            last_exc = exc
    raise last_exc
```

## Checklist

=== "Correctness"
    - validate media subtype before download
    - preserve original metadata for retry

=== "Performance"
    - stream large blobs to disk/object storage
    - avoid storing many large payloads in memory simultaneously

=== "Observability"
    - log message id, chat id, payload size, and retry count

## Related Docs

- [WACore API](../api/wacore.md)
- [Poll Survey Tutorial](poll-survey.md)
- [Performance Operations](../operations/performance.md)
