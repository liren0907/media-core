//! Method manifest — every node kind declares its ports and parameters as DATA.
//!
//! This is the foundation of the whole `/composer` feature: picker filtering,
//! validation and the inspector UI are all driven from here, and the judgement is
//! made on the DECLARATION, never on the node id. Adding a step means touching
//! this file plus `registry.rs` — the frontend needs no change at all.
//!
//! Port types are the fields of `media_core::pipeline::MediaContext` (the shared
//! blackboard). An edge therefore means "slot dependency", not data transfer.
//!
//! See docs/REFERENCE_composer_method_manifest.md for how each entry was derived.

use serde::Serialize;

// ============================================================================
// PRIMITIVES
// ============================================================================

/// A slot on the `MediaContext` blackboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Slot {
    Source,
    Metadata,
    Analysis,
    ExtractedFrames,
    HlsResult,
    AnnotationResult,
    VideoProcessResult,
    ProcessResult,
}

impl Slot {
    pub fn label(self) -> &'static str {
        match self {
            Slot::Source => "media source",
            Slot::Metadata => "metadata",
            Slot::Analysis => "analysis report",
            Slot::ExtractedFrames => "in-memory frames",
            Slot::HlsResult => "HLS output",
            Slot::AnnotationResult => "annotation output",
            Slot::VideoProcessResult => "frames-on-disk output",
            Slot::ProcessResult => "file processing output",
        }
    }
}

/// Which `MediaSource` variants a step can actually cope with.
///
/// `MediaSource::as_path_str()` returns `None` for Stream and Camera, so every
/// step that calls it is File-only. Only `capture_frame` accepts all three.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    File,
    Stream,
    Camera,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteMode {
    /// Overwrites the slot wholesale — can clobber another step's contribution.
    Replace,
    /// Merges into the existing value.
    Merge,
    /// Pushes onto a collection.
    Append,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Source,
    Method,
    Sink,
}

impl NodeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            NodeKind::Source => "source",
            NodeKind::Method => "method",
            NodeKind::Sink => "sink",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Source,
    Extract,
    Analysis,
    Convert,
    Annotate,
    Process,
    Report,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamKind {
    Bool,
    Int,
    Float,
    #[serde(rename = "string")]
    Str,
    Path,
    Dir,
    Enum,
}

/// Where a left-blank parameter may draw its value from at run time.
///
/// A parameter carrying this is what makes the graph deep rather than flat:
/// it turns an upstream node's output into this node's input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FallbackSource {
    /// `video_process_result.output_dir` — produced by `extract_frames_to_disk`.
    VideoProcessOutputDir,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(untagged)]
pub enum DefaultValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(&'static str),
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParamOption {
    pub value: &'static str,
    pub label: &'static str,
}

impl ParamOption {
    const fn new(value: &'static str, label: &'static str) -> Self {
        Self { value, label }
    }
}

// ============================================================================
// PORT / PARAM SPECS
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotRef {
    pub slot: Slot,
    /// Only meaningful for `Slot::Source`. Empty = no constraint.
    #[serde(skip_serializing_if = "slice_is_empty")]
    pub accepts: &'static [SourceKind],
    /// The step still runs when nothing supplies this slot (it just does less).
    pub optional: bool,
    /// Setting this parameter removes the need for an upstream supplier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub satisfied_by_param: Option<&'static str>,
}

impl SlotRef {
    const fn new(slot: Slot) -> Self {
        Self { slot, accepts: &[], optional: false, satisfied_by_param: None }
    }
    const fn accepts(mut self, kinds: &'static [SourceKind]) -> Self {
        self.accepts = kinds;
        self
    }
    const fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
    const fn or_param(mut self, key: &'static str) -> Self {
        self.satisfied_by_param = Some(key);
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotWrite {
    pub slot: Slot,
    pub mode: WriteMode,
}

impl SlotWrite {
    const fn new(slot: Slot, mode: WriteMode) -> Self {
        Self { slot, mode }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParamSpec {
    pub key: &'static str,
    pub kind: ParamKind,
    pub label: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<DefaultValue>,
    pub required: bool,
    #[serde(skip_serializing_if = "slice_is_empty")]
    pub options: &'static [ParamOption],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_from: Option<FallbackSource>,
    /// Extra guidance shown under the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<&'static str>,
}

impl ParamSpec {
    const fn new(key: &'static str, kind: ParamKind, label: &'static str) -> Self {
        Self {
            key,
            kind,
            label,
            default: None,
            required: false,
            options: &[],
            min: None,
            max: None,
            fallback_from: None,
            hint: None,
        }
    }
    const fn required(mut self) -> Self {
        self.required = true;
        self
    }
    const fn def_bool(mut self, v: bool) -> Self {
        self.default = Some(DefaultValue::Bool(v));
        self
    }
    const fn def_int(mut self, v: i64) -> Self {
        self.default = Some(DefaultValue::Int(v));
        self
    }
    const fn def_float(mut self, v: f64) -> Self {
        self.default = Some(DefaultValue::Float(v));
        self
    }
    const fn def_str(mut self, v: &'static str) -> Self {
        self.default = Some(DefaultValue::Str(v));
        self
    }
    const fn options(mut self, o: &'static [ParamOption]) -> Self {
        self.options = o;
        self
    }
    const fn min(mut self, v: f64) -> Self {
        self.min = Some(v);
        self
    }
    const fn max(mut self, v: f64) -> Self {
        self.max = Some(v);
        self
    }
    const fn fallback(mut self, f: FallbackSource) -> Self {
        self.fallback_from = Some(f);
        self
    }
    const fn hint(mut self, h: &'static str) -> Self {
        self.hint = Some(h);
        self
    }
}

// ============================================================================
// METHOD META
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MethodMeta {
    pub id: &'static str,
    pub label: &'static str,
    pub kind: NodeKind,
    pub category: Category,
    pub description: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<&'static str>,
    pub reads: &'static [SlotRef],
    pub writes: &'static [SlotWrite],
    pub params: &'static [ParamSpec],
    /// Needs `ffmpeg` on PATH. A bundled .app launched from Finder does not
    /// inherit the shell PATH — flagged so the UI can warn.
    pub requires_ffmpeg: bool,
}

impl MethodMeta {
    pub fn param(&self, key: &str) -> Option<&'static ParamSpec> {
        self.params.iter().find(|p| p.key == key)
    }
}

fn slice_is_empty<T>(s: &&'static [T]) -> bool {
    s.is_empty()
}

// ============================================================================
// ENUM OPTION TABLES
// ============================================================================

const OPT_MOTION_ALGORITHM: &[ParamOption] = &[
    ParamOption::new("frame_diff", "Frame difference (fastest)"),
    ParamOption::new("mog2", "MOG2 background subtraction"),
    ParamOption::new("knn", "KNN background subtraction"),
    ParamOption::new("optical_flow", "Optical flow"),
];

const OPT_SIMILARITY_METHOD: &[ParamOption] = &[
    ParamOption::new("histogram", "Histogram"),
    ParamOption::new("perceptual_hash", "Perceptual hash"),
    ParamOption::new("feature_matching", "Feature matching"),
];

const OPT_PROCESS_MODE: &[ParamOption] = &[
    ParamOption::new("single", "Single-threaded"),
    ParamOption::new("parallel", "Parallel"),
];

const OPT_SAMPLING_STRATEGY: &[ParamOption] = &[
    ParamOption::new("every_nth", "Every Nth frame"),
    ParamOption::new("first_n", "First N frames"),
    ParamOption::new("range", "Frame range"),
    ParamOption::new("key_frames", "Key frames"),
];

/// `streaming::ExtractionMode` — NOT the same enum as `extractionMode` below.
const OPT_FRAME_MODE: &[ParamOption] = &[
    ParamOption::new("seek", "Seek to each frame (sparse)"),
    ParamOption::new("sequential", "Read sequentially (dense)"),
];

/// `video_process::ExtractionMode` — NOT the same enum as `frameMode` above.
const OPT_EXTRACTION_MODE: &[ParamOption] = &[
    ParamOption::new("opencv_interval", "OpenCV, by interval"),
    ParamOption::new("opencv_sequential", "OpenCV, sequential"),
    ParamOption::new("ffmpeg", "ffmpeg"),
    ParamOption::new("ffmpeg_interval", "ffmpeg, by interval"),
    ParamOption::new("parallel", "Parallel"),
];

const OPT_SAVE_MODE: &[ParamOption] = &[
    ParamOption::new("single_directory", "One directory"),
    ParamOption::new("multiple_directory", "Split directories"),
];

const OPT_PROCESSING_MODE: &[ParamOption] = &[
    ParamOption::new("single_file", "Single file"),
    ParamOption::new("batch_files", "Batch files"),
    ParamOption::new("directory_process", "Directory"),
    ParamOption::new("stream_process", "Stream"),
];

const OPT_ANNOTATION_TYPE: &[ParamOption] = &[
    ParamOption::new("filename", "Filename"),
    ParamOption::new("timestamp", "Timestamp"),
];

const OPT_TEXT_POSITION: &[ParamOption] = &[
    ParamOption::new("top_left", "Top left"),
    ParamOption::new("top_right", "Top right"),
    ParamOption::new("bottom_left", "Bottom left"),
    ParamOption::new("bottom_right", "Bottom right"),
    ParamOption::new("center", "Center"),
];

const ALL_SOURCE_KINDS: &[SourceKind] = &[SourceKind::File, SourceKind::Stream, SourceKind::Camera];
const FILE_ONLY: &[SourceKind] = &[SourceKind::File];

// ============================================================================
// THE TABLE
// ============================================================================

pub const METHODS: &[MethodMeta] = &[
    // ---------------------------------------------------------------- sources
    MethodMeta {
        id: "file",
        label: "File",
        kind: NodeKind::Source,
        category: Category::Source,
        description: "A local video or image file.",
        group: None,
        reads: &[],
        writes: &[SlotWrite::new(Slot::Source, WriteMode::Replace)],
        params: &[ParamSpec::new("path", ParamKind::Path, "File path").required()],
        requires_ffmpeg: false,
    },
    MethodMeta {
        id: "stream",
        label: "Stream",
        kind: NodeKind::Source,
        category: Category::Source,
        description: "A network stream URL (RTSP / RTMP / HTTP). Only Capture frame can consume it.",
        group: None,
        reads: &[],
        writes: &[SlotWrite::new(Slot::Source, WriteMode::Replace)],
        params: &[ParamSpec::new("url", ParamKind::Str, "Stream URL").required()],
        requires_ffmpeg: false,
    },
    MethodMeta {
        id: "camera",
        label: "Camera",
        kind: NodeKind::Source,
        category: Category::Source,
        description: "A local camera device. Only Capture frame can consume it.",
        group: None,
        reads: &[],
        writes: &[SlotWrite::new(Slot::Source, WriteMode::Replace)],
        params: &[ParamSpec::new("cameraId", ParamKind::Int, "Camera ID")
            .def_int(0)
            .min(0.0)],
        requires_ffmpeg: false,
    },
    // ---------------------------------------------------------------- methods
    MethodMeta {
        id: "extract_metadata",
        label: "Extract metadata",
        kind: NodeKind::Method,
        category: Category::Extract,
        description: "Reads codec, resolution, duration and optionally a thumbnail.",
        group: None,
        reads: &[SlotRef::new(Slot::Source).accepts(FILE_ONLY)],
        writes: &[SlotWrite::new(Slot::Metadata, WriteMode::Replace)],
        params: &[ParamSpec::new("includeThumbnail", ParamKind::Bool, "Include thumbnail")
            .def_bool(false)],
        requires_ffmpeg: false,
    },
    MethodMeta {
        id: "detect_motion",
        label: "Detect motion",
        kind: NodeKind::Method,
        category: Category::Analysis,
        description: "Finds motion segments in a video.",
        group: None,
        reads: &[SlotRef::new(Slot::Source).accepts(FILE_ONLY)],
        // NOTE: this one REPLACES the whole analysis report (analysis/pipeline.rs:96),
        // so it wipes similarity groups written by the two steps below.
        writes: &[SlotWrite::new(Slot::Analysis, WriteMode::Replace)],
        params: &[
            ParamSpec::new("algorithm", ParamKind::Enum, "Algorithm")
                .options(OPT_MOTION_ALGORITHM)
                .def_str("frame_diff"),
            ParamSpec::new("threshold", ParamKind::Float, "Threshold")
                .def_float(25.0)
                .min(0.0),
            ParamSpec::new("minArea", ParamKind::Int, "Minimum area")
                .def_int(500)
                .min(0.0),
            ParamSpec::new("outputDir", ParamKind::Dir, "Debug output directory")
                .hint("Leave blank to skip writing debug images."),
        ],
        requires_ffmpeg: false,
    },
    MethodMeta {
        id: "group_similar_images",
        label: "Group similar images",
        kind: NodeKind::Method,
        category: Category::Analysis,
        description: "Clusters a directory of images by visual similarity.",
        group: None,
        // The input directory can come from an upstream frame extraction, which is
        // what makes this node stackable rather than another source-fed leaf.
        reads: &[SlotRef::new(Slot::VideoProcessResult).or_param("inputDir")],
        writes: &[SlotWrite::new(Slot::Analysis, WriteMode::Merge)],
        params: &[
            ParamSpec::new("inputDir", ParamKind::Dir, "Input directory")
                .fallback(FallbackSource::VideoProcessOutputDir)
                .hint("Leave blank to use the upstream frame output directory."),
            ParamSpec::new("outputDir", ParamKind::Dir, "Output directory").required(),
            ParamSpec::new("method", ParamKind::Enum, "Method")
                .options(OPT_SIMILARITY_METHOD)
                .def_str("histogram"),
            ParamSpec::new("processMode", ParamKind::Enum, "Processing mode")
                .options(OPT_PROCESS_MODE)
                .def_str("single"),
            ParamSpec::new("minCategorySize", ParamKind::Int, "Minimum group size").min(0.0),
            ParamSpec::new("groupSimilar", ParamKind::Bool, "Move files into group folders"),
            ParamSpec::new("threshold", ParamKind::Float, "Similarity threshold")
                .min(0.0)
                .max(1.0)
                .hint("Applies to whichever method is selected."),
        ],
        requires_ffmpeg: false,
    },
    MethodMeta {
        id: "compare_images",
        label: "Compare two images",
        kind: NodeKind::Method,
        category: Category::Analysis,
        description: "Scores the similarity of two specific images.",
        group: None,
        reads: &[],
        writes: &[SlotWrite::new(Slot::Analysis, WriteMode::Merge)],
        params: &[
            ParamSpec::new("image1", ParamKind::Path, "First image").required(),
            ParamSpec::new("image2", ParamKind::Path, "Second image").required(),
            ParamSpec::new("method", ParamKind::Enum, "Method")
                .options(OPT_SIMILARITY_METHOD)
                .def_str("perceptual_hash"),
            ParamSpec::new("threshold", ParamKind::Float, "Duplicate threshold")
                .def_float(0.95)
                .min(0.0)
                .max(1.0),
        ],
        requires_ffmpeg: false,
    },
    MethodMeta {
        id: "extract_frames",
        label: "Extract frames (memory)",
        kind: NodeKind::Method,
        category: Category::Extract,
        description: "Samples frames into memory as base64 JPEGs.",
        group: Some("extract_frames"),
        reads: &[SlotRef::new(Slot::Source).accepts(FILE_ONLY)],
        writes: &[SlotWrite::new(Slot::ExtractedFrames, WriteMode::Replace)],
        params: &[
            ParamSpec::new("strategy", ParamKind::Enum, "Sampling strategy")
                .options(OPT_SAMPLING_STRATEGY)
                .def_str("every_nth"),
            ParamSpec::new("strategyParam", ParamKind::Int, "N")
                .min(1.0)
                .hint("Used by 'Every Nth' (default 30) and 'First N' (default 10)."),
            ParamSpec::new("rangeStart", ParamKind::Int, "Range start").min(0.0),
            ParamSpec::new("rangeEnd", ParamKind::Int, "Range end").min(0.0),
            ParamSpec::new("scaleFactor", ParamKind::Float, "Scale factor")
                .min(0.0)
                .max(1.0),
            ParamSpec::new("frameMode", ParamKind::Enum, "Read mode")
                .options(OPT_FRAME_MODE)
                .def_str("seek"),
        ],
        requires_ffmpeg: false,
    },
    MethodMeta {
        id: "extract_frames_to_disk",
        label: "Extract frames (disk)",
        kind: NodeKind::Method,
        category: Category::Extract,
        description: "Writes sampled frames to a directory. The only step other nodes can chain from.",
        group: Some("extract_frames"),
        reads: &[SlotRef::new(Slot::Source).accepts(FILE_ONLY)],
        writes: &[SlotWrite::new(Slot::VideoProcessResult, WriteMode::Replace)],
        params: &[
            ParamSpec::new("outputDir", ParamKind::Dir, "Output directory").required(),
            ParamSpec::new("interval", ParamKind::Int, "Frame interval")
                .def_int(30)
                .min(1.0),
            ParamSpec::new("extractionMode", ParamKind::Enum, "Extraction mode")
                .options(OPT_EXTRACTION_MODE)
                .def_str("opencv_interval"),
            // Defaulted in the registry rather than left to media-core, whose own
            // default (MultipleDirectory) would put the frames in a subdirectory and
            // break the output_dir inheritance downstream nodes rely on.
            ParamSpec::new("saveMode", ParamKind::Enum, "Save mode")
                .options(OPT_SAVE_MODE)
                .def_str("single_directory")
                .hint("Split directories nest frames under the output directory, so downstream steps cannot inherit it."),
        ],
        requires_ffmpeg: false,
    },
    MethodMeta {
        id: "convert_to_hls",
        label: "Convert to HLS",
        kind: NodeKind::Method,
        category: Category::Convert,
        description: "Segments a video into an HLS playlist.",
        group: None,
        reads: &[SlotRef::new(Slot::Source).accepts(FILE_ONLY)],
        writes: &[SlotWrite::new(Slot::HlsResult, WriteMode::Replace)],
        params: &[
            ParamSpec::new("outputDir", ParamKind::Dir, "Output directory").required(),
            ParamSpec::new("segmentDuration", ParamKind::Int, "Segment duration (s)")
                .def_int(5)
                .min(1.0),
            ParamSpec::new("playlistName", ParamKind::Str, "Playlist filename")
                .def_str("playlist.m3u8"),
            ParamSpec::new("forceKeyframes", ParamKind::Bool, "Force keyframes").def_bool(true),
            ParamSpec::new("profile", ParamKind::Str, "H.264 profile").def_str("baseline"),
            ParamSpec::new("level", ParamKind::Str, "H.264 level").def_str("3.0"),
        ],
        requires_ffmpeg: true,
    },
    MethodMeta {
        id: "capture_frame",
        label: "Capture frame",
        kind: NodeKind::Method,
        category: Category::Extract,
        description: "Grabs a single frame. The only step that accepts stream and camera sources.",
        group: None,
        reads: &[SlotRef::new(Slot::Source).accepts(ALL_SOURCE_KINDS)],
        writes: &[SlotWrite::new(Slot::ExtractedFrames, WriteMode::Append)],
        params: &[],
        requires_ffmpeg: false,
    },
    MethodMeta {
        id: "annotate_frame",
        label: "Annotate image",
        kind: NodeKind::Method,
        category: Category::Annotate,
        description: "Overlays text on a single image.",
        group: Some("annotate"),
        reads: &[SlotRef::new(Slot::Source).accepts(FILE_ONLY).or_param("inputPath")],
        writes: &[SlotWrite::new(Slot::AnnotationResult, WriteMode::Replace)],
        params: &[
            ParamSpec::new("inputPath", ParamKind::Path, "Input image")
                .hint("Leave blank to use the graph's source file."),
            ParamSpec::new("outputPath", ParamKind::Path, "Output image").required(),
            ParamSpec::new("annotationType", ParamKind::Enum, "Text content")
                .options(OPT_ANNOTATION_TYPE)
                .def_str("filename"),
            ParamSpec::new("textPosition", ParamKind::Enum, "Text position")
                .options(OPT_TEXT_POSITION)
                .def_str("top_left"),
        ],
        requires_ffmpeg: false,
    },
    MethodMeta {
        id: "annotate_video",
        label: "Annotate video",
        kind: NodeKind::Method,
        category: Category::Annotate,
        description: "Builds an annotated video from a directory of frames.",
        group: Some("annotate"),
        reads: &[SlotRef::new(Slot::Source).accepts(FILE_ONLY).or_param("framesDir")],
        writes: &[SlotWrite::new(Slot::AnnotationResult, WriteMode::Replace)],
        params: &[
            ParamSpec::new("framesDir", ParamKind::Dir, "Frames directory")
                .fallback(FallbackSource::VideoProcessOutputDir)
                .hint("Leave blank to use the upstream frame output directory."),
            ParamSpec::new("outputPath", ParamKind::Path, "Output video").required(),
            ParamSpec::new("annotationType", ParamKind::Enum, "Text content")
                .options(OPT_ANNOTATION_TYPE)
                .def_str("filename"),
            ParamSpec::new("textPosition", ParamKind::Enum, "Text position")
                .options(OPT_TEXT_POSITION)
                .def_str("top_left"),
            ParamSpec::new("fps", ParamKind::Int, "Output FPS").def_int(30).min(1.0),
            ParamSpec::new("sourceFps", ParamKind::Float, "Source FPS")
                .def_float(30.0)
                .min(0.0),
        ],
        requires_ffmpeg: false,
    },
    MethodMeta {
        id: "process_files",
        label: "Process files",
        kind: NodeKind::Method,
        category: Category::Process,
        description: "Runs the file processor over the source. Cannot take its input from an upstream node.",
        group: None,
        reads: &[SlotRef::new(Slot::Source).accepts(FILE_ONLY)],
        writes: &[SlotWrite::new(Slot::ProcessResult, WriteMode::Replace)],
        params: &[
            ParamSpec::new("outputDir", ParamKind::Dir, "Output directory").required(),
            ParamSpec::new("processingMode", ParamKind::Enum, "Processing mode")
                .options(OPT_PROCESSING_MODE)
                .def_str("single_file"),
            ParamSpec::new("overwrite", ParamKind::Bool, "Overwrite existing").def_bool(false),
        ],
        requires_ffmpeg: false,
    },
    // ------------------------------------------------------------------ sinks
    MethodMeta {
        id: "save_report",
        label: "Save report",
        kind: NodeKind::Sink,
        category: Category::Report,
        description: "Writes whatever metadata and analysis the graph produced to a JSON file.",
        group: None,
        // Both optional: the step runs as long as at least one is present, which
        // is checked at run time rather than at validation time.
        reads: &[
            SlotRef::new(Slot::Metadata).optional(),
            SlotRef::new(Slot::Analysis).optional(),
        ],
        writes: &[],
        params: &[
            ParamSpec::new("outputPath", ParamKind::Path, "Output JSON file").required(),
            ParamSpec::new("pretty", ParamKind::Bool, "Pretty-print").def_bool(true),
        ],
        requires_ffmpeg: false,
    },
];

// ============================================================================
// LOOKUP
// ============================================================================

pub fn find(node_ref: &str) -> Option<&'static MethodMeta> {
    METHODS.iter().find(|m| m.id == node_ref)
}

pub fn source_kind_of(node_ref: &str) -> Option<SourceKind> {
    match node_ref {
        "file" => Some(SourceKind::File),
        "stream" => Some(SourceKind::Stream),
        "camera" => Some(SourceKind::Camera),
        _ => None,
    }
}
