"""Media source/sink contracts for the PyO3 VoIP bridge.

These contracts are intentionally independent from AudioPlayer/VideoPlayer.  A
microphone, camera, TTS stream, DSP pipeline, or custom file player can provide
frames without inheriting a concrete player implementation.
"""

from collections.abc import AsyncIterator
from dataclasses import dataclass
from typing import Final

WA_SAMPLE_RATE: Final[int]
WA_FRAME_SAMPLES: Final[int]
WA_FRAME_MS: Final[int]

def validate_audio_frame(frame: bytes | bytearray | memoryview) -> bytes: ...

@dataclass(frozen=True)
class VideoFrame:
    """One H.264 Annex-B access unit received from or sent to a call."""

    data: bytes
    timestamp_us: int
    duration_us: int
    keyframe: bool
    width: int | None
    height: int | None
    orientation: int = 0

class AudioSource:
    """Abstract audio source that produces raw PCM frames."""

    def frames(self) -> AsyncIterator[bytes]:
        """
        Yield raw PCM audio frames (16-bit LE, 48 kHz mono).

        Yields:
            Bytes of raw PCM audio.
        """
        ...
    async def aclose(self) -> None:
        """
        Release resources held by this audio source.
        """
        ...

class AudioPlayer(AudioSource):
    """Built-in audio player that decodes and plays audio files."""

    def __init__(self, buffer_frames: int = 3) -> None:
        """
        Create an audio player with a decoder and command channel.

        Args:
            buffer_frames: Number of decoded frames to buffer (1-30).
        """
        ...
    def play(self, path: str, mode: str | None = None) -> None:
        """
        Start playing an audio file.

        Args:
            path: Filesystem path to the audio file.
            mode: Playback mode (``'replace'``, ``'queue'``, ``'interrupt'``),
                or ``None`` for default replace behavior.
        """
        ...
    def stop(self) -> None:
        """
        Stop playback and flush the command queue.
        """
        ...
    def pause(self) -> None:
        """
        Pause playback (resume with ``resume()``).
        """
        ...
    def resume(self) -> None:
        """
        Resume paused playback.
        """
        ...
    def enqueue(self, path: str) -> None:
        """
        Add an audio file to the playback queue.

        Args:
            path: Filesystem path to the audio file.
        """
        ...
    def skip(self) -> None:
        """
        Skip the currently playing file and play the next in queue.
        """
        ...
    def clear_queue(self) -> None:
        """
        Clear all queued audio files.
        """
        ...
    @property
    def state(self) -> str:
        """
        Return the current player state string.

        Returns:
            State string (``'idle'``, ``'playing'``, ``'paused'``).
        """
        ...

class VideoPlayer(VideoSource):
    """Built-in video player that demuxes and decodes video files."""

    def __init__(self, fps: int = 15) -> None:
        """
        Create a video player with FFmpeg demuxer.

        Args:
            fps: Target frames per second (1-60).
        """
        ...
    def play(self, path: str) -> None:
        """
        Start playing a video file.

        Args:
            path: Filesystem path to the video file.
        """
        ...
    def stop(self) -> None:
        """
        Stop video playback.
        """
        ...

class AudioSink:
    """Abstract audio sink that consumes raw PCM frames."""

    async def write(self, frame: bytes) -> None:
        """
        Write a raw PCM audio frame.

        Args:
            frame: Raw PCM bytes (16-bit LE, 48 kHz mono).
        """
        ...
    async def aclose(self) -> None:
        """
        Release resources held by this audio sink.
        """
        ...

class VideoSource:
    """Abstract video source that produces decoded video frames."""

    def frames(self) -> AsyncIterator[VideoFrame]:
        """
        Yield decoded H.264 video frames.

        Yields:
            VideoFrame access units.
        """
        ...
    def rtp_timestamp_stride(self) -> int:
        """
        Return the RTP timestamp stride for this video source.

        Returns:
            Timestamp stride integer (ticks per frame).
        """
        ...
    async def aclose(self) -> None:
        """
        Release resources held by this video source.
        """
        ...

class VideoSink:
    """Abstract video sink that consumes decoded video frames."""

    async def write(self, frame: VideoFrame) -> None:
        """
        Write a decoded video frame.

        Args:
            frame: VideoFrame access unit.
        """
        ...
    async def aclose(self) -> None:
        """
        Release resources held by this video sink.
        """
        ...

class EncodedAudioSource:
    """Abstract source that produces encoded audio packets (e.g. Opus)."""

    def frames(self) -> AsyncIterator[bytes]:
        """
        Yield encoded audio packets (e.g. Opus).

        Yields:
            Encoded audio packet bytes.
        """
        ...
    async def aclose(self) -> None:
        """
        Release resources held by this encoded audio source.
        """
        ...

class EncodedAudioSink:
    """Abstract sink that consumes encoded audio packets."""

    async def write(self, packet: bytes, timestamp: int, sequence: int) -> None:
        """
        Write an encoded audio packet.

        Args:
            packet: Encoded audio bytes.
            timestamp: RTP timestamp.
            sequence: Packet sequence number.
        """
        ...
    async def aclose(self) -> None:
        """
        Release resources held by this encoded audio sink.
        """
        ...
