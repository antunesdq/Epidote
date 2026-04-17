from __future__ import annotations

import json
from datetime import date, datetime
from textwrap import dedent

from epidote.models import MeetingAnalysis, TodoItem, TranscriptDocument


class ClaudeMeetingAnalyst:
    def __init__(self, api_key: str, model: str) -> None:
        self.api_key = api_key
        self.model = model

    def is_configured(self) -> bool:
        return bool(self.api_key and self.model)

    def analyse(self, transcript: TranscriptDocument, title: str) -> MeetingAnalysis:
        if not self.is_configured():
            raise RuntimeError("Claude is not configured. Set API key and model first.")
        import httpx

        response = httpx.post(
            "https://api.anthropic.com/v1/messages",
            headers={
                "x-api-key": self.api_key,
                "anthropic-version": "2023-06-01",
                "content-type": "application/json",
            },
            json={
                "model": self.model,
                "max_tokens": 4096,
                "messages": [
                    {
                        "role": "user",
                        "content": build_prompt(title=title, transcript=transcript.render_text()),
                    }
                ],
            },
            timeout=180.0,
        )
        response.raise_for_status()
        payload = response.json()
        text_blocks = [
            block.get("text", "")
            for block in payload.get("content", [])
            if block.get("type") == "text"
        ]
        parsed = self._extract_json("".join(text_blocks))
        todos = [todo_from_payload(item) for item in parsed.get("todos", [])]
        return MeetingAnalysis(
            session_id=transcript.session_id,
            summary_markdown=parsed.get("summary_markdown", "").strip(),
            decisions=[str(item).strip() for item in parsed.get("decisions", []) if str(item).strip()],
            architecture_suggestions=[
                str(item).strip()
                for item in parsed.get("architecture_suggestions", [])
                if str(item).strip()
            ],
            todos=todos,
        )

    def _extract_json(self, raw: str) -> dict[str, object]:
        stripped = raw.strip()
        if stripped.startswith("{"):
            return json.loads(stripped)
        start = stripped.find("{")
        end = stripped.rfind("}")
        if start == -1 or end == -1 or end <= start:
            raise ValueError("Claude response did not include a JSON object.")
        return json.loads(stripped[start : end + 1])


def build_prompt(title: str, transcript: str) -> str:
    return dedent(
        f"""
        You are analysing a meeting transcript for a desktop app.

        Meeting title: {title}

        Return only valid JSON with this shape:
        {{
          "summary_markdown": "short markdown summary",
          "decisions": ["decision 1", "decision 2"],
          "architecture_suggestions": [
            "high level architecture guidance when the meeting mentions building software"
          ],
          "todos": [
            {{
              "title": "short task",
              "description": "what needs to happen and why",
              "owner": "me | team | unknown",
              "due_date": "YYYY-MM-DD or null",
              "reminder_at": "ISO-8601 timestamp or null",
              "architecture_notes": "specific architecture notes relevant to this task",
              "code_suggestion": "suggested code or implementation outline when relevant",
              "source_quote": "brief quote or paraphrase from the transcript"
            }}
          ]
        }}

        Rules:
        - If there are no action items, return an empty todos list.
        - Keep architecture_notes and code_suggestion concise but actionable.
        - Use null for unknown dates.
        - Use markdown only in summary_markdown and code_suggestion.
        - Do not include explanations outside the JSON object.

        Transcript:
        {transcript}
        """
    ).strip()


def todo_from_payload(payload: dict[str, object]) -> TodoItem:
    due_date = _read_date(payload.get("due_date"))
    reminder_at = _read_datetime(payload.get("reminder_at"))
    return TodoItem.create(
        title=str(payload.get("title", "Untitled task")).strip(),
        description=str(payload.get("description", "")).strip(),
        owner=str(payload.get("owner", "unknown")).strip() or "unknown",
        due_date=due_date,
        reminder_at=reminder_at,
        architecture_notes=str(payload.get("architecture_notes", "")).strip(),
        code_suggestion=str(payload.get("code_suggestion", "")).strip(),
        source_quote=str(payload.get("source_quote", "")).strip(),
    )


def _read_date(raw: object) -> date | None:
    if not raw or raw == "null":
        return None
    return date.fromisoformat(str(raw))


def _read_datetime(raw: object) -> datetime | None:
    if not raw or raw == "null":
        return None
    return datetime.fromisoformat(str(raw))
