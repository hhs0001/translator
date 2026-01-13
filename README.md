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

## Requirements

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

```text
src/           # React UI (settings, translation flow, editor)
src-tauri/     # Tauri backend (FFmpeg, subtitle parsing, LLM calls)
```
