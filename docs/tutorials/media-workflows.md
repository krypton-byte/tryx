# Tutorial: Media Workflows

## Upload and Send

```python
upload = await client.upload_file("image.jpg", MediaType.Image)
# or use send_photo directly when bytes are available
```

## Download Media

```python
if message := event.data.raw_proto:
    blob = await client.download_media(message.image_message)
```

## Reupload Flow

When media decryption/download fails due expired direct path:

1. collect message metadata (`message_id`, `chat_jid`, `media_key`)
2. call `request_media_reupload(...)`
3. retry download with new path context

## Reliability Checklist

- Validate media type before calling download.
- Add retry with bounded attempts.
- Log message and chat IDs for traceability.
