# Architecture

## Overview

Epidote is split into two runtime layers:

1. A Python desktop application built with PySide6.
2. A native Swift recorder helper for macOS audio capture.

The Python app owns orchestration, storage, transcription, LLM analysis, Google Calendar sync, and Obsidian export. The Swift helper exists because system audio capture on macOS is not reliable from pure Python.

## Data Flow

1. The user starts a meeting from the desktop app.
2. The app creates a session folder in `~/Library/Application Support/Epidote/meetings/<session_id>/`.
3. The Swift recorder helper writes:
   - `microphone.wav`
   - `system_audio.m4a`
   - `recording_manifest.json`
4. When recording stops, Python processes the session:
   - transcribe microphone track as `speaker_1`
   - transcribe system track as `speaker_2`
   - optionally run pyannote diarization on the system track to split additional remote speakers
   - merge transcript segments by timestamp
   - save transcript JSON and Markdown
5. If Claude is configured, Epidote asks it to:
   - summarize the meeting
   - list decisions
   - extract TODO items with due dates and reminders when available
   - suggest architecture and code related to each task when the meeting is about building software
6. If Google Calendar is configured, TODO items with dates are pushed to the configured calendar.
7. If Obsidian is configured, a meeting note is written into the vault.
8. A copy of the processed session is exported to `~/Documents/Epidote/Exports/`.

## Components

### Python UI

- `src/epidote/ui/main_window.py`
- Handles settings, recording controls, meeting history, transcript view, summary view, and task detail rendering.

### Recording Bridge

- `src/epidote/recording.py`
- Builds and launches the Swift helper.

### Transcription

- `src/epidote/transcription.py`
- Uses Faster Whisper for speech-to-text.
- Uses pyannote when a Hugging Face token is configured.

### Analysis

- `src/epidote/claude.py`
- Calls Anthropic's Messages API and expects strict JSON back.

### Integrations

- `src/epidote/calendar.py`
- `src/epidote/obsidian.py`

## Speaker Labeling Notes

Without diarization, Epidote labels:

- `speaker_1`: microphone track, usually you
- `speaker_2`: system audio track, usually everyone else combined

With pyannote enabled, remote speakers are split into `speaker_2`, `speaker_3`, and so on based on diarization turns from the system audio track.

## Tradeoffs

- macOS system audio capture requires Screen Recording permission.
- Google Calendar integration requires OAuth setup.
- Accurate multi-party diarization depends on pyannote and a Hugging Face token.
- The current MVP favors a clear vertical slice over background daemons, signing, auto-updates, or hardened secret storage.
