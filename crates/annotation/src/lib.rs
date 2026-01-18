// Annotation module - core business logic for annotation processing

pub mod annotator;
pub mod ogp_annotation;
pub mod types;

pub use ogp_annotation::{Annotation, VideoAnnotation};
