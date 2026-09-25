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
/// Runs on a dedicated thread: the steps underneath are blocking and several of
/// them touch OpenCV, which must not run on the async executor.
#[command]
pub async fn execute_graph(
    app: tauri::AppHandle,
    recipe: GraphRecipe,
) -> Result<GraphRunResult, String> {
    let run_id = uuid::Uuid::new_v4().to_string();
    let cancel = runner::register_run(&run_id);

    let handle = std::thread::spawn(move || {
        let result = runner::run_graph(&app, &recipe, &run_id, &cancel);
        runner::finish_run(&run_id);
        result
    });

    handle.join().map_err(|_| "The graph run panicked".to_string())?
}

#[command]
pub async fn cancel_graph_run(run_id: String) -> Result<bool, String> {
    Ok(runner::cancel_run(&run_id))
}
