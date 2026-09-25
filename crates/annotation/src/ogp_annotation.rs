use chrono::Local;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};

use crate::annotator::VideoAnnotator;
pub use crate::types::Annotation;

pub struct VideoAnnotation;

impl VideoAnnotation {
    fn ensure_tmp_dir() -> Result<PathBuf, String> {
        let app_tmp_dir = Path::new("tmp");
        if !app_tmp_dir.exists() {
            fs::create_dir_all(app_tmp_dir)
                .map_err(|e| format!("Failed to create tmp directory: {}", e))?;
        }
        Ok(app_tmp_dir.to_path_buf())
    }

    pub fn process_video(
        input_path: &str,
        output_path: &str,
        annotation_data: &Annotation,
    ) -> Result<(), String> {
        let input_path = Path::new(input_path);
        let output_path = Path::new(output_path);

        // Create video annotator
        let mut annotator = VideoAnnotator::new(input_path, output_path)
            .map_err(|e| format!("Failed to create video annotator: {}", e))?;

        // Annotate the video
        annotator
            .annotate_video(annotation_data)
            .map_err(|e| format!("Failed to annotate video: {}", e))?;

        Ok(())
    }

    pub fn start_video_annotation(
        video_path: &str,
        annotation_data: Annotation,
    ) -> Result<String, String> {
        // Validate video file exists
        if !Path::new(video_path).exists() {
            return Err("Video file does not exist".to_string());
        }

        // Get current timestamp in the format YYYYMMDD_HHMMSS
        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();

        // Ensure tmp directory exists
        let tmp_dir = Self::ensure_tmp_dir()?;

        // Create output filename with timestamp
        let input_filename = Path::new(video_path)
            .file_name()
            .ok_or("Invalid input filename")?
            .to_str()
            .ok_or("Invalid filename encoding")?;

        let output_filename = format!("processed_{}_{}", timestamp, input_filename);
        let output_path = tmp_dir.join(&output_filename);
        let output_path_str = output_path.to_str().ok_or("Invalid output path encoding")?;

        // Process the video with annotations
        Self::process_video(video_path, output_path_str, &annotation_data)?;

        // Create response with processed file information
        let response = json!({
            "status": "completed",
            "video_path": video_path,
            "processed_path": output_path_str,
            "timestamp": timestamp,
            "message": "Video annotation completed",
            "frame_count": annotation_data.metadata_count,
            "annotation_timestamp": annotation_data.timestamp
        });

        Ok(response.to_string())
    }

    pub fn cleanup_tmp_files() -> Result<(), String> {
        let tmp_dir = Path::new("tmp");
        if tmp_dir.exists() {
            fs::remove_dir_all(tmp_dir)
                .map_err(|e| format!("Failed to cleanup tmp directory: {}", e))?;
        }
        Ok(())
    }
}
