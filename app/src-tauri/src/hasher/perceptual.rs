use std::path::Path;
use img_hash::{HasherConfig, HashAlg, ImageHash};

// img_hash 3.2 hashes image 0.23 buffers, but that image 0.23 is built without
// any format decoders here (img_hash turns its default features off). Files are
// therefore decoded with the app's own image 0.25 and only the pixels are handed
// over, as an image 0.23 buffer.
use img_hash::image as img_hash_image;

/// The "pHash" toggle: img_hash's DoubleGradient, 40 bits at 8×8.
const PHASH_ALG: HashAlg = HashAlg::DoubleGradient;
/// dHash: img_hash's Gradient, 64 bits at 8×8.
const DHASH_ALG: HashAlg = HashAlg::Gradient;

pub fn phash(path: &Path) -> Result<String, String> {
    Ok(hash_image(&load(path)?, PHASH_ALG))
}

pub fn dhash(path: &Path) -> Result<String, String> {
    Ok(hash_image(&load(path)?, DHASH_ALG))
}

/// Decode `path` with image 0.25 into a greyscale image 0.23 buffer.
///
/// Greyscale is what img_hash hashes anyway (it converts before resizing), and
/// it is a quarter of the memory of RGBA when rayon decodes large files at once.
fn load(path: &Path) -> Result<img_hash_image::DynamicImage, String> {
    let grey = image::open(path)
        .map_err(|e| format!("Failed to open image {}: {}", path.display(), e))?
        .to_luma8();
    let (width, height) = grey.dimensions();
    let buffer = img_hash_image::GrayImage::from_raw(width, height, grey.into_raw())
        .ok_or_else(|| format!("Failed to convert image {}", path.display()))?;
    Ok(img_hash_image::DynamicImage::ImageLuma8(buffer))
}

fn hash_image(img: &img_hash_image::DynamicImage, alg: HashAlg) -> String {
    let hasher = HasherConfig::new()
        .hash_alg(alg)
        .hash_size(8, 8)
        .to_hasher();
    hasher.hash_image(img).to_base64()
}

/// Hamming distance between two hashes, and how many bits each holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HashDistance {
    pub distance: u32,
    pub bits: u32,
}

/// Compare two base64 hashes from `phash` or `dhash`.
///
/// The bit count comes from the hash itself (decoded bytes × 8): `phash`
/// (DoubleGradient at 8×8) holds 40 bits and `dhash` (Gradient) 64, so a
/// caller must not assume 64.
pub fn hamming_distance(hash_a: &str, hash_b: &str) -> Result<HashDistance, String> {
    let a = ImageHash::<Vec<u8>>::from_base64(hash_a)
        .map_err(|e| format!("Failed to parse hash_a: {:?}", e))?;
    let b = ImageHash::<Vec<u8>>::from_base64(hash_b)
        .map_err(|e| format!("Failed to parse hash_b: {:?}", e))?;
    let bits = (a.as_bytes().len() * 8) as u32;
    let bits_b = (b.as_bytes().len() * 8) as u32;
    if bits != bits_b {
        return Err(format!("Hashes have different lengths ({} vs {} bits)", bits, bits_b));
    }
    Ok(HashDistance { distance: a.dist(&b), bits })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An image 0.23 image built in memory, for the hashing step alone.
    fn sample_image() -> img_hash_image::DynamicImage {
        img_hash_image::DynamicImage::ImageRgb8(img_hash_image::RgbImage::from_fn(64, 64, |x, y| {
            img_hash_image::Rgb([(x * 4) as u8, (y * 4) as u8, 128])
        }))
    }

    #[test]
    fn phash_holds_40_bits_and_dhash_64() {
        let img = sample_image();
        let p = hash_image(&img, PHASH_ALG);
        let d = hash_image(&img, DHASH_ALG);
        assert_eq!(hamming_distance(&p, &p).unwrap(), HashDistance { distance: 0, bits: 40 });
        assert_eq!(hamming_distance(&d, &d).unwrap(), HashDistance { distance: 0, bits: 64 });
    }

    #[test]
    fn hashes_of_different_lengths_are_not_compared() {
        let img = sample_image();
        let err = hamming_distance(&hash_image(&img, PHASH_ALG), &hash_image(&img, DHASH_ALG))
            .unwrap_err();
        assert!(err.contains("40 vs 64"), "{err}");
    }

    fn bits_of(hash: &str) -> u32 {
        hamming_distance(hash, hash).unwrap().bits
    }

    #[test]
    fn png_and_jpeg_files_are_hashed() {
        let dir = tempfile::tempdir().unwrap();
        let img = image::RgbImage::from_fn(64, 64, |x, y| image::Rgb([(x * 4) as u8, (y * 4) as u8, 128]));
        for name in ["sample.png", "sample.jpg"] {
            let path = dir.path().join(name);
            img.save(&path).unwrap();
            assert_eq!(bits_of(&phash(&path).unwrap()), 40, "{name}");
            assert_eq!(bits_of(&dhash(&path).unwrap()), 64, "{name}");
        }
    }

    #[test]
    fn reading_a_png_keeps_its_pixels() {
        let value = |x: u32, y: u32| ((x * 3 + y * 5) % 256) as u8;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("grey.png");
        image::GrayImage::from_fn(48, 32, |x, y| image::Luma([value(x, y)])).save(&path).unwrap();

        let in_memory = img_hash_image::DynamicImage::ImageLuma8(img_hash_image::GrayImage::from_fn(
            48,
            32,
            |x, y| img_hash_image::Luma([value(x, y)]),
        ));
        assert_eq!(phash(&path).unwrap(), hash_image(&in_memory, PHASH_ALG));
        assert_eq!(dhash(&path).unwrap(), hash_image(&in_memory, DHASH_ALG));
    }

    #[test]
    fn an_unreadable_file_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("photo.heic");
        std::fs::write(&path, b"not an image").unwrap();
        assert!(phash(&path).is_err());
        assert!(dhash(&path).is_err());
    }
}
