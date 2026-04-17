# Setup

## Prerequisites

- macOS 15 or newer
- Python 3.12+
- Swift 6 / Xcode command line tools
- Anthropic API key and model id
- Optional: Hugging Face token for diarization
- Optional: Google Calendar OAuth client credentials
- Optional: Obsidian vault path

## Install

```bash
uv sync
swift build --package-path native/macos-recorder -c release
```

## Configure

You can either use environment variables from `.env.example` or open the app and fill in the settings tab.

Important settings:

- `Anthropic API key`
- `Anthropic model`
- `Whisper model`
- `Hugging Face token` for speaker diarization
- `Google client secret` for Calendar sync
- `Google calendar id`
- `Obsidian vault`
- `Export folder`

## Run

```bash
uv run epidote
```

## macOS Permissions

The first recording attempt should trigger:

- Microphone permission
- Screen Recording permission

If system audio capture fails, open:

`System Settings -> Privacy & Security -> Screen Recording`

and grant access to the terminal or app you are using to launch Epidote.

## Google Calendar

1. Create an OAuth Desktop App credential in Google Cloud.
2. Put the downloaded client secret JSON somewhere local.
3. Point the `Google client secret` setting at that file.
4. The first sync opens a local OAuth flow and stores a token in `~/Library/Application Support/Epidote/google-token.json`.

## Notes

- Processing requires actual model dependencies; `uv sync` installs them.
- The repository includes a working scaffold, but production distribution still needs app signing and packaging.
