from __future__ import annotations

from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path

from epidote.models import TranscriptDocument, TranscriptSegment, utc_now


@dataclass(slots=True)
class SpeakerTurn:
    start_seconds: float
    end_seconds: float
    speaker: str


class FasterWhisperTranscriber:
    def __init__(self, model_name: str) -> None:
        self.model_name = model_name
        self._model = None

    def transcribe(
        self,
        audio_path: Path,
        speaker: str,
        source_track: str,
    ) -> list[TranscriptSegment]:
        if not audio_path.exists():
            return []
        model = self._load_model()
        raw_segments, _ = model.transcribe(
            str(audio_path),
            vad_filter=True,
            beam_size=5,
        )
        segments: list[TranscriptSegment] = []
        for item in raw_segments:
            text = item.text.strip()
            if not text:
                continue
            segments.append(
                TranscriptSegment(
                    start_seconds=float(item.start),
                    end_seconds=float(item.end),
                    speaker=speaker,
                    text=text,
                    source_track=source_track,
                )
            )
        return segments

    def _load_model(self):
        if self._model is None:
            from faster_whisper import WhisperModel

            self._model = WhisperModel(
                self.model_name,
                device="auto",
                compute_type="default",
            )
        return self._model


class PyannoteDiarizer:
    def __init__(self, token: str) -> None:
        self.token = token
        self._pipeline = None

    def is_configured(self) -> bool:
        return bool(self.token)

    def diarize(self, audio_path: Path) -> list[SpeakerTurn]:
        if not self.is_configured() or not audio_path.exists():
            return []
        pipeline = self._load_pipeline()
        diarization = pipeline(str(audio_path))
        speaker_map = {
            speaker_name: f"speaker_{index}"
            for index, speaker_name in enumerate(
                sorted({speaker for _, _, speaker in diarization.itertracks(yield_label=True)}),
                start=2,
            )
        }
        turns: list[SpeakerTurn] = []
        for turn, _, speaker_name in diarization.itertracks(yield_label=True):
            turns.append(
                SpeakerTurn(
                    start_seconds=float(turn.start),
                    end_seconds=float(turn.end),
                    speaker=speaker_map[speaker_name],
                )
            )
        return turns

    def _load_pipeline(self):
        if self._pipeline is None:
            from pyannote.audio import Pipeline

            self._pipeline = Pipeline.from_pretrained(
                "pyannote/speaker-diarization-3.1",
                use_auth_token=self.token,
            )
        return self._pipeline


class MeetingTranscriptionService:
    def __init__(self, whisper_model: str, hf_token: str) -> None:
        self.transcriber = FasterWhisperTranscriber(model_name=whisper_model)
        self.diarizer = PyannoteDiarizer(token=hf_token)

    def transcribe_session(self, session_id: str, session_dir: Path) -> TranscriptDocument:
        microphone_path = session_dir / "microphone.wav"
        system_audio_path = session_dir / "system_audio.m4a"
        local_segments = self.transcriber.transcribe(
            audio_path=microphone_path,
            speaker="speaker_1",
            source_track="microphone",
        )
        remote_segments = self.transcriber.transcribe(
            audio_path=system_audio_path,
            speaker="speaker_2",
            source_track="system",
        )
        speaker_turns = self.diarizer.diarize(system_audio_path)
        if speaker_turns:
            remote_segments = assign_speakers(remote_segments, speaker_turns)
        all_segments = sorted(
            [*local_segments, *remote_segments],
            key=lambda item: (item.start_seconds, item.end_seconds),
        )
        speakers = sorted({item.speaker for item in all_segments})
        return TranscriptDocument(
            session_id=session_id,
            created_at=utc_now(),
            speakers=speakers,
            segments=all_segments,
        )


def assign_speakers(
    segments: list[TranscriptSegment],
    speaker_turns: list[SpeakerTurn],
) -> list[TranscriptSegment]:
    if not speaker_turns:
        return segments
    assigned: list[TranscriptSegment] = []
    for segment in segments:
        overlaps: dict[str, float] = defaultdict(float)
        for turn in speaker_turns:
            overlap = min(segment.end_seconds, turn.end_seconds) - max(
                segment.start_seconds,
                turn.start_seconds,
            )
            if overlap > 0:
                overlaps[turn.speaker] += overlap
        speaker = max(overlaps, key=overlaps.get) if overlaps else segment.speaker
        assigned.append(
            TranscriptSegment(
                start_seconds=segment.start_seconds,
                end_seconds=segment.end_seconds,
                speaker=speaker,
                text=segment.text,
                source_track=segment.source_track,
            )
        )
    return assigned
