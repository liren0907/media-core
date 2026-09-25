//! Graph wire format — the single contract between frontend and backend.
//!
//! Stored snake_case-free: every field goes over the wire as camelCase.
//! Node coordinates are deliberately NOT part of this format; layout is
//! recomputed from the structure on every load (see PLAN_composer_phase1_frontend.md).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::commands::pipeline::AnalysisResult;
use media_core::analysis::types::AnalysisReport;
use media_core::metadata::MediaMetadata;

/// Node parameters: a weakly-typed map, not named fields.
///
/// `BTreeMap` (not `HashMap`) because serialization order must be stable — the
/// frontend's dirty check compares serialized strings.
///
/// Missing key = not set = the step's own default applies.
pub type NodeParams = BTreeMap<String, serde_json::Value>;

// ============================================================================
// RECIPE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphRecipe {
    #[serde(default)]
    pub label: String,
    pub nodes: Vec<GraphNode>,
    #[serde(default)]
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    pub node_id: String,
    /// `source` | `method` | `sink` — an open string, not a Rust enum, so that a
    /// new node kind never breaks the stored shape.
    pub kind: String,
    /// Manifest id: `file` / `extract_frames_to_disk` / `save_report` / ...
    pub node_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<NodeParams>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
}

impl GraphRecipe {
    pub fn node(&self, node_id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.node_id == node_id)
    }

    /// Direct upstream node ids, in edge order (order is meaningful for fan-in).
    pub fn upstream_of<'a>(&'a self, node_id: &'a str) -> impl Iterator<Item = &'a str> {
        self.edges
            .iter()
            .filter(move |e| e.to == node_id)
            .map(|e| e.from.as_str())
    }

    pub fn in_degree(&self, node_id: &str) -> usize {
        self.edges.iter().filter(|e| e.to == node_id).count()
    }

    pub fn out_degree(&self, node_id: &str) -> usize {
        self.edges.iter().filter(|e| e.from == node_id).count()
    }

    pub fn source_node(&self) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.kind == "source")
    }
}

impl GraphNode {
    /// The only sanctioned way to read params — `None` and an empty map read alike.
    pub fn params(&self) -> NodeParams {
        self.params.clone().unwrap_or_default()
    }
}

// ============================================================================
// VERDICT
// ============================================================================

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphVerdict {
    pub valid: bool,
    /// Why it cannot run (Chinese, shown verbatim by the frontend). None = runnable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Non-blocking findings — the graph still runs.
    pub warnings: Vec<String>,
    /// Topological execution order (node ids). Empty when the graph is invalid.
    pub order: Vec<String>,
}

impl GraphVerdict {
    pub fn invalid(reason: impl Into<String>) -> Self {
        Self {
            valid: false,
            reason: Some(reason.into()),
            warnings: Vec::new(),
            order: Vec::new(),
        }
    }
}

// ============================================================================
// RUN RESULT
// ============================================================================
//
// `MediaContext` itself cannot be serialized (it holds `Box<dyn Any>` resources),
// and none of media-core's result structs derive Serialize. So the runner projects
// the blackboard onto these DTOs.

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HlsResultDto {
    pub output_dir: String,
    pub playlist_path: String,
    pub segment_count: usize,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnotationResultDto {
    pub output_path: String,
    pub annotation_type: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoProcessResultDto {
    pub output_dir: String,
    pub frames_extracted: usize,
    pub extraction_mode: String,
    pub save_mode: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessResultDto {
    pub files_processed: u64,
    pub files_failed: u64,
    pub total_size_bytes: u64,
    pub output_dir: String,
    pub processing_mode: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphRunResult {
    pub run_id: String,
    pub metadata: Option<MediaMetadata>,
    pub analysis: Option<AnalysisResult>,
    /// Frame payloads are deliberately not returned — only how many were produced.
    pub extracted_frame_count: usize,
    pub hls_result: Option<HlsResultDto>,
    pub annotation_result: Option<AnnotationResultDto>,
    pub video_process_result: Option<VideoProcessResultDto>,
    pub process_result: Option<ProcessResultDto>,
    /// Non-critical step failures and validation warnings, in order.
    pub warnings: Vec<String>,
}

/// `AnalysisReport` does not derive Serialize — convert to the DTO the rest of the
/// app already speaks (`commands::pipeline::AnalysisResult`).
pub fn analysis_to_dto(report: &AnalysisReport) -> AnalysisResult {
    use crate::commands::pipeline::{ImageComparisonResult, MotionEvent, SimilarityGroup};

    AnalysisResult {
        motion_events: report
            .motion_events
            .iter()
            .map(|e| MotionEvent {
                start_frame: e.start_frame,
                end_frame: e.end_frame,
                event_type: e.event_type.clone(),
            })
            .collect(),
        similarity_groups: report
            .similarity_groups
            .iter()
            .map(|g| SimilarityGroup {
                group_name: g.group_name.clone(),
                members: g.members.clone(),
            })
            .collect(),
        image_comparison: report.image_comparison.as_ref().map(|c| ImageComparisonResult {
            image1: c.image1.clone(),
            image2: c.image2.clone(),
            similarity_score: c.similarity_score,
            is_duplicate: c.is_duplicate,
        }),
    }
}
