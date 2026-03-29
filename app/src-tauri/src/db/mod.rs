pub mod schema;
pub mod queries;

pub use queries::*;

use surrealdb::Surreal;
use surrealdb::engine::local::{Db, SurrealKv};
use std::path::PathBuf;
use std::sync::OnceLock;

static DB: OnceLock<Surreal<Db>> = OnceLock::new();

pub async fn get_db(app_data_dir: &PathBuf) -> Result<&'static Surreal<Db>, String> {
    if let Some(db) = DB.get() {
        return Ok(db);
    }

    let db_path = app_data_dir.join("dedup.surrealkv");

    // Ensure the directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create DB directory: {}", e))?;
    }

    let db = Surreal::new::<SurrealKv>(db_path.to_string_lossy().as_ref())
        .await
        .map_err(|e| format!("Failed to open SurrealDB: {}", e))?;

    db.use_ns("media").use_db("dedup")
        .await
        .map_err(|e| format!("Failed to select namespace: {}", e))?;

    schema::init_schema(&db).await?;

    let _ = DB.set(db);
    DB.get().ok_or_else(|| "Failed to initialize database".to_string())
}
