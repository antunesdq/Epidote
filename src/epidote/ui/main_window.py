from __future__ import annotations

import traceback
from pathlib import Path

from PySide6.QtCore import QThread, Qt, Signal
from PySide6.QtWidgets import (
    QFormLayout,
    QHBoxLayout,
    QLabel,
    QLineEdit,
    QListWidget,
    QListWidgetItem,
    QMainWindow,
    QMessageBox,
    QPushButton,
    QPlainTextEdit,
    QSplitter,
    QStatusBar,
    QTableWidget,
    QTableWidgetItem,
    QTabWidget,
    QTextEdit,
    QVBoxLayout,
    QWidget,
)

from epidote.config import AppConfig, save_config
from epidote.models import MeetingAnalysis, MeetingMetadata, TodoItem, TranscriptDocument
from epidote.pipeline import MeetingPipeline
from epidote.storage import MeetingRepository


class CallableWorker(QThread):
    succeeded = Signal(object)
    failed = Signal(str)

    def __init__(self, target, *args, **kwargs) -> None:
        super().__init__()
        self._target = target
        self._args = args
        self._kwargs = kwargs

    def run(self) -> None:
        try:
            result = self._target(*self._args, **self._kwargs)
        except Exception:
            self.failed.emit(traceback.format_exc())
            return
        self.succeeded.emit(result)


class MainWindow(QMainWindow):
    def __init__(
        self,
        config: AppConfig,
        repository: MeetingRepository,
        pipeline: MeetingPipeline,
    ) -> None:
        super().__init__()
        self.config = config
        self.repository = repository
        self.pipeline = pipeline
        self._workers: list[CallableWorker] = []
        self._loaded_todos: list[TodoItem] = []

        self.setWindowTitle("Epidote")
        self.resize(1380, 860)

        self.status_bar = QStatusBar()
        self.setStatusBar(self.status_bar)

        self.tabs = QTabWidget()
        self.setCentralWidget(self.tabs)

        self.meeting_tab = QWidget()
        self.settings_tab = QWidget()
        self.tabs.addTab(self.meeting_tab, "Meetings")
        self.tabs.addTab(self.settings_tab, "Settings")

        self._build_meeting_tab()
        self._build_settings_tab()
        self._load_settings_into_form()
        self.refresh_meeting_list()
        self._update_buttons()

    def _build_meeting_tab(self) -> None:
        layout = QVBoxLayout(self.meeting_tab)

        controls = QHBoxLayout()
        self.title_input = QLineEdit()
        self.title_input.setPlaceholderText("Meeting title")
        self.start_button = QPushButton("Start Recording")
        self.stop_button = QPushButton("Stop + Process")
        self.process_button = QPushButton("Process Selected")
        self.refresh_button = QPushButton("Refresh")
        controls.addWidget(QLabel("Title"))
        controls.addWidget(self.title_input, stretch=1)
        controls.addWidget(self.start_button)
        controls.addWidget(self.stop_button)
        controls.addWidget(self.process_button)
        controls.addWidget(self.refresh_button)
        layout.addLayout(controls)

        self.start_button.clicked.connect(self.on_start_clicked)
        self.stop_button.clicked.connect(self.on_stop_clicked)
        self.process_button.clicked.connect(self.on_process_clicked)
        self.refresh_button.clicked.connect(self.refresh_meeting_list)

        main_splitter = QSplitter(Qt.Orientation.Horizontal)
        layout.addWidget(main_splitter, stretch=1)

        self.meeting_list = QListWidget()
        self.meeting_list.currentItemChanged.connect(self.on_meeting_selected)
        main_splitter.addWidget(self.meeting_list)

        detail_widget = QWidget()
        detail_layout = QVBoxLayout(detail_widget)
        main_splitter.addWidget(detail_widget)
        main_splitter.setStretchFactor(0, 0)
        main_splitter.setStretchFactor(1, 1)

        self.detail_tabs = QTabWidget()
        detail_layout.addWidget(self.detail_tabs)

        transcript_tab = QWidget()
        transcript_layout = QVBoxLayout(transcript_tab)
        self.transcript_view = QPlainTextEdit()
        self.transcript_view.setReadOnly(True)
        transcript_layout.addWidget(self.transcript_view)
        self.detail_tabs.addTab(transcript_tab, "Transcript")

        summary_tab = QWidget()
        summary_layout = QVBoxLayout(summary_tab)
        self.summary_view = QTextEdit()
        self.summary_view.setReadOnly(True)
        summary_layout.addWidget(self.summary_view)
        self.detail_tabs.addTab(summary_tab, "Summary")

        todos_tab = QWidget()
        todos_layout = QVBoxLayout(todos_tab)
        todo_splitter = QSplitter(Qt.Orientation.Vertical)
        todos_layout.addWidget(todo_splitter)
        self.todo_table = QTableWidget(0, 4)
        self.todo_table.setHorizontalHeaderLabels(["Title", "Due", "Owner", "Calendar"])
        self.todo_table.currentCellChanged.connect(self.on_todo_selected)
        self.todo_detail = QTextEdit()
        self.todo_detail.setReadOnly(True)
        todo_splitter.addWidget(self.todo_table)
        todo_splitter.addWidget(self.todo_detail)
        todo_splitter.setStretchFactor(0, 1)
        todo_splitter.setStretchFactor(1, 1)
        self.detail_tabs.addTab(todos_tab, "ToDos")

    def _build_settings_tab(self) -> None:
        layout = QVBoxLayout(self.settings_tab)
        form = QFormLayout()
        layout.addLayout(form)

        self.anthropic_key_input = QLineEdit()
        self.anthropic_model_input = QLineEdit()
        self.whisper_model_input = QLineEdit()
        self.hf_token_input = QLineEdit()
        self.google_secret_input = QLineEdit()
        self.google_calendar_input = QLineEdit()
        self.obsidian_vault_input = QLineEdit()
        self.exports_dir_input = QLineEdit()

        self.anthropic_key_input.setEchoMode(QLineEdit.EchoMode.Password)
        self.hf_token_input.setEchoMode(QLineEdit.EchoMode.Password)

        form.addRow("Anthropic API key", self.anthropic_key_input)
        form.addRow("Anthropic model", self.anthropic_model_input)
        form.addRow("Whisper model", self.whisper_model_input)
        form.addRow("Hugging Face token", self.hf_token_input)
        form.addRow("Google client secret", self.google_secret_input)
        form.addRow("Google calendar id", self.google_calendar_input)
        form.addRow("Obsidian vault", self.obsidian_vault_input)
        form.addRow("Export folder", self.exports_dir_input)

        self.save_settings_button = QPushButton("Save Settings")
        self.save_settings_button.clicked.connect(self.on_save_settings)
        layout.addWidget(self.save_settings_button)
        layout.addStretch(1)

    def _load_settings_into_form(self) -> None:
        self.anthropic_key_input.setText(self.config.anthropic_api_key)
        self.anthropic_model_input.setText(self.config.anthropic_model)
        self.whisper_model_input.setText(self.config.whisper_model)
        self.hf_token_input.setText(self.config.hf_token)
        self.google_secret_input.setText(str(self.config.google_client_secret_path or ""))
        self.google_calendar_input.setText(self.config.google_calendar_id)
        self.obsidian_vault_input.setText(str(self.config.obsidian_vault_dir or ""))
        self.exports_dir_input.setText(str(self.config.exports_dir))

    def _apply_form_to_config(self) -> None:
        self.config.anthropic_api_key = self.anthropic_key_input.text().strip()
        self.config.anthropic_model = self.anthropic_model_input.text().strip()
        self.config.whisper_model = self.whisper_model_input.text().strip() or "small"
        self.config.hf_token = self.hf_token_input.text().strip()
        self.config.google_client_secret_path = self._optional_path(self.google_secret_input.text())
        self.config.google_calendar_id = self.google_calendar_input.text().strip() or "primary"
        self.config.obsidian_vault_dir = self._optional_path(self.obsidian_vault_input.text())
        self.config.exports_dir = Path(self.exports_dir_input.text().strip()).expanduser()
        self.config.ensure_directories()

    def _rebuild_runtime(self) -> None:
        self.repository = MeetingRepository(self.config)
        self.pipeline = MeetingPipeline(self.config, self.repository)

    def on_save_settings(self) -> None:
        if self.pipeline.recorder.is_recording:
            self._show_error("Stop the active recording before changing settings.")
            return
        self._apply_form_to_config()
        save_config(self.config)
        self._rebuild_runtime()
        self.status_bar.showMessage("Settings saved.", 5000)
        self.refresh_meeting_list()

    def on_start_clicked(self) -> None:
        self._apply_form_to_config()
        save_config(self.config)
        self._rebuild_runtime()
        title = self.title_input.text().strip() or "Untitled meeting"
        self.status_bar.showMessage("Starting recorder...")
        self._run_in_background(self.pipeline.start_recording, title, success=self._on_recording_started)

    def on_stop_clicked(self) -> None:
        self.status_bar.showMessage("Stopping recorder and processing audio...")
        self._run_in_background(
            self.pipeline.stop_recording_and_process,
            success=self._on_processing_finished,
        )

    def on_process_clicked(self) -> None:
        item = self.meeting_list.currentItem()
        if item is None:
            self._show_error("Select a meeting to process.")
            return
        session_id = item.data(Qt.ItemDataRole.UserRole)
        self.status_bar.showMessage("Processing selected meeting...")
        self._run_in_background(
            self.pipeline.process_session,
            session_id,
            success=self._on_processing_finished,
        )

    def on_meeting_selected(self, current: QListWidgetItem | None, _previous: QListWidgetItem | None) -> None:
        if current is None:
            self.transcript_view.clear()
            self.summary_view.clear()
            self.todo_table.setRowCount(0)
            self.todo_detail.clear()
            self._update_buttons()
            return
        session_id = current.data(Qt.ItemDataRole.UserRole)
        transcript = self.repository.load_transcript(session_id)
        analysis = self.repository.load_analysis(session_id)
        self._render_transcript(transcript)
        self._render_analysis(analysis)
        self._update_buttons()

    def on_todo_selected(self, row: int, _column: int, _previous_row: int, _previous_column: int) -> None:
        if row < 0 or row >= len(self._loaded_todos):
            self.todo_detail.clear()
            return
        todo = self._loaded_todos[row]
        due_text = todo.due_date.isoformat() if todo.due_date else "Not set"
        reminder_text = todo.reminder_at.isoformat() if todo.reminder_at else "Not set"
        body = (
            f"## {todo.title}\n\n"
            f"**Owner:** {todo.owner}\n\n"
            f"**Due:** {due_text}\n\n"
            f"**Reminder:** {reminder_text}\n\n"
            f"**Description**\n{todo.description or 'No description'}\n\n"
            f"**Architecture notes**\n{todo.architecture_notes or 'None'}\n\n"
            f"**Code suggestion**\n{todo.code_suggestion or 'None'}\n\n"
            f"**Source**\n{todo.source_quote or 'None'}"
        )
        self.todo_detail.setMarkdown(body)

    def refresh_meeting_list(self) -> None:
        selected_session_id = None
        if self.meeting_list.currentItem():
            selected_session_id = self.meeting_list.currentItem().data(Qt.ItemDataRole.UserRole)
        self.meeting_list.clear()
        restored_item = None
        for meeting in self.repository.list_meetings():
            item = QListWidgetItem(
                f"{meeting.created_at:%Y-%m-%d %H:%M}  {meeting.title}  [{meeting.status}]"
            )
            item.setData(Qt.ItemDataRole.UserRole, meeting.session_id)
            self.meeting_list.addItem(item)
            if meeting.session_id == selected_session_id:
                restored_item = item
        if restored_item is not None:
            self.meeting_list.setCurrentItem(restored_item)
        elif self.meeting_list.count() > 0:
            self.meeting_list.setCurrentRow(0)
        self._update_buttons()

    def _render_transcript(self, transcript: TranscriptDocument | None) -> None:
        self.transcript_view.setPlainText(transcript.render_text() if transcript else "")

    def _render_analysis(self, analysis: MeetingAnalysis | None) -> None:
        if analysis is None:
            self.summary_view.clear()
            self.todo_table.setRowCount(0)
            self.todo_detail.clear()
            self._loaded_todos = []
            return
        extra_sections: list[str] = []
        if analysis.decisions:
            extra_sections.append("## Decisions\n\n" + "\n".join(f"- {item}" for item in analysis.decisions))
        if analysis.architecture_suggestions:
            extra_sections.append(
                "## Architecture Suggestions\n\n"
                + "\n".join(f"- {item}" for item in analysis.architecture_suggestions)
            )
        summary = analysis.summary_markdown
        if extra_sections:
            summary = summary.rstrip() + "\n\n" + "\n\n".join(extra_sections)
        self.summary_view.setMarkdown(summary)
        self._loaded_todos = list(analysis.todos)
        self.todo_table.setRowCount(len(self._loaded_todos))
        for row, todo in enumerate(self._loaded_todos):
            self.todo_table.setItem(row, 0, QTableWidgetItem(todo.title))
            self.todo_table.setItem(
                row,
                1,
                QTableWidgetItem(todo.due_date.isoformat() if todo.due_date else ""),
            )
            self.todo_table.setItem(row, 2, QTableWidgetItem(todo.owner))
            self.todo_table.setItem(
                row,
                3,
                QTableWidgetItem("Yes" if todo.calendar_event_id else "No"),
            )
        if self._loaded_todos:
            self.todo_table.selectRow(0)
            self.on_todo_selected(0, 0, -1, -1)
        else:
            self.todo_detail.clear()

    def _run_in_background(self, target, *args, success) -> None:
        worker = CallableWorker(target, *args)
        worker.succeeded.connect(success)
        worker.failed.connect(self._on_worker_failed)
        worker.finished.connect(lambda: self._cleanup_worker(worker))
        self._workers.append(worker)
        self._update_buttons(disable_all=True)
        worker.start()

    def _cleanup_worker(self, worker: CallableWorker) -> None:
        self._workers = [item for item in self._workers if item is not worker]
        self._update_buttons()

    def _on_recording_started(self, metadata: MeetingMetadata) -> None:
        self.status_bar.showMessage(f"Recording: {metadata.title}", 5000)
        self.refresh_meeting_list()

    def _on_processing_finished(self, _result) -> None:
        self.status_bar.showMessage("Meeting processed.", 5000)
        self.refresh_meeting_list()

    def _on_worker_failed(self, message: str) -> None:
        self._show_error(message)
        self.status_bar.clearMessage()

    def _update_buttons(self, disable_all: bool = False) -> None:
        if disable_all:
            self.start_button.setEnabled(False)
            self.stop_button.setEnabled(False)
            self.process_button.setEnabled(False)
            self.save_settings_button.setEnabled(False)
            return
        recording = self.pipeline.recorder.is_recording
        has_selection = self.meeting_list.currentItem() is not None
        self.start_button.setEnabled(not recording)
        self.stop_button.setEnabled(recording)
        self.process_button.setEnabled(not recording and has_selection)
        self.save_settings_button.setEnabled(not recording)

    def _show_error(self, message: str) -> None:
        dialog = QMessageBox(self)
        dialog.setIcon(QMessageBox.Icon.Critical)
        dialog.setWindowTitle("Epidote")
        dialog.setText(message)
        dialog.exec()
        self._update_buttons()

    @staticmethod
    def _optional_path(raw: str) -> Path | None:
        value = raw.strip()
        return Path(value).expanduser() if value else None
