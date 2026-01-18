# HLS Playback Architecture and Technical Rationale

## 1. Objective

The primary goal is to enable users to watch a live RTSP (Real-Time Streaming Protocol) video stream directly within the application's user interface. To achieve this, we convert the RTSP stream into the HLS (HTTP Live Streaming) format, which is widely supported by web-based video players.

## 2. The Core Challenge: The Tauri Security Sandbox

Tauri applications are composed of two distinct parts with different privilege levels:

-   **Rust Backend**: A native application with high privileges. It can freely access the local file system, execute commands (like FFmpeg), and perform other system-level operations.
-   **Svelte Frontend**: A web-based interface running inside a sandboxed WebView. For security reasons, the frontend is treated like a public website and is **prohibited from directly accessing the local file system**.

This security model is fundamental to protecting users from malicious web content and is not something that can be disabled.

## 3. Initial Implementation & The "Path" Problem

The process begins when the user clicks "Play in App":

1.  **Frontend `invoke`s Rust**: The Svelte frontend calls a Rust function (`start_hls_playback`).
2.  **Rust Creates Files**: The Rust backend uses FFmpeg to transcode the RTSP stream into HLS segments (`.ts` files) and a manifest (`.m3u8` file) in a local directory (e.g., `hls_media_dev/`).
3.  **Rust Returns a Path**: The Rust function returns the absolute file system path to the `playlist.m3u8` manifest file.

Here, we hit the first problem: The frontend receives a path like `"/Users/user/app/hls_media_dev/playlist.m3u8"`, but it cannot use this path directly due to the security sandbox.

## 4. Tauri's Bridge: The `asset://` Protocol

Tauri provides a specific solution for this problem: the `asset` custom protocol.

-   The frontend uses a special function, `convertFileSrc`, to transform the file system path into a special URL, like `asset://localhost/%2FUsers%2F...%2Fplaylist.m3u8`.
-   This URL is handled by a built-in Tauri server that acts as a secure bridge, fetching the local file and serving it to the frontend.

## 5. The Root Cause of Failure: `asset://` Protocol Limitations

Our debugging revealed the critical point of failure:

-   **SUCCESS**: The frontend successfully used the `asset://` URL to fetch and load the `.m3u8` playlist file. This proved the file was being created correctly and the initial path was correct.
-   **FAILURE**: When the HLS player (`hls.js`) then tried to fetch the `.ts` video segment files (as instructed by the playlist), the Tauri asset server returned a `500 Internal Server Error`.

This indicates a limitation or a bug within Tauri's asset protocol handler when dealing with binary media files (`.ts`) as opposed to simple text files (`.m3u8`). The server itself was crashing when trying to serve the video data.

## 6. The Final Solution: A Dedicated Local HTTP Server

Since the official bridge (`asset://`) is unreliable for our use case, we must create our own, more robust bridge. The most standard and reliable way to do this is with a local HTTP server.

The architecture is as follows:

1.  **Server Launch**: When `start_hls_playback` is called, the Rust backend spawns a lightweight, local-only HTTP server in a background thread. This server binds to a specific port (e.g., `127.0.0.1:1521`) and is configured to serve files exclusively from our `hls_media_dev` directory.
2.  **Return a Real URL**: Instead of returning a file path, the Rust function now returns a standard, universally understood HTTP URL, such as `http://127.0.0.1:1521/playlist.m3u8`.
3.  **Frontend Simplicity**: The Svelte frontend receives this standard URL. No special conversion (`convertFileSrc`) is needed. It passes the URL directly to the `hls.js` player.
4.  **Reliable Playback**: The HLS player now communicates directly with our custom, robust HTTP server over the standard HTTP protocol, which can reliably serve any type of file.

This approach completely bypasses the faulty `asset://` protocol for media files and resolves the playback issue in a stable and standards-compliant way. 