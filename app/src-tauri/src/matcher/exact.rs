use crate::db;
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use std::collections::HashMap;
use super::MatchPair;

/// Find exact matches for `target_files` against a pre-loaded `pool` of source files.
/// Each match pair has new_file = target, existing_file = source pool file.
pub fn find_exact_matches_against_pool(
    target_files: &[db::MediaFile],
    pool: &[db::MediaFile],
    on_progress: impl Fn(usize, usize),
) -> Vec<MatchPair> {
    let mut matches = Vec::new();
    let total = target_files.len();

    // Index pool files by content_hash
    let mut pool_by_hash: HashMap<String, Vec<&db::MediaFile>> = HashMap::new();
    for file in pool {
        if let Some(ref hash) = file.content_hash {
            pool_by_hash.entry(hash.clone()).or_default().push(file);
        }
    }

    for (i, target) in target_files.iter().enumerate() {
        on_progress(i + 1, total);

        let target_id = target.id.as_ref().map(|t| t.to_string()).unwrap_or_default();

        if let Some(ref hash) = target.content_hash {
            if let Some(pool_files) = pool_by_hash.get(hash) {
                for source_file in pool_files {
                    let source_id = source_file.id.as_ref().map(|t| t.to_string()).unwrap_or_default();
                    if source_id == target_id {
                        continue;
                    }
                    matches.push(MatchPair {
                        new_file_id: target_id.clone(),
                        existing_file_id: source_id,
                        new_file_path: target.file_path.clone(),
                        existing_file_path: source_file.file_path.clone(),
                        algorithm: "blake3".to_string(),
                        similarity_score: 1.0,
                        match_type: "exact".to_string(),
                    });
                }
            }
        }
    }

    matches
}

/// Find exact content hash matches for files from a given source.
/// When `include_self` is true, also finds duplicates within the same source.
/// Groups files by content_hash to avoid N+1 queries.
pub async fn find_exact_matches(
    db: &Surreal<Db>,
    source_files: &[db::MediaFile],
    include_self: bool,
    on_progress: impl Fn(usize, usize),
) -> Result<Vec<MatchPair>, String> {
    let mut matches = Vec::new();
    let total = source_files.len();

    // Group source files by content_hash to batch lookups
    let mut hash_to_files: HashMap<String, Vec<&db::MediaFile>> = HashMap::new();
    for file in source_files {
        if let Some(ref hash) = file.content_hash {
            hash_to_files.entry(hash.clone()).or_default().push(file);
        }
    }

    let mut processed = 0usize;
    for (hash, files_with_hash) in &hash_to_files {
        let existing = db::get_files_by_content_hash(db, hash).await?;

        for file in files_with_hash {
            processed += 1;
            on_progress(processed, total);

            let file_id = file.id.as_ref().map(|t| t.to_string()).unwrap_or_default();

            for existing_file in &existing {
                let existing_id = existing_file.id.as_ref().map(|t| t.to_string()).unwrap_or_default();

                // Always skip comparing a file with itself
                if existing_id == file_id {
                    continue;
                }

                // Skip same-source files unless include_self is enabled
                if !include_self && existing_file.source == file.source {
                    continue;
                }

                matches.push(MatchPair {
                    new_file_id: file_id.clone(),
                    existing_file_id: existing_id,
                    new_file_path: file.file_path.clone(),
                    existing_file_path: existing_file.file_path.clone(),
                    algorithm: "blake3".to_string(),
                    similarity_score: 1.0,
                    match_type: "exact".to_string(),
                });
            }
        }
    }

    Ok(matches)
}
