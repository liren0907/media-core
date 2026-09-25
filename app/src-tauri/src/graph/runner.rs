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
    analysis_to_dto, AnnotationResultDto, GraphRecipe, GraphRunResult, HlsResultDto,
    ProcessResultDto, VideoProcessResultDto,
};
use super::{registry, validate};

// ============================================================================
// CANCELLATION REGISTRY
// ============================================================================

fn active_runs() -> &'static Mutex<HashMap<String, Arc<AtomicBool>>> {
    static ACTIVE: OnceLock<Mutex<HashMap<String, Arc<AtomicBool>>>> = OnceLock::new();
    ACTIVE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn register_run(run_id: &str) -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    if let Ok(mut runs) = active_runs().lock() {
        runs.insert(run_id.to_string(), flag.clone());
    }
    flag
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
    run_graph_with(recipe, run_id, cancel, &|event, payload| {
        let _ = app.emit(event, payload);
    })
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

        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled".to_string());
        }

        emit_progress(emit, run_id, node, index, total, "start");

        // Built here, not up front — see the module comment.
        let step = registry::build_step(&node.node_ref, &node.params(), &ctx)?;

        match step.execute(&mut ctx) {
            Ok(()) => {}
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

    let result = project(run_id, ctx, warnings);

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

fn emit_progress(
    emit: Emit<'_>,
    run_id: &str,
    node: &super::types::GraphNode,
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
    ctx: media_core::pipeline::MediaContext,
    warnings: Vec<String>,
) -> GraphRunResult {
    GraphRunResult {
        run_id: run_id.to_string(),
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
        warnings,
    }
}
