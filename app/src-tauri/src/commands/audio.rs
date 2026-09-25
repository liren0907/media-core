use audio_core::metadata::{self, AudioMetadata};
use audio_core::recording::{self, RecordingInfo};
use std::path::PathBuf;
use tauri::{command, AppHandle, Manager};

/// Recordings live under the app data dir, next to the dedup database.
fn recordings_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("recordings"))
}

#[command]
pub async fn save_audio_recording(
    app: AppHandle,
    audio_data: Vec<u8>,
    filename: String,
) -> Result<RecordingInfo, String> {
    recording::save_recording(&recordings_dir(&app)?, &audio_data, &filename)
}

#[command]
pub async fn list_audio_recordings(app: AppHandle) -> Result<Vec<RecordingInfo>, String> {
    recording::list_recordings(&recordings_dir(&app)?)
}

#[command]
pub async fn delete_audio_recording(app: AppHandle, filename: String) -> Result<(), String> {
    recording::delete_recording(&recordings_dir(&app)?, &filename)
}

#[command]
pub async fn rename_audio_recording(
    app: AppHandle,
    old_filename: String,
    new_filename: String,
) -> Result<RecordingInfo, String> {
    recording::rename_recording(&recordings_dir(&app)?, &old_filename, &new_filename)
}

#[command]
pub async fn get_recording_info(app: AppHandle, filename: String) -> Result<RecordingInfo, String> {
    recording::get_recording_metadata(&recordings_dir(&app)?, &filename)
}

#[command]
pub async fn get_audio_metadata(
    audio_path: String,
    srt_path: Option<String>,
) -> Result<AudioMetadata, String> {
    metadata::analyze_audio_metadata(&audio_path, srt_path.as_deref())
}
