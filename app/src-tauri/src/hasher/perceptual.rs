use std::path::Path;
use img_hash::{HasherConfig, HashAlg, ImageHash};

// img_hash 3.2 depends on image 0.23, but the project uses image 0.25.
// Use img_hash's re-exported image crate for loading to avoid version conflicts.
use img_hash::image as img_hash_image;

pub fn phash(path: &Path) -> Result<String, String> {
    let img = img_hash_image::open(path)
        .map_err(|e| format!("Failed to open image {}: {}", path.display(), e))?;
    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::DoubleGradient)
        .hash_size(8, 8)
        .to_hasher();
    let hash = hasher.hash_image(&img);
    Ok(hash.to_base64())
}

pub fn dhash(path: &Path) -> Result<String, String> {
    let img = img_hash_image::open(path)
        .map_err(|e| format!("Failed to open image {}: {}", path.display(), e))?;
    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::Gradient)
        .hash_size(8, 8)
        .to_hasher();
    let hash = hasher.hash_image(&img);
    Ok(hash.to_base64())
}

pub fn hamming_distance(hash_a: &str, hash_b: &str) -> Result<u32, String> {
    let a = ImageHash::<Vec<u8>>::from_base64(hash_a)
        .map_err(|e| format!("Failed to parse hash_a: {:?}", e))?;
    let b = ImageHash::<Vec<u8>>::from_base64(hash_b)
        .map_err(|e| format!("Failed to parse hash_b: {:?}", e))?;
    Ok(a.dist(&b))
}
