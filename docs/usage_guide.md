# Usage Guide

This guide explains how to use the Video Annotation and Streaming application. Since this is a Tauri-based GUI application, all operations are performed through the user interface.

## 1. Annotating a Video

This workflow allows you to overlay annotation data (like bounding boxes and labels) onto a video file. This is done on the `/annotator` page.

### Step-by-Step Instructions

1.  **Open the Application**: Launch the application to see the main dashboard.

2.  **Navigate to the Annotator**: Click on the **"Video Annotation Tool"** card from the main dashboard to go to the `/annotator` page.

3.  **Select a Video File**:
    -   Click the **"Open Video"** button.
    -   A file dialog will appear. Navigate to and select the video file you want to process (e.g., `.mp4`, `.avi`).
    -   A preview of the video and its technical details will be displayed.

4.  **Select an Annotation File**:
    -   Click the **"Open Annotation"** button.
    -   Select the corresponding JSON file that contains the annotation data for your video.
    -   The application will load the file and display a list of all object labels found.

5.  **Filter Labels**:
    -   Below the file selection, you will see a list of all unique labels from the annotation file with checkboxes.
    -   By default, all labels are selected. Uncheck any labels you wish to *exclude* from the final annotated video.

6.  **Choose an Output Directory**:
    -   Click the **"Start Annotation"** (or **"Process Video"**) button.
    -   A dialog will ask you to select a directory where the output video will be saved.

7.  **Processing**:
    -   The application will begin processing. A progress indicator may be shown.
    -   The backend reads the video frame by frame, draws the bounding boxes and labels for your selected objects, and encodes a new video file.

8.  **View the Result**:
    -   Once complete, navigate to the output directory you selected to find the processed video.
    -   If using the `/inferencer` page, the resulting video will be automatically displayed in the UI.

## 2. Streaming RTSP

The application can connect to and stream live video from RTSP sources with two playback modes.

### Step-by-Step Instructions

1.  **Navigate to the Stream Manager**: Click on the **"Stream Manager"** card from the main dashboard.

2.  **Enter the RTSP URL**: Input your RTSP stream URL in the provided field.

3.  **Choose a Playback Mode**:

    **Mode A: In-App HLS Playback**
    -   Click the **"Play in App"** button.
    -   The stream will be transcoded to HLS and played directly in the application's video player.
    -   Use this for convenient viewing without leaving the app.

    **Mode B: Native Window Playback**
    -   Click the **"Play in New Window"** button.
    -   A new native OS window will open, displaying the raw RTSP feed.
    -   Use this for low-latency, real-time viewing.

## 3. Viewing a Video

The `/viewVideo` page is a simple tool for quick video playback and inspection.

1.  **Navigate to the Video Viewer Page**.
2.  Click **"Select Video"**.
3.  Choose a local video file.
4.  The video will load in the player, and its technical details (codec, resolution, etc.) will be displayed below.

## 4. Viewing an HLS Stream

The `/hlsViewer` page allows you to monitor and watch a live HLS stream being generated from an RTSP source by the backend.

### Step-by-Step Instructions

1.  **Navigate to the HLS Viewer Page**.
2.  **Start the HLS Stream**: Ensure that you have already configured and started an RTSP-to-HLS stream using the `/stream` page.
3.  **Monitor Status**:
    -   The "HLS Status Panel" will automatically update every few seconds.
    -   It shows whether the stream is `active`, if the `playlist.m3u8` file exists, and the current number of video `segments`.
4.  **Load and Play the Stream**:
    -   Click the **"Load Playlist"** button.
    -   The application will load the stream into the video player. It may take a moment to buffer before playback begins.
    -   The video will play the live stream, continuously fetching new segments as they become available.

## 5. Multi-Stream Viewer

The **Multi-Stream Viewer** is designed for monitoring multiple worksite cameras simultaneously.

1.  Click on the **"Multi-Stream Viewer"** card from the main dashboard.
2.  The application will load and display all configured HLS streams in a grid layout.