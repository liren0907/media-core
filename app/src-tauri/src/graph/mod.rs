//! Composer graph — a DAG of media-core pipeline steps that can be described as
//! data, validated, and executed.
//!
//! The port types are the fields of `media_core::pipeline::MediaContext`: an edge
//! declares "the downstream step depends on a slot the upstream step writes".
//! Steps themselves come unchanged from `crates/media-core` in this workspace; the
//! manifest's declaration of which slots each step writes is checked against what
//! the step actually does by `tests_manifest`.
//!
//! See docs/PLAN_composer_graph_editor.md for the design and its trade-offs.

pub mod manifest;
pub mod registry;
pub mod runner;
pub mod steps;
pub mod store;
pub mod types;
pub mod validate;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_manifest;
#[cfg(test)]
mod tests_run;
#[cfg(test)]
mod tests_store;
