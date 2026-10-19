//! Graph runner.
//!
//! Why this is not `media_core::pipeline::Pipeline`: that executor collects every
//! step into a `Vec<Box<dyn PipelineStep>>` and only then runs them. Our downstream
//! steps take *constructor arguments* that depend on an upstream step's *result*
//! (a blank `inputDir` inherits the frame directory the previous step just wrote),
//! so construction has to happen one step at a time, interleaved with execution.
//!
//! Everything else is reused verbatim: the steps, the trait, and the `MediaContext`
//! blackboard all come from media-core untouched.
//!
//! Branches drawn side by side execute in topological order, not in parallel. Data
//! lives on the blackboard rather than on the edges, so the result is the same; the
//! only thing given up is a speed-up.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use tauri::Emitter;

use super::types::{
    analysis_to_dto, AnnotationResultDto, GraphNode, GraphRecipe, GraphRunResult, HlsResultDto,
    ProcessResultDto, RunOutputDto, RunStatus, VideoProcessResultDto,
};
use super::{manifest, registry, validate};

/// The error a run returns when a step panics.
pub const PANICKED: &str = "The graph run panicked";

// ============================================================================
// CANCELLATION REGISTRY
// ============================================================================

fn active_runs() -> &'static Mutex<HashMap<String, Arc<AtomicBool>>> {
    static ACTIVE: OnceLock<Mutex<HashMap<String, Arc<AtomicBool>>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Refuses an id that is already running: its cancel flag and its `graph:finished`
/// belong to the run that registered it first.
pub fn register_run(run_id: &str) -> Result<Arc<AtomicBool>, String> {
    let mut runs = active_runs()
        .lock()
        .map_err(|_| "The run registry is unavailable".to_string())?;
    if runs.contains_key(run_id) {
        return Err(format!("A run with id '{}' is already in progress", run_id));
    }
    let flag = Arc::new(AtomicBool::new(false));
    runs.insert(run_id.to_string(), flag.clone());
    Ok(flag)
}

pub fn finish_run(run_id: &str) {
    if let Ok(mut runs) = active_runs().lock() {
        runs.remove(run_id);
    }
}

/// Returns false when the run id is unknown (already finished, or never started).
pub fn cancel_run(run_id: &str) -> bool {
    if let Ok(runs) = active_runs().lock() {
        if let Some(flag) = runs.get(run_id) {
            flag.store(true, Ordering::Relaxed);
            return true;
        }
    }
    false
}

// ============================================================================
// EXECUTION
// ============================================================================

/// Where progress goes. Injected rather than assumed so the runner can be exercised
/// without a Tauri app handle.
pub type Emit<'a> = &'a dyn Fn(&str, serde_json::Value);

pub fn run_graph(
    app: &tauri::AppHandle,
    recipe: &GraphRecipe,
    run_id: &str,
    cancel: &AtomicBool,
) -> Result<GraphRunResult, String> {
    execute_run(recipe, run_id, cancel, &|event, payload| {
        let _ = app.emit(event, payload);
    })
}

/// One registered run, start to finish (the caller registers it first, with
/// `register_run`). This is the only way the app runs a graph:
/// whatever `run_graph_with` does — return, fail, stop at a cancel check, or panic —
/// the guard's `Drop` is the single exit.
pub fn execute_run(
    recipe: &GraphRecipe,
    run_id: &str,
    cancel: &AtomicBool,
    emit: Emit<'_>,
) -> Result<GraphRunResult, String> {
    let mut guard = RunGuard::new(run_id, emit);
    let result = run_graph_with(recipe, run_id, cancel, emit);
    guard.settle(&result);
    result
}

/// Clears the registry entry and emits `graph:finished` when dropped — including
/// while a panic unwinds through it, which is why the workspace must never set
/// `panic = "abort"`.
pub struct RunGuard<'a> {
    run_id: String,
    emit: Emit<'a>,
    outcome: Option<(RunStatus, Option<String>)>,
}

impl<'a> RunGuard<'a> {
    pub fn new(run_id: &str, emit: Emit<'a>) -> Self {
        Self {
            run_id: run_id.to_string(),
            emit,
            outcome: None,
        }
    }

    fn settle(&mut self, result: &Result<GraphRunResult, String>) {
        self.outcome = Some(match result {
            Ok(r) => (r.status, None),
            Err(e) => (RunStatus::Failed, Some(e.clone())),
        });
    }
}

impl Drop for RunGuard<'_> {
    fn drop(&mut self) {
        // Never settled means the run did not return: it is unwinding from a panic.
        let (status, message) = self
            .outcome
            .take()
            .unwrap_or_else(|| (RunStatus::Failed, Some(PANICKED.to_string())));

        // Unregister first, so a listener reacting to `graph:finished` can no
        // longer cancel a run that is already over.
        finish_run(&self.run_id);

        let mut payload = serde_json::json!({ "runId": self.run_id, "status": status });
        if let Some(message) = message {
            payload["message"] = serde_json::Value::String(message);
        }
        (self.emit)("graph:finished", payload);
    }
}

pub fn run_graph_with(
    recipe: &GraphRecipe,
    run_id: &str,
    cancel: &AtomicBool,
    emit: Emit<'_>,
) -> Result<GraphRunResult, String> {
    let verdict = validate::validate(recipe);
    if !verdict.valid {
        return Err(verdict.reason.unwrap_or_else(|| "The graph cannot run".to_string()));
    }

    let mut warnings = verdict.warnings;
    let mut outputs = Vec::new();
    let order = verdict.order;

    let source = recipe
        .source_node()
        .ok_or_else(|| "The graph has no source node".to_string())?;
    let mut ctx = registry::build_context(&source.node_ref, &source.params())?;

    // The source contributes no work of its own, so progress counts the rest.
    let total = order
        .iter()
        .filter(|id| recipe.node(id).is_some_and(|n| n.kind != "source"))
        .count();
    let mut index = 0usize;

    for node_id in &order {
        let node = recipe
            .node(node_id)
            .ok_or_else(|| format!("Node '{}' vanished mid-run", node_id))?;
        if node.kind == "source" {
            continue;
        }

        // Cancelling is the user's call, not a failure: hand back what the steps
        // before the stop produced.
        if cancel.load(Ordering::Relaxed) {
            return Ok(project(run_id, RunStatus::Cancelled, ctx, outputs, warnings));
        }

        emit_progress(emit, run_id, node, index, total, "start");

        // Built here, not up front — see the module comment.
        let step = registry::build_step(&node.node_ref, &node.params(), &ctx)?;

        match step.execute(&mut ctx) {
            Ok(()) => outputs.extend(declared_outputs(node)),
            Err(e) => {
                if step.is_critical() {
                    let message = format!("{}", e);
                    emit(
                        "graph:error",
                        serde_json::json!({
                            "runId": run_id,
                            "nodeId": node.node_id,
                            "message": message,
                        }),
                    );
                    return Err(message);
                }
                warnings.push(format!("{} (non-critical): {}", node.node_ref, e));
            }
        }

        index += 1;
        emit_progress(emit, run_id, node, index, total, "done");
    }

    let result = project(run_id, RunStatus::Completed, ctx, outputs, warnings);

    emit(
        "graph:complete",
        serde_json::json!({
            "runId": run_id,
            "succeeded": true,
            "warnings": result.warnings,
        }),
    );

    Ok(result)
}

/// The paths a node's parameters name as outputs, per the manifest. Collected only
/// after the step has succeeded, so a failed step never claims a file.
fn declared_outputs(node: &GraphNode) -> Vec<RunOutputDto> {
    let Some(meta) = manifest::find(&node.node_ref) else {
        return Vec::new();
    };
    let params = node.params();
    meta.params
        .iter()
        .filter(|spec| spec.output)
        .filter_map(|spec| {
            let path = params.get(spec.key)?.as_str()?.trim();
            (!path.is_empty()).then(|| RunOutputDto {
                node_id: node.node_id.clone(),
                node_ref: node.node_ref.clone(),
                key: spec.key.to_string(),
                path: path.to_string(),
            })
        })
        .collect()
}

fn emit_progress(
    emit: Emit<'_>,
    run_id: &str,
    node: &GraphNode,
    index: usize,
    total: usize,
    phase: &str,
) {
    let percent = if total == 0 {
        100.0
    } else {
        (index as f64 / total as f64) * 100.0
    };
    emit(
        "graph:progress",
        serde_json::json!({
            "runId": run_id,
            "nodeId": node.node_id,
            "nodeRef": node.node_ref,
            "stepIndex": index,
            "totalSteps": total,
            "phase": phase,
            "progressPercent": percent,
        }),
    );
}

/// Project the blackboard onto serializable DTOs.
///
/// `MediaContext` cannot be serialized directly (it carries `Box<dyn Any>` resource
/// handles) and none of media-core's result structs derive Serialize.
fn project(
    run_id: &str,
    status: RunStatus,
    ctx: media_core::pipeline::MediaContext,
    outputs: Vec<RunOutputDto>,
    warnings: Vec<String>,
) -> GraphRunResult {
    GraphRunResult {
        run_id: run_id.to_string(),
        status,
        metadata: ctx.metadata,
        analysis: ctx.analysis.as_ref().map(analysis_to_dto),
        // Frame payloads are base64 and can be enormous; only the count goes back.
        extracted_frame_count: ctx.extracted_frames.len(),
        hls_result: ctx.hls_result.map(|r| HlsResultDto {
            output_dir: r.output_dir,
            playlist_path: r.playlist_path,
            segment_count: r.segment_count,
            success: r.success,
        }),
        annotation_result: ctx.annotation_result.map(|r| AnnotationResultDto {
            output_path: r.output_path,
            annotation_type: r.annotation_type,
            success: r.success,
        }),
        video_process_result: ctx.video_process_result.map(|r| VideoProcessResultDto {
            output_dir: r.output_dir,
            frames_extracted: r.frames_extracted,
            extraction_mode: r.extraction_mode,
            save_mode: r.save_mode,
            success: r.success,
        }),
        process_result: ctx.process_result.map(|r| ProcessResultDto {
            files_processed: r.files_processed,
            files_failed: r.files_failed,
            total_size_bytes: r.total_size_bytes,
            output_dir: r.output_dir,
            processing_mode: r.processing_mode,
            success: r.success,
        }),
        outputs,
        warnings,
    }
}
