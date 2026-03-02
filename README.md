# Translator (Tauri + React)

English | [Português (Brasil)](README.pt-BR.md)

Translator is a desktop app for translating subtitle files (and subtitles embedded in videos) using LLM APIs. It is built with Tauri, React, and Vite, and focuses on fast batch translation workflows with editable results and flexible output options.

## Features

- Drag-and-drop queue for subtitle files and videos, with single and batch modes.
- Subtitle editor with live progress and per-line tweaks.
- LLM API configuration with auto-detected OpenAI/Anthropic formats and model listing.
- Prompt templates and a prompt editor for consistent translation styles.
- Batch sizing, parallel request control, retry limits, and multi-file concurrency.
- Optional language detection to auto-fill mux metadata.
- FFmpeg-backed subtitle extraction and muxing for video files.
- Output options for separate subtitle files or muxed MKV output.

## Supported formats

- Subtitle files: SRT, ASS, SSA (VTT planned).
- Video files: MKV, MP4, AVI (subtitle extraction via FFmpeg).

## Download

Pre-built binaries for all platforms are available on the [Releases page](https://github.com/hhs0001/translator/releases).

### Supported Platforms

| Platform | Architecture          | File             |
| -------- | --------------------- | ---------------- |
| Windows  | x64                   | `.exe` installer |
| macOS    | ARM64 (Apple Silicon) | `.app`           |
| macOS    | x64 (Intel)           | `.app`           |
| Linux    | AMD64                 | `.deb`           |

## Requirements (for development)

- **Bun** (recommended) or another Node.js package manager.
- **Rust + Tauri CLI** for desktop builds.
- **FFmpeg** for video subtitle extraction/muxing.
- An LLM API endpoint (OpenAI-compatible or Anthropic).

## Development

```bash
bun install
bun run dev
```

## Run the desktop app (Tauri)

```bash
bun run tauri dev
```

## Build

```bash
bun run tauri build
```

## Project structure

```
src/           # React UI (settings, translation flow, editor)
src-tauri/     # Tauri backend (FFmpeg, subtitle parsing, LLM calls)
```

## CI/CD

This project uses GitHub Actions for automated builds and releases:

- **Lint & Type Check**: Runs on every PR to validate code quality
- **Test Build**: Compiles the app for all platforms without releasing
- **Release**: Automatically creates releases with binaries for all platforms

To create a new release, update the version in `package.json` and merge to the `release` branch.
