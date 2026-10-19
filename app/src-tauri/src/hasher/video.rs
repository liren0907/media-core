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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hasher::perceptual::hamming_distance;

    /// A 2 second test-pattern clip. Fails rather than skips without ffmpeg:
    /// a skipped check would stay green forever.
    fn fixture_video(dir: &Path) -> std::path::PathBuf {
        let path = dir.join("fixture.mp4");
        let status = std::process::Command::new("ffmpeg")
            .args(["-y", "-f", "lavfi", "-i", "testsrc=duration=2:size=160x120:rate=15", "-pix_fmt", "yuv420p"])
            .arg(&path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap_or_else(|e| panic!("ffmpeg is required for the video hash tests: {e}"));
        assert!(status.success() && path.exists(), "ffmpeg could not make the fixture video");
        path
    }

    #[test]
    fn videos_are_hashed() {
        let dir = TempDir::new().unwrap();
        let video = fixture_video(dir.path());
        let p = video_phash(&video).unwrap();
        let d = video_dhash(&video).unwrap();
        assert_eq!(hamming_distance(&p, &p).unwrap().bits, 40);
        assert_eq!(hamming_distance(&d, &d).unwrap().bits, 64);
    }
}
