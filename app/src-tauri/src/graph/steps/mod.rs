//! App-side pipeline steps.
//!
//! `media_core::pipeline::PipelineStep` is a public trait, so the app can add nodes
//! to the graph without modifying the media-core submodule.

mod save_report;

pub use save_report::SaveReport;
