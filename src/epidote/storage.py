from __future__ import annotations

import json
from pathlib import Path

from epidote.config import AppConfig
from epidote.models import (
    MeetingAnalysis,
    MeetingMetadata,
    TranscriptDocument,
    format_timestamp,
    new_id,
    utc_now,
)


class MeetingRepository:
    def __init__(self, config: AppConfig) -> None:
        self.config = config
        self.config.ensure_directories()

    def list_meetings(self) -> list[MeetingMetadata]:
        meetings: list[MeetingMetadata] = []
        for session_dir in sorted(self.config.meetings_dir.glob("*"), reverse=True):
            metadata_path = session_dir / "metadata.json"
            if not metadata_path.exists():
                continue
            meetings.append(self.load_metadata(session_dir.name))
        meetings.sort(key=lambda item: item.created_at, reverse=True)
        return meetings

    def create_session(self, title: str) -> MeetingMetadata:
        session_id = new_id("meeting")
        now = utc_now()
        metadata = MeetingMetadata(
            session_id=session_id,
            title=title or "Untitled meeting",
            created_at=now,
            updated_at=now,
            status="recording",
        )
        session_dir = self.session_dir(session_id)
        session_dir.mkdir(parents=True, exist_ok=True)
        self.save_metadata(metadata)
        return metadata

    def session_dir(self, session_id: str) -> Path:
        return self.config.meetings_dir / session_id

    def metadata_path(self, session_id: str) -> Path:
        return self.session_dir(session_id) / "metadata.json"

    def transcript_path(self, session_id: str) -> Path:
        return self.session_dir(session_id) / "transcript.json"

    def transcript_markdown_path(self, session_id: str) -> Path:
        return self.session_dir(session_id) / "transcript.md"

    def analysis_path(self, session_id: str) -> Path:
        return self.session_dir(session_id) / "analysis.json"

    def summary_markdown_path(self, session_id: str) -> Path:
        return self.session_dir(session_id) / "summary.md"

    def todos_path(self, session_id: str) -> Path:
        return self.session_dir(session_id) / "todos.json"

    def load_metadata(self, session_id: str) -> MeetingMetadata:
        payload = json.loads(self.metadata_path(session_id).read_text(encoding="utf-8"))
        return MeetingMetadata.from_dict(payload)

    def save_metadata(self, metadata: MeetingMetadata) -> None:
        metadata.updated_at = utc_now()
        self.metadata_path(metadata.session_id).write_text(
            json.dumps(metadata.to_dict(), indent=2),
            encoding="utf-8",
        )

    def load_transcript(self, session_id: str) -> TranscriptDocument | None:
        path = self.transcript_path(session_id)
        if not path.exists():
            return None
        payload = json.loads(path.read_text(encoding="utf-8"))
        return TranscriptDocument.from_dict(payload)

    def save_transcript(self, transcript: TranscriptDocument) -> None:
        self.transcript_path(transcript.session_id).write_text(
            json.dumps(transcript.to_dict(), indent=2),
            encoding="utf-8",
        )
        self.transcript_markdown_path(transcript.session_id).write_text(
            render_transcript_markdown(transcript),
            encoding="utf-8",
        )

    def load_analysis(self, session_id: str) -> MeetingAnalysis | None:
        path = self.analysis_path(session_id)
        if not path.exists():
            return None
        payload = json.loads(path.read_text(encoding="utf-8"))
        return MeetingAnalysis.from_dict(payload)

    def save_analysis(self, analysis: MeetingAnalysis) -> None:
        self.analysis_path(analysis.session_id).write_text(
            json.dumps(analysis.to_dict(), indent=2),
            encoding="utf-8",
        )
        self.summary_markdown_path(analysis.session_id).write_text(
            analysis.summary_markdown,
            encoding="utf-8",
        )
        self.todos_path(analysis.session_id).write_text(
            json.dumps([item.to_dict() for item in analysis.todos], indent=2),
            encoding="utf-8",
        )

    def export_session(
        self,
        metadata: MeetingMetadata,
        transcript: TranscriptDocument | None,
        analysis: MeetingAnalysis | None,
    ) -> Path:
        safe_title = slugify(metadata.title)
        export_dir = self.config.exports_dir / f"{metadata.created_at:%Y-%m-%d}_{safe_title}"
        export_dir.mkdir(parents=True, exist_ok=True)
        self.save_metadata(metadata)
        source_dir = self.session_dir(metadata.session_id)
        for filename in [
            "metadata.json",
            "transcript.json",
            "transcript.md",
            "analysis.json",
            "summary.md",
            "todos.json",
        ]:
            source_path = source_dir / filename
            if source_path.exists():
                target_path = export_dir / filename
                target_path.write_text(source_path.read_text(encoding="utf-8"), encoding="utf-8")
        return export_dir


def slugify(raw: str) -> str:
    reduced = "".join(character.lower() if character.isalnum() else "-" for character in raw)
    while "--" in reduced:
        reduced = reduced.replace("--", "-")
    return reduced.strip("-") or "meeting"


def render_transcript_markdown(transcript: TranscriptDocument) -> str:
    lines = ["# Transcript", ""]
    for segment in transcript.segments:
        lines.append(
            f"- `{format_timestamp(segment.start_seconds)} - "
            f"{format_timestamp(segment.end_seconds)}` "
            f"**{segment.speaker}** ({segment.source_track}): {segment.text}"
        )
    return "\n".join(lines) + "\n"
