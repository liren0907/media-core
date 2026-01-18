use opencv::prelude::*;
use serde_json::json;

use crate::handlers::{DialogHandler, ImageHandler, VideoHandler};

#[tauri::command]
pub fn get_video_info(filename: &str) -> Result<String, String> {
    let video = opencv::videoio::VideoCapture::from_file(filename, opencv::videoio::CAP_ANY)
        .map_err(|e| format!("Failed to open video file: {}", e))?;

    let fps = video
        .get(opencv::videoio::CAP_PROP_FPS)
        .map_err(|e| format!("Failed to get FPS: {}", e))?;
    let frame_count = video
        .get(opencv::videoio::CAP_PROP_FRAME_COUNT)
        .map_err(|e| format!("Failed to get frame count: {}", e))?;
    let width = video
        .get(opencv::videoio::CAP_PROP_FRAME_WIDTH)
        .map_err(|e| format!("Failed to get width: {}", e))?;
    let height = video
        .get(opencv::videoio::CAP_PROP_FRAME_HEIGHT)
        .map_err(|e| format!("Failed to get height: {}", e))?;

    let codec = video
        .get(opencv::videoio::CAP_PROP_FOURCC)
        .map_err(|e| format!("Failed to get codec: {}", e))? as i32;

    let codec_str = format!(
        "{}{}{}{}",
        (codec & 0xFF) as u8 as char,
        ((codec >> 8) & 0xFF) as u8 as char,
        ((codec >> 16) & 0xFF) as u8 as char,
        ((codec >> 24) & 0xFF) as u8 as char
    );

    let codec_name = match codec_str.as_str() {
        "avc1" | "h264" => "H264",
        "hev1" | "hvc1" => "H265",
        "mp4v" => "MPEG-4 Part 2",
        "mp4a" => "MPEG-4 AAC",
        _ => "Unknown",
    }
    .to_string();

    let duration = if fps > 0.0 { frame_count / fps } else { 0.0 };
    let minutes = (duration / 60.0) as i32;
    let seconds = (duration % 60.0) as i32;
    let resolution = format!("{}x{}", width as i32, height as i32);

    // Return the JSON object directly, not as a string
    Ok(json!({
        "filename": filename,
        "fps": fps,
        "frame_count": frame_count,
        "width": width,
        "height": height,
        "resolution": resolution,
        "codec_str": codec_str,
        "codec_name": codec_name,
        "duration_seconds": duration,
        "duration": {
            "total_seconds": duration,
            "minutes": minutes,
            "seconds": seconds
        }
    })
    .to_string())
}

#[tauri::command]
pub fn read_video_file(file_path: String) -> Result<String, String> {
    // Convert the file path to a URL that can be used by the video player
    Ok(format!("asset://localhost/{}", file_path))
}

#[tauri::command]
pub fn read_image_file(file_path: String) -> Result<String, String> {
    ImageHandler::read_image_info(&file_path)
}

#[tauri::command]
pub fn open_file_dialog() -> Result<String, String> {
    DialogHandler::select_video_file()
}

#[tauri::command]
pub async fn extract_all_frames_ffmpeg() -> Result<(), String> {
    let filename = DialogHandler::select_video_file()?;
    
    // Use rfd for blocking dialog in backend
    let output_dir = rfd::FileDialog::new()
        .set_title("Select Output Directory for Frames")
        .pick_folder()
        .ok_or_else(|| "No output directory selected".to_string())?;
        
    VideoHandler::extract_all_frames_ffmpeg(&filename, output_dir.to_str().unwrap()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn extract_frames_ffmpeg_interval(frame_interval: usize) -> Result<(), String> {
    let filename = DialogHandler::select_video_file()?;
    
    // Use rfd for blocking dialog in backend
    let output_dir = rfd::FileDialog::new()
        .set_title("Select Output Directory for Frames")
        .pick_folder()
        .ok_or_else(|| "No output directory selected".to_string())?;

    VideoHandler::extract_frames_ffmpeg_interval(&filename, output_dir.to_str().unwrap(), frame_interval)
        .map_err(|e| e.to_string())
}
