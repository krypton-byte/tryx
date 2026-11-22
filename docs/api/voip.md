# :material-phone: VoIP Namespace (`client.voip`)

`client.voip` bridges `whatsapp-rust` VoIP sessions (RTP/WebRTC, Signal media encryption, native relay, and codec negotiation) with bounded asynchronous Python media adapters.

## Media Contracts

| Media | Contract |
|-------|----------|
| Audio | Mono signed PCM16 little-endian, 16,000 Hz, 960 samples (1,920 bytes) per 60 ms frame |
| Video | H.264 Annex-B access units wrapped in `tryx.media.VideoFrame` |

## Audio & Video Adapters

Implement `AudioSource` (`frames() -> AsyncIterator[bytes]`) and `AudioSink` (`write(frame: bytes)`) from `tryx.media`, or use the built-in Rust-backed `AudioPlayer` and `VideoPlayer`:

```python
from tryx.media import AudioPlayer, VideoPlayer

player = AudioPlayer(buffer_frames=3)
player.play("intro.mp3", mode="replace")
player.enqueue("next.wav")

video_source = VideoPlayer(fps=15)
video_source.play("clip.mp4")
```

## Starting & Handling Calls

```python
from tryx.client import IncomingCallEvent


@app.on(IncomingCallEvent)
async def on_call(_client, call_event: IncomingCallEvent):
    call = await call_event.accept(Microphone(), Speaker())
    print("Call ID:", call.call_id, "Phase:", call.phase)
    await call.wait_ended()
```

## `CallHandle` Controls & State

| Member | Type | Description |
|--------|------|-------------|
| `call_id` | `str` | Unique call identifier |
| `peer` | `JID` | Remote participant JID |
| `phase` | `str \| None` | `"idle"`, `"calling"`, `"ringing"`, `"waiting_room"`, `"connecting"`, `"active"`, `"ended"` |
| `media_connected` | `bool` | Whether RTP/media transport is connected |
| `is_answered` | `bool` | `True` when phase is `"connecting"` or `"active"` |
| `is_muted()` / `await set_muted(muted)` | `bool` / `Awaitable[None]` | Query or toggle microphone mute |
| `await hangup()` | `Awaitable[None]` | Terminate the call remotely and locally |
| `await hangup_local()` | `Awaitable[None]` | Tear down local call media/session |
| `await wait_ended()` | `Awaitable[None]` | Wait until the call ends |
| `await start_video(...)` / `await stop_video()` / `await resume_video(...)` | `Awaitable[None]` | Control video mid-call |
| `await invite_participant(target)` / `await ring_participant(target)` | `Awaitable[None]` | Group call participant controls |
