use crate::asr::{self, AsrModelInfo, TranscriptionResult};
use tauri::{command, AppHandle};

#[command]
pub async fn list_asr_models(app: AppHandle) -> Result<Vec<AsrModelInfo>, String> {
    asr::list_models(&app)
}

#[command]
pub async fn download_asr_model(app: AppHandle, model: String) -> Result<AsrModelInfo, String> {
    asr::download_model(app, model).await
}

#[command]
pub async fn transcribe_audio(
    app: AppHandle,
    audio_path: String,
    model: String,
    language: Option<String>,
) -> Result<TranscriptionResult, String> {
    asr::transcribe_file(app, audio_path, model, language).await
}

#[command]
pub async fn cancel_asr() -> Result<(), String> {
    asr::request_cancel();
    Ok(())
}

/// Write exported transcription output (TXT / SRT / JSON) to a path the user
/// picked via the save dialog.
#[command]
pub async fn save_text_file(path: String, contents: String) -> Result<(), String> {
    std::fs::write(&path, contents).map_err(|e| format!("Failed to save file: {}", e))
}
