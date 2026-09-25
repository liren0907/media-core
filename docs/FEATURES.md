# Features and Logic

## Feature Summary

This table provides a quick summary of the features and corresponding Rust backend `invoke` calls for each page in the application.

| Page Route | Features | Rust Backend Functions Invoked |
| :--- | :--- | :--- |
| **`/`** | <ul><li>Central dashboard hub</li><li>Quick access to all features</li><li>Navigate to Stream Manager, Multi-Stream Viewer, or Video Annotation Tool</li></ul> | *(Navigation only)* |
| **`/annotator`** | <ul><li>Select Video & Annotation files</li><li>Display video preview & metadata</li><li>Filter annotation labels</li><li>Select output directory</li><li>Start the annotation process</li></ul> | `get_video_info`<br/>`read_annotation_file`<br/>`start_video_annotation` |
| **`/inferencer`** | <ul><li>Select Video & Annotation files</li><li>Filter annotation labels</li><li>Process the video</li><li>Display the resulting annotated video</li></ul> | `read_annotation_file`<br/>`start_video_annotation` |
| **`/viewVideo`** | <ul><li>Select a local video file</li><li>Playback the video</li><li>Display video metadata</li></ul> | `get_video_info` |
| **`/stream`** | <ul><li>Configure RTSP stream URLs</li><li>Play stream in-app (HLS) or in native window</li><li>Configure HLS settings</li><li>Start stream capture and recording</li></ul> | `process_rtsp_config`<br/>`start_hls_playback`<br/>`start_direct_playback` |
| **`/multi-stream-viewer`** | <ul><li>View multiple RTSP streams simultaneously</li><li>Grid layout for monitoring multiple cameras</li></ul> | `start_multiple_hls_playback` |
| **`/hlsViewer`** | <ul><li>Monitor the status of a live HLS stream</li><li>View the video stream from a generated playlist</li><li>Validate the HLS configuration file</li><li>Dynamically load and attach the stream to the player</li></ul> | `get_hls_status`<br/>`validate_config` |
| **(All Pages)** | <ul><li>File/Directory selection dialogs</li></ul> | *(Handled by `tauri-plugin-dialog`)* | 

## Feature Details

This document outlines the application's features, organized by frontend pages, and details the corresponding backend logic.

## 1. Main Dashboard (`/`)

The central hub of the application, providing quick access to all features.

### Features

-   **Navigation Cards**: Quick access to Multi-Stream Viewer, Stream Manager, and Video Annotation Tool.
-   **Modern UI**: Clean, professional dashboard design.

### Backend Calls

-   *(Navigation only - no direct backend calls)*

## 2. Video Annotation Page (`/annotator`)

This is the dedicated page for preparing and processing video annotations.

### Features

-   **File Input**: Allows users to select a local video file (`.mp4`, `.avi`, `.mkv`) and a corresponding JSON annotation file.
-   **Video Preview**: Displays the selected video.
-   **Video Details**: Shows metadata for the selected video, such as resolution, frame rate, and duration.
-   **Label Filtering**: Provides checkboxes for the user to select or deselect which object labels should be processed and rendered onto the video.
-   **Output Selection**: Prompts the user to choose an output directory where the final annotated video will be saved.
-   **Processing Trigger**: A button to start the annotation process.

### Backend Calls

-   `get_video_info(filename: &str)`: Invoked when a user selects a video. Uses OpenCV to extract and return video metadata.
-   `read_annotation_file(path: &str)`: Reads the JSON annotation data and returns unique labels for filtering.
-   `start_video_annotation(payload: &str)`: Triggers the video processing logic to draw annotations frame by frame.

## 3. Inferencer Page (`/inferencer`)

A streamlined page for processing a video and immediately viewing the result.

### Features

-   **Simplified Workflow**: Combines file selection, processing, and viewing into a single interface.
-   **File Input**: Select a video and an annotation file.
-   **Label Filtering**: Select which labels to process.
-   **Integrated Viewer**: After processing is complete, the resulting annotated video is displayed directly on the page for immediate review.

### Backend Calls

-   `read_annotation_file(path: &str)`: Used to get the list of available labels from the annotation file for the filtering UI.
-   `start_video_annotation(payload: &str)`: Starts the annotation process. The backend returns a success status and the path to the newly created video, which the frontend then uses to display the result.

## 4. Video Viewer Page (`/viewVideo`)

A simple utility page for inspecting video files.

### Features

-   **Video Selection**: Allows the user to open any local video file.
-   **Playback**: Provides standard video playback controls.
-   **Metadata Display**: Shows detailed technical information about the video, including codec, format, resolution, and frame rate.

### Backend Calls

-   `get_video_info(filename: &str)`: This is the main backend interaction on this page. It's called after the user selects a video to fetch and display its technical details.

## 5. RTSP Stream Manager Page (`/stream`)

This page is dedicated to configuring and managing live RTSP streams with two playback modes.

### Features

-   **Stream Configuration**: Input an RTSP stream URL.
-   **In-App HLS Playback**: Play the stream directly in the application using HLS transcoding.
-   **Native Window Playback**: Open the stream in a separate native OS window for low-latency viewing.
-   **HLS Configuration**: Configure segment duration and playlist size for HLS streaming.

### Backend Calls

-   `start_hls_playback(url: &str)`: Starts HLS streaming and returns the playlist URL for in-app playback.
-   `start_direct_playback(url: &str)`: Opens the stream in a native window for direct viewing.
-   `process_rtsp_config(config_path: &str)`: Processes an RTSP configuration file to start stream capture.

## 6. Multi-Stream Viewer Page (`/multi-stream-viewer`)

A dashboard for monitoring multiple RTSP streams simultaneously.

### Features

-   **Grid Layout**: Displays multiple streams in a responsive grid.
-   **Concurrent Streaming**: Handles multiple HLS streams efficiently.

### Backend Calls

-   `start_multiple_hls_playback(urls: Vec<String>)`: Initiates HLS playback for multiple streams. 

## 7. HLS Stream Viewer (`/hlsViewer`)

A dedicated page for monitoring and viewing live RTSP streams that have been converted to the HLS (HTTP Live Streaming) format by the backend.

### Features

-   **Live Status Monitoring**: Periodically checks and displays the status of the HLS stream, including whether the playlist is active and how many video segments are available.
-   **Dynamic Video Player**: Uses the HLS.js library to dynamically load the `playlist.m3u8` file and play the live stream.
-   **Configuration Validation**: Allows the user to trigger a validation check on the `rtsp_hls_config.json` file to ensure its format is correct before starting a stream.

### Backend Calls

-   `get_hls_status(output_dir: &str)`: Called periodically to fetch the current status of the HLS stream from the specified output directory.
-   `validate_config(config_path: &str)`: Invoked to check the syntax and structure of the HLS configuration file. 