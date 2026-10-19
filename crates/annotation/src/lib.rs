// Detection overlay: reads a detection-results JSON (per-frame objects with
// labels, boxes and confidences) and draws each object's box and label on
// every frame of a video, writing a new video.
//
// Not to be confused with media_core's `annotation` module, which stamps a
// line of text (file name, timecode or custom string) on images and videos.

pub mod annotator;
pub mod ogp_annotation;
pub mod types;

pub use ogp_annotation::{Annotation, VideoAnnotation};
