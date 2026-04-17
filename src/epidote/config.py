from __future__ import annotations

import json
import os
from dataclasses import dataclass
from pathlib import Path


def project_root() -> Path:
    return Path(__file__).resolve().parents[2]


def default_app_home() -> Path:
    override = os.getenv("EPIDOTE_HOME", "").strip()
    if override:
        return Path(override).expanduser()
    return Path.home() / "Library" / "Application Support" / "Epidote"


def default_exports_dir() -> Path:
    override = os.getenv("EPIDOTE_EXPORTS_DIR", "").strip()
    if override:
        return Path(override).expanduser()
    return Path.home() / "Documents" / "Epidote" / "Exports"


@dataclass(slots=True)
class AppConfig:
    app_home: Path
    meetings_dir: Path
    exports_dir: Path
    settings_path: Path
    recorder_package_dir: Path
    recorder_binary_path: Path
    anthropic_api_key: str = ""
    anthropic_model: str = ""
    whisper_model: str = "small"
    hf_token: str = ""
    google_client_secret_path: Path | None = None
    google_token_path: Path | None = None
    google_calendar_id: str = "primary"
    obsidian_vault_dir: Path | None = None

    def ensure_directories(self) -> None:
        self.app_home.mkdir(parents=True, exist_ok=True)
        self.meetings_dir.mkdir(parents=True, exist_ok=True)
        self.exports_dir.mkdir(parents=True, exist_ok=True)

    def to_dict(self) -> dict[str, str]:
        return {
            "app_home": str(self.app_home),
            "meetings_dir": str(self.meetings_dir),
            "exports_dir": str(self.exports_dir),
            "settings_path": str(self.settings_path),
            "recorder_package_dir": str(self.recorder_package_dir),
            "recorder_binary_path": str(self.recorder_binary_path),
            "anthropic_api_key": self.anthropic_api_key,
            "anthropic_model": self.anthropic_model,
            "whisper_model": self.whisper_model,
            "hf_token": self.hf_token,
            "google_client_secret_path": (
                str(self.google_client_secret_path)
                if self.google_client_secret_path
                else ""
            ),
            "google_token_path": (
                str(self.google_token_path) if self.google_token_path else ""
            ),
            "google_calendar_id": self.google_calendar_id,
            "obsidian_vault_dir": (
                str(self.obsidian_vault_dir) if self.obsidian_vault_dir else ""
            ),
        }

    @classmethod
    def from_dict(cls, payload: dict[str, str]) -> "AppConfig":
        app_home = Path(payload["app_home"])
        meetings_dir = Path(payload["meetings_dir"])
        return cls(
            app_home=app_home,
            meetings_dir=meetings_dir,
            exports_dir=Path(payload["exports_dir"]),
            settings_path=Path(payload["settings_path"]),
            recorder_package_dir=Path(payload["recorder_package_dir"]),
            recorder_binary_path=Path(payload["recorder_binary_path"]),
            anthropic_api_key=payload.get("anthropic_api_key", ""),
            anthropic_model=payload.get("anthropic_model", ""),
            whisper_model=payload.get("whisper_model", "small"),
            hf_token=payload.get("hf_token", ""),
            google_client_secret_path=_optional_path(
                payload.get("google_client_secret_path", "")
            ),
            google_token_path=_optional_path(payload.get("google_token_path", "")),
            google_calendar_id=payload.get("google_calendar_id", "primary"),
            obsidian_vault_dir=_optional_path(payload.get("obsidian_vault_dir", "")),
        )


def _optional_path(raw: str) -> Path | None:
    return Path(raw).expanduser() if raw else None


def load_config() -> AppConfig:
    app_home = default_app_home()
    settings_path = app_home / "settings.json"
    repo_root = project_root()
    defaults = AppConfig(
        app_home=app_home,
        meetings_dir=app_home / "meetings",
        exports_dir=default_exports_dir(),
        settings_path=settings_path,
        recorder_package_dir=repo_root / "native" / "macos-recorder",
        recorder_binary_path=repo_root
        / "native"
        / "macos-recorder"
        / ".build"
        / "release"
        / "EpidoteRecorderCLI",
        anthropic_api_key=os.getenv("ANTHROPIC_API_KEY", ""),
        anthropic_model=os.getenv("ANTHROPIC_MODEL", ""),
        whisper_model=os.getenv("EPIDOTE_WHISPER_MODEL", "small"),
        hf_token=os.getenv("HF_TOKEN", ""),
        google_client_secret_path=_optional_path(
            os.getenv("EPIDOTE_GOOGLE_CLIENT_SECRET", "")
        ),
        google_token_path=app_home / "google-token.json",
        google_calendar_id=os.getenv("EPIDOTE_GOOGLE_CALENDAR_ID", "primary"),
        obsidian_vault_dir=_optional_path(os.getenv("EPIDOTE_OBSIDIAN_VAULT", "")),
    )
    defaults.ensure_directories()
    if settings_path.exists():
        payload = json.loads(settings_path.read_text(encoding="utf-8"))
        config = AppConfig.from_dict({**defaults.to_dict(), **payload})
        config.ensure_directories()
        return config
    return defaults


def save_config(config: AppConfig) -> None:
    config.ensure_directories()
    config.settings_path.write_text(
        json.dumps(config.to_dict(), indent=2),
        encoding="utf-8",
    )
