use media_core::camera::{Camera, is_camera_available};
use tauri::command;
use base64::prelude::*;


// Global state for the active camera to allow persistent access across commands
// We use a lazy_static or similar pattern if we needed singleton, but here we'll 
// just instantiate on demand for simple actions, or we'd need a managed state in main.rs.
// For now, we will expose the listing and simple capture 'one-shot' capabilities 
// to avoid complex state management in this iteration.

#[command]
pub async fn get_available_cameras() -> Result<Vec<String>, String> {
    // This function in media_core seems to print to stdout/stderr mostly
    // We might need to adjust media_core to return the list more cleanly, 
    // but based on signatures: pub use utils::{default_camera_id, has_camera, is_camera_available, list_available_cameras};
    // Let's see if we can just wrap list_available_cameras if it returns a Vec.
    // Checking signature... it might just print. 
    // If it just prints, we might need to rely on `is_camera_available` for specific IDs.
    
    // Fallback: Check first 10 indices
    let mut available = Vec::new();
    for i in 0..10 {
        if is_camera_available(i) {
            available.push(format!("Camera {}", i));
        }
    }
    Ok(available)
}

#[command]
pub async fn check_camera_access(index: i32) -> bool {
    is_camera_available(index)
}

#[command]
pub async fn capture_camera_snapshot(index: i32, output_path: String) -> Result<String, String> {
    // Run blocking camera operation in a separate thread
    std::thread::spawn(move || {
        let mut cam = Camera::new(index).map_err(|e| format!("Failed to open camera: {:?}", e))?;
        
        // Capture frame (returns CameraFrame with base64 data)
        let frame = cam.capture_frame().map_err(|e| format!("Failed to capture frame: {:?}", e))?;
        
        // Decode and save to file
        let image_data = BASE64_STANDARD
            .decode(&frame.data)
            .map_err(|e| format!("Failed to decode base64 image data: {}", e))?;

        std::fs::write(&output_path, image_data)
            .map_err(|e| format!("Failed to save snapshot to {}: {}", output_path, e))?;
        
        Ok(format!("Snapshot saved to {}", output_path))
    })
    .join()
    .map_err(|_| "Thread panicked".to_string())?
}

