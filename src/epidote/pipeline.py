from __future__ import annotations

from pathlib import Path

from epidote.calendar import GoogleCalendarSync
from epidote.claude import ClaudeMeetingAnalyst
from epidote.config import AppConfig
from epidote.models import MeetingAnalysis, MeetingMetadata, ProcessingResult
from epidote.obsidian import ObsidianExporter
from epidote.recording import RecorderBridge
from epidote.storage import MeetingRepository
from epidote.transcription import MeetingTranscriptionService


class MeetingPipeline:
    def __init__(self, config: AppConfig, repository: MeetingRepository) -> None:
        self.config = config
        self.repository = repository
        self.recorder = RecorderBridge(config)
        self.transcriber = MeetingTranscriptionService(
            whisper_model=config.whisper_model,
            hf_token=config.hf_token,
        )
        self.analyst = ClaudeMeetingAnalyst(
            api_key=config.anthropic_api_key,
            model=config.anthropic_model,
        )
        self.calendar = GoogleCalendarSync(
            client_secret_path=config.google_client_secret_path,
            token_path=config.google_token_path,
            calendar_id=config.google_calendar_id,
        )
        self.obsidian = ObsidianExporter(config.obsidian_vault_dir)
        self._current_metadata: MeetingMetadata | None = None

    def start_recording(self, title: str) -> MeetingMetadata:
        metadata = self.repository.create_session(title=title)
        self.recorder.start_recording(metadata, self.repository.session_dir(metadata.session_id))
        self._current_metadata = metadata
        return metadata

    def stop_recording_and_process(self) -> ProcessingResult:
        metadata = self.recorder.stop_recording()
        self._current_metadata = None
        return self.process_session(metadata.session_id)

    def process_session(self, session_id: str) -> ProcessingResult:
        metadata = self.repository.load_metadata(session_id)
        metadata.status = "processing"
        self.repository.save_metadata(metadata)
        session_dir = self.repository.session_dir(session_id)
        transcript = self.transcriber.transcribe_session(session_id=session_id, session_dir=session_dir)
        self.repository.save_transcript(transcript)
        analysis: MeetingAnalysis | None = None
        if self.analyst.is_configured():
            analysis = self.analyst.analyse(transcript=transcript, title=metadata.title)
            analysis.todos = self.calendar.sync_todos(metadata.title, analysis.todos)
            self.repository.save_analysis(analysis)
        metadata.status = "complete"
        self.repository.save_metadata(metadata)
        export_dir = self.repository.export_session(metadata, transcript, analysis)
        exported_note_path = self.obsidian.export(metadata, transcript, analysis)
        return ProcessingResult(
            metadata=metadata,
            transcript=transcript,
            analysis=analysis,
            exported_note_path=exported_note_path or export_dir,
        )

    @property
    def current_metadata(self) -> MeetingMetadata | None:
        return self._current_metadata
