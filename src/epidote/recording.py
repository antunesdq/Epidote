from __future__ import annotations

import json
import signal
import subprocess
from dataclasses import dataclass
from pathlib import Path

from epidote.config import AppConfig
from epidote.models import MeetingMetadata


@dataclass(slots=True)
class RecordingHandle:
    metadata: MeetingMetadata
    process: subprocess.Popen[str]


class RecorderBridge:
    def __init__(self, config: AppConfig) -> None:
        self.config = config
        self._active: RecordingHandle | None = None

    def start_recording(self, metadata: MeetingMetadata, session_dir: Path) -> MeetingMetadata:
        self.ensure_binary()
        if self._active is not None:
            raise RuntimeError("A recording session is already active.")
        command = [
            str(self.config.recorder_binary_path),
            "--output-dir",
            str(session_dir),
            "--title",
            metadata.title,
        ]
        process = subprocess.Popen(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            start_new_session=True,
        )
        self._active = RecordingHandle(metadata=metadata, process=process)
        return metadata

    def stop_recording(self) -> MeetingMetadata:
        if self._active is None:
            raise RuntimeError("No active recording session.")
        handle = self._active
        process = handle.process
        process.send_signal(signal.SIGINT)
        stdout, stderr = process.communicate(timeout=30)
        if process.returncode not in (0, None):
            raise RuntimeError(stderr.strip() or stdout.strip() or "Recorder failed to stop.")
        self._active = None
        return handle.metadata

    def ensure_binary(self) -> None:
        if self.config.recorder_binary_path.exists():
            return
        subprocess.run(
            [
                "swift",
                "build",
                "--package-path",
                str(self.config.recorder_package_dir),
                "-c",
                "release",
            ],
            check=True,
        )
        if not self.config.recorder_binary_path.exists():
            raise RuntimeError("Recorder helper build succeeded but binary was not found.")

    @property
    def is_recording(self) -> bool:
        return self._active is not None
