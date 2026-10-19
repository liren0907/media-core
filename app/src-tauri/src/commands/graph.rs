//! Composer graph commands.
//!
//! The frontend hard-codes no method names: `list_graph_methods` is the single
//! source of truth for what can be placed on the canvas and what can be tuned.

use std::path::PathBuf;

use tauri::{command, AppHandle, Manager};

use crate::graph::manifest::{MethodMeta, METHODS};
use crate::graph::types::{GraphListItem, GraphRecipe, GraphRunResult, GraphVerdict, SavedGraph};
use crate::graph::{runner, store, validate};

/// Saved graphs live under the app data dir, next to the recordings and the dedup
/// database.
fn graphs_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("composer")
        .join("graphs"))
}

/// Everything the canvas needs to build its picker and its inspector.
#[command]
pub async fn list_graph_methods() -> Result<Vec<&'static MethodMeta>, String> {
    Ok(METHODS.iter().collect())
}

/// Check a graph without running it. Never errors on a bad graph — the verdict
/// carries the reason so the UI can show it inline.
#[command]
pub async fn validate_graph(recipe: GraphRecipe) -> Result<GraphVerdict, String> {
    Ok(validate::validate(&recipe))
}

/// Validate, then execute in topological order.
///
/// Runs on the blocking pool: the steps underneath are blocking and several of
/// them touch OpenCV, which must not run on the async executor — and awaiting the
/// pool, unlike joining a thread, leaves the async worker free meanwhile.
///
/// `run_id` lets the caller know the id before the first progress event, so a
/// cancel issued right after Run already reaches this run. Without one, an id is
/// generated here.
#[command]
pub async fn execute_graph(
    app: tauri::AppHandle,
    recipe: GraphRecipe,
    run_id: Option<String>,
) -> Result<GraphRunResult, String> {
    let run_id = run_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let cancel = runner::register_run(&run_id)?;

    // Registry cleanup and `graph:finished` happen inside, on every path; see
    // `runner::RunGuard`.
    tauri::async_runtime::spawn_blocking(move || {
        runner::run_graph(&app, &recipe, &run_id, &cancel)
    })
    .await
    .map_err(|_| runner::PANICKED.to_string())?
}

#[command]
pub async fn cancel_graph_run(run_id: String) -> Result<bool, String> {
    Ok(runner::cancel_run(&run_id))
}

/// Saved graphs, newest first, each judged as it stands now.
#[command]
pub async fn list_graphs(app: AppHandle) -> Result<Vec<GraphListItem>, String> {
    store::list(&graphs_dir(&app)?)
}

#[command]
pub async fn load_graph(app: AppHandle, id: String) -> Result<GraphRecipe, String> {
    store::load(&graphs_dir(&app)?, &id)
}

/// Without an id this saves a new graph; with one it overwrites that graph. Only a
/// graph that could not be read back is refused — one that cannot run yet is saved.
#[command]
pub async fn save_graph(
    app: AppHandle,
    id: Option<String>,
    recipe: GraphRecipe,
) -> Result<SavedGraph, String> {
    store::save(&graphs_dir(&app)?, id.as_deref(), recipe)
}

#[command]
pub async fn delete_graph(app: AppHandle, id: String) -> Result<(), String> {
    store::delete(&graphs_dir(&app)?, &id)
}
