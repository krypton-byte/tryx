"""Media source/sink contracts for the PyO3 VoIP bridge.

These contracts are intentionally independent from AudioPlayer/VideoPlayer.  A
microphone, camera, TTS stream, DSP pipeline, or custom file player can provide
frames without inheriting a concrete player implementation.
"""

from __future__ import annotations

import asyncio
from abc import ABC, abstractmethod
from collections.abc import AsyncIterator
from dataclasses import dataclass
from typing import Any, Awaitable, Callable

WA_SAMPLE_RATE = 16_000
WA_FRAME_SAMPLES = 960
WA_FRAME_MS = 60


def validate_audio_frame(frame: object) -> bytes:
    """Validate and normalize one PCM frame without accepting ambiguous shapes."""
    if not isinstance(frame, (bytes, bytearray, memoryview)):
        raise TypeError("audio frame must be bytes-like PCM int16 data")
    value = bytes(frame)
    expected = WA_FRAME_SAMPLES * 2
    if len(value) != expected:
        raise ValueError(
            f"audio frame must contain {WA_FRAME_SAMPLES} int16 samples "
            f"({expected} bytes), got {len(value)} bytes"
        )
    return value


@dataclass(frozen=True, slots=True)
class VideoFrame:
    """One H.264 Annex-B access unit received from or sent to a call."""

    data: bytes
    timestamp_us: int
    duration_us: int
    keyframe: bool
    width: int | None
    height: int | None
    orientation: int = 0

    def __post_init__(self) -> None:
        if not self.data:
            raise ValueError("video frame data cannot be empty")
        if self.timestamp_us < 0 or self.duration_us <= 0:
            raise ValueError("video timestamps must be monotonic and duration positive")
        if self.width is not None and self.width <= 0:
            raise ValueError("video dimensions must be positive")
        if self.height is not None and self.height <= 0:
            raise ValueError("video dimensions must be positive")
        if self.orientation not in (0, 1, 2, 3):
            raise ValueError("orientation must be 0, 1, 2, or 3")


class AudioSource(ABC):
    """Outbound 16 kHz mono PCM source, one 60 ms frame per iteration."""

    @abstractmethod
    def frames(self) -> AsyncIterator[bytes]:
        raise NotImplementedError

    async def aclose(self) -> None:
        return None


class AudioSink(ABC):
    """Inbound 16 kHz mono PCM sink."""

    @abstractmethod
    async def write(self, frame: bytes) -> None:
        validate_audio_frame(frame)

    async def aclose(self) -> None:
        return None


class VideoSource(ABC):
    """Outbound H.264 Annex-B access-unit source."""

    @abstractmethod
    def frames(self) -> AsyncIterator[VideoFrame]:
        raise NotImplementedError

    def rtp_timestamp_stride(self) -> int:
        return 90_000 // 15

    async def aclose(self) -> None:
        return None


class VideoSink(ABC):
    """Inbound H.264 Annex-B access-unit sink."""

    @abstractmethod
    async def write(self, frame: VideoFrame) -> None:
        if not isinstance(frame, VideoFrame):
            raise TypeError("video sink expects VideoFrame")

    async def aclose(self) -> None:
        return None


class EncodedAudioSource(ABC):
    """Advanced source for complete negotiated raw Opus/MLOW packets."""

    @abstractmethod
    def frames(self) -> AsyncIterator[bytes]:
        raise NotImplementedError

    async def aclose(self) -> None:
        return None


class EncodedAudioSink(ABC):
    """Advanced sink for complete decrypted codec packets."""

    @abstractmethod
    async def write(self, packet: bytes, timestamp: int, sequence: int) -> None:
        raise NotImplementedError

    async def aclose(self) -> None:
        return None


class _PythonPrototypeAudioPlayer(AudioSource):
    """Bounded PCM player/source with playlist and interruption support.

    The player consumes an async iterable of exact PCM16 frames. File decoding
    is intentionally supplied by a separate decoder/source adapter; this keeps
    playback control independent from MP3/MP4 backend choice.
    """

    def __init__(self, queue_size: int = 6) -> None:
        self._queue: asyncio.Queue[bytes | None] = asyncio.Queue(
            maxsize=max(1, queue_size)
        )
        self._pending: asyncio.Queue[
            tuple[AsyncIterator[bytes], bool]
        ] = asyncio.Queue()
        self._producer: asyncio.Task[None] | None = None
        self._generation = 0
        self._state = "idle"
        self._finish_handlers: list[Callable[[Any], Any | Awaitable[Any]]] = []

    @property
    def state(self) -> str:
        return self._state

    def on_finish(self, callback: Callable[[Any], Any | Awaitable[Any]]) -> None:
        self._finish_handlers.append(callback)

    async def _notify_finish(self, reason: str, generation: int) -> None:
        if generation != self._generation:
            return
        event = type(
            "PlaybackFinished",
            (),
            {"reason": reason, "generation": generation},
        )()
        for callback in tuple(self._finish_handlers):
            result = callback(event)
            if asyncio.iscoroutine(result):
                await result

    async def _run(self, frames: AsyncIterator[bytes], generation: int) -> None:
        self._state = "playing"
        try:
            async for frame in frames:
                while self._state == "paused":
                    await asyncio.sleep(0.01)
                if generation != self._generation or self._state == "stopped":
                    return
                await self._queue.put(validate_audio_frame(frame))
            if generation == self._generation:
                self._state = "finished"
                await self._queue.put(None)
                await self._notify_finish("finished", generation)
        except asyncio.CancelledError:
            raise
        except Exception:
            self._state = "error"
            raise

    async def play(self, frames: AsyncIterator[bytes], mode: str = "replace") -> None:
        if mode not in {"replace", "queue", "interrupt"}:
            raise ValueError("mode must be replace, queue, or interrupt")
        if (
            mode == "queue"
            and self._producer is not None
            and not self._producer.done()
        ):
            await self._pending.put((frames, False))
            return
        if (
            mode == "interrupt"
            and self._producer is not None
            and not self._producer.done()
        ):
            await self._pending.put((frames, True))
            return
        await self.stop()
        self._generation += 1
        self._state = "loading"
        self._producer = asyncio.create_task(self._run(frames, self._generation))

    async def pause(self) -> None:
        if self._state == "playing":
            self._state = "paused"

    async def resume(self) -> None:
        if self._state == "paused":
            self._state = "playing"

    async def stop(self) -> None:
        self._generation += 1
        if self._producer is not None and not self._producer.done():
            self._producer.cancel()
            await asyncio.gather(self._producer, return_exceptions=True)
        self._producer = None
        while not self._queue.empty():
            self._queue.get_nowait()
        self._state = "idle"

    async def frames(self) -> AsyncIterator[bytes]:
        while True:
            frame = await self._queue.get()
            if frame is None:
                return
            yield frame


AudioPlayer = _PythonPrototypeAudioPlayer
VideoPlayer = VideoSource


__all__ = [
    "AudioSink",
    "AudioSource",
    "EncodedAudioSink",
    "EncodedAudioSource",
    "VideoFrame",
    "VideoSink",
    "VideoSource",
    "WA_FRAME_MS",
    "WA_FRAME_SAMPLES",
    "WA_SAMPLE_RATE",
    "validate_audio_frame",
]

# The production file player is implemented in Rust and registered in the
# extension's client namespace. Keep the abstract/source classes above Python
# native, but make AudioPlayer resolve to the Rust implementation.
try:
    from ._tryx import client as _native_client  # type: ignore

    AudioPlayer = _native_client.AudioPlayer
    VideoPlayer = _native_client.VideoPlayer
except (ImportError, AttributeError):
    # Source-only environments can still import the contracts before the
    # extension is built.
    pass
