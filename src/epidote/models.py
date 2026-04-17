from __future__ import annotations

from dataclasses import asdict, dataclass, field, fields, is_dataclass
from datetime import UTC, date, datetime
from pathlib import Path
from typing import Any
from uuid import uuid4


def utc_now() -> datetime:
    return datetime.now(tz=UTC)


def new_id(prefix: str) -> str:
    return f"{prefix}_{uuid4().hex[:10]}"


def _serialize(value: Any) -> Any:
    if is_dataclass(value):
        return {key: _serialize(inner) for key, inner in asdict(value).items()}
    if isinstance(value, datetime):
        return value.isoformat()
    if isinstance(value, date):
        return value.isoformat()
    if isinstance(value, Path):
        return str(value)
    if isinstance(value, list):
        return [_serialize(item) for item in value]
    if isinstance(value, dict):
        return {key: _serialize(inner) for key, inner in value.items()}
    return value


def _read_optional_datetime(raw: str | None) -> datetime | None:
    if not raw:
        return None
    return datetime.fromisoformat(raw)


def _read_optional_date(raw: str | None) -> date | None:
    if not raw:
        return None
    return date.fromisoformat(raw)


@dataclass(slots=True)
class MeetingMetadata:
    session_id: str
    title: str
    created_at: datetime
    updated_at: datetime
    status: str = "idle"

    def to_dict(self) -> dict[str, Any]:
        return _serialize(self)

    @classmethod
    def from_dict(cls, payload: dict[str, Any]) -> "MeetingMetadata":
        return cls(
            session_id=payload["session_id"],
            title=payload["title"],
            created_at=datetime.fromisoformat(payload["created_at"]),
            updated_at=datetime.fromisoformat(payload["updated_at"]),
            status=payload.get("status", "idle"),
        )


@dataclass(slots=True)
class TranscriptSegment:
    start_seconds: float
    end_seconds: float
    speaker: str
    text: str
    source_track: str

    def to_dict(self) -> dict[str, Any]:
        return _serialize(self)

    @classmethod
    def from_dict(cls, payload: dict[str, Any]) -> "TranscriptSegment":
        return cls(
            start_seconds=float(payload["start_seconds"]),
            end_seconds=float(payload["end_seconds"]),
            speaker=payload["speaker"],
            text=payload["text"],
            source_track=payload.get("source_track", "unknown"),
        )


@dataclass(slots=True)
class TranscriptDocument:
    session_id: str
    created_at: datetime
    speakers: list[str] = field(default_factory=list)
    segments: list[TranscriptSegment] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return _serialize(self)

    @classmethod
    def from_dict(cls, payload: dict[str, Any]) -> "TranscriptDocument":
        return cls(
            session_id=payload["session_id"],
            created_at=datetime.fromisoformat(payload["created_at"]),
            speakers=list(payload.get("speakers", [])),
            segments=[
                TranscriptSegment.from_dict(segment)
                for segment in payload.get("segments", [])
            ],
        )

    def render_text(self) -> str:
        lines: list[str] = []
        for segment in self.segments:
            lines.append(
                f"[{format_timestamp(segment.start_seconds)} - "
                f"{format_timestamp(segment.end_seconds)}] "
                f"{segment.speaker}: {segment.text}"
            )
        return "\n".join(lines)


@dataclass(slots=True)
class TodoItem:
    item_id: str
    title: str
    description: str
    owner: str
    due_date: date | None = None
    reminder_at: datetime | None = None
    architecture_notes: str = ""
    code_suggestion: str = ""
    source_quote: str = ""
    status: str = "open"
    calendar_event_id: str | None = None

    @classmethod
    def create(
        cls,
        title: str,
        description: str,
        owner: str = "unknown",
        due_date: date | None = None,
        reminder_at: datetime | None = None,
        architecture_notes: str = "",
        code_suggestion: str = "",
        source_quote: str = "",
    ) -> "TodoItem":
        return cls(
            item_id=new_id("todo"),
            title=title,
            description=description,
            owner=owner,
            due_date=due_date,
            reminder_at=reminder_at,
            architecture_notes=architecture_notes,
            code_suggestion=code_suggestion,
            source_quote=source_quote,
        )

    def to_dict(self) -> dict[str, Any]:
        return _serialize(self)

    @classmethod
    def from_dict(cls, payload: dict[str, Any]) -> "TodoItem":
        return cls(
            item_id=payload["item_id"],
            title=payload["title"],
            description=payload["description"],
            owner=payload.get("owner", "unknown"),
            due_date=_read_optional_date(payload.get("due_date")),
            reminder_at=_read_optional_datetime(payload.get("reminder_at")),
            architecture_notes=payload.get("architecture_notes", ""),
            code_suggestion=payload.get("code_suggestion", ""),
            source_quote=payload.get("source_quote", ""),
            status=payload.get("status", "open"),
            calendar_event_id=payload.get("calendar_event_id"),
        )


@dataclass(slots=True)
class MeetingAnalysis:
    session_id: str
    summary_markdown: str
    decisions: list[str] = field(default_factory=list)
    architecture_suggestions: list[str] = field(default_factory=list)
    todos: list[TodoItem] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return _serialize(self)

    @classmethod
    def from_dict(cls, payload: dict[str, Any]) -> "MeetingAnalysis":
        return cls(
            session_id=payload["session_id"],
            summary_markdown=payload.get("summary_markdown", ""),
            decisions=list(payload.get("decisions", [])),
            architecture_suggestions=list(payload.get("architecture_suggestions", [])),
            todos=[TodoItem.from_dict(item) for item in payload.get("todos", [])],
        )


@dataclass(slots=True)
class ProcessingResult:
    metadata: MeetingMetadata
    transcript: TranscriptDocument | None
    analysis: MeetingAnalysis | None
    exported_note_path: Path | None = None


def dataclass_from_dict(cls: type[Any], payload: dict[str, Any]) -> Any:
    values: dict[str, Any] = {}
    for item in fields(cls):
        values[item.name] = payload.get(item.name)
    return cls(**values)


def format_timestamp(value: float) -> str:
    total_seconds = int(max(0, value))
    hours, remainder = divmod(total_seconds, 3600)
    minutes, seconds = divmod(remainder, 60)
    return f"{hours:02d}:{minutes:02d}:{seconds:02d}"
