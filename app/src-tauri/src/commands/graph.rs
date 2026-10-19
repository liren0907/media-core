//! Composer graph commands.
//!
//! The frontend hard-codes no method names: `list_graph_methods` is the single
//! source of truth for what can be placed on the canvas and what can be tuned.

use tauri::command;

use crate::graph::manifest::{MethodMeta, METHODS};
use crate::graph::types::{GraphRecipe, GraphRunResult, GraphVerdict};
use crate::graph::{runner, validate};

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
