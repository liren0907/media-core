# Pipeline Integration Documentation

This document describes the pipeline-based command layer and frontend integrations implemented for the Media Core application.

## Overview

The implementation provides a unified interface between the Svelte frontend and the Rust `media_core` crate's pipeline builder pattern. All media processing operations now flow through typed pipeline commands that leverage the `Pipeline<MediaContext>` architecture.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (Svelte)                        │
│  Dashboard │ ViewVideo │ Analysis │ Stream │ HLS Viewer         │
│  Camera    │ Inferencer │ Multi-Stream                          │
│                                                                  │
│  src/lib/events.ts (Event utilities & batch processing)         │
└──────────────────────────┬──────────────────────────────────────┘
                           │ invoke() + listen()
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Tauri Command Layer                           │
│  commands/pipeline.rs   (Pipeline execution + batch processing)  │
│  commands/system.rs     (System metrics + stream stats)          │
│  commands/events.rs     (Real-time events + pipeline tracking)   │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                      media_core Crate                            │
│  Pipeline<MediaContext> + PipelineStep implementations           │
│  ├── metadata/pipeline.rs   (ExtractMetadata)                    │
│  ├── analysis/pipeline.rs   (DetectMotion, GroupSimilarImages)   │
│  ├── streaming/pipeline.rs  (ExtractFrames)                      │
│  ├── hls/pipeline.rs        (ConvertToHLS)                       │
│  ├── video_process/pipeline.rs (ExtractFramesToDisk)             │
│  └── camera/pipeline.rs     (CaptureFrame)                       │
└─────────────────────────────────────────────────────────────────┘
```

---

## Backend Commands

### Pipeline Commands (`src-tauri/src/commands/pipeline.rs`)

All pipeline commands follow a consistent pattern:
1. Accept a configuration struct (deserialized from frontend JSON)
2. Create a `MediaContext` from the source
3. Build a `Pipeline` with appropriate steps
4. Execute and return serializable results

#### Metadata Pipeline

```typescript
// Frontend usage
const metadata = await invoke('execute_metadata_pipeline', {
  filePath: '/path/to/video.mp4',
  includeThumbnail: true
});
```

**Returns:** `MediaMetadata` object with:
- `filename`, `filePath`, `fileSize`, `mimeType`
- `duration`, `width`, `height`, `fps`, `frameCount`
- `bitrate`, `codec`, `audioCodec`, `audioSampleRate`, `audioChannels`
- `createdAt`, `modifiedAt`, `thumbnail` (base64)

---

#### Motion Detection Pipeline

```typescript
const result = await invoke('execute_motion_pipeline', {
  config: {
    videoPath: '/path/to/video.mp4',
    outputDir: '/optional/debug/output',  // optional
    algorithm: 'frame_diff',  // 'frame_diff' | 'mog2' | 'knn' | 'optical_flow'
    threshold: 25.0,          // 1-100
    minArea: 500              // minimum pixel area
  }
});
```

**Returns:** `AnalysisResult` with:
```typescript
{
  motionEvents: [
    { startFrame: 120, endFrame: 180, eventType: 'Motion' },
    // ...
  ],
  similarityGroups: [],
  imageComparison: null
}
```

**Algorithm Options:**
| Algorithm | Description | Speed |
|-----------|-------------|-------|
| `frame_diff` | Simple frame difference | Fastest |
| `mog2` | MOG2 background subtraction | Medium |
| `knn` | KNN background subtraction | Medium |
| `optical_flow` | Dense optical flow | Slowest |

---

#### Similarity Grouping Pipeline

```typescript
const result = await invoke('execute_similarity_pipeline', {
  config: {
    inputDir: '/path/to/images',
    outputDir: '/path/to/grouped',
    method: 'phash',      // 'phash' | 'histogram' | 'feature'
    threshold: 0.95,      // 0.0-1.0
    minGroupSize: 2
  }
});
```

**Returns:** `AnalysisResult` with:
```typescript
{
  motionEvents: [],
  similarityGroups: [
    { groupName: 'group_001', members: ['img1.jpg', 'img2.jpg'] },
    // ...
  ],
  imageComparison: null
}
```

**Method Options:**
| Method | Description | Best For |
|--------|-------------|----------|
| `phash` | Perceptual hash | General use, recommended |
| `histogram` | Color histogram | Color-based matching |
| `feature` | SIFT feature matching | Rotated/scaled images |

---

#### Image Comparison Pipeline

```typescript
const result = await invoke('execute_compare_pipeline', {
  config: {
    image1: '/path/to/image1.jpg',
    image2: '/path/to/image2.jpg',
    method: 'phash',
    threshold: 0.95
  }
});
```

**Returns:** `AnalysisResult` with:
```typescript
{
  motionEvents: [],
  similarityGroups: [],
  imageComparison: {
    image1: '/path/to/image1.jpg',
    image2: '/path/to/image2.jpg',
    similarityScore: 0.92,
    isDuplicate: false
  }
}
```

---

#### Frame Extraction Pipeline (In-Memory)

```typescript
const frames = await invoke('execute_extract_frames_pipeline', {
  config: {
    videoPath: '/path/to/video.mp4',
    strategy: 'every_nth',    // 'every_nth' | 'first_n' | 'range' | 'keyframes'
    strategyParam: 30,        // N for every_nth/first_n
    rangeStart: 0,            // for 'range' strategy
    rangeEnd: 100,
    scaleFactor: 0.5          // optional, 0.0-1.0
  }
});
```

**Returns:** Array of `FrameDataResult`:
```typescript
[
  { index: 0, data: 'base64...' },
  { index: 30, data: 'base64...' },
  // ...
]
```

---

#### HLS Conversion Pipeline

```typescript
const result = await invoke('execute_hls_pipeline', {
  config: {
    videoPath: '/path/to/video.mp4',
    outputDir: '/path/to/hls_output',
    segmentDuration: 4,       // seconds
    playlistName: 'stream',   // creates stream.m3u8
    profile: 'main',          // H.264 profile
    level: '4.0'              // H.264 level
  }
});
```

**Returns:** `HLSConversionResult`:
```typescript
{
  outputDir: '/path/to/hls_output',
  playlistPath: '/path/to/hls_output/stream.m3u8',
  segmentCount: 42,
  success: true
}
```

---

#### Frame Extraction to Disk Pipeline

```typescript
const result = await invoke('execute_extract_to_disk_pipeline', {
  config: {
    videoPath: '/path/to/video.mp4',
    outputDir: '/path/to/frames',
    interval: 30,             // every 30th frame
    extractionMode: 'opencv_interval',  // 'opencv_sequential' | 'opencv_interval' | 'ffmpeg' | 'parallel'
    saveMode: 'single'        // 'single' | 'multiple'
  }
});
```

**Returns:** `ExtractToDiskResult`:
```typescript
{
  outputDir: '/path/to/frames',
  framesExtracted: 120,
  extractionMode: 'OpenCVInterval',
  saveMode: 'SingleDirectory',
  success: true
}
```

---

#### Camera Capture Pipeline

```typescript
const result = await invoke('execute_camera_pipeline', {
  config: {
    cameraId: 0,
    outputPath: '/optional/path/to/save.jpg'  // optional
  }
});
```

**Returns:** `CameraCaptureResult`:
```typescript
{
  cameraId: 0,
  outputPath: '/optional/path/to/save.jpg',
  frameData: 'base64...',  // always returned
  success: true
}
```

---

#### Full Analysis Pipeline

Combines metadata extraction and motion detection in a single pipeline execution:

```typescript
const result = await invoke('execute_full_analysis_pipeline', {
  videoPath: '/path/to/video.mp4',
  includeThumbnail: true,
  detectMotion: true,
  motionThreshold: 25.0
});
```

**Returns:** `FullAnalysisResult`:
```typescript
{
  metadata: { /* MediaMetadata */ },
  analysis: { /* AnalysisResult with motionEvents */ }
}
```

---

### System Metrics Commands (`src-tauri/src/commands/system.rs`)

#### Get System Metrics

```typescript
const metrics = await invoke('get_system_metrics');
```

**Returns:** `SystemMetrics`:
```typescript
{
  cpu: {
    usagePercent: 45.2,
    coreCount: 8,
    frequencyMhz: null
  },
  memory: {
    totalGb: 16.0,
    usedGb: 9.6,
    availableGb: 6.4,
    usagePercent: 60.0
  },
  disk: {
    totalGb: 500.0,
    usedGb: 300.0,
    availableGb: 200.0,
    usagePercent: 60.0,
    readSpeedMbps: null,
    writeSpeedMbps: null
  },
  uptimeSeconds: 3600
}
```

**Platform Notes:**
- macOS: Uses `sysctl`, `vm_stat`, and `df` for real metrics
- Other platforms: Returns reasonable fallback values

---

#### Get Stream Statistics

```typescript
const stats = await invoke('get_stream_stats');
```

**Returns:** `StreamStats`:
```typescript
{
  activeCount: 2,
  totalCount: 5,
  avgLatencyMs: 45.0,
  totalBitrateKbps: 8000,
  streams: [
    {
      id: 'stream_0',
      name: 'camera_feed',
      status: 'active',    // 'active' | 'idle' | 'error' | 'paused'
      streamType: 'hls',
      codec: 'H.264',
      resolution: '1920x1080',
      fps: 30.0,
      bitrateKbps: 4000,
      durationSeconds: null,
      latencyMs: 45
    }
  ]
}
```

---

#### Get Throughput History

```typescript
const history = await invoke('get_throughput_history', { periodSeconds: 3600 });
```

**Returns:** `ThroughputHistory`:
```typescript
{
  points: [
    { timestamp: 1707100800, networkMbps: 15.2, fps: 28.5, cpuPercent: 35.0 },
    // ... 20 data points
  ],
  periodSeconds: 3600
}
```

---

## Frontend Pages

### Dashboard (`src/routes/+page.svelte`)

Real-time system monitoring dashboard with:

- **CPU Card**: Live CPU percentage with 8-bar history chart
- **Memory Card**: Usage percentage with progress bar and GB values
- **RAM Card**: Available memory with visual indicator
- **Throughput Chart**: Network and FPS over time (1H/24H periods)
- **Storage Panel**: Disk usage donut chart with total/write speed
- **Active Streams Table**: Live stream status with codec, latency, status badges

**Polling Interval:** 2 seconds for metrics, streams update in real-time.

---

### Video Player (`src/routes/viewVideo/+page.svelte`)

Enhanced video player with full metadata extraction:

- Native HTML5 video playback
- **Video Information Panel**: Duration, resolution, FPS, frame count
- **Technical Details**: Video/audio codecs, bitrate, file size, MIME type, sample rate
- **Preview Panel**: Thumbnail from pipeline, file dates

Uses `execute_metadata_pipeline` for comprehensive metadata extraction.

---

### Analysis Page (`src/routes/analysis/+page.svelte`)

Three-mode analysis interface:

#### Motion Detection Mode
- Video file selection
- Algorithm picker (Frame Diff, MOG2, KNN, Optical Flow)
- Threshold slider (1-100)
- Minimum area slider (100-5000 px)
- Results: Table of motion events with frame ranges

#### Similarity Grouping Mode
- Input/output directory selection
- Method picker (Perceptual Hash, Histogram, Feature Matching)
- Threshold slider (50-100%)
- Minimum group size
- Results: Cards showing similar image groups

#### Image Comparison Mode
- Two image file selection
- Method picker
- Duplicate threshold slider
- Results: Large similarity score with duplicate/different verdict

---

## File Changes Summary

### New Files Created

| File | Purpose |
|------|---------|
| `src-tauri/src/commands/pipeline.rs` | Pipeline executor commands (600 lines) |
| `src-tauri/src/commands/system.rs` | System metrics commands (350 lines) |
| `src/routes/analysis/+page.svelte` | Analysis UI page |

### Modified Files

| File | Changes |
|------|---------|
| `src-tauri/src/commands/mod.rs` | Added `pipeline` and `system` modules |
| `src-tauri/src/main.rs` | Registered 15 new commands |
| `src/routes/+page.svelte` | Dashboard with real metrics |
| `src/routes/viewVideo/+page.svelte` | Pipeline-based metadata |
| `src/routes/+layout.svelte` | Added Analysis and Video Player nav links |

---

## Usage Examples

### Running a Full Video Analysis

```typescript
import { invoke } from '@tauri-apps/api/core';

async function analyzeVideo(videoPath: string) {
  // Get metadata + motion in one call
  const result = await invoke('execute_full_analysis_pipeline', {
    videoPath,
    includeThumbnail: true,
    detectMotion: true,
    motionThreshold: 20.0
  });

  console.log('Duration:', result.metadata?.duration);
  console.log('Motion events:', result.analysis?.motionEvents.length);
}
```

### Monitoring System Health

```typescript
import { invoke } from '@tauri-apps/api/core';

async function monitorSystem() {
  setInterval(async () => {
    const metrics = await invoke('get_system_metrics');
    
    if (metrics.cpu.usagePercent > 90) {
      console.warn('High CPU usage!');
    }
    
    if (metrics.memory.usagePercent > 85) {
      console.warn('Low memory!');
    }
  }, 2000);
}
```

### Finding Duplicate Images

```typescript
import { invoke } from '@tauri-apps/api/core';

async function findDuplicates(inputDir: string, outputDir: string) {
  const result = await invoke('execute_similarity_pipeline', {
    config: {
      inputDir,
      outputDir,
      method: 'phash',
      threshold: 0.95,
      minGroupSize: 2
    }
  });

  for (const group of result.similarityGroups) {
    console.log(`Found ${group.members.length} similar images in ${group.groupName}`);
  }
}
```

---

## Error Handling

All pipeline commands return `Result<T, String>` where errors are human-readable messages:

```typescript
try {
  const result = await invoke('execute_motion_pipeline', { config });
} catch (error) {
  // error is a string like "Motion detection failed: File not found"
  console.error('Pipeline error:', error);
}
```

Common error patterns:
- `"Pipeline execution failed: ..."` - General pipeline error
- `"No metadata extracted"` - File couldn't be parsed
- `"Thread panicked"` - Unexpected runtime error (rare)

---

## Event System

The application supports real-time event-driven updates from the backend using Tauri's event system. This enables push-based updates instead of polling.

### Event Types

| Event Name | Payload Type | Description |
|------------|--------------|-------------|
| `pipeline:progress` | `PipelineProgressEvent` | Progress updates during pipeline execution |
| `pipeline:complete` | `PipelineCompleteEvent` | Emitted when a batch pipeline completes |
| `stream:status` | `StreamStatusEvent` | Stream state changes |
| `system:metrics` | `SystemMetricsEvent` | System resource metrics |

### Frontend Event Utilities (`src/lib/events.ts`)

```typescript
import { 
  onPipelineProgress, 
  onSystemMetrics,
  startMetricsStream,
  stopMetricsStream,
  cancelPipeline,
  executeBatchPipeline
} from '$lib/events';

// Subscribe to pipeline progress
const unlisten = await onPipelineProgress((event) => {
  console.log(`${event.stepName}: ${event.progressPercent.toFixed(1)}%`);
  console.log(event.message);
});

// Start receiving system metrics every 2 seconds
await startMetricsStream(2000);

// Listen for metrics
const unlistenMetrics = await onSystemMetrics((metrics) => {
  console.log(`CPU: ${metrics.cpuPercent}%, Memory: ${metrics.memoryPercent}%`);
});

// Stop metrics when done
await stopMetricsStream();

// Cleanup
unlisten();
unlistenMetrics();
```

### Event Commands

#### Start Metrics Streaming
```typescript
await invoke('start_metrics_stream', { intervalMs: 2000 });
```

#### Stop Metrics Streaming
```typescript
await invoke('stop_metrics_stream');
```

---

## Batch Processing

Process multiple files in a single operation with progress tracking and cancellation support.

### Configuration

```typescript
interface BatchProcessConfig {
  filePaths: string[];           // Array of file paths to process
  operation: 'metadata' | 'motion' | 'hls' | 'extract';
  parallel?: boolean;            // Run in parallel (default: false)
  options?: Record<string, unknown>;  // Operation-specific options
}
```

### Usage

```typescript
import { executeBatchPipeline, onPipelineProgress, cancelPipeline } from '$lib/events';

// Subscribe to progress updates
const unlisten = await onPipelineProgress((event) => {
  updateProgressBar(event.progressPercent);
  showMessage(event.message);
});

// Execute batch processing
const result = await executeBatchPipeline({
  filePaths: ['/video1.mp4', '/video2.mp4', '/video3.mp4'],
  operation: 'metadata',
  parallel: true
});

console.log(`Processed ${result.succeeded}/${result.total} files`);

// Check individual results
for (const item of result.results) {
  if (item.success) {
    console.log(`${item.filePath}: OK`);
  } else {
    console.error(`${item.filePath}: ${item.error}`);
  }
}

unlisten();
```

### Cancellation

```typescript
// Cancel a running pipeline by its ID
const cancelled = await cancelPipeline(pipelineId);

// Or use the tracker helper
import { createPipelineTracker } from '$lib/events';

const tracker = createPipelineTracker('my-batch-operation');

tracker
  .onProgress((event) => console.log(event.message))
  .onComplete((event) => console.log(`Done: ${event.succeeded}/${event.total}`));

await tracker.start();

// Later, to cancel:
await tracker.cancel();

// Cleanup when done
await tracker.cleanup();
```

---

## Pipeline Tracking Commands

### Start Tracking
```typescript
const id = await invoke('start_pipeline_tracking', { pipelineId: 'my-operation' });
```

### Cancel Pipeline
```typescript
const cancelled = await invoke('cancel_pipeline', { pipelineId: 'my-operation' });
```

### Check Cancellation Status
```typescript
const isCancelled = await invoke('is_pipeline_cancelled', { pipelineId: 'my-operation' });
```

### Finish Tracking
```typescript
await invoke('finish_pipeline_tracking', { pipelineId: 'my-operation' });
```

### Get Active Pipelines
```typescript
const activePipelines: string[] = await invoke('get_active_pipelines');
```

---

## HLS Viewer Enhancements

The HLS Viewer page now displays real-time stream quality and latency metrics:

### Stream Quality Metrics
- **Resolution**: Current video resolution and frame rate
- **Bitrate**: Encoded bitrate and estimated bandwidth
- **Codec**: Video codec information
- **Quality Level**: Current ABR level with manual override

### Buffer Health
- **Buffer Length**: Seconds of buffered content
- **Status**: Healthy (>3s), Warning (1-3s), or Critical (<1s)

### Latency Metrics
- **Current Latency**: Live stream latency
- **Target Latency**: Configured target for low-latency mode
- **Drift**: Deviation from target

### Playback Quality
- **Dropped Frames**: Count and percentage of dropped frames

---

## Future Enhancements

1. **GPU metrics**: Add NVIDIA GPU monitoring via `nvml-wrapper` crate
2. **Result caching**: Cache metadata for frequently accessed files
3. **WebSocket streams**: Alternative to Tauri events for web builds
4. **Pipeline composition**: Chain multiple pipelines with intermediate results
