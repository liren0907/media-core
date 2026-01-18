use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

use annotation::{Annotation, VideoAnnotation};

#[derive(Debug, Serialize, Deserialize)]
pub struct AnnotationPayload {
    pub video_path: String,
    pub annotation_path: String,
    pub output_directory: String,
    pub label_selected: Vec<String>,
}

#[tauri::command]
pub fn read_annotation_file(path: &str) -> Result<Value, String> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    let annotation: Annotation = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse annotation data: {}", e))?;

    let (unique_labels, total_objects) = annotation.get_total_labels();
    let total_frames = annotation.frame_metadata.len();

    Ok(json!({
        "total_frames": total_frames,
        "total_objects": total_objects,
        "unique_labels": unique_labels,
        "annotation": annotation
    }))
}

#[tauri::command]
pub fn start_video_annotation(payload: AnnotationPayload) -> Result<serde_json::Value, String> {
    println!(
        "Starting video annotation with video: {}, annotation: {}, output: {}",
        payload.video_path, payload.annotation_path, payload.output_directory
    );
    if !payload.label_selected.is_empty() {
        println!(
            "Selected labels for annotation: {:?}",
            payload.label_selected
        );
    }

    // Generate output filename with timestamp
    let input_path = Path::new(&payload.video_path);
    let file_stem = input_path
        .file_stem()
        .ok_or_else(|| "Invalid video filename".to_string())?
        .to_str()
        .ok_or_else(|| "Invalid video filename characters".to_string())?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let output_filename = format!("{}_annotated_{}.mp4", file_stem, timestamp);
    let output_path = Path::new(&payload.output_directory).join(&output_filename);

    // Read and parse the annotation file
    let content = fs::read_to_string(&payload.annotation_path)
        .map_err(|e| format!("Failed to read annotation file: {}", e))?;

    let mut annotation_data: Annotation = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse annotation file: {}", e))?;

    // Filter annotations if label_selected is provided
    if !payload.label_selected.is_empty() {
        annotation_data.filter_by_labels(&payload.label_selected);
    }

    // Process the video with annotations
    VideoAnnotation::process_video(
        &payload.video_path,
        output_path.to_str().unwrap(),
        &annotation_data,
    )?;

    // Create response with processed file information
    let response = json!({
        "status": "success",
        "data": {
            "input_video": payload.video_path,
            "output_video": output_path.to_str().unwrap(),
            "annotation_file": payload.annotation_path,
            "timestamp": timestamp.to_string()
        },
        "message": "Video annotation completed successfully"
    });

    Ok(response)
}
