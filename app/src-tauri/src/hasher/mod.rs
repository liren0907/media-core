pub mod content_hash;
pub mod perceptual;
pub mod video;

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum HashAlgorithm {
    Blake3,
    PHash,
    DHash,
}

#[derive(Debug, Clone, Default)]
pub struct FileHashes {
    pub content_hash: Option<String>,
    pub phash: Option<String>,
    pub dhash: Option<String>,
}

pub fn compute_hashes(
    file_path: &Path,
    file_type: &str,
    algorithms: &[HashAlgorithm],
) -> Result<FileHashes, String> {
    let mut result = FileHashes::default();

    for algo in algorithms {
        match algo {
            HashAlgorithm::Blake3 => {
                result.content_hash = Some(content_hash::blake3_hash(file_path)?);
            }
            HashAlgorithm::PHash => {
                let ph_result = if file_type == "image" {
                    perceptual::phash(file_path)
                } else if file_type == "video" {
                    video::video_phash(file_path)
                } else {
                    Err(format!("Unsupported file type for pHash: {}", file_type))
                };
                match ph_result {
                    Ok(h) => result.phash = Some(h),
                    Err(e) => eprintln!("pHash failed for {}: {}", file_path.display(), e),
                }
            }
            HashAlgorithm::DHash => {
                let dh_result = if file_type == "image" {
                    perceptual::dhash(file_path)
                } else if file_type == "video" {
                    video::video_dhash(file_path)
                } else {
                    Err(format!("Unsupported file type for dHash: {}", file_type))
                };
                match dh_result {
                    Ok(h) => result.dhash = Some(h),
                    Err(e) => eprintln!("dHash failed for {}: {}", file_path.display(), e),
                }
            }
        }
    }

    Ok(result)
}
