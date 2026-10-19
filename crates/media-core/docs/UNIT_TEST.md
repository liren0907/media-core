# Unit Tests

## Table of Contents

- [Test Video](#test-video)
- [Config Generation Tests](#config-generation-tests)
- [RTSP Stream Extraction Tests](#rtsp-stream-extraction-tests)
- [HLS Conversion Tests](#hls-conversion-tests)
- [Process Module Tests](#process-module-tests)
- [Metadata Module Tests](#metadata-module-tests)
- [Annotation Module Tests](#annotation-module-tests)
- [Benchmark Module Tests](#benchmark-module-tests)
- [Streaming Module Tests](#streaming-module-tests)
- [Video Process Module Tests](#video-process-module-tests)

---

## Test Video

Most integration tests read `data/test.mp4`. It is not stored in git: the first test that needs it
runs `script/gen_test_data.sh`, which uses FFmpeg to generate a 10-second synthetic clip (640×360,
30 fps, a moving test pattern with a 1 kHz tone). To create or recreate it by hand:

```bash
sh script/gen_test_data.sh          # does nothing if the file exists
sh script/gen_test_data.sh --force  # regenerate
```

Without FFmpeg the video cannot be generated, and the tests that need it fail with the reason
instead of being skipped.

Run the commands in this file from `crates/media-core/`, or add `-p media_core` when running them
from the repository root.

---

## Config Generation Tests

**File**: `tests/test_config_generation.rs`

### Tests

1. **`test_rtsp_config_generation`** - Generates and validates default RTSP config
2. **`test_process_config_generation`** - Generates and validates default process config

### Run
```bash
cargo test --test test_config_generation
```

---

## RTSP Stream Extraction Tests

**File**: `tests/test_rtsp_extraction.rs`

### Tests

1. **`test_rtsp_stream_extraction`** - FFmpeg mode (records `.mp4` to `output/`)
2. **`test_hls_streaming`** - HLS mode (generates `.m3u8` + `.ts` to `hls_test_output/`)

### Prerequisites
- FFmpeg installed
- [MediaMTX](https://github.com/bluenviron/mediamtx) installed (`brew install mediamtx` on macOS)
- A live stream at `rtsp://localhost:8554/mystream`, served by `script/rtsp_test_server.sh`

Both tests read a live stream, so they are marked `#[ignore]`: a plain `cargo test` lists them as
ignored instead of failing. The script starts MediaMTX and loops `data/test.mp4` into it (generating
the video first if needed) until you press Ctrl-C, which stops both.

### Run All RTSP Tests
```bash
sh script/rtsp_test_server.sh                                     # Terminal 1
cargo test --test test_rtsp_extraction -- --ignored --nocapture   # Terminal 2
```

### Run Individual Tests
```bash
# FFmpeg mode
cargo test --test test_rtsp_extraction test_rtsp_stream_extraction -- --ignored --nocapture

# HLS mode
cargo test --test test_rtsp_extraction test_hls_streaming -- --ignored --nocapture
```

### Test Comparison

| Test | Mode | Output | Duration |
|------|------|--------|----------|
| `test_rtsp_stream_extraction` | FFmpeg | `output/camera_*/*.mp4` | ~36s |
| `test_hls_streaming` | HLS | `hls_test_output/*.m3u8, *.ts` | ~10s |

### Troubleshooting

**The tests fail within a second**: nothing is serving the stream; start `script/rtsp_test_server.sh` first.

**`mediamtx did not start`**: another program is listening on port 8554.

**FFmpeg or MediaMTX not found**: Install them
```bash
brew install ffmpeg mediamtx  # macOS
sudo apt-get install ffmpeg   # Ubuntu; MediaMTX: https://github.com/bluenviron/mediamtx/releases
```

---

## HLS Conversion Tests

**File**: `tests/test_hls_conversion.rs`

### Tests

1. **`test_hls_config_generation`** - Validates HLS config defaults and JSON serialization
2. **`test_hls_conversion`** - Converts `data/test.mp4` to HLS format

### Prerequisites
- FFmpeg installed
- Test video `data/test.mp4`, generated automatically (see [Test Video](#test-video))

### Run
```bash
cargo test --test test_hls_conversion -- --nocapture
```

### Test Comparison

| Test | Output | Duration |
|------|--------|----------|
| `test_hls_config_generation` | None | <1s |
| `test_hls_conversion` | `hls_test_output/*.m3u8, *.ts` | ~5s |

---

## Process Module Tests

**File**: `tests/test_process_module.rs`

### Tests

1.  **`test_process_config_generation`** - Validates Process config defaults and JSON serialization.
2.  **`test_video_extraction`** - **Integration Test**: Extracts frames from `data/test.mp4`.
    -   Mode: "skip" (Direct frame extraction, no video creation).
    -   Output: Temporary directory (cleaned up automatically).
    -   Verification: Checks that `.jpg` files are created.
3.  **`test_video_creation`** - **Integration Test**: Creates a time-lapse video from `data/test.mp4`.
    -   Mode: "direct" (Direct video creation).
    -   Output: Temporary directory.
    -   Verification: Checks that `.mp4` file is created and has size > 0.

### Prerequisites

-   Test video `data/test.mp4`, generated automatically (see [Test Video](#test-video))

### Run

```bash
cargo test --test test_process_module -- --nocapture
```

### Test Comparison

| Test | Mode | Output | Duration |
| :--- | :--- | :--- | :--- |
| `test_video_extraction` | Skip (Direct) | `.jpg` frames | ~2s |
| `test_video_creation` | Direct | `.mp4` video | ~2s |

---

## Metadata Module Tests

**File**: `tests/test_metadata.rs`

### Tests

1. **`test_get_media_info`** - **Unit Test**: Validates metadata extraction.
    -   **Input**: `data/test.mp4`
    -   **Verification**: Checks that:
        -   Function returns `Ok`.
        -   `media_type` is "video".
        -   Basic fields (`width`, `height`, `duration_seconds`) are valid (> 0).

### Prerequisites

-   Test video `data/test.mp4`, generated automatically (see [Test Video](#test-video))

### Run

```bash
cargo test --test test_metadata -- --nocapture
```

### Test Comparison

| Test | Output | Duration |
| :--- | :--- | :--- |
| `test_get_media_info` | Metadata logs | <1s |

---

## Annotation Module Tests

**File**: `tests/test_annotation.rs`

### Tests

1. **`test_single_frame_annotation`** - Example 1: Single image with filename overlay (TopLeft)
2. **`test_video_from_frames_filename`** - Example 2: Video from frames with filename annotation
3. **`test_video_from_frames_timestamp`** - Example 3: Video from frames with timestamp annotation (BottomLeft)
4. **`test_video_from_frames_custom_text`** - Example 4: Video from frames with custom watermark (TopRight)

### Prerequisites

- Test video `data/test.mp4`, generated automatically (see [Test Video](#test-video))
- FFmpeg installed

### Run

```bash
cargo test --test test_annotation -- --nocapture
```

### Test Comparison

| Test | Annotation Type | Output | Duration |
| :--- | :--- | :--- | :--- |
| `test_single_frame_annotation` | Filename | `.jpg` image | <1s |
| `test_video_from_frames_filename` | Filename | `.mp4` video | ~1s |
| `test_video_from_frames_timestamp` | Timestamp | `.mp4` video | ~1s |
| `test_video_from_frames_custom_text` | Custom | `.mp4` video | ~1s |

---

## Benchmark Module Tests

**File**: `tests/test_benchmark.rs`

### Tests

1. **`test_benchmark_single_function`** - Benchmark a simple function with multiple runs
2. **`test_benchmark_result_summary`** - Verify `print_summary()` output
3. **`test_benchmark_result_to_json`** - Export benchmark results to JSON file

### Prerequisites

- None (uses simple in-memory operations)

### Run

```bash
cargo test --test test_benchmark -- --nocapture
```

### Test Comparison

| Test | Output | Duration |
| :--- | :--- | :--- |
| `test_benchmark_single_function` | Console stats | <1s |
| `test_benchmark_result_summary` | Console summary | <1s |
| `test_benchmark_result_to_json` | `.json` file | <1s |

---

## Streaming Module Tests

**File**: `tests/test_streaming.rs`

### Tests

1. **`test_stream_extractor_single_frame`** - Extract single frame with Custom strategy
2. **`test_stream_extractor_every_nth`** - Extract with EveryNth(50) strategy
3. **`test_stream_extractor_first_n`** - Extract with FirstN(5) strategy
4. **`test_stream_extractor_range`** - Extract with Range(0, 10) strategy
5. **`test_stream_extractor_scale_factor`** - Compare scale factors (100%, 50%, 25%)

### Prerequisites

- Test video `data/test.mp4`, generated automatically (see [Test Video](#test-video))

### Run

```bash
cargo test --test test_streaming -- --nocapture
```

### Test Comparison

| Test | Strategy | Output | Duration |
| :--- | :--- | :--- | :--- |
| `test_stream_extractor_single_frame` | Custom | 1 frame | <1s |
| `test_stream_extractor_every_nth` | EveryNth(50) | 6 frames | ~1s |
| `test_stream_extractor_first_n` | FirstN(5) | 5 frames | <1s |
| `test_stream_extractor_range` | Range(0,10) | 10 frames | <1s |
| `test_stream_extractor_scale_factor` | FirstN(1) | Size comparison | ~2s |

---

## Video Process Module Tests

**File**: `tests/test_video_process.rs`

### Tests

1. **`test_frame_extractor_opencv_interval`** - Extract frames with OpenCVInterval mode + SingleDirectory
2. **`test_frame_extractor_parallel`** - Parallel extraction with Rayon + SingleDirectory
3. **`test_frame_extractor_multiple_directory`** - MultipleDirectory save mode (subdirectory per video)

### Prerequisites

- Test video `data/test.mp4`, generated automatically (see [Test Video](#test-video))

### Run

```bash
cargo test --test test_video_process -- --nocapture
```

### Test Comparison

| Test | Mode | SaveMode | Duration |
| :--- | :--- | :--- | :--- |
| `test_frame_extractor_opencv_interval` | OpenCVInterval | SingleDirectory | ~2s |
| `test_frame_extractor_parallel` | Parallel | SingleDirectory | ~1s |
| `test_frame_extractor_multiple_directory` | OpenCVInterval | MultipleDirectory | ~2s |
