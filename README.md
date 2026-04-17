# Epidote

Epidote is a macOS desktop meeting recorder and post-processing assistant.

Current stack:

- Python desktop app with PySide6
- Swift command-line helper for macOS system audio + microphone capture
- Local transcription with Faster Whisper
- Optional diarization with pyannote
- Claude for summary, action items, and architecture/code suggestions
- Optional Google Calendar sync and Obsidian export

See [docs/setup.md](/Users/mduzch/projects/Epidote/docs/setup.md) for setup and [docs/architecture.md](/Users/mduzch/projects/Epidote/docs/architecture.md) for the system design.
