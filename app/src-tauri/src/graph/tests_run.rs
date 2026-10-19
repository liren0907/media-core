//! Phase 0 execution acceptance — see docs/PLAN_composer_phase0_backend.md §8 (3)-(4).
//!
//! These actually run steps, so they need a real video. The fixture is synthesised
//! with ffmpeg; if ffmpeg is unavailable the test reports that it was skipped rather
//! than passing quietly.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use super::runner;
use super::types::{GraphRecipe, RunStatus};

fn no_op_emit(_event: &str, _payload: serde_json::Value) {}

/// Records every emitted event so a test can assert on what a run announced.
#[derive(Default)]
struct Recorder(RefCell<Vec<(String, serde_json::Value)>>);

impl Recorder {
    fn emit(&self, event: &str, payload: serde_json::Value) {
        self.0.borrow_mut().push((event.to_string(), payload));
    }

    fn finished(&self) -> Vec<serde_json::Value> {
        self.0
            .borrow()
            .iter()
            .filter(|(event, _)| event == "graph:finished")
            .map(|(_, payload)| payload.clone())
            .collect()
    }
}

/// `cancel_run` only finds ids that are still registered.
fn is_registered(run_id: &str) -> bool {
    runner::cancel_run(run_id)
}

fn metadata_recipe(video: &Path) -> GraphRecipe {
    serde_json::from_value(serde_json::json!({
        "nodes": [
            { "nodeId": "n0", "kind": "source", "nodeRef": "file",
              "params": { "path": video.to_str().unwrap() } },
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_metadata",
              "params": { "includeThumbnail": false } }
        ],
        "edges": [{ "from": "n0", "to": "n1" }]
    }))
    .unwrap()
}

/// A 2 second colour-bar clip. Returns None when ffmpeg is not on PATH.
fn fixture_video(dir: &Path) -> Option<PathBuf> {
    let path = dir.join("fixture.mp4");
    let status = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=duration=2:size=160x120:rate=15",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok()?;

    (status.success() && path.exists()).then_some(path)
}

fn run(recipe: GraphRecipe) -> Result<super::types::GraphRunResult, String> {
    let cancel = AtomicBool::new(false);
    runner::run_graph_with(&recipe, "test-run", &cancel, &no_op_emit)
}

/// A fan: one source feeding two independent steps.
#[test]
fn executes_a_fan_out_graph() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let Some(video) = fixture_video(tmp.path()) else {
        eprintln!("SKIPPED executes_a_fan_out_graph: ffmpeg not available");
        return;
    };

    let result = run(
        serde_json::from_value(serde_json::json!({
            "label": "fan",
            "nodes": [
                { "nodeId": "n0", "kind": "source", "nodeRef": "file",
                  "params": { "path": video.to_str().unwrap() } },
                { "nodeId": "n1", "kind": "method", "nodeRef": "extract_metadata",
                  "params": { "includeThumbnail": false } },
                { "nodeId": "n2", "kind": "method", "nodeRef": "extract_frames_to_disk",
                  "params": { "outputDir": tmp.path().join("frames").to_str().unwrap(),
                              "interval": 10 } }
            ],
            "edges": [{ "from": "n0", "to": "n1" }, { "from": "n0", "to": "n2" }]
        }))
        .unwrap(),
    )
    .expect("fan graph should run");

    assert!(result.metadata.is_some(), "metadata slot should be filled");
    let frames = result
        .video_process_result
        .expect("frames-on-disk slot should be filled");
    assert!(
        frames.frames_extracted > 0,
        "expected frames on disk, got {}",
        frames.frames_extracted
    );
}

/// The Phase 0 headline: `group_similar_images` is given no `inputDir`, and must pick
/// up the directory `extract_frames_to_disk` wrote during the same run. This is the
/// case that cannot work if every step is constructed before the first one executes.
#[test]
fn executes_a_chain_that_inherits_the_upstream_directory() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let Some(video) = fixture_video(tmp.path()) else {
        eprintln!("SKIPPED executes_a_chain...: ffmpeg not available");
        return;
    };

    let frames_dir = tmp.path().join("frames");
    let groups_dir = tmp.path().join("groups");

    let result = run(
        serde_json::from_value(serde_json::json!({
            "label": "chain",
            "nodes": [
                { "nodeId": "n0", "kind": "source", "nodeRef": "file",
                  "params": { "path": video.to_str().unwrap() } },
                { "nodeId": "n1", "kind": "method", "nodeRef": "extract_frames_to_disk",
                  "params": { "outputDir": frames_dir.to_str().unwrap(), "interval": 10 } },
                // No inputDir on purpose.
                { "nodeId": "n2", "kind": "method", "nodeRef": "group_similar_images",
                  "params": { "outputDir": groups_dir.to_str().unwrap(),
                              "method": "perceptual_hash" } }
            ],
            "edges": [{ "from": "n0", "to": "n1" }, { "from": "n1", "to": "n2" }]
        }))
        .unwrap(),
    )
    .expect("chain graph should run");

    let frames = result
        .video_process_result
        .expect("frames-on-disk slot should be filled");
    assert_eq!(frames.output_dir, frames_dir.to_string_lossy());
    assert!(frames.frames_extracted > 0);

    // Reaching the analysis slot at all proves the second step found the directory:
    // with no inputDir and no inheritance it would have failed to build.
    assert!(
        result.analysis.is_some(),
        "the downstream grouping step did not contribute an analysis report"
    );
}

/// The sink turns the two otherwise-unread slots into a file on disk.
#[test]
fn executes_a_graph_that_writes_a_report() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let Some(video) = fixture_video(tmp.path()) else {
        eprintln!("SKIPPED executes_a_graph_that_writes_a_report: ffmpeg not available");
        return;
    };

    let report = tmp.path().join("out").join("report.json");

    run(serde_json::from_value(serde_json::json!({
        "label": "report",
        "nodes": [
            { "nodeId": "n0", "kind": "source", "nodeRef": "file",
              "params": { "path": video.to_str().unwrap() } },
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_metadata" },
            { "nodeId": "n2", "kind": "sink", "nodeRef": "save_report",
              "params": { "outputPath": report.to_str().unwrap(), "pretty": true } }
        ],
        "edges": [{ "from": "n0", "to": "n1" }, { "from": "n1", "to": "n2" }]
    }))
    .unwrap())
    .expect("report graph should run");

    assert!(report.exists(), "the report file was not written");
    let text = std::fs::read_to_string(&report).expect("report should be readable");
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("report should be JSON");
    assert!(parsed.get("metadata").is_some(), "report has no metadata section");
    assert!(parsed.get("source").is_some(), "report has no source section");
}

/// An invalid graph must be refused before anything executes.
#[test]
fn refuses_to_run_an_invalid_graph() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let groups = tmp.path().join("groups");

    let err = run(
        serde_json::from_value(serde_json::json!({
            "nodes": [
                { "nodeId": "n0", "kind": "source", "nodeRef": "file",
                  "params": { "path": "/definitely/not/here.mp4" } },
                { "nodeId": "n1", "kind": "method", "nodeRef": "group_similar_images",
                  "params": { "outputDir": groups.to_str().unwrap() } }
            ],
            "edges": [{ "from": "n0", "to": "n1" }]
        }))
        .unwrap(),
    )
    .expect_err("a graph with an unsatisfiable input must not run");

    assert!(err.contains("inputDir"), "unhelpful reason: {}", err);
    assert!(!groups.exists(), "nothing should have been created");
}

// ============================================================================
// SINGLE EXIT — every run ends with exactly one `graph:finished` and leaves the
// registry clean, whichever way it ends.
// ============================================================================

#[test]
fn a_completed_run_announces_completed_and_unregisters() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let Some(video) = fixture_video(tmp.path()) else {
        eprintln!("SKIPPED a_completed_run...: ffmpeg not available");
        return;
    };

    let run_id = "exit-completed";
    let cancel = runner::register_run(run_id).expect("fresh id registers");
    let events = Recorder::default();

    let result =
        runner::execute_run(&metadata_recipe(&video), run_id, &cancel, &|e, p| events.emit(e, p))
            .expect("metadata graph should run");

    assert_eq!(result.status, RunStatus::Completed);
    let finished = events.finished();
    assert_eq!(finished.len(), 1, "expected exactly one graph:finished");
    assert_eq!(finished[0]["runId"], run_id);
    assert_eq!(finished[0]["status"], "completed");
    assert!(finished[0].get("message").is_none());
    assert!(!is_registered(run_id), "the run id should be unregistered");
}

#[test]
fn a_cancelled_run_announces_cancelled_and_unregisters() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let Some(video) = fixture_video(tmp.path()) else {
        eprintln!("SKIPPED a_cancelled_run...: ffmpeg not available");
        return;
    };

    let run_id = "exit-cancelled";
    let cancel = runner::register_run(run_id).expect("fresh id registers");
    // Cancelled before the first step: the earliest a user can press Cancel.
    assert!(runner::cancel_run(run_id));
    assert!(cancel.load(Ordering::Relaxed));
    let events = Recorder::default();

    let result = runner::execute_run(&metadata_recipe(&video), run_id, &cancel, &|e, p| {
        events.emit(e, p)
    })
    .expect("cancelling is not a failure");

    assert_eq!(result.status, RunStatus::Cancelled);
    assert!(result.metadata.is_none(), "no step should have run");
    let finished = events.finished();
    assert_eq!(finished.len(), 1, "expected exactly one graph:finished");
    assert_eq!(finished[0]["status"], "cancelled");
    assert!(!is_registered(run_id), "the run id should be unregistered");
}

#[test]
fn a_failed_run_announces_failed_with_the_reason_and_unregisters() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let run_id = "exit-failed";
    let cancel = runner::register_run(run_id).expect("fresh id registers");
    let events = Recorder::default();

    // Unsatisfiable input: refused by validation before any step runs.
    let recipe: GraphRecipe = serde_json::from_value(serde_json::json!({
        "nodes": [
            { "nodeId": "n0", "kind": "source", "nodeRef": "file",
              "params": { "path": "/definitely/not/here.mp4" } },
            { "nodeId": "n1", "kind": "method", "nodeRef": "group_similar_images",
              "params": { "outputDir": tmp.path().join("groups").to_str().unwrap() } }
        ],
        "edges": [{ "from": "n0", "to": "n1" }]
    }))
    .unwrap();

    let err = runner::execute_run(&recipe, run_id, &cancel, &|e, p| events.emit(e, p))
        .expect_err("an invalid graph must not run");

    let finished = events.finished();
    assert_eq!(finished.len(), 1, "expected exactly one graph:finished");
    assert_eq!(finished[0]["status"], "failed");
    assert_eq!(finished[0]["message"], err.as_str());
    assert!(!is_registered(run_id), "the run id should be unregistered");
}

/// Cancel pressed while the first of two steps runs: the second never starts, and the
/// first one's output comes back with the cancelled status.
#[test]
fn a_run_cancelled_mid_way_returns_what_finished() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let Some(video) = fixture_video(tmp.path()) else {
        eprintln!("SKIPPED a_run_cancelled_mid_way...: ffmpeg not available");
        return;
    };

    let run_id = "cancel-mid-way";
    let cancel = runner::register_run(run_id).expect("fresh id registers");
    let frames_dir = tmp.path().join("frames");
    // Declaration order makes n1 run before n2.
    let recipe: GraphRecipe = serde_json::from_value(serde_json::json!({
        "nodes": [
            { "nodeId": "n0", "kind": "source", "nodeRef": "file",
              "params": { "path": video.to_str().unwrap() } },
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_metadata" },
            { "nodeId": "n2", "kind": "method", "nodeRef": "extract_frames_to_disk",
              "params": { "outputDir": frames_dir.to_str().unwrap(), "interval": 10 } }
        ],
        "edges": [{ "from": "n0", "to": "n1" }, { "from": "n0", "to": "n2" }]
    }))
    .unwrap();

    let result = runner::execute_run(&recipe, run_id, &cancel, &|event, payload| {
        if event == "graph:progress" && payload["nodeId"] == "n1" && payload["phase"] == "start" {
            assert!(runner::cancel_run(run_id), "the run should still be cancellable");
        }
    })
    .expect("cancelling is not a failure");

    assert_eq!(result.status, RunStatus::Cancelled);
    assert!(result.metadata.is_some(), "the step that finished should be in the result");
    assert!(result.video_process_result.is_none(), "the step after the cancel ran anyway");
    assert!(!frames_dir.exists(), "the step after the cancel wrote frames");
}

/// The manifest marks the report path as an output, so the result says where it went.
#[test]
fn a_sink_reports_where_it_wrote() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let Some(video) = fixture_video(tmp.path()) else {
        eprintln!("SKIPPED a_sink_reports_where_it_wrote: ffmpeg not available");
        return;
    };

    let report = tmp.path().join("report.json");
    let result = run(serde_json::from_value(serde_json::json!({
        "nodes": [
            { "nodeId": "n0", "kind": "source", "nodeRef": "file",
              "params": { "path": video.to_str().unwrap() } },
            { "nodeId": "n1", "kind": "method", "nodeRef": "extract_metadata" },
            { "nodeId": "n2", "kind": "sink", "nodeRef": "save_report",
              "params": { "outputPath": report.to_str().unwrap() } }
        ],
        "edges": [{ "from": "n0", "to": "n1" }, { "from": "n1", "to": "n2" }]
    }))
    .unwrap())
    .expect("report graph should run");

    assert_eq!(result.outputs.len(), 1, "outputs: {:?}", result.outputs);
    let output = &result.outputs[0];
    assert_eq!(output.node_id, "n2");
    assert_eq!(output.node_ref, "save_report");
    assert_eq!(output.key, "outputPath");
    assert_eq!(output.path, report.to_string_lossy());
    assert!(report.exists(), "the reported path was not written");
}

/// A step that panics is tested through the guard itself rather than through a
/// panicking test-only step, which would have to be declared in the manifest.
#[test]
fn a_panicking_run_still_announces_failed_and_unregisters() {
    let run_id = "exit-panicked";
    runner::register_run(run_id).expect("fresh id registers");
    let events = Recorder::default();

    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let emit = |e: &str, p: serde_json::Value| events.emit(e, p);
        let _guard = runner::RunGuard::new(run_id, &emit);
        panic!("a step blew up");
    }));

    assert!(unwound.is_err(), "the panic should have propagated");
    let finished = events.finished();
    assert_eq!(finished.len(), 1, "expected exactly one graph:finished");
    assert_eq!(finished[0]["status"], "failed");
    assert_eq!(finished[0]["message"], runner::PANICKED);
    assert!(!is_registered(run_id), "the run id should be unregistered");
}

#[test]
fn a_run_id_that_is_already_running_is_refused() {
    let run_id = "exit-duplicate";
    runner::register_run(run_id).expect("fresh id registers");

    let err = runner::register_run(run_id).expect_err("a live id must not register twice");
    assert!(err.contains(run_id), "unhelpful reason: {}", err);

    runner::finish_run(run_id);
    runner::register_run(run_id).expect("a finished id can be used again");
    runner::finish_run(run_id);
}
