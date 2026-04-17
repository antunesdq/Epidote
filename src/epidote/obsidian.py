from __future__ import annotations

from pathlib import Path

from epidote.models import MeetingAnalysis, MeetingMetadata, TranscriptDocument


class ObsidianExporter:
    def __init__(self, vault_dir: Path | None) -> None:
        self.vault_dir = vault_dir

    def is_configured(self) -> bool:
        return self.vault_dir is not None

    def export(
        self,
        metadata: MeetingMetadata,
        transcript: TranscriptDocument | None,
        analysis: MeetingAnalysis | None,
    ) -> Path | None:
        if not self.vault_dir:
            return None
        note_dir = self.vault_dir / "Meetings" / f"{metadata.created_at:%Y}"
        note_dir.mkdir(parents=True, exist_ok=True)
        note_path = note_dir / f"{metadata.created_at:%Y-%m-%d}_{slugify(metadata.title)}.md"
        note_path.write_text(
            render_note(metadata, transcript, analysis),
            encoding="utf-8",
        )
        return note_path


def render_note(
    metadata: MeetingMetadata,
    transcript: TranscriptDocument | None,
    analysis: MeetingAnalysis | None,
) -> str:
    lines = [
        "---",
        f"title: {metadata.title}",
        f"session_id: {metadata.session_id}",
        f"created_at: {metadata.created_at.isoformat()}",
        "tags:",
        "  - meetings",
        "  - epidote",
        "---",
        "",
        f"# {metadata.title}",
        "",
    ]
    if analysis:
        lines.extend(["## Summary", "", analysis.summary_markdown, ""])
        if analysis.decisions:
            lines.extend(["## Decisions", ""])
            lines.extend([f"- {item}" for item in analysis.decisions])
            lines.append("")
        if analysis.todos:
            lines.extend(["## Action Items", ""])
            for item in analysis.todos:
                due_suffix = f" (due {item.due_date.isoformat()})" if item.due_date else ""
                lines.append(f"- [ ] {item.title}{due_suffix}")
                if item.description:
                    lines.append(f"  {item.description}")
            lines.append("")
        if analysis.architecture_suggestions:
            lines.extend(["## Architecture Suggestions", ""])
            lines.extend([f"- {item}" for item in analysis.architecture_suggestions])
            lines.append("")
    if transcript:
        lines.extend(["## Transcript", "", transcript.render_text(), ""])
    return "\n".join(lines).strip() + "\n"


def slugify(raw: str) -> str:
    reduced = "".join(character.lower() if character.isalnum() else "-" for character in raw)
    while "--" in reduced:
        reduced = reduced.replace("--", "-")
    return reduced.strip("-") or "meeting"
