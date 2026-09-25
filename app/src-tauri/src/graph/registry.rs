//! Registry — turns `(node_ref, params, blackboard)` into a live pipeline step.
//!
//! Steps are built one at a time, immediately before they run, NOT all up front.
//! That is deliberate: a downstream step's *constructor arguments* can depend on an
//! upstream step's *execution result* (see `resolve_dir`), which is impossible if
//! every step is constructed before the first one executes. This is also why the
//! runner cannot reuse `media_core::pipeline::Pipeline`.

use std::path::PathBuf;

use media_core::pipeline::{MediaContext, PipelineStep};

use super::steps::SaveReport;
use super::types::NodeParams;

// ============================================================================
// PARAMETER ACCESS
// ============================================================================

/// Typed, error-reporting access to a node's parameter map.
///
/// Missing key, JSON null and empty string all read as "not set" — the step's own
/// default then applies. Manifest defaults are for the UI to display; they are
/// deliberately not re-applied here, so a step that is left alone behaves byte for
/// byte like the hand-written commands in `commands/pipeline.rs`.
pub struct Params<'a> {
    values: &'a NodeParams,
    node_ref: &'a str,
}

impl<'a> Params<'a> {
    pub fn new(values: &'a NodeParams, node_ref: &'a str) -> Self {
        Self { values, node_ref }
    }

    fn get(&self, key: &str) -> Option<&'a serde_json::Value> {
        match self.values.get(key) {
            None => None,
            Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::String(s)) if s.trim().is_empty() => None,
            Some(v) => Some(v),
        }
    }

    fn wrong_type(&self, key: &str, expected: &str) -> String {
        format!("{}: parameter '{}' must be {}", self.node_ref, key, expected)
    }

    pub fn bool(&self, key: &str) -> Result<Option<bool>, String> {
        match self.get(key) {
            None => Ok(None),
            Some(v) => v.as_bool().map(Some).ok_or_else(|| self.wrong_type(key, "a boolean")),
        }
    }

    pub fn int(&self, key: &str) -> Result<Option<i64>, String> {
        match self.get(key) {
            None => Ok(None),
            Some(v) => v
                .as_i64()
                .or_else(|| v.as_f64().map(|f| f as i64))
                .map(Some)
                .ok_or_else(|| self.wrong_type(key, "a whole number")),
        }
    }

    pub fn float(&self, key: &str) -> Result<Option<f64>, String> {
        match self.get(key) {
            None => Ok(None),
            Some(v) => v.as_f64().map(Some).ok_or_else(|| self.wrong_type(key, "a number")),
        }
    }

    pub fn str(&self, key: &str) -> Result<Option<&'a str>, String> {
        match self.get(key) {
            None => Ok(None),
            Some(v) => v.as_str().map(Some).ok_or_else(|| self.wrong_type(key, "a string")),
        }
    }

    pub fn path(&self, key: &str) -> Result<Option<PathBuf>, String> {
        Ok(self.str(key)?.map(PathBuf::from))
    }

    pub fn req_str(&self, key: &str) -> Result<&'a str, String> {
        self.str(key)?
            .ok_or_else(|| format!("{}: parameter '{}' is required", self.node_ref, key))
    }

    pub fn req_path(&self, key: &str) -> Result<PathBuf, String> {
        Ok(PathBuf::from(self.req_str(key)?))
    }
}

/// Resolve a directory parameter that may be left blank to inherit from upstream.
///
/// Order: explicit parameter → the upstream frames-on-disk output → nothing.
fn resolve_dir(
    p: &Params<'_>,
    key: &str,
    ctx: &MediaContext,
) -> Result<Option<PathBuf>, String> {
    if let Some(explicit) = p.path(key)? {
        return Ok(Some(explicit));
    }
    Ok(ctx
        .video_process_result
        .as_ref()
        .map(|r| PathBuf::from(&r.output_dir)))
}

// ============================================================================
// SOURCE → CONTEXT
// ============================================================================

pub fn build_context(node_ref: &str, params: &NodeParams) -> Result<MediaContext, String> {
    let p = Params::new(params, node_ref);
    match node_ref {
        "file" => Ok(MediaContext::from_file(p.req_path("path")?)),
        "stream" => Ok(MediaContext::from_stream(p.req_str("url")?.to_string())),
        "camera" => Ok(MediaContext::from_camera(
            p.int("cameraId")?.unwrap_or(0) as i32
        )),
        other => Err(format!("Unknown source type '{}'", other)),
    }
}

// ============================================================================
// METHOD / SINK → STEP
// ============================================================================

pub fn build_step(
    node_ref: &str,
    params: &NodeParams,
    ctx: &MediaContext,
) -> Result<Box<dyn PipelineStep<MediaContext>>, String> {
    let p = Params::new(params, node_ref);

    match node_ref {
        // ------------------------------------------------------------ metadata
        "extract_metadata" => {
            use media_core::metadata::pipeline::ExtractMetadata;
            Ok(Box::new(ExtractMetadata::new(
                p.bool("includeThumbnail")?.unwrap_or(false),
            )))
        }

        // ------------------------------------------------------------ analysis
        "detect_motion" => {
            use media_core::analysis::config::MotionAlgorithm;
            use media_core::analysis::pipeline::DetectMotion;

            let mut s = DetectMotion::new();
            if let Some(v) = p.str("algorithm")? {
                s = s.algorithm(match v {
                    "mog2" => MotionAlgorithm::Mog2,
                    "knn" => MotionAlgorithm::Knn,
                    "optical_flow" => MotionAlgorithm::OpticalFlow,
                    "frame_diff" => MotionAlgorithm::FrameDiff,
                    other => return Err(unknown_option(node_ref, "algorithm", other)),
                });
            }
            if let Some(v) = p.float("threshold")? {
                s = s.threshold(v);
            }
            if let Some(v) = p.int("minArea")? {
                s = s.min_area(v as i32);
            }
            if let Some(v) = p.path("outputDir")? {
                s = s.output_dir(v);
            }
            Ok(Box::new(s))
        }

        "group_similar_images" => {
            use media_core::analysis::config::{ProcessMode, SimilarityMethod};
            use media_core::analysis::pipeline::GroupSimilarImages;

            // Blank input directory means "use what the upstream extraction wrote".
            let input = resolve_dir(&p, "inputDir", ctx)?.ok_or_else(|| {
                format!(
                    "{}: 'inputDir' is empty and no upstream step has produced a frame directory",
                    node_ref
                )
            })?;
            let output = p.req_path("outputDir")?;

            // NOTE: `new<P: AsRef<Path>>(input: P, output: P)` binds BOTH arguments to
            // the same P, so they must be the same concrete type.
            let mut s = GroupSimilarImages::new(input, output);

            let method = p.str("method")?;
            if let Some(v) = method {
                s = s.method(match v {
                    "histogram" => SimilarityMethod::Histogram,
                    "feature_matching" => SimilarityMethod::FeatureMatching,
                    "perceptual_hash" => SimilarityMethod::PerceptualHash,
                    other => return Err(unknown_option(node_ref, "method", other)),
                });
            }
            if let Some(v) = p.str("processMode")? {
                s = s.process_mode(match v {
                    "parallel" => ProcessMode::Parallel,
                    "single" => ProcessMode::Single,
                    other => return Err(unknown_option(node_ref, "processMode", other)),
                });
            }
            if let Some(v) = p.int("minCategorySize")? {
                s = s.min_category_size(v as i32);
            }
            if let Some(v) = p.bool("groupSimilar")? {
                s = s.group_similar(v);
            }
            // One knob, three setters — the destination depends on the method.
            if let Some(v) = p.float("threshold")? {
                s = match method {
                    Some("histogram") => s.histogram_threshold(v),
                    Some("feature_matching") => s.feature_threshold(v),
                    _ => s.phash_threshold(v),
                };
            }
            Ok(Box::new(s))
        }

        "compare_images" => {
            use media_core::analysis::config::SimilarityMethod;
            use media_core::analysis::pipeline::CompareImages;

            let image1 = p.req_path("image1")?;
            let image2 = p.req_path("image2")?;
            let mut s = CompareImages::new(image1, image2);

            if let Some(v) = p.str("method")? {
                s = s.method(match v {
                    "histogram" => SimilarityMethod::Histogram,
                    "feature_matching" => SimilarityMethod::FeatureMatching,
                    "perceptual_hash" => SimilarityMethod::PerceptualHash,
                    other => return Err(unknown_option(node_ref, "method", other)),
                });
            }
            if let Some(v) = p.float("threshold")? {
                s = s.threshold(v);
            }
            Ok(Box::new(s))
        }

        // ------------------------------------------------------------- extract
        "extract_frames" => {
            use media_core::streaming::pipeline::ExtractFrames;
            use media_core::streaming::{ExtractionMode as FrameMode, SamplingStrategy};

            let n = p.int("strategyParam")?;
            let strategy = match p.str("strategy")?.unwrap_or("every_nth") {
                "first_n" => SamplingStrategy::FirstN(n.unwrap_or(10).max(1) as usize),
                "range" => SamplingStrategy::Range(
                    p.int("rangeStart")?.unwrap_or(0).max(0) as usize,
                    p.int("rangeEnd")?.unwrap_or(100).max(0) as usize,
                ),
                "key_frames" => SamplingStrategy::KeyFrames,
                "every_nth" => SamplingStrategy::EveryNth(n.unwrap_or(30).max(1) as usize),
                other => return Err(unknown_option(node_ref, "strategy", other)),
            };

            let mut s = ExtractFrames::new(strategy);
            if let Some(v) = p.float("scaleFactor")? {
                s = s.with_scale(v);
            }
            // `streaming::ExtractionMode` — NOT the same enum as `extractionMode` on
            // extract_frames_to_disk. Same name, different crate module, different values.
            if let Some(v) = p.str("frameMode")? {
                s = s.with_mode(match v {
                    "sequential" => FrameMode::Sequential,
                    "seek" => FrameMode::Seek,
                    other => return Err(unknown_option(node_ref, "frameMode", other)),
                });
            }
            Ok(Box::new(s))
        }

        "extract_frames_to_disk" => {
            use media_core::video_process::pipeline::ExtractFramesToDisk;
            use media_core::video_process::{ExtractionMode as DiskMode, SaveMode};

            let mut s = ExtractFramesToDisk::new(p.req_path("outputDir")?);
            if let Some(v) = p.int("interval")? {
                s = s.interval(v.max(1) as usize);
            }
            if let Some(v) = p.str("extractionMode")? {
                s = s.mode(match v {
                    "opencv_sequential" => DiskMode::OpenCVSequential,
                    "opencv_interval" => DiskMode::OpenCVInterval,
                    "ffmpeg" => DiskMode::FFmpeg,
                    "ffmpeg_interval" => DiskMode::FFmpegInterval,
                    "parallel" => DiskMode::Parallel,
                    other => return Err(unknown_option(node_ref, "extractionMode", other)),
                });
            }
            // The one place a manifest default is applied here rather than left to
            // the step. media-core's `SaveMode::default()` is MultipleDirectory,
            // which writes frames to `{outputDir}/{video_name}/` — that would make
            // `output_dir` the PARENT of the frames, so any downstream node
            // inheriting it would look in the wrong place, and media-core's own
            // frame count (which only scans the top level) would always report 0.
            // Single-directory keeps `output_dir` equal to "where the frames are",
            // which is what the inheritance contract needs.
            s = s.save_mode(match p.str("saveMode")?.unwrap_or("single_directory") {
                "multiple_directory" => SaveMode::MultipleDirectory,
                "single_directory" => SaveMode::SingleDirectory,
                other => return Err(unknown_option(node_ref, "saveMode", other)),
            });
            Ok(Box::new(s))
        }

        "capture_frame" => {
            use media_core::camera::pipeline::CaptureFrame;
            Ok(Box::new(CaptureFrame))
        }

        // ------------------------------------------------------------- convert
        "convert_to_hls" => {
            use media_core::hls::pipeline::ConvertToHLS;

            let mut s = ConvertToHLS::new(p.req_path("outputDir")?);
            if let Some(v) = p.int("segmentDuration")? {
                s = s.segment_duration(v.max(1) as u32);
            }
            if let Some(v) = p.str("playlistName")? {
                s = s.playlist_name(v);
            }
            if let Some(v) = p.bool("forceKeyframes")? {
                s = s.force_keyframes(v);
            }
            if let Some(v) = p.str("profile")? {
                s = s.profile(v);
            }
            if let Some(v) = p.str("level")? {
                s = s.level(v);
            }
            Ok(Box::new(s))
        }

        // ------------------------------------------------------------ annotate
        "annotate_frame" => {
            use media_core::annotation::pipeline::AnnotateFrame;

            let output = p.req_path("outputPath")?;
            let mut s = match p.path("inputPath")? {
                Some(input) => AnnotateFrame::new(input, output),
                None => AnnotateFrame::with_context_source(output),
            };
            if let Some(v) = p.str("annotationType")? {
                s = s.annotation_type(annotation_type(node_ref, v)?);
            }
            if let Some(v) = p.str("textPosition")? {
                s = s.position(text_position(node_ref, v)?);
            }
            Ok(Box::new(s))
        }

        "annotate_video" => {
            use media_core::annotation::pipeline::AnnotateVideo;

            let output = p.req_path("outputPath")?;
            // Blank frames directory falls back to the upstream extraction, then to
            // the graph source (which `AnnotateVideo` resolves itself).
            let mut s = match resolve_dir(&p, "framesDir", ctx)? {
                Some(frames) => AnnotateVideo::new(frames, output),
                None => AnnotateVideo::with_context_source(output),
            };
            if let Some(v) = p.str("annotationType")? {
                s = s.annotation_type(annotation_type(node_ref, v)?);
            }
            if let Some(v) = p.str("textPosition")? {
                s = s.position(text_position(node_ref, v)?);
            }
            if let Some(v) = p.int("fps")? {
                s = s.fps(v.max(1) as i32);
            }
            if let Some(v) = p.float("sourceFps")? {
                s = s.source_fps(v);
            }
            Ok(Box::new(s))
        }

        // ------------------------------------------------------------- process
        "process_files" => {
            use media_core::process::pipeline::ProcessFiles;
            use media_core::process::ProcessingMode;

            let mut s = ProcessFiles::new(p.req_path("outputDir")?);
            if let Some(v) = p.str("processingMode")? {
                s = s.mode(match v {
                    "batch_files" => ProcessingMode::BatchFiles,
                    "directory_process" => ProcessingMode::DirectoryProcess,
                    "stream_process" => ProcessingMode::StreamProcess,
                    "single_file" => ProcessingMode::SingleFile,
                    other => return Err(unknown_option(node_ref, "processingMode", other)),
                });
            }
            if let Some(v) = p.bool("overwrite")? {
                s = s.overwrite(v);
            }
            Ok(Box::new(s))
        }

        // ---------------------------------------------------------------- sink
        "save_report" => Ok(Box::new(SaveReport::new(
            p.req_path("outputPath")?,
            p.bool("pretty")?.unwrap_or(true),
        ))),

        other => Err(format!("Unknown method '{}'", other)),
    }
}

// ============================================================================
// SHARED ENUM MAPPING
// ============================================================================

fn unknown_option(node_ref: &str, key: &str, value: &str) -> String {
    format!("{}: '{}' is not a valid value for '{}'", node_ref, value, key)
}

fn annotation_type(
    node_ref: &str,
    v: &str,
) -> Result<media_core::annotation::types::AnnotationType, String> {
    use media_core::annotation::types::AnnotationType;
    match v {
        "filename" => Ok(AnnotationType::Filename),
        "timestamp" => Ok(AnnotationType::Timestamp),
        other => Err(unknown_option(node_ref, "annotationType", other)),
    }
}

fn text_position(
    node_ref: &str,
    v: &str,
) -> Result<media_core::annotation::types::TextPosition, String> {
    use media_core::annotation::types::TextPosition;
    match v {
        "top_left" => Ok(TextPosition::TopLeft),
        "top_right" => Ok(TextPosition::TopRight),
        "bottom_left" => Ok(TextPosition::BottomLeft),
        "bottom_right" => Ok(TextPosition::BottomRight),
        "center" => Ok(TextPosition::Center),
        other => Err(unknown_option(node_ref, "textPosition", other)),
    }
}
