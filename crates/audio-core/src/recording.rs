use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingInfo {
    pub filename: String,
    /// Absolute path, so the frontend can play the file via convertFileSrc.
    pub file_path: String,
    pub size_bytes: u64,
    pub size_kb: f64,
    pub size_mb: f64,
    pub created_timestamp: u64,
    pub created_date: String,
}

/// Recordings are addressed by bare filename only — reject anything that
/// could escape the recordings directory.
fn safe_path(recordings_dir: &Path, filename: &str) -> Result<PathBuf, String> {
    if filename.is_empty()
        || filename.contains('/')
        || filename.contains('\\')
        || filename.contains("..")
    {
        return Err(format!("Invalid recording filename: {}", filename));
    }
    Ok(recordings_dir.join(filename))
}

/// Save audio recording data to file
pub fn save_recording(
    recordings_dir: &Path,
    audio_data: &[u8],
    filename: &str,
) -> Result<RecordingInfo, String> {
    let file_path = safe_path(recordings_dir, filename)?;

    std::fs::create_dir_all(recordings_dir)
        .map_err(|e| format!("Failed to create recordings directory: {}", e))?;

    let mut file =
        File::create(&file_path).map_err(|e| format!("Failed to create audio file: {}", e))?;

    file.write_all(audio_data)
        .map_err(|e| format!("Failed to write audio data: {}", e))?;

    get_recording_metadata(recordings_dir, filename)
}

/// List all audio recordings, newest first
pub fn list_recordings(recordings_dir: &Path) -> Result<Vec<RecordingInfo>, String> {
    if !recordings_dir.exists() {
        return Ok(vec![]);
    }

    let entries = std::fs::read_dir(recordings_dir)
        .map_err(|e| format!("Failed to read recordings directory: {}", e))?;

    let mut recordings = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if filename.starts_with('.') {
            continue;
        }
        recordings.push(get_recording_metadata(recordings_dir, filename)?);
    }

    recordings.sort_by(|a, b| {
        b.created_timestamp
            .cmp(&a.created_timestamp)
            .then_with(|| b.filename.cmp(&a.filename))
    });

    Ok(recordings)
}

/// Rename an audio recording, returning the renamed file's metadata
pub fn rename_recording(
    recordings_dir: &Path,
    old_filename: &str,
    new_filename: &str,
) -> Result<RecordingInfo, String> {
    let old_path = safe_path(recordings_dir, old_filename)?;
    let new_path = safe_path(recordings_dir, new_filename)?;

    if !old_path.exists() {
        return Err(format!("Recording file not found: {}", old_filename));
    }
    if old_filename == new_filename {
        return get_recording_metadata(recordings_dir, old_filename);
    }
    if new_path.exists() {
        return Err(format!("A recording named {} already exists", new_filename));
    }

    std::fs::rename(&old_path, &new_path)
        .map_err(|e| format!("Failed to rename recording: {}", e))?;

    get_recording_metadata(recordings_dir, new_filename)
}

/// Delete an audio recording
pub fn delete_recording(recordings_dir: &Path, filename: &str) -> Result<(), String> {
    let file_path = safe_path(recordings_dir, filename)?;

    if !file_path.exists() {
        return Err(format!("Recording file not found: {}", filename));
    }

    std::fs::remove_file(&file_path).map_err(|e| format!("Failed to delete recording: {}", e))
}

/// Get metadata for a recording file
pub fn get_recording_metadata(
    recordings_dir: &Path,
    filename: &str,
) -> Result<RecordingInfo, String> {
    let file_path = safe_path(recordings_dir, filename)?;

    if !file_path.exists() {
        return Err(format!("Recording file not found: {}", filename));
    }

    let metadata =
        std::fs::metadata(&file_path).map_err(|e| format!("Failed to get file metadata: {}", e))?;

    let size_bytes = metadata.len();

    // created() is unsupported on some filesystems — fall back to modified()
    let created = metadata
        .created()
        .or_else(|_| metadata.modified())
        .map_err(|e| format!("Failed to get file time: {}", e))?;

    let created_timestamp = created
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Failed to convert timestamp: {}", e))?
        .as_secs();

    let created_date = chrono::DateTime::from_timestamp(created_timestamp as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    Ok(RecordingInfo {
        filename: filename.to_string(),
        file_path: file_path.to_string_lossy().to_string(),
        size_bytes,
        size_kb: size_bytes as f64 / 1024.0,
        size_mb: size_bytes as f64 / (1024.0 * 1024.0),
        created_timestamp,
        created_date,
    })
}
