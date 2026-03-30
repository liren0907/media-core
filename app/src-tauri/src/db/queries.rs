use serde::{Deserialize, Serialize};
use surrealdb::Surreal;
use surrealdb::engine::local::Db;
use surrealdb::types::{Datetime, RecordId, SurrealValue};

fn record_id_to_string(id: &RecordId) -> String {
    let key = match &id.key {
        surrealdb::types::RecordIdKey::String(s) => s.clone(),
        surrealdb::types::RecordIdKey::Number(n) => n.to_string(),
        surrealdb::types::RecordIdKey::Uuid(u) => u.to_string(),
        other => format!("{:?}", other),
    };
    format!("{}:{}", id.table, key)
}

// ──────────────────────────────────────────────
// Internal DB types (for SurrealDB query results)
// ──────────────────────────────────────────────

#[derive(Debug, Clone, SurrealValue)]
struct DbScanSource {
    id: Option<RecordId>,
    path: String,
    label: String,
    file_count: i64,
    status: String,
    role: String,
    created_at: Option<Datetime>,
    last_scanned_at: Option<Datetime>,
}

impl DbScanSource {
    fn into_api(self) -> ScanSource {
        ScanSource {
            id: self.id.as_ref().map(record_id_to_string),
            path: self.path,
            label: self.label,
            file_count: self.file_count,
            status: self.status,
            role: self.role,
            created_at: self.created_at.map(|d| d.to_string()),
            last_scanned_at: self.last_scanned_at.map(|d| d.to_string()),
        }
    }
}

#[derive(Debug, Clone, SurrealValue)]
struct DbMediaFile {
    id: Option<RecordId>,
    file_path: String,
    file_name: String,
    file_size: i64,
    file_type: String,
    parent_dir: String,
    content_hash: Option<String>,
    phash: Option<String>,
    dhash: Option<String>,
    source: Option<RecordId>,
    scanned_at: Option<Datetime>,
}

impl DbMediaFile {
    fn into_api(self) -> MediaFile {
        MediaFile {
            id: self.id.as_ref().map(record_id_to_string),
            file_path: self.file_path,
            file_name: self.file_name,
            file_size: self.file_size,
            file_type: self.file_type,
            parent_dir: self.parent_dir,
            content_hash: self.content_hash,
            phash: self.phash,
            dhash: self.dhash,
            source: self.source.as_ref().map(record_id_to_string),
            scanned_at: self.scanned_at.map(|d| d.to_string()),
        }
    }
}

#[derive(Debug, Clone, SurrealValue)]
struct DbDuplicateGroup {
    id: Option<RecordId>,
    match_type: String,
    similarity_score: f64,
    algorithm: String,
    members: Vec<String>,
    source_file: Option<String>,
    target_file: Option<String>,
    source: Option<RecordId>,
    created_at: Option<Datetime>,
}

impl DbDuplicateGroup {
    fn into_api(self) -> DuplicateGroup {
        DuplicateGroup {
            id: self.id.as_ref().map(record_id_to_string),
            match_type: self.match_type,
            similarity_score: self.similarity_score,
            algorithm: self.algorithm,
            members: self.members,
            source_file: self.source_file,
            target_file: self.target_file,
            source: self.source.as_ref().map(record_id_to_string),
            created_at: self.created_at.map(|d| d.to_string()),
        }
    }
}

#[derive(Debug, Clone, SurrealValue)]
struct CountResult {
    c: i64,
}

// ──────────────────────────────────────────────
// API types (serialized to frontend via Tauri commands)
// ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSource {
    pub id: Option<String>,
    pub path: String,
    pub label: String,
    pub file_count: i64,
    pub status: String,
    pub role: String,
    pub created_at: Option<String>,
    pub last_scanned_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaFile {
    pub id: Option<String>,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub file_type: String,
    pub parent_dir: String,
    pub content_hash: Option<String>,
    pub phash: Option<String>,
    pub dhash: Option<String>,
    pub source: Option<String>,
    pub scanned_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    pub id: Option<String>,
    pub match_type: String,
    pub similarity_score: f64,
    pub algorithm: String,
    pub members: Vec<String>,
    pub source_file: Option<String>,
    pub target_file: Option<String>,
    pub source: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroupExpanded {
    pub id: String,
    pub match_type: String,
    pub similarity_score: f64,
    pub algorithm: String,
    pub members: Vec<MediaFile>,
    pub source_member: Option<MediaFile>,
    pub target_member: Option<MediaFile>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedupStats {
    pub total_sources: i64,
    pub total_files: i64,
    pub total_hashed: i64,
    pub total_duplicate_groups: i64,
    pub total_duplicates: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashResult {
    pub trashed: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalScanResult {
    pub added: usize,
    pub removed: usize,
    pub updated: usize,
    pub unchanged: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<DirectoryNode>,
    pub file_size: Option<i64>,
}

// ──────────────────────────────────────────────
// Scan Source Queries
// ──────────────────────────────────────────────

pub async fn insert_scan_source(db: &Surreal<Db>, path: &str, label: &str, role: &str) -> Result<ScanSource, String> {
    let mut result = db
        .query("CREATE scan_source SET path = $path, label = $label, role = $role")
        .bind(("path", path.to_string()))
        .bind(("label", label.to_string()))
        .bind(("role", role.to_string()))
        .await
        .map_err(|e| format!("Failed to insert scan source: {}", e))?;

    let source: Option<DbScanSource> = result.take(0).map_err(|e| format!("Failed to parse result: {}", e))?;
    source.map(|s| s.into_api()).ok_or_else(|| "Failed to create scan source".to_string())
}

pub async fn get_all_scan_sources(db: &Surreal<Db>) -> Result<Vec<ScanSource>, String> {
    let mut result = db
        .query("SELECT * FROM scan_source ORDER BY created_at DESC")
        .await
        .map_err(|e| format!("Failed to query sources: {}", e))?;

    let sources: Vec<DbScanSource> = result.take(0).map_err(|e| format!("Failed to parse sources: {}", e))?;
    Ok(sources.into_iter().map(|s| s.into_api()).collect())
}

pub async fn get_scan_source(db: &Surreal<Db>, id: &str) -> Result<ScanSource, String> {
    let mut result = db
        .query("SELECT * FROM type::record($id)")
        .bind(("id", id.to_string()))
        .await
        .map_err(|e| format!("Failed to query source: {}", e))?;

    let source: Option<DbScanSource> = result.take(0).map_err(|e| format!("Failed to parse source: {}", e))?;
    source.map(|s| s.into_api()).ok_or_else(|| format!("Source not found: {}", id))
}

pub async fn update_source_status(db: &Surreal<Db>, id: &str, status: &str, file_count: i64) -> Result<(), String> {
    db.query("UPDATE type::record($id) SET status = $status, file_count = $file_count, last_scanned_at = time::now()")
        .bind(("id", id.to_string()))
        .bind(("status", status.to_string()))
        .bind(("file_count", file_count))
        .await
        .map_err(|e| format!("Failed to update source: {}", e))?;
    Ok(())
}

pub async fn update_source_role(db: &Surreal<Db>, id: &str, role: &str) -> Result<ScanSource, String> {
    if role == "source" {
        // Demote any existing source to target first (singleton constraint)
        db.query("UPDATE scan_source SET role = 'target' WHERE role = 'source'")
            .await
            .map_err(|e| format!("Failed to demote existing source: {}", e))?;
    }
    let mut result = db
        .query("UPDATE type::record($id) SET role = $role")
        .bind(("id", id.to_string()))
        .bind(("role", role.to_string()))
        .await
        .map_err(|e| format!("Failed to update role: {}", e))?;

    let source: Option<DbScanSource> = result.take(0).map_err(|e| format!("Failed to parse source: {}", e))?;
    source.map(|s| s.into_api()).ok_or_else(|| format!("Source not found: {}", id))
}

pub async fn get_source_role_source(db: &Surreal<Db>) -> Result<Option<ScanSource>, String> {
    let mut result = db
        .query("SELECT * FROM scan_source WHERE role = 'source' LIMIT 1")
        .await
        .map_err(|e| format!("Failed to query source role: {}", e))?;

    let source: Option<DbScanSource> = result.take(0).map_err(|e| format!("Failed to parse source: {}", e))?;
    Ok(source.map(|s| s.into_api()))
}

pub async fn get_scan_sources_by_role(db: &Surreal<Db>, role: &str) -> Result<Vec<ScanSource>, String> {
    let mut result = db
        .query("SELECT * FROM scan_source WHERE role = $role ORDER BY created_at DESC")
        .bind(("role", role.to_string()))
        .await
        .map_err(|e| format!("Failed to query sources by role: {}", e))?;

    let sources: Vec<DbScanSource> = result.take(0).map_err(|e| format!("Failed to parse sources: {}", e))?;
    Ok(sources.into_iter().map(|s| s.into_api()).collect())
}

pub async fn get_hashed_files_by_source_ids(db: &Surreal<Db>, source_ids: &[String]) -> Result<Vec<MediaFile>, String> {
    let mut all_files = Vec::new();
    for sid in source_ids {
        let mut result = db
            .query("SELECT * FROM media_file WHERE source = type::record($sid) AND (content_hash IS NOT NONE OR phash IS NOT NONE OR dhash IS NOT NONE)")
            .bind(("sid", sid.to_string()))
            .await
            .map_err(|e| format!("Failed to query hashed files: {}", e))?;

        let files: Vec<DbMediaFile> = result.take(0).map_err(|e| format!("Failed to parse files: {}", e))?;
        all_files.extend(files.into_iter().map(|f| f.into_api()));
    }
    Ok(all_files)
}

pub async fn clear_all_duplicate_groups(db: &Surreal<Db>) -> Result<(), String> {
    db.query("DELETE duplicate_group")
        .await
        .map_err(|e| format!("Failed to clear all groups: {}", e))?;
    Ok(())
}

pub async fn delete_scan_source_cascade(db: &Surreal<Db>, id: &str) -> Result<(), String> {
    db.query("
        DELETE duplicate_group WHERE source = type::record($id);
        DELETE media_file WHERE source = type::record($id);
        DELETE type::record($id);
    ")
    .bind(("id", id.to_string()))
    .await
    .map_err(|e| format!("Failed to delete source: {}", e))?;
    Ok(())
}

// ──────────────────────────────────────────────
// Media File Queries
// ──────────────────────────────────────────────

pub async fn insert_media_files_batch(
    db: &Surreal<Db>,
    files: &[MediaFile],
    source_id: &str,
) -> Result<usize, String> {
    let mut count = 0;
    for chunk in files.chunks(100) {
        // Build a transaction for each chunk to reduce per-record commit overhead
        let mut query_str = String::from("BEGIN TRANSACTION;\n");
        for (i, _file) in chunk.iter().enumerate() {
            query_str.push_str(&format!(
                "INSERT INTO media_file {{
                    file_path: $fp{i},
                    file_name: $fn{i},
                    file_size: $fs{i},
                    file_type: $ft{i},
                    parent_dir: $pd{i},
                    source: type::record($sid)
                }} ON DUPLICATE KEY UPDATE file_size = $fs{i};\n"
            ));
        }
        query_str.push_str("COMMIT TRANSACTION;");

        let mut q = db.query(&query_str);
        q = q.bind(("sid", source_id.to_string()));
        for (i, file) in chunk.iter().enumerate() {
            q = q.bind((format!("fp{i}"), file.file_path.clone()));
            q = q.bind((format!("fn{i}"), file.file_name.clone()));
            q = q.bind((format!("fs{i}"), file.file_size));
            q = q.bind((format!("ft{i}"), file.file_type.clone()));
            q = q.bind((format!("pd{i}"), file.parent_dir.clone()));
        }

        q.await.map_err(|e| format!("Failed to batch insert files: {}", e))?;
        count += chunk.len();
    }
    Ok(count)
}

pub async fn get_files_by_source(db: &Surreal<Db>, source_id: &str) -> Result<Vec<MediaFile>, String> {
    let mut result = db
        .query("SELECT * FROM media_file WHERE source = type::record($source_id)")
        .bind(("source_id", source_id.to_string()))
        .await
        .map_err(|e| format!("Failed to query files: {}", e))?;

    let files: Vec<DbMediaFile> = result.take(0).map_err(|e| format!("Failed to parse files: {}", e))?;
    Ok(files.into_iter().map(|f| f.into_api()).collect())
}

pub async fn update_file_hashes(
    db: &Surreal<Db>,
    file_id: &str,
    content_hash: Option<&str>,
    phash: Option<&str>,
    dhash: Option<&str>,
) -> Result<(), String> {
    db.query("UPDATE type::record($id) SET content_hash = $content_hash, phash = $phash, dhash = $dhash")
        .bind(("id", file_id.to_string()))
        .bind(("content_hash", content_hash.map(|s| s.to_string())))
        .bind(("phash", phash.map(|s| s.to_string())))
        .bind(("dhash", dhash.map(|s| s.to_string())))
        .await
        .map_err(|e| format!("Failed to update hashes: {}", e))?;
    Ok(())
}

pub async fn get_file_by_id(db: &Surreal<Db>, id: &str) -> Result<MediaFile, String> {
    let mut result = db
        .query("SELECT * FROM type::record($id)")
        .bind(("id", id.to_string()))
        .await
        .map_err(|e| format!("Failed to query file: {}", e))?;

    let file: Option<DbMediaFile> = result.take(0).map_err(|e| format!("Failed to parse file: {}", e))?;
    file.map(|f| f.into_api()).ok_or_else(|| format!("File not found: {}", id))
}

pub async fn get_files_by_content_hash(db: &Surreal<Db>, hash: &str) -> Result<Vec<MediaFile>, String> {
    let mut result = db
        .query("SELECT * FROM media_file WHERE content_hash = $hash")
        .bind(("hash", hash.to_string()))
        .await
        .map_err(|e| format!("Failed to query by hash: {}", e))?;

    let files: Vec<DbMediaFile> = result.take(0).map_err(|e| format!("Failed to parse files: {}", e))?;
    Ok(files.into_iter().map(|f| f.into_api()).collect())
}

pub async fn get_all_hashed_files(db: &Surreal<Db>) -> Result<Vec<MediaFile>, String> {
    let mut result = db
        .query("SELECT * FROM media_file WHERE content_hash IS NOT NONE OR phash IS NOT NONE OR dhash IS NOT NONE")
        .await
        .map_err(|e| format!("Failed to query hashed files: {}", e))?;

    let files: Vec<DbMediaFile> = result.take(0).map_err(|e| format!("Failed to parse files: {}", e))?;
    Ok(files.into_iter().map(|f| f.into_api()).collect())
}

pub async fn get_all_hashed_files_excluding_source(db: &Surreal<Db>, source_id: &str) -> Result<Vec<MediaFile>, String> {
    let mut result = db
        .query("SELECT * FROM media_file WHERE source != type::record($source_id) AND (content_hash IS NOT NONE OR phash IS NOT NONE OR dhash IS NOT NONE)")
        .bind(("source_id", source_id.to_string()))
        .await
        .map_err(|e| format!("Failed to query hashed files: {}", e))?;

    let files: Vec<DbMediaFile> = result.take(0).map_err(|e| format!("Failed to parse files: {}", e))?;
    Ok(files.into_iter().map(|f| f.into_api()).collect())
}

pub async fn delete_media_files_by_paths(db: &Surreal<Db>, source_id: &str, paths: &[String]) -> Result<usize, String> {
    let mut deleted = 0;
    for chunk in paths.chunks(100) {
        let chunk_vec: Vec<String> = chunk.to_vec();
        db.query("DELETE media_file WHERE source = type::record($source_id) AND file_path IN $paths")
            .bind(("source_id", source_id.to_string()))
            .bind(("paths", chunk_vec))
            .await
            .map_err(|e| format!("Failed to delete files: {}", e))?;
        deleted += chunk.len();
    }
    Ok(deleted)
}

pub async fn clear_file_hashes_by_id(db: &Surreal<Db>, file_id: &str) -> Result<(), String> {
    db.query("UPDATE type::record($id) SET content_hash = NONE, phash = NONE, dhash = NONE")
        .bind(("id", file_id.to_string()))
        .await
        .map_err(|e| format!("Failed to clear hashes: {}", e))?;
    Ok(())
}

// ──────────────────────────────────────────────
// Duplicate Group Queries
// ──────────────────────────────────────────────

pub async fn insert_duplicate_group(
    db: &Surreal<Db>,
    match_type: &str,
    similarity_score: f64,
    algorithm: &str,
    member_ids: &[String],
    source_id: &str,
    source_file: Option<&str>,
    target_file: Option<&str>,
) -> Result<(), String> {
    db.query("CREATE duplicate_group SET
        match_type = $match_type,
        similarity_score = $similarity_score,
        algorithm = $algorithm,
        members = $members,
        source_file = $source_file,
        target_file = $target_file,
        source = type::record($source_id)
    ")
    .bind(("match_type", match_type.to_string()))
    .bind(("similarity_score", similarity_score))
    .bind(("algorithm", algorithm.to_string()))
    .bind(("members", member_ids.to_vec()))
    .bind(("source_file", source_file.map(|s| s.to_string())))
    .bind(("target_file", target_file.map(|s| s.to_string())))
    .bind(("source_id", source_id.to_string()))
    .await
    .map_err(|e| format!("Failed to insert duplicate group: {}", e))?;
    Ok(())
}

pub async fn get_duplicate_groups_by_source(db: &Surreal<Db>, source_id: &str) -> Result<Vec<DuplicateGroup>, String> {
    let mut result = db
        .query("SELECT * FROM duplicate_group WHERE source = type::record($source_id) ORDER BY similarity_score DESC")
        .bind(("source_id", source_id.to_string()))
        .await
        .map_err(|e| format!("Failed to query groups: {}", e))?;

    let groups: Vec<DbDuplicateGroup> = result.take(0).map_err(|e| format!("Failed to parse groups: {}", e))?;
    Ok(groups.into_iter().map(|g| g.into_api()).collect())
}

pub async fn clear_duplicate_groups_for_source(db: &Surreal<Db>, source_id: &str) -> Result<(), String> {
    db.query("DELETE duplicate_group WHERE source = type::record($source_id)")
        .bind(("source_id", source_id.to_string()))
        .await
        .map_err(|e| format!("Failed to clear groups: {}", e))?;
    Ok(())
}

/// Remove deleted file IDs from duplicate_group members, and delete groups with ≤1 member.
pub async fn clean_orphan_duplicate_members(db: &Surreal<Db>, source_id: &str, deleted_file_ids: &[String]) -> Result<(), String> {
    if deleted_file_ids.is_empty() {
        return Ok(());
    }

    let groups = get_duplicate_groups_by_source(db, source_id).await?;

    for group in groups {
        let group_id = match &group.id {
            Some(id) => id.clone(),
            None => continue,
        };

        let remaining: Vec<String> = group.members
            .iter()
            .filter(|m| !deleted_file_ids.contains(m))
            .cloned()
            .collect();

        if remaining.len() <= 1 {
            db.query("DELETE type::record($id)")
                .bind(("id", group_id))
                .await
                .map_err(|e| format!("Failed to delete orphan group: {}", e))?;
        } else if remaining.len() < group.members.len() {
            db.query("UPDATE type::record($id) SET members = $members")
                .bind(("id", group_id))
                .bind(("members", remaining))
                .await
                .map_err(|e| format!("Failed to update group members: {}", e))?;
        }
    }

    Ok(())
}

// ──────────────────────────────────────────────
// Stats
// ──────────────────────────────────────────────

pub async fn get_dedup_stats(db: &Surreal<Db>) -> Result<DedupStats, String> {
    let mut result = db
        .query("
            SELECT count() AS c FROM scan_source GROUP ALL;
            SELECT count() AS c FROM media_file GROUP ALL;
            SELECT count() AS c FROM media_file WHERE content_hash IS NOT NONE OR phash IS NOT NONE GROUP ALL;
            SELECT count() AS c FROM duplicate_group GROUP ALL;
            SELECT math::sum(array::len(members)) AS c FROM duplicate_group GROUP ALL;
        ")
        .await
        .map_err(|e| format!("Failed to get stats: {}", e))?;

    let sources: Option<CountResult> = result.take(0).unwrap_or(None);
    let files: Option<CountResult> = result.take(1).unwrap_or(None);
    let hashed: Option<CountResult> = result.take(2).unwrap_or(None);
    let groups: Option<CountResult> = result.take(3).unwrap_or(None);
    let duplicates: Option<CountResult> = result.take(4).unwrap_or(None);

    Ok(DedupStats {
        total_sources: sources.map(|r| r.c).unwrap_or(0),
        total_files: files.map(|r| r.c).unwrap_or(0),
        total_hashed: hashed.map(|r| r.c).unwrap_or(0),
        total_duplicate_groups: groups.map(|r| r.c).unwrap_or(0),
        total_duplicates: duplicates.map(|r| r.c).unwrap_or(0),
    })
}

// ──────────────────────────────────────────────
// Directory Tree Builder
// ──────────────────────────────────────────────

pub async fn get_directory_tree(db: &Surreal<Db>, source_id: &str) -> Result<Vec<DirectoryNode>, String> {
    let files = get_files_by_source(db, source_id).await?;
    let source = get_scan_source(db, source_id).await?;
    let root_path = &source.path;
    build_tree_from_files(&files, root_path)
}

fn build_tree_from_files(files: &[MediaFile], root_path: &str) -> Result<Vec<DirectoryNode>, String> {
    use std::collections::HashMap;

    let mut dir_children: HashMap<String, Vec<DirectoryNode>> = HashMap::new();
    let mut all_dirs: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    all_dirs.insert(root_path.to_string());

    for file in files {
        let file_node = DirectoryNode {
            name: file.file_name.clone(),
            path: file.file_path.clone(),
            is_dir: false,
            children: vec![],
            file_size: Some(file.file_size),
        };

        dir_children
            .entry(file.parent_dir.clone())
            .or_default()
            .push(file_node);

        let mut current = file.parent_dir.clone();
        while current != root_path && !current.is_empty() {
            all_dirs.insert(current.clone());
            match std::path::Path::new(&current).parent() {
                Some(parent) => {
                    let parent_str = parent.to_string_lossy().to_string();
                    if parent_str == current { break; }
                    current = parent_str;
                }
                None => break,
            }
        }
    }

    fn build_node(
        path: &str,
        dir_children: &HashMap<String, Vec<DirectoryNode>>,
        all_dirs: &std::collections::BTreeSet<String>,
    ) -> DirectoryNode {
        let name = std::path::Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string());

        let mut children: Vec<DirectoryNode> = Vec::new();

        for dir in all_dirs {
            if let Some(parent) = std::path::Path::new(dir).parent() {
                if parent.to_string_lossy() == path && dir != path {
                    children.push(build_node(dir, dir_children, all_dirs));
                }
            }
        }

        if let Some(file_nodes) = dir_children.get(path) {
            children.extend(file_nodes.iter().cloned());
        }

        children.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        DirectoryNode {
            name,
            path: path.to_string(),
            is_dir: true,
            children,
            file_size: None,
        }
    }

    let root = build_node(root_path, &dir_children, &all_dirs);
    Ok(root.children)
}
