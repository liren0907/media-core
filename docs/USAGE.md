# Application Usage Guide

This guide will walk you through the exciting new features of the Intelligent Worksite Monitoring application, designed to be more powerful, user-friendly, and versatile.

---

## 1. The Main Dashboard

When you launch the application, you will now be greeted by a modern, clean dashboard. This is your new central hub for accessing all of the application's powerful features.

![image](https://i.imgur.com/your-dashboard-image.png)

The dashboard features several cards, each representing a core function:

-   **Multi-Stream Viewer**: Takes you to a grid view for monitoring several cameras at once.
-   **Stream Manager**: Your all-in-one tool for configuring, viewing, and recording individual RTSP streams.
-   **Video Annotation Tool**: The dedicated page for running AI analysis on video files.

---

## 2. Stream Manager: Viewing and Recording

The **Stream Manager**, accessible from the dashboard, is where you will handle individual video streams. This interface provides two powerful playback modes for different needs.

### How to Access
1.  Click on the **"Stream Manager"** card from the main dashboard.
2.  This will take you to the `/stream` page.

### Mode A: In-App HLS Playback (For Convenience)

This mode transcodes an RTSP stream into the web-friendly HLS format and plays it directly within the application's user interface.

**Use this mode when:** You want to quickly view a stream without leaving the app.

**How to use:**
1.  Enter your RTSP stream URL in the "Main RTSP URL" field.
2.  Click the **"Play in App"** button.
3.  The video player will appear directly on the page and begin streaming.

![image](https://i.imgur.com/your-in-app-play-image.png)

### Mode B: Native Window Playback (For Low-Latency)

This mode opens the RTSP stream directly from the source in a new, separate native OS window. This bypasses the web UI and any transcoding, resulting in the lowest possible latency.

**Use this mode when:** You need a real-time, high-performance view of a stream where every second counts.

**How to use:**
1.  Enter your RTSP stream URL in the "Main RTSP URL" field.
2.  Click the **"Play in New Window"** button.
3.  A new window will open on your desktop, displaying the raw video feed.

![image](https://i.imgur.com/your-native-window-image.png)

### Recording a Stream

The Stream Manager also allows you to record streams based on a rich set of configuration options:
1.  Fill out the form with your desired settings (e.g., additional URLs, segment duration, FPS).
2.  Click the **"Start Capture"** button at the bottom to begin recording.

---

## 3. Multi-Stream Viewer

The **Multi-Stream Viewer** is designed for monitoring multiple worksite cameras simultaneously. It presents all configured streams in a dynamic, easy-to-view grid.

**How to Access:**
1. Click on the **"Multi-Stream Viewer"** card from the main dashboard.
2. The application will load and display all pre-configured HLS streams.

![image](https://i.imgur.com/your-multi-stream-image.png)

---
## 4. Video Annotation

The powerful video annotation tool is now located on its own dedicated page for better organization.

**How to Access:**
1. Click on the **"Video Annotation Tool"** card from the main dashboard.
2. This will navigate you to the `/annotator` page where you can load a video and a JSON file to begin the annotation process. 