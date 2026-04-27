use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{command, AppHandle, Emitter, Manager};

use crate::db;
use crate::hasher::{self, HashAlgorithm};
use crate::matcher;
use crate::scanner;

// ──────────────────────────────────────────────
// Event types
// ──────────────────────────────────────────────

const EVENT_SCAN_PROGRESS: &str = "dedup:scan-progress";
const EVENT_HASH_PROGRESS: &str = "dedup:hash-progress";
const EVENT_COMPARE_PROGRESS: &str = "dedup:compare-progress";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanProgressEvent {
    source_id: String,
    files_found: usize,
    current_path: String,
    phase: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct HashProgressEvent {
    source_id: String,
    current: usize,
    total: usize,
    current_file: String,
    algorithm: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompareProgressEvent {
    source_id: String,
    current: usize,
    total: usize,
    matches_found: usize,
}

// ──────────────────────────────────────────────
// Cancellation
// ──────────────────────────────────────────────

lazy_static::lazy_static! {
    static ref CANCEL_FLAGS: std::sync::Mutex<HashMap<String, std::sync::Arc<AtomicBool>>> =
        std::sync::Mutex::new(HashMap::new());
}

fn get_cancel_flag(source_id: &str) -> std::sync::Arc<AtomicBool> {
    let mut flags = CANCEL_FLAGS.lock().unwrap();
    flags.entry(source_id.to_string())
        .or_insert_with(|| std::sync::Arc::new(AtomicBool::new(false)))
        .clone()
}

fn is_cancelled(source_id: &str) -> bool {
    CANCEL_FLAGS.lock()
        .map(|flags| flags.get(source_id).map(|f| f.load(Ordering::Relaxed)).unwrap_or(false))
        .unwrap_or(false)
}

// ──────────────────────────────────────────────
// Commands
// ──────────────────────────────────────────────

#[command]
pub async fn add_scan_source(
    app: AppHandle,
    path: String,
    label: String,
    role: Option<String>,
) -> Result<db::ScanSource, String> {
    if !std::path::Path::new(&path).is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    let role_val = role.unwrap_or_else(|| "target".to_string());
    if role_val != "source" && role_val != "target" {
        return Err(format!("Invalid role: {}. Must be 'source' or 'target'", role_val));
    }

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let database = db::get_db(&app_data_dir).await?;

    // If adding as source, demote any existing source first
    if role_val == "source" {
        if let Ok(Some(existing)) = db::get_source_role_source(database).await {
            if let Some(ref id) = existing.id {
                db::update_source_role(database, id, "target").await?;
            }
        }
    }

    db::insert_scan_source(database, &path, &label, &role_val).await
}

#[command]
pub async fn get_scan_sources(app: AppHandle) -> Result<Vec<db::ScanSource>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let database = db::get_db(&app_data_dir).await?;
    db::get_all_scan_sources(database).await
}

#[command]
pub async fn delete_scan_source(app: AppHandle, source_id: String) -> Result<(), String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let database = db::get_db(&app_data_dir).await?;
    db::delete_scan_source_cascade(database, &source_id).await
}

#[command]
pub async fn get_source_tree(
    app: AppHandle,
    source_id: String,
) -> Result<Vec<db::DirectoryNode>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let database = db::get_db(&app_data_dir).await?;
    db::get_directory_tree(database, &source_id).await
}

#[command]
pub async fn start_scan(
    app: AppHandle,
    source_id: String,
) -> Result<db::IncrementalScanResult, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let database = db::get_db(&app_data_dir).await?;
    let source = db::get_scan_source(database, &source_id).await?;

    // Reset cancel flag
    get_cancel_flag(&source_id).store(false, Ordering::Relaxed);

    // Update status to scanning
    db::update_source_status(database, &source_id, "scanning", 0).await?;

    let sid = source_id.clone();
    let app_clone = app.clone();

    // Scan directory in a blocking thread
    let scan_result = std::thread::spawn(move || {
        scanner::scan_directory(
            std::path::Path::new(&source.path),
            |count, path| {
                let _ = app_clone.emit(EVENT_SCAN_PROGRESS, ScanProgressEvent {
                    source_id: sid.clone(),
                    files_found: count,
                    current_path: path.to_string(),
                    phase: "scanning".to_string(),
                });
            },
            || is_cancelled(&sid),
        )
    })
    .join()
    .map_err(|_| "Scan thread panicked".to_string())??;

    // Emit diffing phase
    let _ = app.emit(EVENT_SCAN_PROGRESS, ScanProgressEvent {
        source_id: source_id.clone(),
        files_found: scan_result.files.len(),
        current_path: String::new(),
        phase: "diffing".to_string(),
    });

    // Get existing DB records for this source
    let existing_files = db::get_files_by_source(database, &source_id).await?;
    let existing_map: HashMap<String, db::MediaFile> = existing_files
        .into_iter()
        .map(|f| (f.file_path.clone(), f))
        .collect();

    let scanned_map: HashMap<String, &scanner::ScannedFile> = scan_result.files
        .iter()
        .map(|f| (f.file_path.to_string_lossy().to_string(), f))
        .collect();

    // Diff: find new, removed, and changed files
    let mut new_files: Vec<db::MediaFile> = Vec::new();
    let mut removed_paths: Vec<String> = Vec::new();
    let mut updated = 0usize;
    let mut unchanged = 0usize;

    // New or changed files
    for (path, scanned) in &scanned_map {
        if let Some(existing) = existing_map.get(path) {
            // File exists in DB — check if size changed
            if existing.file_size != scanned.file_size as i64 {
                // Size changed → clear hashes so user can re-fingerprint
                if let Some(ref id) = existing.id {
                    db::clear_file_hashes_by_id(database, id).await?;
                }
                updated += 1;
            } else {
                unchanged += 1;
            }
        } else {
            // New file
            new_files.push(db::MediaFile {
                id: None,
                file_path: scanned.file_path.to_string_lossy().to_string(),
                file_name: scanned.file_name.clone(),
                file_size: scanned.file_size as i64,
                file_type: scanned.file_type.clone(),
                parent_dir: scanned.parent_dir.clone(),
                content_hash: None,
                phash: None,
                dhash: None,
    
                source: None,
                scanned_at: None,
            });
        }
    }

    // Removed files (in DB but not on filesystem)
    let mut removed_file_ids: Vec<String> = Vec::new();
    for (path, existing) in &existing_map {
        if !scanned_map.contains_key(path) {
            removed_paths.push(path.clone());
            if let Some(ref id) = existing.id {
                removed_file_ids.push(id.clone());
            }
        }
    }

    // Apply changes
    let added = if !new_files.is_empty() {
        db::insert_media_files_batch(database, &new_files, &source_id).await?
    } else {
        0
    };

    let removed = if !removed_paths.is_empty() {
        db::delete_media_files_by_paths(database, &source_id, &removed_paths).await?
    } else {
        0
    };

    // Clean up orphan duplicate group members
    if !removed_file_ids.is_empty() {
        db::clean_orphan_duplicate_members(database, &source_id, &removed_file_ids).await?;
    }

    let total_files = (added + unchanged + updated) as i64;
    db::update_source_status(database, &source_id, "scanned", total_files).await?;

    Ok(db::IncrementalScanResult { added, removed, updated, unchanged })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FingerprintConfig {
    pub source_id: String,
    pub algorithms: Vec<HashAlgorithm>,
}

#[command]
pub async fn start_fingerprinting(
    app: AppHandle,
    config: FingerprintConfig,
) -> Result<usize, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let database = db::get_db(&app_data_dir).await?;

    // Get files that need hashing
    let files = db::get_files_by_source(database, &config.source_id).await?;
    let total = files.len();

    if total == 0 {
        return Ok(0);
    }

    // Reset cancel flag
    get_cancel_flag(&config.source_id).store(false, Ordering::Relaxed);
    db::update_source_status(database, &config.source_id, "hashing", total as i64).await?;

    let source_id = config.source_id.clone();
    let algorithms = config.algorithms.clone();
    let app_clone = app.clone();

    // Hash files in parallel using rayon
    let results: Vec<(String, String, hasher::FileHashes)> = std::thread::spawn(move || {
        use rayon::prelude::*;
        use std::sync::atomic::AtomicUsize;

        let counter = AtomicUsize::new(0);

        let results: Vec<_> = files.par_iter().filter_map(|file| {
            if is_cancelled(&source_id) {
                return None;
            }

            let hashes = match hasher::compute_hashes(
                std::path::Path::new(&file.file_path),
                &file.file_type,
                &algorithms,
            ) {
                Ok(h) => h,
                Err(e) => {
                    eprintln!("Hash failed for {}: {}", file.file_path, e);
                    return None;
                }
            };

            let current = counter.fetch_add(1, Ordering::Relaxed) + 1;
            let _ = app_clone.emit(EVENT_HASH_PROGRESS, HashProgressEvent {
                source_id: source_id.clone(),
                current,
                total,
                current_file: file.file_name.clone(),
                algorithm: format!("{:?}", algorithms),
            });

            let file_id = file.id.as_ref().map(|t| t.to_string()).unwrap_or_default();
            Some((file_id, file.file_type.clone(), hashes))
        }).collect();

        results
    })
    .join()
    .map_err(|_| "Hash thread panicked".to_string())?;

    // Update DB with computed hashes
    let mut updated = 0;
    for (file_id, _file_type, hashes) in &results {
        if !file_id.is_empty() {
            db::update_file_hashes(
                database,
                file_id,
                hashes.content_hash.as_deref(),
                hashes.phash.as_deref(),
                hashes.dhash.as_deref(),
            ).await?;
            updated += 1;
        }
    }

    db::update_source_status(database, &config.source_id, "hashed", updated as i64).await?;
    Ok(updated)
}

#[command]
pub async fn compare_source(
    app: AppHandle,
    source_id: String,
    include_self: Option<bool>,
    threshold: Option<u32>,
    algorithms: Option<Vec<String>>,
) -> Result<Vec<db::DuplicateGroupExpanded>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let database = db::get_db(&app_data_dir).await?;

    // Clear old comparison results for this source
    db::clear_duplicate_groups_for_source(database, &source_id).await?;

    let source_files = db::get_files_by_source(database, &source_id).await?;

    if source_files.is_empty() {
        return Ok(vec![]);
    }

    let include_self_flag = include_self.unwrap_or(false);
    let threshold_val = threshold.unwrap_or(matcher::similar::DEFAULT_SIMILARITY_THRESHOLD);
    let algos = algorithms.unwrap_or_else(|| vec!["blake3".into(), "pHash".into()]);

    let use_blake3 = algos.iter().any(|a| a == "blake3");
    let use_phash = algos.iter().any(|a| a == "pHash");
    let use_dhash = algos.iter().any(|a| a == "dHash");

    let mut all_matches: Vec<matcher::MatchPair> = Vec::new();

    // Find exact matches (Blake3)
    if use_blake3 {
        let app_clone = app.clone();
        let sid = source_id.clone();
        let exact_matches = matcher::exact::find_exact_matches(
            database,
            &source_files,
            include_self_flag,
            |current, total| {
                let _ = app_clone.emit(EVENT_COMPARE_PROGRESS, CompareProgressEvent {
                    source_id: sid.clone(),
                    current,
                    total,
                    matches_found: 0,
                });
            },
        ).await?;
        all_matches.extend(exact_matches);
    }

    // Find similar matches (pHash)
    if use_phash {
        let app_clone = app.clone();
        let sid = source_id.clone();
        let phash_matches = matcher::similar::find_similar_matches(
            database,
            &source_id,
            &source_files,
            include_self_flag,
            threshold_val,
            "phash",
            |current, total| {
                let _ = app_clone.emit(EVENT_COMPARE_PROGRESS, CompareProgressEvent {
                    source_id: sid.clone(),
                    current,
                    total,
                    matches_found: 0,
                });
            },
        ).await?;
        all_matches.extend(phash_matches);
    }

    // Find similar matches (dHash)
    if use_dhash {
        let app_clone = app.clone();
        let sid = source_id.clone();
        let dhash_matches = matcher::similar::find_similar_matches(
            database,
            &source_id,
            &source_files,
            include_self_flag,
            threshold_val,
            "dhash",
            |current, total| {
                let _ = app_clone.emit(EVENT_COMPARE_PROGRESS, CompareProgressEvent {
                    source_id: sid.clone(),
                    current,
                    total,
                    matches_found: 0,
                });
            },
        ).await?;
        all_matches.extend(dhash_matches);
    }

    // Group by new file -> collect all matching existing files
    let mut groups: HashMap<String, Vec<&matcher::MatchPair>> = HashMap::new();
    for m in &all_matches {
        groups.entry(m.new_file_id.clone()).or_default().push(m);
    }

    for (new_file_id, matches) in &groups {
        // Collect unique member IDs (new file + all matching existing files)
        let mut member_ids: Vec<String> = vec![new_file_id.clone()];
        let mut seen: HashSet<String> = HashSet::new();
        seen.insert(new_file_id.clone());

        let mut best_score = 0.0f64;
        let mut best_algo = String::new();
        let mut best_type = String::new();

        for m in matches {
            if seen.insert(m.existing_file_id.clone()) {
                member_ids.push(m.existing_file_id.clone());
            }
            if m.similarity_score > best_score {
                best_score = m.similarity_score;
                best_algo = m.algorithm.clone();
                best_type = m.match_type.clone();
            }
        }

        db::insert_duplicate_group(
            database,
            &best_type,
            best_score,
            &best_algo,
            &member_ids,
            &source_id,
            None,
            None,
        ).await?;
    }

    // Return expanded groups with member details
    expand_groups(database, &source_id).await
}

/// Shared helper: fetch duplicate groups and expand member IDs into full MediaFile records.
async fn expand_groups(
    database: &'static surrealdb::Surreal<surrealdb::engine::local::Db>,
    source_id: &str,
) -> Result<Vec<db::DuplicateGroupExpanded>, String> {
    let groups = db::get_duplicate_groups_by_source(database, source_id).await?;
    let mut expanded = Vec::new();

    for group in groups {
        let mut members = Vec::new();
        for member_id in &group.members {
            if let Ok(file) = db::get_file_by_id(database, member_id).await {
                members.push(file);
            }
        }

        let source_member = if let Some(ref sf_id) = group.source_file {
            db::get_file_by_id(database, sf_id).await.ok()
        } else {
            None
        };
        let target_member = if let Some(ref tf_id) = group.target_file {
            db::get_file_by_id(database, tf_id).await.ok()
        } else {
            None
        };

        expanded.push(db::DuplicateGroupExpanded {
            id: group.id.unwrap_or_default(),
            match_type: group.match_type,
            similarity_score: group.similarity_score,
            algorithm: group.algorithm,
            members,
            source_member,
            target_member,
            created_at: group.created_at,
        });
    }

    Ok(expanded)
}

#[command]
pub async fn get_duplicate_groups(
    app: AppHandle,
    source_id: String,
) -> Result<Vec<db::DuplicateGroupExpanded>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let database = db::get_db(&app_data_dir).await?;
    expand_groups(database, &source_id).await
}

#[command]
pub async fn get_files_by_source(app: AppHandle, source_id: String) -> Result<Vec<db::MediaFile>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let database = db::get_db(&app_data_dir).await?;
    db::get_files_by_source(database, &source_id).await
}

#[command]
pub async fn get_dedup_stats(app: AppHandle) -> Result<db::DedupStats, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let database = db::get_db(&app_data_dir).await?;
    db::get_dedup_stats(database).await
}

#[command]
pub async fn set_source_role(
    app: AppHandle,
    source_id: String,
    role: String,
) -> Result<db::ScanSource, String> {
    if role != "source" && role != "target" {
        return Err(format!("Invalid role: {}. Must be 'source' or 'target'", role));
    }

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let database = db::get_db(&app_data_dir).await?;

    // Clear comparison results when roles change
    db::clear_all_duplicate_groups(database).await?;

    db::update_source_role(database, &source_id, &role).await
}

#[command]
pub async fn compare_targets(
    app: AppHandle,
    threshold: Option<u32>,
    algorithms: Option<Vec<String>>,
) -> Result<Vec<db::DuplicateGroupExpanded>, String> {
    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let database = db::get_db(&app_data_dir).await?;

    // Find the source-role scan_source
    let source_entry = db::get_source_role_source(database).await?
        .ok_or_else(|| "No source directory designated. Please set a directory as Source first.".to_string())?;
    let source_scan_id = source_entry.id
        .ok_or_else(|| "Source has no ID".to_string())?;

    // Load source files (the reference pool)
    let source_files = db::get_files_by_source(database, &source_scan_id).await?;
    if source_files.is_empty() {
        return Err("Source directory has no files. Please scan it first.".to_string());
    }

    // Find all target-role scan_sources
    let targets = db::get_scan_sources_by_role(database, "target").await?;
    if targets.is_empty() {
        return Err("No target directories to compare against.".to_string());
    }

    // Clear old comparison results
    db::clear_duplicate_groups_for_source(database, &source_scan_id).await?;

    let threshold_val = threshold.unwrap_or(matcher::similar::DEFAULT_SIMILARITY_THRESHOLD);
    let algos = algorithms.unwrap_or_else(|| vec!["blake3".into(), "pHash".into()]);
    let use_blake3 = algos.iter().any(|a| a == "blake3");
    let use_phash = algos.iter().any(|a| a == "pHash");
    let use_dhash = algos.iter().any(|a| a == "dHash");

    let mut all_matches: Vec<matcher::MatchPair> = Vec::new();
    let mut total_target_files = 0usize;
    let mut processed_target_files = 0usize;

    // Count total target files for progress
    for target in &targets {
        if let Some(ref tid) = target.id {
            let files = db::get_files_by_source(database, tid).await?;
            total_target_files += files.len();
        }
    }

    // Compare each target's files against the source pool
    for target in &targets {
        let target_id = match target.id {
            Some(ref id) => id.clone(),
            None => continue,
        };

        let target_files = db::get_files_by_source(database, &target_id).await?;
        if target_files.is_empty() {
            continue;
        }

        if use_blake3 {
            let app_clone = app.clone();
            let sid = source_scan_id.clone();
            let offset = processed_target_files;
            let total = total_target_files;
            let exact = matcher::exact::find_exact_matches_against_pool(
                &target_files,
                &source_files,
                |current, _| {
                    let _ = app_clone.emit(EVENT_COMPARE_PROGRESS, CompareProgressEvent {
                        source_id: sid.clone(),
                        current: offset + current,
                        total,
                        matches_found: 0,
                    });
                },
            );
            all_matches.extend(exact);
        }

        if use_phash {
            let app_clone = app.clone();
            let sid = source_scan_id.clone();
            let offset = processed_target_files;
            let total = total_target_files;
            let phash_matches = matcher::similar::find_similar_matches_against_pool(
                &target_files,
                &source_files,
                threshold_val,
                "phash",
                |current, _| {
                    let _ = app_clone.emit(EVENT_COMPARE_PROGRESS, CompareProgressEvent {
                        source_id: sid.clone(),
                        current: offset + current,
                        total,
                        matches_found: 0,
                    });
                },
            );
            all_matches.extend(phash_matches);
        }

        if use_dhash {
            let app_clone = app.clone();
            let sid = source_scan_id.clone();
            let offset = processed_target_files;
            let total = total_target_files;
            let dhash_matches = matcher::similar::find_similar_matches_against_pool(
                &target_files,
                &source_files,
                threshold_val,
                "dhash",
                |current, _| {
                    let _ = app_clone.emit(EVENT_COMPARE_PROGRESS, CompareProgressEvent {
                        source_id: sid.clone(),
                        current: offset + current,
                        total,
                        matches_found: 0,
                    });
                },
            );
            all_matches.extend(dhash_matches);
        }

        processed_target_files += target_files.len();
    }

    // Create one DuplicateGroup per match pair (source file left, target file right)
    for m in &all_matches {
        let member_ids = vec![m.existing_file_id.clone(), m.new_file_id.clone()];
        db::insert_duplicate_group(
            database,
            &m.match_type,
            m.similarity_score,
            &m.algorithm,
            &member_ids,
            &source_scan_id,
            Some(&m.existing_file_id),  // source_file = the reference file
            Some(&m.new_file_id),       // target_file = the duplicate
        ).await?;
    }

    expand_groups(database, &source_scan_id).await
}

#[command]
pub async fn trash_files(
    app: AppHandle,
    file_ids: Vec<String>,
) -> Result<db::TrashResult, String> {
    if file_ids.is_empty() {
        return Ok(db::TrashResult { trashed: 0, failed: 0, errors: Vec::new() });
    }

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let database = db::get_db(&app_data_dir).await?;

    let mut failed = 0usize;
    let mut errors: Vec<String> = Vec::new();

    // Phase 1 — resolve each file_id against the DB. Anything we cannot resolve
    // counts as a failure now and is excluded from the batch trash call.
    struct Resolved {
        id: String,
        path: String,
        source_id: String,
    }
    let mut resolved: Vec<Resolved> = Vec::with_capacity(file_ids.len());

    for file_id in &file_ids {
        match db::get_file_by_id(database, file_id).await {
            Ok(file) => resolved.push(Resolved {
                id: file_id.clone(),
                path: file.file_path,
                source_id: file.source.unwrap_or_default(),
            }),
            Err(e) => {
                failed += 1;
                errors.push(format!("{}: {}", file_id, e));
            }
        }
    }

    // Phase 2 — batch the trash call into a single OS invocation so Finder only
    // plays one trash sound (and one swoosh animation) regardless of count.
    let mut trashed = 0usize;
    if !resolved.is_empty() {
        let paths: Vec<&std::path::Path> = resolved
            .iter()
            .map(|r| std::path::Path::new(&r.path))
            .collect();

        match trash::delete_all(&paths) {
            Ok(_) => {
                trashed = resolved.len();

                // Phase 3 — DB cleanup, grouped by source. Only runs on a
                // successful batch so the DB stays consistent with disk.
                let mut source_to_paths: HashMap<String, Vec<String>> = HashMap::new();
                let mut source_to_ids: HashMap<String, Vec<String>> = HashMap::new();
                for r in &resolved {
                    source_to_paths
                        .entry(r.source_id.clone())
                        .or_default()
                        .push(r.path.clone());
                    source_to_ids
                        .entry(r.source_id.clone())
                        .or_default()
                        .push(r.id.clone());
                }
                for (source_id, paths) in &source_to_paths {
                    let _ = db::delete_media_files_by_paths(database, source_id, paths).await;
                }
                for (source_id, ids) in &source_to_ids {
                    let _ = db::clean_orphan_duplicate_members(database, source_id, ids).await;
                }
            }
            Err(e) => {
                failed += resolved.len();
                errors.push(format!("batch trash failed: {}", e));
            }
        }
    }

    Ok(db::TrashResult { trashed, failed, errors })
}

#[command]
pub async fn add_files_as_target(
    app: AppHandle,
    file_paths: Vec<String>,
) -> Result<db::ScanSource, String> {
    use crate::scanner::filter;

    let app_data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let database = db::get_db(&app_data_dir).await?;

    // Validate files
    let valid_files: Vec<&String> = file_paths.iter()
        .filter(|p| {
            let path = std::path::Path::new(p);
            path.is_file() && filter::is_supported_media(path)
        })
        .collect();

    if valid_files.is_empty() {
        return Err("No supported media files found in the selection.".to_string());
    }

    // Find or create the "Individual Files" virtual source
    let all_sources = db::get_all_scan_sources(database).await?;
    let virtual_source = all_sources.iter().find(|s| s.label == "Individual Files" && s.role == "target");

    let source = if let Some(existing) = virtual_source {
        existing.clone()
    } else {
        // Use a placeholder path for the virtual source
        let placeholder = format!("__individual_files_{}", uuid::Uuid::new_v4());
        db::insert_scan_source(database, &placeholder, "Individual Files", "target").await?
    };

    let source_id = source.id.as_ref().ok_or("Source has no ID")?;

    // Build MediaFile records
    let files: Vec<db::MediaFile> = valid_files.iter().map(|p| {
        let path = std::path::Path::new(p);
        let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let file_size = std::fs::metadata(path).map(|m| m.len() as i64).unwrap_or(0);
        let file_type = filter::classify_extension(path);
        let parent_dir = path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();

        db::MediaFile {
            id: None,
            file_path: p.to_string(),
            file_name,
            file_size,
            file_type,
            parent_dir,
            content_hash: None,
            phash: None,
            dhash: None,
            source: None,
            scanned_at: None,
        }
    }).collect();

    let added = db::insert_media_files_batch(database, &files, source_id).await?;
    db::update_source_status(database, source_id, "scanned", added as i64).await?;

    // Return updated source
    db::get_scan_source(database, source_id).await
}

#[command]
pub fn cancel_dedup(source_id: String) -> Result<(), String> {
    get_cancel_flag(&source_id).store(true, Ordering::Relaxed);
    Ok(())
}
