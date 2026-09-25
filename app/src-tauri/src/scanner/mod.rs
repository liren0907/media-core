pub mod filter;

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct ScannedFile {
    pub file_path: PathBuf,
    pub file_name: String,
    pub file_size: u64,
    pub file_type: String,
    pub parent_dir: String,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub files: Vec<ScannedFile>,
}

pub fn scan_directory(
    root: &Path,
    on_progress: impl Fn(usize, &str),
    is_cancelled: impl Fn() -> bool,
) -> Result<ScanResult, String> {
    if !root.is_dir() {
        return Err(format!("Path is not a directory: {}", root.display()));
    }

    let mut files = Vec::new();

    for entry in WalkDir::new(root).follow_links(false).into_iter().filter_map(|e| e.ok()) {
        if is_cancelled() {
            return Err("Scan cancelled".to_string());
        }

        if entry.file_type().is_dir() {
            continue;
        }

        let path = entry.path();
        if !filter::is_supported_media(path) {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let file_name = path.file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let file_type = filter::classify_extension(path);
        let parent = path.parent()
            .unwrap_or(root)
            .to_string_lossy()
            .to_string();

        let scanned = ScannedFile {
            file_path: path.to_path_buf(),
            file_name,
            file_size: metadata.len(),
            file_type,
            parent_dir: parent,
        };

        files.push(scanned);

        on_progress(files.len(), &path.to_string_lossy());
    }

    Ok(ScanResult { files })
}
