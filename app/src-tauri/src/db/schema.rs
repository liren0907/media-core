use surrealdb::Surreal;
use surrealdb::engine::local::Db;

pub async fn init_schema(db: &Surreal<Db>) -> Result<(), String> {
    db.query("
        -- Scan source: imported directory paths
        DEFINE TABLE IF NOT EXISTS scan_source SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS path         ON scan_source TYPE string;
        DEFINE FIELD IF NOT EXISTS label        ON scan_source TYPE string;
        DEFINE FIELD IF NOT EXISTS file_count   ON scan_source TYPE int DEFAULT 0;
        DEFINE FIELD IF NOT EXISTS status       ON scan_source TYPE string DEFAULT 'pending';
        DEFINE FIELD IF NOT EXISTS role         ON scan_source TYPE string DEFAULT 'target';
        DEFINE FIELD IF NOT EXISTS created_at      ON scan_source TYPE datetime DEFAULT time::now();
        DEFINE FIELD IF NOT EXISTS last_scanned_at ON scan_source TYPE option<datetime>;
        DEFINE INDEX IF NOT EXISTS idx_scan_source_path ON scan_source COLUMNS path UNIQUE;

        -- Media file: individual file records with fingerprints
        DEFINE TABLE IF NOT EXISTS media_file SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS file_path       ON media_file TYPE string;
        DEFINE FIELD IF NOT EXISTS file_name       ON media_file TYPE string;
        DEFINE FIELD IF NOT EXISTS file_size       ON media_file TYPE int;
        DEFINE FIELD IF NOT EXISTS file_type       ON media_file TYPE string;
        DEFINE FIELD IF NOT EXISTS parent_dir      ON media_file TYPE string;
        DEFINE FIELD IF NOT EXISTS content_hash    ON media_file TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS phash           ON media_file TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS dhash           ON media_file TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS source          ON media_file TYPE record<scan_source>;
        DEFINE FIELD IF NOT EXISTS scanned_at      ON media_file TYPE datetime DEFAULT time::now();
        DEFINE INDEX IF NOT EXISTS idx_media_file_path         ON media_file COLUMNS file_path UNIQUE;
        DEFINE INDEX IF NOT EXISTS idx_media_file_content_hash ON media_file COLUMNS content_hash;
        DEFINE INDEX IF NOT EXISTS idx_media_file_phash        ON media_file COLUMNS phash;
        DEFINE INDEX IF NOT EXISTS idx_media_file_source       ON media_file COLUMNS source;

        -- Duplicate group: comparison results
        DEFINE TABLE IF NOT EXISTS duplicate_group SCHEMAFULL;
        DEFINE FIELD IF NOT EXISTS match_type        ON duplicate_group TYPE string;
        DEFINE FIELD IF NOT EXISTS similarity_score   ON duplicate_group TYPE float;
        DEFINE FIELD IF NOT EXISTS algorithm          ON duplicate_group TYPE string;
        DEFINE FIELD IF NOT EXISTS members            ON duplicate_group TYPE array;
        DEFINE FIELD IF NOT EXISTS source_file       ON duplicate_group TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS target_file       ON duplicate_group TYPE option<string>;
        DEFINE FIELD IF NOT EXISTS source             ON duplicate_group TYPE record<scan_source>;
        DEFINE FIELD IF NOT EXISTS created_at         ON duplicate_group TYPE datetime DEFAULT time::now();
    ")
    .await
    .map_err(|e| format!("Schema init failed: {}", e))?;

    // Backfill existing records that lack the new role field
    db.query("UPDATE scan_source SET role = 'target' WHERE role = NONE;")
        .await
        .map_err(|e| format!("Migration failed: {}", e))?;

    Ok(())
}
