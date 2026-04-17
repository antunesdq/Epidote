from __future__ import annotations

from datetime import timedelta
from pathlib import Path

from epidote.models import TodoItem


class GoogleCalendarSync:
    def __init__(
        self,
        client_secret_path: Path | None,
        token_path: Path | None,
        calendar_id: str,
    ) -> None:
        self.client_secret_path = client_secret_path
        self.token_path = token_path
        self.calendar_id = calendar_id

    def is_configured(self) -> bool:
        return bool(self.client_secret_path and self.token_path and self.calendar_id)

    def sync_todos(self, meeting_title: str, todos: list[TodoItem]) -> list[TodoItem]:
        if not self.is_configured():
            return todos
        service = self._build_service()
        synced: list[TodoItem] = []
        for item in todos:
            if item.calendar_event_id or (item.due_date is None and item.reminder_at is None):
                synced.append(item)
                continue
            event = build_event_payload(meeting_title, item)
            created = (
                service.events()
                .insert(calendarId=self.calendar_id, body=event)
                .execute()
            )
            item.calendar_event_id = created.get("id")
            synced.append(item)
        return synced

    def _build_service(self):
        from google.auth.transport.requests import Request
        from google.oauth2.credentials import Credentials
        from google_auth_oauthlib.flow import InstalledAppFlow
        from googleapiclient.discovery import build

        scopes = ["https://www.googleapis.com/auth/calendar"]
        credentials = None
        if self.token_path and self.token_path.exists():
            credentials = Credentials.from_authorized_user_file(str(self.token_path), scopes)
        if credentials and credentials.expired and credentials.refresh_token:
            credentials.refresh(Request())
        if not credentials or not credentials.valid:
            if not self.client_secret_path:
                raise RuntimeError("Google Calendar client secret path is missing.")
            flow = InstalledAppFlow.from_client_secrets_file(
                str(self.client_secret_path),
                scopes,
            )
            credentials = flow.run_local_server(port=0)
            if self.token_path:
                self.token_path.write_text(credentials.to_json(), encoding="utf-8")
        return build("calendar", "v3", credentials=credentials)


def build_event_payload(meeting_title: str, item: TodoItem) -> dict[str, object]:
    description_parts = [item.description]
    if item.architecture_notes:
        description_parts.append(f"Architecture:\n{item.architecture_notes}")
    if item.code_suggestion:
        description_parts.append(f"Code suggestion:\n{item.code_suggestion}")
    body: dict[str, object] = {
        "summary": f"[Epidote] {item.title}",
        "description": "\n\n".join(part for part in description_parts if part),
    }
    if item.reminder_at:
        end_time = item.reminder_at + timedelta(minutes=30)
        body["start"] = {"dateTime": item.reminder_at.isoformat()}
        body["end"] = {"dateTime": end_time.isoformat()}
    elif item.due_date:
        next_day = item.due_date + timedelta(days=1)
        body["start"] = {"date": item.due_date.isoformat()}
        body["end"] = {"date": next_day.isoformat()}
    return body
