# AGENTS.md

This file provides guidance to coding agents when working with code in this repository.

## Project Overview

Subtitle translation desktop application built with **GPUI** (Rust UI) and **gpui-component**. Translates subtitle files (SRT/ASS) using LLM APIs, with FFmpeg integration for video subtitle extraction and muxing.

There is no Tauri, React, Vite, or Bun. The UI and business logic are a single Rust binary.

## Development Commands

```bash
# Run the desktop app
cargo run

# Unit tests (core modules)
cargo test

# Release build
cargo build --release

# Format / lint
cargo fmt
cargo clippy
```

FFmpeg must be on `PATH` for video extract/mux. An LLM API endpoint (OpenAI-compatible or Anthropic) is required for translation.

## Architecture

```
GPUI App (src/main.rs)
├── RootView (src/app.rs)          # navbar + page switcher
├── views/                         # Translation / Settings / editor / queue
├── state/                         # GPUI Entities (queue, settings, logs)
└── core/                          # business logic (NO gpui)
    ├── subtitle/                  # SRT/ASS parsers
    ├── translator.rs              # LLM client + batching
    ├── ffmpeg.rs                  # extract / mux / track list
    ├── settings.rs / templates.rs # JSON persistence
    └── files.rs / paths.rs        # file helpers + app data dir
```

`src/core` must stay free of GPUI so tests can run with `cargo test`. HTTP work uses a process-wide Tokio runtime (`src/core/runtime.rs`); do not block the GPUI UI thread.

Persistence: `directories::ProjectDirs::from("com", "translator", "translator")` → `data_dir()`, files `settings.json` and `templates.json` (`rename_all = "camelCase"`).

## Translation Workflow

1. User adds files (video or subtitle) → queue with UUID
2. For videos: FFmpeg extracts subtitle track → temporary ASS file
3. Parse subtitle → detect encoding → convert to UTF-8
4. Optional: detect target language via LLM
5. Batch translate with LLM (configurable batch size, parallelism, retries)
6. Save translated subtitle
7. Optional: mux translated subtitle back into video (creates `.muxed.mkv`)

## Tech Stack Notes

- **GPUI 0.2** + **gpui-component 0.5** for native UI
- **Tokio** runtime (dedicated thread) for LLM HTTP via **reqwest**
- **encoding_rs** for character encoding detection
- **FFmpeg / FFprobe** on PATH for video subtitle tracks
