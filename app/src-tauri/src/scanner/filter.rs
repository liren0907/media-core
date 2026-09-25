use std::path::Path;

const IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "tif",
    "heic", "heif", "avif",
];

const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm",
    "m4v", "mpg", "mpeg", "3gp", "ts", "mts",
];

pub fn is_supported_media(path: &Path) -> bool {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some(e) => IMAGE_EXTENSIONS.contains(&e) || VIDEO_EXTENSIONS.contains(&e),
        None => false,
    }
}

pub fn classify_extension(path: &Path) -> String {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
        "image".to_string()
    } else if VIDEO_EXTENSIONS.contains(&ext.as_str()) {
        "video".to_string()
    } else {
        "unknown".to_string()
    }
}

