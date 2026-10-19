use crate::db;
use crate::hasher::perceptual;
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use super::MatchPair;

/// Find similar matches for `target_files` against a pre-loaded `pool` of source files.
/// Each match pair has new_file = target, existing_file = source pool file.
/// `threshold_percent` is the largest share of differing bits (0–100) for a match.
pub fn find_similar_matches_against_pool(
    target_files: &[db::MediaFile],
    pool: &[db::MediaFile],
    threshold_percent: u32,
    algorithm: &str,
    on_progress: impl Fn(usize, usize),
) -> Vec<MatchPair> {
    let mut matches = Vec::new();
    let total = target_files.len();

    for (i, target) in target_files.iter().enumerate() {
        on_progress(i + 1, total);

        let target_id = target.id.as_ref().map(|t| t.to_string()).unwrap_or_default();

        for source_file in pool {
            let source_id = source_file.id.as_ref().map(|t| t.to_string()).unwrap_or_default();
            if source_id == target_id {
                continue;
            }

            let (target_hash, source_hash) = match algorithm {
                "phash" => (&target.phash, &source_file.phash),
                "dhash" => (&target.dhash, &source_file.dhash),
                _ => continue,
            };

            if let (Some(ref th), Some(ref sh)) = (target_hash, source_hash) {
                if let Some((dist, score)) = score_pair(th, sh, threshold_percent) {
                    matches.push(MatchPair {
                        new_file_id: target_id.clone(),
                        existing_file_id: source_id.clone(),
                        new_file_path: target.file_path.clone(),
                        existing_file_path: source_file.file_path.clone(),
                        algorithm: algorithm.to_string(),
                        similarity_score: score,
                        match_type: if dist == 0 { "exact" } else { "similar" }.to_string(),
                    });
                }
            }
        }
    }

    matches
}

/// Default threshold: the largest share of differing bits, in percent, that still
/// counts as similar. 16% is 10 of dHash's 64 bits and 6 of pHash's 40.
pub const DEFAULT_SIMILARITY_THRESHOLD_PERCENT: u32 = 16;

/// Largest Hamming distance a `percent` threshold allows for a `bits`-bit hash.
/// Integer division rounds down, so a boundary never lets an extra bit through.
pub fn max_distance(percent: u32, bits: u32) -> u32 {
    percent.min(100) * bits / 100
}

/// Similarity in [0, 1] of two `bits`-bit hashes that are `distance` bits apart.
pub fn similarity_score(distance: u32, bits: u32) -> f64 {
    if bits == 0 {
        return 0.0;
    }
    1.0 - (distance as f64 / bits as f64)
}

/// Distance and score of a hash pair, or `None` when it is over the threshold
/// (or the hashes can't be compared).
fn score_pair(hash_a: &str, hash_b: &str, threshold_percent: u32) -> Option<(u32, f64)> {
    let d = perceptual::hamming_distance(hash_a, hash_b).ok()?;
    (d.distance <= max_distance(threshold_percent, d.bits))
        .then(|| (d.distance, similarity_score(d.distance, d.bits)))
}

/// Find similar files using a specific perceptual hash algorithm.
/// `algorithm` must be `"phash"` or `"dhash"`.
/// When `include_self` is true, also compares files within the same source.
/// `threshold_percent` is the largest share of differing bits (0–100) for a match;
/// it is converted per hash, since pHash holds 40 bits and dHash 64.
pub async fn find_similar_matches(
    db: &Surreal<Db>,
    source_id: &str,
    source_files: &[db::MediaFile],
    include_self: bool,
    threshold_percent: u32,
    algorithm: &str,
    on_progress: impl Fn(usize, usize),
) -> Result<Vec<MatchPair>, String> {
    let existing_files = if include_self {
        db::get_all_hashed_files(db).await?
    } else {
        db::get_all_hashed_files_excluding_source(db, source_id).await?
    };
    let mut matches = Vec::new();
    let total = source_files.len();

    for (i, new_file) in source_files.iter().enumerate() {
        on_progress(i, total);

        let new_file_id = new_file.id.as_ref().map(|t| t.to_string()).unwrap_or_default();

        for existing_file in &existing_files {
            let existing_id = existing_file.id.as_ref().map(|t| t.to_string()).unwrap_or_default();

            // Always skip comparing a file with itself
            if existing_id == new_file_id {
                continue;
            }

            // Get the hash pair based on the selected algorithm
            let (new_hash, ex_hash) = match algorithm {
                "phash" => (&new_file.phash, &existing_file.phash),
                "dhash" => (&new_file.dhash, &existing_file.dhash),
                _ => continue,
            };

            if let (Some(ref new_h), Some(ref ex_h)) = (new_hash, ex_hash) {
                if let Some((dist, score)) = score_pair(new_h, ex_h, threshold_percent) {
                    matches.push(MatchPair {
                        new_file_id: new_file_id.clone(),
                        existing_file_id: existing_id.clone(),
                        new_file_path: new_file.file_path.clone(),
                        existing_file_path: existing_file.file_path.clone(),
                        algorithm: algorithm.to_string(),
                        similarity_score: score,
                        match_type: if dist == 0 { "exact" } else { "similar" }.to_string(),
                    });
                }
            }
        }
    }

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use img_hash::ImageHash;

    fn hash(bytes: &[u8]) -> String {
        ImageHash::<Vec<u8>>::from_bytes(bytes).unwrap().to_base64()
    }

    fn file(id: &str, phash: Option<String>, dhash: Option<String>) -> db::MediaFile {
        db::MediaFile {
            id: Some(id.to_string()),
            file_path: format!("/media/{id}.jpg"),
            file_name: format!("{id}.jpg"),
            file_size: 1,
            file_type: "image".to_string(),
            parent_dir: "/media".to_string(),
            content_hash: None,
            phash,
            dhash,
            source: None,
            scanned_at: None,
        }
    }

    /// Matches of one target against one pool file.
    fn compare(algorithm: &str, a: &str, b: &str, threshold_percent: u32) -> Vec<MatchPair> {
        let (target, pool) = match algorithm {
            "phash" => (file("t", Some(a.into()), None), file("p", Some(b.into()), None)),
            _ => (file("t", None, Some(a.into())), file("p", None, Some(b.into()))),
        };
        find_similar_matches_against_pool(&[target], &[pool], threshold_percent, algorithm, |_, _| {})
    }

    #[test]
    fn a_percent_threshold_becomes_a_distance_for_each_hash_length() {
        assert_eq!(max_distance(DEFAULT_SIMILARITY_THRESHOLD_PERCENT, 64), 10);
        assert_eq!(max_distance(DEFAULT_SIMILARITY_THRESHOLD_PERCENT, 40), 6);
        assert_eq!(max_distance(25, 40), 10);
        assert_eq!(max_distance(25, 64), 16);
        assert_eq!(max_distance(0, 64), 0);
        assert_eq!(max_distance(100, 40), 40);
        assert_eq!(max_distance(250, 40), 40);
    }

    #[test]
    fn a_40_bit_phash_scores_against_40_bits() {
        let zeros = hash(&[0; 5]);
        let opposite = compare("phash", &zeros, &hash(&[0xff; 5]), 100);
        assert_eq!(opposite.len(), 1);
        assert_eq!(opposite[0].similarity_score, 0.0);

        let same = compare("phash", &zeros, &zeros, 0);
        assert_eq!(same[0].similarity_score, 1.0);
        assert_eq!(same[0].match_type, "exact");

        // 6 bits apart: 85% (1 - 6/40), not the 91% of 1 - 6/64.
        let six = compare("phash", &zeros, &hash(&[0b0011_1111, 0, 0, 0, 0]), 16);
        assert_eq!(six[0].similarity_score, 1.0 - 6.0 / 40.0);
        assert_eq!(six[0].match_type, "similar");
    }

    #[test]
    fn a_64_bit_dhash_scores_as_before() {
        let zeros = hash(&[0; 8]);
        let ten = compare("dhash", &zeros, &hash(&[0xff, 0b0000_0011, 0, 0, 0, 0, 0, 0]), 16);
        assert_eq!(ten.len(), 1);
        assert_eq!(ten[0].similarity_score, 1.0 - 10.0 / 64.0);

        let eleven = compare("dhash", &zeros, &hash(&[0xff, 0b0000_0111, 0, 0, 0, 0, 0, 0]), 16);
        assert!(eleven.is_empty());
    }

    #[test]
    fn the_default_threshold_allows_6_phash_bits_and_not_7() {
        let zeros = hash(&[0; 5]);
        let six = compare("phash", &zeros, &hash(&[0b0011_1111, 0, 0, 0, 0]), 16);
        let seven = compare("phash", &zeros, &hash(&[0b0111_1111, 0, 0, 0, 0]), 16);
        assert_eq!(six.len(), 1);
        assert!(seven.is_empty());
    }
}
