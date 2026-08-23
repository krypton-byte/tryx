import asyncio

import pytest

from tryx.media import (
    AudioSink,
    AudioSource,
    VideoFrame,
    validate_audio_frame,
)
from tryx.media import _PythonPrototypeAudioPlayer as AudioPlayer


async def _frames(*frames: bytes):
    for frame in frames:
        yield frame


def test_audio_frame_contract_requires_exact_60ms_pcm16():
    frame = bytes(960 * 2)
    assert validate_audio_frame(frame) == frame
    with pytest.raises(ValueError):
        validate_audio_frame(bytes(959 * 2))


def test_audio_contracts_are_abstract():
    with pytest.raises(TypeError):
        AudioSource()
    with pytest.raises(TypeError):
        AudioSink()


def test_video_frame_validates_metadata():
    frame = VideoFrame(b"\x00\x00\x00\x01\x09", 0, 66_666, True, 640, 360)
    assert frame.orientation == 0
    with pytest.raises(ValueError):
        VideoFrame(b"x", 0, 0, False, 640, 360)


def test_audio_player_emits_frames_and_finishes_once():
    async def run():
        player = AudioPlayer(queue_size=2)
        finished = []
        player.on_finish(lambda event: finished.append(event.reason))
        frame = bytes(960 * 2)
        await player.play(_frames(frame))
        assert [value async for value in player.frames()] == [frame]
        assert finished == ["finished"]

    asyncio.run(run())
