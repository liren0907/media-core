use opencv::core::Mat;

use super::analyzer::SimilarityMethodImpl;
use crate::analysis::config::PerceptualHashConfig;
use crate::analysis::hashing::Hasher;
use crate::analysis::types::AnalysisError;

/// Perceptual hash (aHash) based image similarity.
///
/// The hashing itself is `Hasher`'s, so the crate has a single aHash.
pub struct PerceptualHashMethod {
    hasher: Hasher,
}

impl PerceptualHashMethod {
    #[allow(dead_code)]
    pub fn new(_resize_width: i32, _resize_height: i32) -> Result<Self, AnalysisError> {
        Ok(Self {
            hasher: Hasher::new(PerceptualHashConfig::default().hash_size),
        })
    }

    /// Create with custom configuration
    pub fn with_config(config: &PerceptualHashConfig) -> Result<Self, AnalysisError> {
        Ok(Self {
            hasher: Hasher::new(config.hash_size),
        })
    }
}

impl SimilarityMethodImpl for PerceptualHashMethod {
    fn compute_similarity(&mut self, img1: &Mat, img2: &Mat) -> Result<f64, AnalysisError> {
        let hash1 = self.hasher.compute_phash_from_mat(img1)?;
        let hash2 = self.hasher.compute_phash_from_mat(img2)?;

        // Similarity in 0-1, where 1 is identical
        Ok(self.hasher.calculate_similarity(&hash1, &hash2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opencv::core::{CV_8UC3, Scalar, Vec3b};
    use opencv::prelude::*;

    /// A 64×64 grey image whose pixel value is `f(row, col)`.
    fn image(f: impl Fn(i32, i32) -> u8) -> Mat {
        let mut img =
            Mat::new_rows_cols_with_default(64, 64, CV_8UC3, Scalar::all(0.0)).unwrap();
        for row in 0..64 {
            for col in 0..64 {
                let v = f(row, col);
                *img.at_2d_mut::<Vec3b>(row, col).unwrap() = Vec3b::from([v, v, v]);
            }
        }
        img
    }

    #[test]
    fn similarity_of_generated_images_is_unchanged() {
        let horizontal = image(|_, c| (c * 4) as u8);
        let vertical = image(|r, _| (r * 4) as u8);
        let inverted = image(|_, c| 255 - (c * 4) as u8);
        let checker = image(|r, c| if (r / 8 + c / 8) % 2 == 0 { 0 } else { 255 });
        let diagonal = image(|r, c| ((r + c) * 2) as u8);

        let pairs = [
            (&horizontal, &horizontal),
            (&horizontal, &vertical),
            (&horizontal, &inverted),
            (&horizontal, &checker),
            (&vertical, &diagonal),
            (&checker, &diagonal),
        ];
        let mut method = PerceptualHashMethod::with_config(&PerceptualHashConfig::default()).unwrap();
        let scores: Vec<f64> = pairs
            .iter()
            .map(|(a, b)| method.compute_similarity(a, b).unwrap())
            .collect();

        // Recorded from the implementation before it was switched to `Hasher`.
        assert_eq!(scores, vec![1.0, 0.5, 0.0, 0.5, 0.75, 0.4375]);
    }
}
