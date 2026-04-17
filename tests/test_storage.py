from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from epidote.config import AppConfig
from epidote.models import TranscriptDocument, TranscriptSegment, utc_now
from epidote.storage import MeetingRepository


def build_config(tmp_path: Path) -> AppConfig:
    return AppConfig(
        app_home=tmp_path / "home",
        meetings_dir=tmp_path / "home" / "meetings",
        exports_dir=tmp_path / "exports",
        settings_path=tmp_path / "home" / "settings.json",
        recorder_package_dir=tmp_path / "native",
        recorder_binary_path=tmp_path / "native" / "bin",
    )


class StorageTests(unittest.TestCase):
    def test_repository_session_and_transcript_roundtrip(self) -> None:
        with tempfile.TemporaryDirectory() as raw_directory:
            tmp_path = Path(raw_directory)
            config = build_config(tmp_path)
            repository = MeetingRepository(config)
            metadata = repository.create_session("Architecture sync")
            transcript = TranscriptDocument(
                session_id=metadata.session_id,
                created_at=utc_now(),
                speakers=["speaker_1", "speaker_2"],
                segments=[
                    TranscriptSegment(
                        start_seconds=0,
                        end_seconds=3,
                        speaker="speaker_1",
                        text="Let's do it.",
                        source_track="microphone",
                    )
                ],
            )
            repository.save_transcript(transcript)

            restored = repository.load_transcript(metadata.session_id)
            self.assertIsNotNone(restored)
            self.assertEqual(restored.segments[0].text, "Let's do it.")
            self.assertTrue(repository.transcript_markdown_path(metadata.session_id).exists())


if __name__ == "__main__":
    unittest.main()
