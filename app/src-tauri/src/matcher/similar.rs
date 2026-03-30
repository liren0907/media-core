use crate::db;
use crate::hasher::perceptual;
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use super::MatchPair;

/// Find similar matches for `target_files` against a pre-loaded `pool` of source files.
/// Each match pair has new_file = target, existing_file = source pool file.
pub fn find_similar_matches_against_pool(
    target_files: &[db::MediaFile],
    pool: &[db::MediaFile],
    threshold: u32,
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
                if let Ok(dist) = perceptual::hamming_distance(th, sh) {
                    if dist <= threshold {
                        let score = 1.0 - (dist as f64 / 64.0);
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
    }

    matches
}

/// Default Hamming distance threshold for perceptual hash comparison.
pub const DEFAULT_SIMILARITY_THRESHOLD: u32 = 10;

/// Find similar files using a specific perceptual hash algorithm.
/// `algorithm` must be `"phash"` or `"dhash"`.
/// When `include_self` is true, also compares files within the same source.
/// `threshold` controls the max Hamming distance (0–64) for a match.
pub async fn find_similar_matches(
    db: &Surreal<Db>,
    source_id: &str,
    source_files: &[db::MediaFile],
    include_self: bool,
    threshold: u32,
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
                if let Ok(dist) = perceptual::hamming_distance(new_h, ex_h) {
                    if dist <= threshold {
                        let score = 1.0 - (dist as f64 / 64.0);
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
    }

    Ok(matches)
}
