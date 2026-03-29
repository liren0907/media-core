use std::path::Path;
use tempfile::TempDir;

/// Extract a representative I-frame from a video using ffmpeg CLI, then hash it.
/// Phase 1: single frame only. Phase 2 will add multi-frame strategy.
pub fn video_phash(path: &Path) -> Result<String, String> {
    hash_video_frame(path, super::perceptual::phash)
}

pub fn video_dhash(path: &Path) -> Result<String, String> {
    hash_video_frame(path, super::perceptual::dhash)
}

/// Extract first I-frame, hash it, then let TempDir clean up automatically.
fn hash_video_frame(
    path: &Path,
    hash_fn: fn(&Path) -> Result<String, String>,
) -> Result<String, String> {
    let tmp = TempDir::new().map_err(|e| format!("Failed to create temp dir: {}", e))?;
    let frame_path = tmp.path().join("frame.png");

    let status = std::process::Command::new("ffmpeg")
        .args([
            "-i", &path.to_string_lossy(),
            "-vf", "select=eq(pict_type\\,I)",
            "-vframes", "1",
            "-y",
            &frame_path.to_string_lossy(),
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("ffmpeg not found or failed: {}", e))?;

    if !status.success() {
        return Err("ffmpeg frame extraction failed".to_string());
    }

    if !frame_path.exists() {
        return Err("ffmpeg produced no output frame".to_string());
    }

    // Hash the frame while TempDir is still alive, then let it drop and clean up
    let result = hash_fn(&frame_path);
    // tmp drops here, cleaning up the temp directory
    result
}
