//! Composer graph — a DAG of media-core pipeline steps that can be described as
//! data, validated, and executed.
//!
//! The port types are the fields of `media_core::pipeline::MediaContext`: an edge
//! declares "the downstream step depends on a slot the upstream step writes".
//! Steps themselves come from media-core unchanged; nothing in this module requires
//! a modification to that submodule.
//!
//! See docs/PLAN_composer_graph_editor.md for the design and its trade-offs.

pub mod manifest;
pub mod registry;
pub mod runner;
pub mod steps;
pub mod types;
pub mod validate;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_run;
