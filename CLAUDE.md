# AGENTS.md

This file provides guidance to coding agents when working with code in this repository.

## Project Overview

Subtitle translation desktop application built with **GPUI** (Rust UI) and **gpui-component**. Translates subtitle files (SRT/ASS) using LLM APIs, with FFmpeg integration for video subtitle extraction and muxing.

There is no Tauri, React, Vite, or Bun. The UI and business logic are a single Rust binary.

## Development Commands

```bash
# Run the desktop app (accepts subtitle/video paths as arguments)
cargo run -- path/to/file.srt

# Tests: core unit tests + integration tests against a fake LLM server
cargo test

# Release build
cargo build --release

# Format / lint
cargo fmt
cargo clippy --all-targets
```

Helper scripts:

```bash
python scripts/gen_icons.py      # regenerates assets/icons/*.svg and src/assets.rs
python scripts/make_samples.py   # sample .srt/.ass files for manual testing
python scripts/mock_api.py 8123  # fake OpenAI-compatible server for end-to-end runs
```

`TRANSLATOR_DATA_DIR` overrides the config directory, which is how you exercise
the app against throwaway settings without touching a real install.

FFmpeg must be on `PATH` for video extract/mux. An LLM API endpoint (OpenAI-compatible or Anthropic) is required for translation.

## Architecture

```
src/main.rs                        # thin binary, calls translator::run()
src/lib.rs                         # library root (so tests can drive the core)
├── app.rs                         # RootView: navbar, tabs, toasts, logs drawer
├── theme.rs                       # brand palette on top of gpui-component
├── icons.rs + assets.rs           # embedded lucide-style SVGs (AssetSource)
├── views/                         # Translation / Settings / editor / queue / logs
│   └── ui.rs                      # shared visual primitives (cards, chips, fields)
├── state/                         # GPUI Entities (queue, settings, logs)
└── core/                          # business logic (NO gpui)
    ├── subtitle/                  # SRT/ASS parsers
    ├── translator.rs              # LLM client, batching, parallel scheduler
    ├── translate.rs               # per-file orchestration + callbacks
    ├── ffmpeg.rs                  # extract / mux / track list
    ├── settings.rs / templates.rs # JSON persistence
    └── files.rs / paths.rs        # file helpers + app data dir
```

The UI never clones parsed subtitles per frame: the queue renders from
`QueueState::summaries()` and the editor keeps an incrementally patched row
cache (`views/editor.rs`) feeding a virtualized `gpui::list`.

`src/core` must stay free of GPUI so tests can run with `cargo test`. HTTP work uses a process-wide Tokio runtime (`src/core/runtime.rs`); do not block the GPUI UI thread.

Persistence: `directories::ProjectDirs::from("com", "translator", "translator")` → `data_dir()`, files `settings.json` and `templates.json` (`rename_all = "camelCase"`).

Settings stay compatible with the old Tauri builds: the schema is unchanged,
`core/paths.rs` imports `settings.json`/`templates.json` from the legacy
`com.translator*` folders on first run, and `core/settings.rs` falls back to a
field-by-field load (keeping a `.bak`) instead of discarding a file it cannot
parse strictly.

## Translation Workflow

1. User adds files (video or subtitle) → queue with UUID
2. For videos: FFmpeg extracts subtitle track → temporary ASS file
3. Parse subtitle → detect encoding → convert to UTF-8
4. Optional: detect target language via LLM
5. Batch translate with LLM. Batches run through a `buffer_unordered` queue so
   exactly `parallelRequests` calls stay in flight; each batch reports its state
   (`BatchUpdate`) for the segmented progress bar, and retries happen inside the
   batch so a slow one never blocks the others.
6. Save translated subtitle, keeping the source format (`.srt` in → `.translated.srt` out)
7. Optional: mux translated subtitle back into video (creates `.muxed.mkv`)

## Tech Stack Notes

- **GPUI 0.2** + **gpui-component 0.5** for native UI
- **Tokio** runtime (dedicated thread) for LLM HTTP via **reqwest**
- **encoding_rs** for character encoding detection
- **FFmpeg / FFprobe** on PATH for video subtitle tracks
