# RTSP HLS Configuration (`rtsp_hls_config.json`)

This document outlines the structure and purpose of the `rtsp_hls_config.json` file, which is used to configure RTSP stream processing and HLS output.

## Root Object Structure

The JSON configuration is an array of stream configuration objects. Each object in the array represents a unique RTSP source to be processed.

```json
[
  {
    "rtsp_url": "rtsp://example.com/stream1",
    "output_directory": "media/hls/stream1",
    "show_preview": false,
    "saving_option": "single",
    "saved_time_duration": 10,
    "use_fps": false,
    "fps": 30.0,
    "hls": {
      "enabled": true,
      "segment_duration": 10,
      "playlist_size": 5
    }
  }
]
```

## Stream Object Fields

Each object in the top-level array has the following fields:

-   **`rtsp_url`** (string, required): The full URL of the source RTSP stream.
-   **`output_directory`** (string, required): The path to the directory where HLS files (`.m3u8` playlist and `.ts` video segments) will be saved. Each stream should have a unique output directory.
-   **`show_preview`** (boolean, optional): If `true`, the application will attempt to open a preview window to display the stream in real-time. Defaults to `false`.
-   **`saving_option`** (string, optional): Defines the recording strategy. Can be `"single"` (only this stream), `"list"` (if part of a larger process), or `"both"`. Defaults to `"single"`.
-   **`saved_time_duration`** (integer, optional): The duration in seconds for each video segment if saving as regular video files (not HLS). Defaults to `60`.
-   **`use_fps`** (boolean, optional): If `true`, the stream will be re-encoded to the `fps` value specified. If `false`, the backend will attempt a direct, more efficient stream copy. Defaults to `false`.
-   **`fps`** (float, optional): The target frames per second to use if `use_fps` is `true`. Defaults to `30.0`.
-   **`hls`** (object, optional): A container for HLS-specific settings.

## HLS Object Fields

The `hls` object contains the following fields:

-   **`enabled`** (boolean, required): Must be `true` to enable HLS stream generation for this RTSP source.
-   **`segment_duration`** (integer, required): The target duration in seconds for each HLS video segment (`.ts` file). A common value is between 2 and 10 seconds.
-   **`playlist_size`** (integer, required): The maximum number of segments to list in the `playlist.m3u8` file. A larger number allows clients to seek further back in a live stream.

## Usage

This file is processed by the `process_rtsp_config` backend command, which spawns background processes to handle each stream defined in the array. 