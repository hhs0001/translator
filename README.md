# Translator (GPUI)

English | [Português (Brasil)](README.pt-BR.md)

Translator is a desktop app for translating subtitle files (and subtitles embedded in videos) using LLM APIs. It is a native Rust app built with [GPUI](https://gpui.rs) and [gpui-component](https://longbridge.github.io/gpui-component/), and focuses on fast batch translation workflows with editable results and flexible output options.

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
- Video files: MKV, MP4, AVI, MOV, WEBM, M4V, TS (subtitle extraction via FFmpeg).

## Download

Pre-built binaries for all platforms are available on the [Releases page](https://github.com/hhs0001/translator/releases).

### Supported Platforms

| Platform | Architecture          | File        |
| -------- | --------------------- | ----------- |
| Windows  | x64                   | `.exe`      |
| macOS    | ARM64 (Apple Silicon) | binary      |
| macOS    | x64 (Intel)           | binary      |
| Linux    | AMD64                 | binary      |

## Requirements (for development)

- **Rust** 1.85+ (stable).
- **FFmpeg** on `PATH` for video subtitle extraction/muxing.
- An LLM API endpoint (OpenAI-compatible or Anthropic).

On Linux you also need GPUI system libraries, typically:

```bash
sudo apt-get install -y \
  libxkbcommon-dev libwayland-dev libvulkan-dev \
  libx11-dev libxrandr-dev libxi-dev libxcursor-dev libxinerama-dev \
  libssl-dev cmake pkg-config libfontconfig-dev
```

## Development

```bash
cargo run
```

## Build

```bash
cargo build --release
```

The binary is written to `target/release/translator` (or `translator.exe` on Windows).

## Tests

```bash
cargo test
```

## Project structure

```
src/main.rs      # GPUI entry
src/app.rs       # RootView (navbar + pages)
src/core/        # business logic (subtitle, FFmpeg, LLM, settings)
src/state/       # GPUI entities (queue, settings, logs)
src/views/       # UI pages
assets/i18n/     # en / pt-BR strings
```

## CI/CD

This project uses GitHub Actions for automated builds and releases:

- **Lint**: `cargo fmt` and `cargo clippy` on Ubuntu and Windows
- **Test Build**: `cargo test` and `cargo build --release` on macOS, Ubuntu, and Windows
- **Release**: cargo release binaries for all platforms

To create a new release, update the version in `Cargo.toml` and merge to the `release` branch.
