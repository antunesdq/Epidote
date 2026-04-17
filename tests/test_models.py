from __future__ import annotations

import unittest
from datetime import UTC, date, datetime

from epidote.models import MeetingAnalysis, TodoItem, TranscriptDocument, TranscriptSegment


class ModelTests(unittest.TestCase):
    def test_transcript_render_text(self) -> None:
        transcript = TranscriptDocument(
            session_id="meeting_123",
            created_at=datetime(2026, 4, 17, 20, 0, tzinfo=UTC),
            speakers=["speaker_1"],
            segments=[
                TranscriptSegment(
                    start_seconds=0,
                    end_seconds=4,
                    speaker="speaker_1",
                    text="Hello there",
                    source_track="microphone",
                )
            ],
        )
        rendered = transcript.render_text()
        self.assertIn("speaker_1: Hello there", rendered)
        self.assertIn("[00:00:00 - 00:00:04]", rendered)

    def test_meeting_analysis_roundtrip(self) -> None:
        analysis = MeetingAnalysis(
            session_id="meeting_123",
            summary_markdown="Summary",
            todos=[
                TodoItem.create(
                    title="Ship feature",
                    description="Implement the thing",
                    owner="me",
                    due_date=date(2026, 4, 20),
                )
            ],
        )
        payload = analysis.to_dict()
        restored = MeetingAnalysis.from_dict(payload)
        self.assertEqual(restored.todos[0].title, "Ship feature")
        self.assertEqual(restored.todos[0].due_date, date(2026, 4, 20))


if __name__ == "__main__":
    unittest.main()
