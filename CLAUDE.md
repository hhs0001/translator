# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Subtitle translation desktop application built with Tauri 2 (Rust backend) and React 19 (TypeScript frontend). Translates subtitle files (SRT/ASS) using LLM APIs, with FFmpeg integration for video subtitle extraction and muxing.

## Development Commands

```bash
# Development (starts Vite dev server + Tauri with hot reload)
bun run tauri dev

# Production build
bun run build && bun run tauri build

# TypeScript check only
bun run build

# Preview production frontend
bun run preview
```

Package manager is **Bun** (not npm/yarn).

## Architecture

### IPC-Based Client-Server Pattern

```
React Frontend (Webview)          Rust Backend (Native)
├── UI State (Zustand stores)     ├── Subtitle parsing (SRT/ASS)
├── Component tree                ├── FFmpeg integration
├── Tauri IPC calls ──────────────├── LLM API client
└── Event listeners ◄─────────────└── File I/O, settings persistence
```

### Frontend Structure (`src/`)

- **`stores/`** - Zustand stores: `translationStore` (file queue, orchestration), `settingsStore` (app config, templates), `logsStore` (log messages)
- **`components/translation/`** - Translation workflow UI (FileDropZone, FileQueue, SubtitleEditor)
- **`components/config/`** - Settings pages (API, FFmpeg, prompts, templates)
- **`utils/tauri.ts`** - Wrapper functions for Tauri IPC commands
- **`types/index.ts`** - All shared TypeScript interfaces

### Backend Structure (`src-tauri/src/`)

- **`lib.rs`** - Main entry point with 30+ Tauri commands exported via `#[tauri::command]`
- **`translator.rs`** - LLM client, batch translation logic, language detection
- **`ffmpeg.rs`** - FFmpeg/FFprobe integration (extraction, muxing, track detection)
- **`subtitle/`** - Format-specific parsers: `srt.rs`, `ass.rs`

### Key Tauri Commands

Subtitle: `load_subtitle`, `save_subtitle`, `detect_subtitle_format`
FFmpeg: `check_ffmpeg_installed`, `extract_subtitle_track`, `mux_subtitle_to_video`, `list_video_subtitle_tracks`
Translation: `translate_subtitle_full`, `translate_subtitle_batch`, `continue_translation`, `detect_language`
Settings: `load_settings`, `save_settings`, `load_templates`, `add_template`, `update_template`, `delete_template`

### Event Communication (Backend → Frontend)

- `translation:progress` - Real-time progress updates during translation
- `translation:error` - Error events with retry information

## Translation Workflow

1. User adds files (video or subtitle) → queue with UUID
2. For videos: FFmpeg extracts subtitle track → temporary ASS file
3. Parse subtitle → detect encoding → convert to UTF-8
4. Optional: detect target language via LLM
5. Batch translate with LLM (configurable batch size, parallelism, retries)
6. Save translated subtitle
7. Optional: mux translated subtitle back into video (creates `.muxed.mkv`)

## Tech Stack Notes

- **HeroUI** (beta) for component library - built on Radix UI
- **TailwindCSS 4.1** for styling
- **Tokio** async runtime in Rust backend
- **reqwest** for HTTP client (LLM API calls)
- **encoding_rs** for character encoding detection
