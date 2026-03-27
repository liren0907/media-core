# Frontend Gaps Analysis

Backend vs Frontend coverage review (2026-03-26).

---

## 1. Backend Features with No Frontend UI

### Local Video to HLS Conversion
- **Backend**: `execute_hls_pipeline` — converts local video to HLS segments + playlist
- **Gap**: Stream page only handles RTSP→HLS. No UI for local file→HLS conversion
- **Priority**: High

### Full Analysis Pipeline
- **Backend**: `execute_full_analysis_pipeline` — runs metadata + motion detection in one call
- **Gap**: Analysis page only exposes motion/similarity/compare separately, no "full analysis" one-click entry
- **Priority**: Medium

### Standalone Metadata Viewer
- **Backend**: `get_media_metadata` / `check_media_support` — rich metadata extraction
- **Gap**: Only used indirectly via `execute_metadata_pipeline` in Video Viewer. No dedicated quick-inspect tool for any media file
- **Priority**: Low (Video Viewer partially covers this)

### Benchmark Suite
- **Backend**: `run_benchmark_suite` — runs all benchmark operations at once
- **Gap**: Benchmark page only uses `run_benchmark` with manual operation selection, no "Run All" button
- **Priority**: Low

---

## 2. Existing Pages with Incomplete Feature Coverage

### Stream Page
- [ ] Add config validation button — backend `validate_config` exists but is not wired to any UI element
- [ ] Support loading config from file — backend `process_rtsp_config` accepts a config file path, frontend only has manual URL input

### Video Viewer (`/viewVideo`)
- [ ] Display all metadata fields returned by backend: `quality_category`, `estimated_memory_mb`, `color_space`, `bit_depth`, `aspect_ratio`, `bitrate_mbps`, `total_pixels`

### HLS Viewer (`/hlsViewer`)
- [ ] Periodic polling of `get_hls_status` to show real-time segment generation progress, not just initial load

### Pipeline Page (`/pipeline`)
- [ ] Expose per-operation parameters: motion algorithm/threshold, HLS segment duration, extraction strategy
- [ ] Show clear "Cancelled" status after `cancel_pipeline` is called

### Benchmark Page (`/benchmark`)
- [ ] Add "Run All" button calling `run_benchmark_suite`
- [ ] Result history or export functionality

### Camera Page (`/camera`)
- [ ] Improve camera discovery — `get_available_cameras` returns a static list (0-10), consider filtering by `check_camera_access` results

---

## 3. UX Improvements

### Inferencer vs Annotator Overlap
- Both pages call `start_video_annotation` with nearly identical workflows
- **Action**: Merge into one page or clearly differentiate their purpose (e.g., Inferencer = view/inspect, Annotator = process/export)

### Error Handling
- Most pages show generic error messages from `catch` blocks
- **Action**: Parse backend error strings and provide actionable guidance per error type

### No Unified Media Browser
- Every page implements its own file picker independently
- **Action**: Consider a shared media library / recent files component

---

## 4. Suggested Priority Order

1. Local Video → HLS conversion UI (new feature, high utility)
2. Stream page: add validate config button (trivial, backend ready)
3. Merge or clarify Inferencer / Annotator
4. Benchmark "Run All" button
5. Video Viewer: show full metadata fields
6. Pipeline page: per-operation parameter inputs
7. HLS Viewer: periodic status polling
