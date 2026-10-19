# media-core

Monorepo for the media processing core library and the desktop app built on top of it.
It is a Cargo workspace (Rust crates under `crates/`) plus a Tauri 2 + SvelteKit app under `app/`.

## Layout

```
Cargo.toml        workspace root (members, shared dependency versions)
Cargo.lock        single lockfile for every crate and the app backend
crates/
  media-core/     core library: video processing, HLS, RTSP, analysis, pipelines (+ CLI bins)
  annotation/     video annotation library
  audio-core/     audio recording I/O and audio/SRT metadata analysis
  whisper-rs/     vendored whisper.cpp bindings (ASR), pregenerated FFI bindings
app/              desktop app (Tauri 2 backend in app/src-tauri, SvelteKit frontend in app/src)
examples/         standalone example programs (local sample media is gitignored)
docs/             architecture notes and plans
```

## Prerequisites

- Rust (stable) and Cargo
- Node.js and Yarn (for the app frontend)
- OpenCV and FFmpeg installed on the system (e.g. via Homebrew on macOS)
- macOS: Xcode or Command Line Tools whose linker matches the active SDK

## Common commands

Run frontend commands inside `app/` (or with `yarn --cwd app …`); run Cargo commands from the repo root.

| Task | Command |
|---|---|
| Install frontend dependencies | `yarn --cwd app install` |
| Run the desktop app (Rust + hot reload) | `cd app && yarn tauri dev` |
| Frontend type check | `yarn --cwd app run check` |
| Check the app backend | `cargo check -p media-app` |
| Build everything in the workspace | `cargo build --workspace` |
| Composer graph tests | `cargo test -p media-app --bin media-app graph::` |
| Core library tests | `cargo test -p media_core` |

The workspace deliberately declares no `[profile.*]` settings; see `crates/media-core/README.md`
for why `panic = "abort"` must not be added at the root.

## History

`crates/media-core/` is this repository's original crate. The app, `annotation`, `audio-core`,
`whisper-rs` and `docs/` were imported with full history from the former `media-app` repository,
which previously consumed this crate as a git submodule.
