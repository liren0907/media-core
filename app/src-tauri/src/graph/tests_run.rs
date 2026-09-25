//! Phase 0 execution acceptance — see docs/PLAN_composer_phase0_backend.md §8 (3)-(4).
//!
//! These actually run steps, so they need a real video. The fixture is synthesised
//! with ffmpeg; if ffmpeg is unavailable the test reports that it was skipped rather
//! than passing quietly.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use super::runner;
use super::types::GraphRecipe;

fn no_op_emit(_event: &str, _payload: serde_json::Value) {}

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
