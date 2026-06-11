# Audio & ASR Architecture

This document describes the `/audio` feature: in-app voice recording, whisper.cpp
transcription (ASR), and audio metadata inspection. The feature was modeled on the
standalone `reference/asr-app` project and rebuilt to media-app conventions as a
fully isolated addition — every backend module and frontend component is new, and
the only changes to existing files are additive registrations (nav entry, icon
mappings, type/event declarations, command registration).

## Overview

The `/audio` route hosts three self-contained tabs:

| Tab | What it does | Backbone |
|-----|--------------|----------|
| **Recorder** | Capture mic audio with a live level meter; list / play / rename / delete recordings | Web Audio API + MediaRecorder → `audio-core` crate |
| **Transcribe** | Download whisper models, transcribe audio files or recordings with live segments, export TXT/SRT/JSON | `src/asr.rs` → vendored `whisper-rs` (whisper.cpp) |
| **Metadata** | Inspect duration / sample rate / bitrate / channels, plus optional SRT statistics | `audio-core` crate (lofty + SRT parser) |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Frontend (Svelte 5)                       │
│  routes/audio/+page.svelte (TabBar)                              │
│  ├── RecorderTab    (getUserMedia + AnalyserNode + MediaRecorder)│
│  ├── TranscribeTab  (model download, live segments, export)      │
│  ├── MetadataTab    (audio + optional SRT analysis)              │
│  └── LevelMeter     (ProgressBar + SparklineBar)                 │
│                                                                  │
│  lib/events.ts — onAsrProgress / onAsrModelProgress              │
│  lib/types.ts  — RecordingInfo, AudioMetadata, AsrModelInfo, …   │
└──────────────────────────┬──────────────────────────────────────┘
                           │ invoke() + listen()
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Tauri Command Layer                          │
│  commands/audio.rs  (recordings + metadata, 6 commands)          │
│  commands/asr.rs    (models + transcription, 5 commands)         │
└───────────┬───────────────────────────────┬─────────────────────┘
            ▼                               ▼
┌───────────────────────────┐  ┌──────────────────────────────────┐
│   crates/audio-core       │  │   src/asr.rs (engine)            │
│   recording.rs (file I/O) │  │   model registry + download      │
│   metadata.rs  (lofty,    │  │   symphonia decode → 16 kHz mono │
│                SRT stats) │  │   whisper-rs transcription       │
└───────────────────────────┘  │   AtomicBool cancellation        │
                               └────────────┬─────────────────────┘
                                            ▼
                               ┌──────────────────────────────────┐
                               │   crates/whisper-rs (vendored)   │
                               │   whisper.cpp via CMake + FFI    │
                               │   (pregenerated bindings —       │
                               │    NO bindgen, see below)        │
                               └──────────────────────────────────┘
```

## Storage layout

Everything lives under the Tauri app data dir (`app.path().app_data_dir()`):

```
{app_data_dir}/
├── recordings/                  # saved mic recordings (.m4a on macOS)
└── whisper/models/              # downloaded ggml models
    ├── ggml-tiny.bin
    └── ggml-base.bin.download   # in-flight temp file, renamed on completion
```

## Recording pipeline (RecorderTab)

1. `getUserMedia({ audio: { echoCancellation, noiseSuppression, autoGainControl } })`.
2. An `AnalyserNode` (fftSize 256) feeds the LevelMeter: RMS level every frame,
   pushed into a 48-slot rolling history every 4th frame.
3. `MediaRecorder` collects 1-second chunks. The MIME type is picked at runtime
   via `MediaRecorder.isTypeSupported()`, preferring `audio/mp4`:

   | Priority | MIME | Extension | Why |
   |----------|------|-----------|-----|
   | 1 | `audio/mp4` | `.m4a` | What macOS WKWebView supports; AAC decodes in symphonia → transcribable |
   | 2 | `audio/webm;codecs=opus` | `.webm` | Chromium webviews; **Opus is NOT decodable by our symphonia feature set** — playable but not transcribable |
   | 3 | `audio/webm` | `.webm` | last resort |

   The reference asr-app hardcoded `audio/webm;codecs=opus`, which WKWebView
   doesn't support at all — this ordering is a deliberate fix.
4. On stop, the blob is sent to `save_audio_recording` and written into
   `recordings/` with a `recording_<ISO-stamp>` filename.
5. Recordings are addressed by **bare filename only** — `audio-core::recording`
   rejects names containing `/`, `\`, or `..` (path-traversal guard). Playback
   uses `convertFileSrc(filePath)` on the absolute path returned by the backend.

macOS mic access requires `NSMicrophoneUsageDescription`, provided via
`src-tauri/Info.plist` (Tauri merges it into the bundle).

## ASR engine (`src/asr.rs`)

### Model registry

Ten models with pinned HuggingFace URLs (`ggerganov/whisper.cpp`): `tiny`,
`tiny.en`, `base`, `base.en`, `small`, `small.en`, `medium`, `medium.en`,
`large-v3`, `large-v3-turbo`. Each entry carries an `approx_mb` for the UI.
`list_asr_models` reports per-model `downloaded` + on-disk `sizeBytes`.

### Model download

`download_asr_model` runs a blocking chunked download in `spawn_blocking`:
256 KB reads, written to `<model>.download` and renamed on success (a partial
download never shadows a real model). Events are throttled to one per whole
percent (or per 16 MB when the server sends no Content-Length). The HTTP client
has a 30 s connect timeout but **no overall request timeout** — model files run
into the GB range; a stalled transfer is handled by cancellation instead.

### Transcription

`transcribe_audio(audioPath, model, language?)` takes a **path**, not bytes —
the file is read on the Rust side to keep IPC payloads small.

1. Decode: symphonia probes with a filename-extension `Hint`, decodes any
   track to f32, downmixes to mono (channel average), then linearly resamples
   to 16 kHz. Enabled symphonia features: defaults (wav/flac/ogg/vorbis…) +
   `mp3`, `aac`, `isomp4`. **No Opus.**
2. Whisper: `BeamSearch { beam_size: 5 }`, `n_threads = available_parallelism`,
   language forced when given, otherwise auto-detect.
3. Output: full text, generated SRT, and a JSON value with per-segment
   timestamps — returned as one `TranscriptionResult { text, srt, json }`.

### Cancellation

A process-wide `static CANCELLED: AtomicBool` (one ASR operation runs at a
time; the UI enforces that). `cancel_asr` sets it; it is checked by:
- the download loop (removes the partial `.download` file),
- the decode loop (per packet),
- whisper itself via `params.set_abort_callback_safe(is_cancelled)`.

A cancelled `state.full()` error is mapped to `"Transcription cancelled"`.

### Event protocol

`asr:progress` (camelCase payload):

| kind | fields used |
|------|-------------|
| `progress` | `progress` (0–100), `message` |
| `segment` | `segmentIndex`, `startSeconds`, `endSeconds`, `text` |
| `complete` | `progress: 100` |

`asr:model-progress`: `{ model, downloadedBytes, totalBytes?, percent?, done }`.

Frontend listeners live in `lib/events.ts` (`onAsrProgress`,
`onAsrModelProgress`); TranscribeTab registers them in `onMount` inside a
try/catch so the page still renders in a plain browser without the Tauri runtime.

## Command reference

| Command | Signature (frontend view) | Notes |
|---------|---------------------------|-------|
| `save_audio_recording` | `(audioData: number[], filename) → RecordingInfo` | writes into `recordings/` |
| `list_audio_recordings` | `() → RecordingInfo[]` | newest first |
| `delete_audio_recording` | `(filename) → void` | filename-only, traversal-guarded |
| `rename_audio_recording` | `(oldFilename, newFilename) → RecordingInfo` | rejects existing target |
| `get_recording_info` | `(filename) → RecordingInfo` | |
| `get_audio_metadata` | `(audioPath, srtPath?) → AudioMetadata` | lofty + optional SRT stats |
| `list_asr_models` | `() → AsrModelInfo[]` | registry + download state |
| `download_asr_model` | `(model) → AsrModelInfo` | emits `asr:model-progress` |
| `transcribe_audio` | `(audioPath, model, language?) → TranscriptionResult` | emits `asr:progress` |
| `cancel_asr` | `() → void` | cancels download or transcription |
| `save_text_file` | `(path, contents) → void` | export helper (plugin-fs isn't registered) |

All commands return `Result<T, String>`; response structs use
`#[serde(rename_all = "camelCase")]`.

## Vendored whisper-rs and the bindgen × OpenCV conflict

`src-tauri/crates/whisper-rs` is a vendored copy of whisper-rs 0.15.1 (FFI
bindings to whisper.cpp, built via CMake) with one critical local change:
**bindgen is removed from `whisper-rs-sys`**.

Why: adding bindgen to the workspace dependency graph changes clang-sys
feature unification, which breaks the `opencv` crate's binding generator with
a `"a libclang shared library is not loaded on this thread"` panic — even
though OpenCV builds fine without whisper-rs in the tree. The reference
asr-app's `.cargo/config.toml` workaround (forcing `LIBCLANG_PATH` to Xcode's
libclang) *also* breaks OpenCV, so it was deliberately not copied.

Instead, `sys/src/bindings.rs` is **pregenerated** (2026-06-12, against the
vendored whisper.cpp 1.8.3, using Xcode's libclang) and `sys/build.rs` simply
copies it to `OUT_DIR`. Consequences:

- Normal builds never need libclang for whisper — faster and env-independent.
- If the vendored whisper.cpp is ever upgraded, regenerate `sys/src/bindings.rs`
  manually with bindgen. Use **Xcode's** libclang (`xcrun --find clang` →
  adjacent lib dir); Homebrew LLVM 22's libclang fails.
- On macOS the whisper.cpp CMake build needs the toolchain set up per
  `.cargo/config.toml` notes in asr-app **only for bindgen runs** — the normal
  app build is unaffected.

## Gallery integration

`/gallery/pages/audio` catalogs the page per the standard pattern: a
CompositionTree of the three tabs, one section per building block (TabBar,
Panel, LevelMeter, StatusBadge, RunButton, ProgressBar, FilePicker, FormField,
StatCard, ErrorAlert, EmptyState) with static demos and usage snippets, and an
"Assembled result" section rendering the real tabs.

## Testing notes

- Mic capture, model downloads, and transcription require the real Tauri
  runtime (`yarn tauri dev`) — the browser preview renders the UI but every
  `invoke` fails and `getUserMedia` permission semantics differ.
- The first `yarn tauri dev` after a clean checkout compiles whisper.cpp via
  CMake (~2–3 minutes); subsequent builds are cached.
- ffmpeg is **not** required for audio — decoding is pure-Rust via symphonia
  (ffmpeg remains required for the video dedup/thumbnail features).
