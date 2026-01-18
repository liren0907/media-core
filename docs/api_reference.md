# API Reference

This document provides a reference for the core Rust structs and Tauri commands used in the application.

## Core Data Structures (`video_annotation.rs`)

For detailed documentation on these data structures, please see the [Annotation Documentation](./annotation.md).

## Video Processing Structs

### `VideoAnnotator` (`video_annotation.rs`)
Handles the process of drawing annotations onto a video.
```rust
pub struct VideoAnnotator {
    // private fields
}
```
**Methods**:
- `new(video_path: &Path, output_path: &Path) -> Result<Self>`: Creates a new annotator.
- `annotate_video(&mut self, annotation: &Annotation) -> Result<()>`: Processes the video and applies annotations.

### `VideoHandler` (`video_handler.rs`)
Provides utility functions for video file operations.
```rust
pub struct VideoHandler;
```
**Methods (static)**:
- `get_video_info(filename: &str) -> Result<String, String>`: Extracts video metadata.
- `extract_all_frames_ffmpeg(filename: &str, output_dir: &str) -> Result<()>`: Extracts all frames from a video using FFmpeg.

## Tauri Commands (`main.rs`)

These are the functions exposed from the Rust backend to the Svelte frontend.

```rust
#[tauri::command]
fn get_video_info(filename: &str) -> Result<String, String>
```
**Description**: Takes a video file path and returns a JSON string with its metadata (resolution, FPS, etc.).

```rust
#[tauri::command]
fn read_video_file(file_path: String) -> Result<String, String>
```
**Description**: Reads a video file and returns a formatted string with its technical details.

```rust
#[tauri::command]
fn read_image_file(file_path: String) -> Result<String, String>
```
**Description**: Reads an image file and returns its Base64-encoded data URL.

```rust
#[tauri::command]
fn open_file_dialog() -> Result<String, String>
```
**Description**: A command to open the native file dialog (note: this is often handled by the Tauri dialog plugin directly in the frontend).

```rust
#[tauri::command]
fn extract_all_frames_ffmpeg() -> Result<(), String>
```
**Description**: Extracts all frames from a pre-configured video file using FFmpeg and saves them to an output directory.

```rust
#[tauri::command]
fn extract_frames_ffmpeg_interval(frame_interval: usize) -> Result<(), String>
```
**Description**: Extracts frames from a video at a specific interval (e.g., every Nth frame).

```rust
#[tauri::command]
fn read_annotation_file(path: &str) -> Result<Value, String>
```
**Description**: Reads a JSON annotation file and returns statistics like total frames, objects, and unique labels.

```rust
#[tauri::command]
fn start_video_annotation(payload: &str) -> Result<Value, String>
```
**Description**: Receives a JSON string payload with paths and selected labels, orchestrates the video annotation process, and returns the result status.

```rust
#[tauri::command]
fn get_hls_status(output_dir: &str) -> Result<String, String>
```
**Description**: Checks the status of an HLS stream, returning details like playlist existence and segment count.

```rust
#[tauri::command]
fn start_hls_playback(url: &str) -> Result<String, String>
```
**Description**: Starts HLS streaming from an RTSP URL and returns the URL to the generated playlist.

```rust
#[tauri::command]
fn start_direct_playback(url: &str) -> Result<(), String>
```
**Description**: Opens the RTSP stream directly in a native OS window for low-latency playback.

```rust
#[tauri::command]
fn start_multiple_hls_playback(urls: Vec<String>) -> Result<Vec<String>, String>
```
**Description**: Starts HLS playback for multiple RTSP streams and returns the playlist URLs.

### `AnnotationPayload`
Used for `start_video_annotation`.
```rust
struct AnnotationPayload {
    video_path: String,
    annotation_path: String,
    output_directory: String,
    label_selected: Vec<String>,
}
``` 