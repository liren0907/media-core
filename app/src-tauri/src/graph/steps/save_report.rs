//! `save_report` — an app-side pipeline step, not part of media-core.
//!
//! It exists because `metadata` and `analysis` are the two blackboard slots that
//! no media-core step consumes; without a reader they can only ever be leaves.
//! This is the graph's sink node, and it doubles as proof that custom steps can be
//! added on the app side without touching the media-core submodule.

use std::fs;
use std::path::PathBuf;

use media_core::pipeline::{MediaContext, MediaSource, PipelineError, PipelineStep};

use crate::graph::types::analysis_to_dto;

pub struct SaveReport {
    output_path: PathBuf,
    pretty: bool,
}

impl SaveReport {
    pub fn new(output_path: PathBuf, pretty: bool) -> Self {
        Self { output_path, pretty }
    }
}

impl PipelineStep<MediaContext> for SaveReport {
    fn name(&self) -> &str {
        "SaveReport"
    }

    fn execute(&self, context: &mut MediaContext) -> Result<(), PipelineError> {
        if context.metadata.is_none() && context.analysis.is_none() {
            return Err(PipelineError::StepFailed {
                step_name: self.name().to_string(),
                error: "nothing to report: no upstream step produced metadata or analysis"
                    .to_string(),
            });
        }

        let mut root = serde_json::Map::new();

        root.insert(
            "source".to_string(),
            serde_json::Value::String(match &context.source {
                MediaSource::File(p) => p.to_string_lossy().to_string(),
                MediaSource::Stream(s) => s.clone(),
                MediaSource::Camera(id) => format!("camera:{}", id),
            }),
        );

        if let Some(metadata) = &context.metadata {
            let value = serde_json::to_value(metadata).map_err(|e| PipelineError::StepFailed {
                step_name: self.name().to_string(),
                error: format!("failed to serialize metadata: {}", e),
            })?;
            root.insert("metadata".to_string(), value);
        }

        if let Some(report) = &context.analysis {
            // `AnalysisReport` does not derive Serialize — go through the DTO.
            let value =
                serde_json::to_value(analysis_to_dto(report)).map_err(|e| {
                    PipelineError::StepFailed {
                        step_name: self.name().to_string(),
                        error: format!("failed to serialize analysis: {}", e),
                    }
                })?;
            root.insert("analysis".to_string(), value);
        }

        let value = serde_json::Value::Object(root);
        let text = if self.pretty {
            serde_json::to_string_pretty(&value)
        } else {
            serde_json::to_string(&value)
        }
        .map_err(|e| PipelineError::StepFailed {
            step_name: self.name().to_string(),
            error: format!("failed to encode report: {}", e),
        })?;

        if let Some(parent) = self.output_path.parent() {
            fs::create_dir_all(parent).map_err(|e| PipelineError::StepFailed {
                step_name: self.name().to_string(),
                error: format!("failed to create output directory: {}", e),
            })?;
        }

        fs::write(&self.output_path, text).map_err(|e| PipelineError::StepFailed {
            step_name: self.name().to_string(),
            error: format!("failed to write {}: {}", self.output_path.display(), e),
        })?;

        Ok(())
    }
}
