//! The manifest says which `MediaContext` slots each media-core step writes; the
//! validator and the picker trust that declaration. This module checks it against
//! what the step really does: every core method in the manifest is built through
//! the registry, run once on a fresh context, and the fields it changed must be
//! exactly the slots it declares.
//!
//! Only writes are checked. Reads cannot be observed from the result, so every
//! step is given what it needs up front. The write mode (replace, merge, append)
//! is not checked either — on a fresh context they all look like "was written".
//!
//! Needs ffmpeg for the fixture video and FAILS without it: a check that skips
//! itself would stay green forever.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use media_core::pipeline::MediaContext;
use serde_json::json;

use super::manifest::{self, NodeKind, Slot};
use super::registry;
use super::tests_run::fixture_video;
use super::types::NodeParams;

/// Steps that cannot run in this test, each with the reason. Keep it empty if
/// at all possible: a step listed here has an unchecked declaration.
const NOT_RUN: &[(&str, &str)] = &[];

/// The `MediaContext` field behind a slot.
fn slot_field(slot: Slot) -> &'static str {
    match slot {
        Slot::Source => "source",
        Slot::Metadata => "metadata",
        Slot::Analysis => "analysis",
        Slot::ExtractedFrames => "extracted_frames",
        Slot::HlsResult => "hls_result",
        Slot::AnnotationResult => "annotation_result",
        Slot::VideoProcessResult => "video_process_result",
        Slot::ProcessResult => "process_result",
    }
}

/// What can be seen of each public `MediaContext` field.
#[derive(PartialEq)]
struct Snapshot {
    source: String,
    metadata: bool,
    analysis: bool,
    annotation_result: bool,
    extracted_frames: usize,
    hls_result: bool,
    benchmark_result: bool,
    video_process_result: bool,
    process_result: bool,
}

impl Snapshot {
    fn of(ctx: &MediaContext) -> Self {
        Self {
            source: format!("{:?}", ctx.source),
            metadata: ctx.metadata.is_some(),
            analysis: ctx.analysis.is_some(),
            annotation_result: ctx.annotation_result.is_some(),
            extracted_frames: ctx.extracted_frames.len(),
            hls_result: ctx.hls_result.is_some(),
            benchmark_result: ctx.benchmark_result.is_some(),
            video_process_result: ctx.video_process_result.is_some(),
            process_result: ctx.process_result.is_some(),
        }
    }

    /// Fields that differ from `before`. `benchmark_result` is not a slot, so a
    /// step writing it can never match a declaration.
    fn written_since(&self, before: &Snapshot) -> BTreeSet<&'static str> {
        [
            ("source", self.source != before.source),
            ("metadata", self.metadata != before.metadata),
            ("analysis", self.analysis != before.analysis),
            ("annotation_result", self.annotation_result != before.annotation_result),
            ("extracted_frames", self.extracted_frames != before.extracted_frames),
            ("hls_result", self.hls_result != before.hls_result),
            ("benchmark_result", self.benchmark_result != before.benchmark_result),
            ("video_process_result", self.video_process_result != before.video_process_result),
            ("process_result", self.process_result != before.process_result),
        ]
        .into_iter()
        .filter_map(|(field, changed)| changed.then_some(field))
        .collect()
    }
}

/// Inputs shared by the steps: the video plus frames already extracted from it.
struct Fixtures {
    video: PathBuf,
    frames_dir: PathBuf,
    frame: PathBuf,
    other_frame: PathBuf,
    out: PathBuf,
}

impl Fixtures {
    fn new(dir: &Path) -> Self {
        let video = fixture_video(dir).unwrap_or_else(|| {
            panic!("ffmpeg is required to check the manifest against media-core: install it and put it on PATH")
        });

        let frames_dir = dir.join("frames");
        let mut ctx = MediaContext::from_file(video.clone());
        registry::build_step(
            "extract_frames_to_disk",
            &params(json!({ "outputDir": frames_dir, "interval": 5 })),
            &ctx,
        )
        .and_then(|step| step.execute(&mut ctx).map_err(|e| e.to_string()))
        .expect("extracting the fixture frames");

        let mut frames: Vec<PathBuf> = std::fs::read_dir(&frames_dir)
            .expect("frames directory")
            .map(|entry| entry.unwrap().path())
            .collect();
        frames.sort();
        assert!(frames.len() >= 2, "expected at least two fixture frames, got {:?}", frames);

        let out = dir.join("out");
        std::fs::create_dir_all(&out).unwrap();
        Self {
            video,
            frames_dir,
            frame: frames[0].clone(),
            other_frame: frames[frames.len() - 1].clone(),
            out,
        }
    }

    /// Parameters that let `node_ref` run. Outputs go to a directory of their own.
    /// `None` for a method this test does not know yet.
    fn params_for(&self, node_ref: &str) -> Option<NodeParams> {
        let out = self.out.join(node_ref);
        std::fs::create_dir_all(&out).unwrap();
        let values = match node_ref {
            "extract_metadata" => json!({ "includeThumbnail": false }),
            "detect_motion" => json!({}),
            "group_similar_images" => json!({ "inputDir": self.frames_dir, "outputDir": out }),
            "compare_images" => json!({ "image1": self.frame, "image2": self.other_frame }),
            "extract_frames" => json!({}),
            "extract_frames_to_disk" => json!({ "outputDir": out }),
            "convert_to_hls" => json!({ "outputDir": out }),
            "capture_frame" => json!({}),
            "annotate_frame" => json!({ "inputPath": self.frame, "outputPath": out.join("annotated.jpg") }),
            "annotate_video" => json!({ "framesDir": self.frames_dir, "outputPath": out.join("annotated.mp4") }),
            // In the default single_file mode media-core copies the source TO this
            // path, so it must not be an existing directory.
            "process_files" => json!({ "outputDir": out.join("copy.mp4") }),
            _ => return None,
        };
        Some(params(values))
    }
}

fn params(values: serde_json::Value) -> NodeParams {
    serde_json::from_value(values).expect("node params")
}

#[test]
fn each_core_step_writes_exactly_the_slots_it_declares() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let fixtures = Fixtures::new(tmp.path());

    let mut mismatches = Vec::new();
    for method in manifest::METHODS.iter().filter(|m| m.kind == NodeKind::Method) {
        if NOT_RUN.iter().any(|(id, _)| *id == method.id) {
            continue;
        }
        let params = fixtures.params_for(method.id).unwrap_or_else(|| {
            panic!("'{}' is in the manifest but not in this test: add it to params_for", method.id)
        });

        let mut ctx = MediaContext::from_file(fixtures.video.clone());
        let before = Snapshot::of(&ctx);
        let step = registry::build_step(method.id, &params, &ctx)
            .unwrap_or_else(|e| panic!("building '{}': {}", method.id, e));
        step.execute(&mut ctx)
            .unwrap_or_else(|e| panic!("running '{}': {}", method.id, e));

        let actual = Snapshot::of(&ctx).written_since(&before);
        let declared: BTreeSet<&'static str> =
            method.writes.iter().map(|w| slot_field(w.slot)).collect();
        if actual != declared {
            mismatches.push(format!(
                "{}: declares {:?}, but the step wrote {:?}",
                method.id, declared, actual
            ));
        }
    }

    assert!(mismatches.is_empty(), "manifest out of step with media-core:\n{}", mismatches.join("\n"));
}

#[test]
fn steps_that_are_not_run_are_in_the_manifest() {
    for (id, reason) in NOT_RUN {
        assert!(manifest::find(id).is_some(), "'{}' is not in the manifest ({})", id, reason);
    }
}
