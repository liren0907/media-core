use media_core::metadata::{get_media_info, MediaMetadata};
use tauri::command;

#[command]
pub async fn get_media_metadata(file_path: String, include_thumbnail: bool) -> Result<MediaMetadata, String> {
    std::thread::spawn(move || {
        get_media_info(&file_path, include_thumbnail)
            // Error is already a String based on signature
            .map_err(|e: String| e)
    })
    .join()
    .map_err(|_| "Thread panicked".to_string())?
}

#[command]
pub async fn check_media_support(file_path: String) -> bool {
    media_core::metadata::is_supported_media(&file_path)
}

