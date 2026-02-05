//! Pipeline Executor Commands
//!
//! This module provides Tauri commands that execute media_core pipelines.
//! Each command builds a pipeline using the builder pattern and returns
//! the results stored in the MediaContext.

use media_core::pipeline::{MediaContext, Pipeline};
use media_core::metadata::pipeline::ExtractMetadata;
use media_core::metadata::MediaMetadata;
use media_core::analysis::pipeline::{DetectMotion, GroupSimilarImages, CompareImages};
use media_core::analysis::config::{MotionAlgorithm, SimilarityMethod};
use media_core::streaming::pipeline::ExtractFrames;
use media_core::streaming::SamplingStrategy;
use media_core::hls::pipeline::ConvertToHLS;
use media_core::video_process::pipeline::ExtractFramesToDisk;
use media_core::video_process::{ExtractionMode, SaveMode};
use media_core::camera::pipeline::CaptureFrame;

use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::command;

// ============================================================================
// SERIALIZABLE ANALYSIS TYPES (wrappers for media_core types)
// ============================================================================

/// Serializable motion event
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MotionEvent {
    pub start_frame: i32,
    pub end_frame: i32,
    pub event_type: String,
}

/// Serializable similarity group
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarityGroup {
    pub group_name: String,
    pub members: Vec<String>,
}

/// Serializable image comparison result
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageComparisonResult {
    pub image1: String,
    pub image2: String,
    pub similarity_score: f64,
    pub is_duplicate: bool,
}

/// Serializable analysis report
#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResult {
    pub motion_events: Vec<MotionEvent>,
    pub similarity_groups: Vec<SimilarityGroup>,
    pub image_comparison: Option<ImageComparisonResult>,
}

/// Serializable frame data
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameDataResult {
    pub index: usize,
    pub data: String,  // Base64 encoded
}

// ============================================================================
// METADATA PIPELINE
// ============================================================================

/// Execute metadata extraction pipeline
#[command]
pub async fn execute_metadata_pipeline(
    file_path: String,
    include_thumbnail: bool,
) -> Result<MediaMetadata, String> {
    std::thread::spawn(move || {
        let context = MediaContext::from_file(PathBuf::from(&file_path));
        
        let pipeline = Pipeline::new()
            .add_node(ExtractMetadata::new(include_thumbnail));
        
        let result = pipeline.execute(context)
            .map_err(|e| format!("Pipeline execution failed: {}", e))?;
        
        result.metadata.ok_or_else(|| "No metadata extracted".to_string())
    })
    .join()
    .map_err(|_| "Thread panicked".to_string())?
}

// ============================================================================
// ANALYSIS PIPELINE
// ============================================================================

/// Configuration for motion detection
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MotionDetectionConfig {
    pub video_path: String,
    pub output_dir: Option<String>,
    pub algorithm: Option<String>,  // "frame_diff", "mog2", "knn", "optical_flow"
    pub threshold: Option<f64>,
    pub min_area: Option<i32>,
}

/// Execute motion detection pipeline
#[command]
pub async fn execute_motion_pipeline(config: MotionDetectionConfig) -> Result<AnalysisResult, String> {
    std::thread::spawn(move || {
        let context = MediaContext::from_file(PathBuf::from(&config.video_path));
        
        let mut step = DetectMotion::new();
        
        // Apply configuration
        if let Some(algo) = &config.algorithm {
            step = step.algorithm(match algo.as_str() {
                "mog2" => MotionAlgorithm::Mog2,
                "knn" => MotionAlgorithm::Knn,
                "optical_flow" => MotionAlgorithm::OpticalFlow,
                _ => MotionAlgorithm::FrameDiff,
            });
        }
        
        if let Some(threshold) = config.threshold {
            step = step.threshold(threshold);
        }
        
        if let Some(min_area) = config.min_area {
            step = step.min_area(min_area);
        }
        
        if let Some(output_dir) = &config.output_dir {
            step = step.output_dir(output_dir);
        }
        
        let pipeline = Pipeline::new().add_node(step);
        
        let result = pipeline.execute(context)
            .map_err(|e| format!("Motion detection failed: {}", e))?;
        
        let analysis = result.analysis.ok_or_else(|| "No motion analysis results".to_string())?;
        
        // Convert to serializable type
        Ok(AnalysisResult {
            motion_events: analysis.motion_events.into_iter().map(|e| MotionEvent {
                start_frame: e.start_frame,
                end_frame: e.end_frame,
                event_type: e.event_type,
            }).collect(),
            similarity_groups: vec![],
            image_comparison: None,
        })
    })
    .join()
    .map_err(|_| "Thread panicked".to_string())?
}

/// Configuration for similarity grouping
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarityGroupConfig {
    pub input_dir: String,
    pub output_dir: String,
    pub method: Option<String>,  // "histogram", "phash", "feature"
    pub threshold: Option<f64>,
    pub min_group_size: Option<i32>,
}

/// Execute similarity grouping pipeline
#[command]
pub async fn execute_similarity_pipeline(config: SimilarityGroupConfig) -> Result<AnalysisResult, String> {
    std::thread::spawn(move || {
        // For similarity, we use the input_dir as the source
        let context = MediaContext::from_file(PathBuf::from(&config.input_dir));
        
        let mut step = GroupSimilarImages::new(&config.input_dir, &config.output_dir);
        
        // Apply configuration
        if let Some(method) = &config.method {
            step = step.method(match method.as_str() {
                "histogram" => SimilarityMethod::Histogram,
                "feature" => SimilarityMethod::FeatureMatching,
                _ => SimilarityMethod::PerceptualHash,
            });
        }
        
        if let Some(threshold) = config.threshold {
            step = match config.method.as_deref() {
                Some("histogram") => step.histogram_threshold(threshold),
                Some("feature") => step.feature_threshold(threshold),
                _ => step.phash_threshold(threshold),
            };
        }
        
        if let Some(min_size) = config.min_group_size {
            step = step.min_category_size(min_size);
        }
        
        let pipeline = Pipeline::new().add_node(step);
        
        let result = pipeline.execute(context)
            .map_err(|e| format!("Similarity analysis failed: {}", e))?;
        
        let analysis = result.analysis.ok_or_else(|| "No similarity results".to_string())?;
        
        // Convert to serializable type
        Ok(AnalysisResult {
            motion_events: vec![],
            similarity_groups: analysis.similarity_groups.into_iter().map(|g| SimilarityGroup {
                group_name: g.group_name,
                members: g.members,
            }).collect(),
            image_comparison: None,
        })
    })
    .join()
    .map_err(|_| "Thread panicked".to_string())?
}

/// Configuration for image comparison
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageCompareConfig {
    pub image1: String,
    pub image2: String,
    pub method: Option<String>,
    pub threshold: Option<f64>,
}

/// Execute image comparison pipeline
#[command]
pub async fn execute_compare_pipeline(config: ImageCompareConfig) -> Result<AnalysisResult, String> {
    std::thread::spawn(move || {
        let context = MediaContext::from_file(PathBuf::from(&config.image1));
        
        let mut step = CompareImages::new(&config.image1, &config.image2);
        
        if let Some(method) = &config.method {
            step = step.method(match method.as_str() {
                "histogram" => SimilarityMethod::Histogram,
                "feature" => SimilarityMethod::FeatureMatching,
                _ => SimilarityMethod::PerceptualHash,
            });
        }
        
        if let Some(threshold) = config.threshold {
            step = step.threshold(threshold);
        }
        
        let pipeline = Pipeline::new().add_node(step);
        
        let result = pipeline.execute(context)
            .map_err(|e| format!("Image comparison failed: {}", e))?;
        
        let analysis = result.analysis.ok_or_else(|| "No comparison results".to_string())?;
        
        // Convert to serializable type
        Ok(AnalysisResult {
            motion_events: vec![],
            similarity_groups: vec![],
            image_comparison: analysis.image_comparison.map(|c| ImageComparisonResult {
                image1: c.image1,
                image2: c.image2,
                similarity_score: c.similarity_score,
                is_duplicate: c.is_duplicate,
            }),
        })
    })
    .join()
    .map_err(|_| "Thread panicked".to_string())?
}

// ============================================================================
// STREAMING PIPELINE
// ============================================================================

/// Configuration for frame extraction
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameExtractionConfig {
    pub video_path: String,
    pub strategy: String,      // "every_nth", "first_n", "range", "keyframes"
    pub strategy_param: Option<usize>,  // N for every_nth/first_n
    pub range_start: Option<usize>,
    pub range_end: Option<usize>,
    pub scale_factor: Option<f64>,
}

/// Execute frame extraction pipeline (in-memory)
#[command]
pub async fn execute_extract_frames_pipeline(config: FrameExtractionConfig) -> Result<Vec<FrameDataResult>, String> {
    std::thread::spawn(move || {
        let context = MediaContext::from_file(PathBuf::from(&config.video_path));
        
        let strategy = match config.strategy.as_str() {
            "every_nth" => SamplingStrategy::EveryNth(config.strategy_param.unwrap_or(30)),
            "first_n" => SamplingStrategy::FirstN(config.strategy_param.unwrap_or(10)),
            "range" => SamplingStrategy::Range(
                config.range_start.unwrap_or(0),
                config.range_end.unwrap_or(100),
            ),
            "keyframes" => SamplingStrategy::KeyFrames,
            _ => SamplingStrategy::EveryNth(30),
        };
        
        let mut step = ExtractFrames::new(strategy);
        
        if let Some(scale) = config.scale_factor {
            step = step.with_scale(scale);
        }
        
        let pipeline = Pipeline::new().add_node(step);
        
        let result = pipeline.execute(context)
            .map_err(|e| format!("Frame extraction failed: {}", e))?;
        
        // Convert to serializable type
        Ok(result.extracted_frames.into_iter().map(|f| FrameDataResult {
            index: f.index,
            data: f.data,
        }).collect())
    })
    .join()
    .map_err(|_| "Thread panicked".to_string())?
}

// ============================================================================
// HLS PIPELINE
// ============================================================================

/// Configuration for HLS conversion
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HLSConversionConfig {
    pub video_path: String,
    pub output_dir: String,
    pub segment_duration: Option<u32>,
    pub playlist_name: Option<String>,
    pub profile: Option<String>,
    pub level: Option<String>,
}

/// Result of HLS conversion
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HLSConversionResult {
    pub output_dir: String,
    pub playlist_path: String,
    pub segment_count: usize,
    pub success: bool,
}

/// Execute HLS conversion pipeline
#[command]
pub async fn execute_hls_pipeline(config: HLSConversionConfig) -> Result<HLSConversionResult, String> {
    std::thread::spawn(move || {
        let context = MediaContext::from_file(PathBuf::from(&config.video_path));
        
        let mut step = ConvertToHLS::new(&config.output_dir);
        
        if let Some(duration) = config.segment_duration {
            step = step.segment_duration(duration);
        }
        
        if let Some(name) = &config.playlist_name {
            step = step.playlist_name(name);
        }
        
        if let Some(profile) = &config.profile {
            step = step.profile(profile);
        }
        
        if let Some(level) = &config.level {
            step = step.level(level);
        }
        
        let pipeline = Pipeline::new().add_node(step);
        
        let result = pipeline.execute(context)
            .map_err(|e| format!("HLS conversion failed: {}", e))?;
        
        let hls = result.hls_result.ok_or_else(|| "No HLS result".to_string())?;
        
        Ok(HLSConversionResult {
            output_dir: hls.output_dir,
            playlist_path: hls.playlist_path,
            segment_count: hls.segment_count,
            success: hls.success,
        })
    })
    .join()
    .map_err(|_| "Thread panicked".to_string())?
}

// ============================================================================
// VIDEO PROCESS PIPELINE
// ============================================================================

/// Configuration for extracting frames to disk
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractToDiskConfig {
    pub video_path: String,
    pub output_dir: String,
    pub interval: Option<usize>,
    pub extraction_mode: Option<String>,  // "opencv_sequential", "opencv_interval", "ffmpeg", "parallel"
    pub save_mode: Option<String>,        // "single", "multiple"
}

/// Result of frame extraction to disk
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractToDiskResult {
    pub output_dir: String,
    pub frames_extracted: usize,
    pub extraction_mode: String,
    pub save_mode: String,
    pub success: bool,
}

/// Execute frame extraction to disk pipeline
#[command]
pub async fn execute_extract_to_disk_pipeline(config: ExtractToDiskConfig) -> Result<ExtractToDiskResult, String> {
    std::thread::spawn(move || {
        let context = MediaContext::from_file(PathBuf::from(&config.video_path));
        
        let mut step = ExtractFramesToDisk::new(&config.output_dir);
        
        if let Some(interval) = config.interval {
            step = step.interval(interval);
        }
        
        if let Some(mode) = &config.extraction_mode {
            step = step.mode(match mode.as_str() {
                "opencv_sequential" => ExtractionMode::OpenCVSequential,
                "ffmpeg" => ExtractionMode::FFmpeg,
                "ffmpeg_interval" => ExtractionMode::FFmpegInterval,
                "parallel" => ExtractionMode::Parallel,
                _ => ExtractionMode::OpenCVInterval,  // Default
            });
        }
        
        if let Some(save) = &config.save_mode {
            step = step.save_mode(match save.as_str() {
                "multiple" => SaveMode::MultipleDirectory,
                _ => SaveMode::SingleDirectory,
            });
        }
        
        let pipeline = Pipeline::new().add_node(step);
        
        let result = pipeline.execute(context)
            .map_err(|e| format!("Frame extraction failed: {}", e))?;
        
        let vp_result = result.video_process_result
            .ok_or_else(|| "No video process result".to_string())?;
        
        Ok(ExtractToDiskResult {
            output_dir: vp_result.output_dir,
            frames_extracted: vp_result.frames_extracted,
            extraction_mode: vp_result.extraction_mode,
            save_mode: vp_result.save_mode,
            success: vp_result.success,
        })
    })
    .join()
    .map_err(|_| "Thread panicked".to_string())?
}

// ============================================================================
// CAMERA PIPELINE
// ============================================================================

/// Configuration for camera capture
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraCaptureConfig {
    pub camera_id: i32,
    pub output_path: Option<String>,
}

/// Result of camera capture
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraCaptureResult {
    pub camera_id: i32,
    pub output_path: Option<String>,
    pub frame_data: Option<String>,  // Base64 encoded
    pub success: bool,
}

/// Execute camera capture pipeline
#[command]
pub async fn execute_camera_pipeline(config: CameraCaptureConfig) -> Result<CameraCaptureResult, String> {
    std::thread::spawn(move || {
        let context = MediaContext::from_camera(config.camera_id);
        
        // CaptureFrame is a unit struct - no builder pattern
        let step = CaptureFrame;
        
        let pipeline = Pipeline::new().add_node(step);
        
        let result = pipeline.execute(context)
            .map_err(|e| format!("Camera capture failed: {}", e))?;
        
        // Get frame data from extracted_frames
        let frame_data = result.extracted_frames.first()
            .map(|f| f.data.clone());
        
        // If output_path is specified, save the frame to disk
        if let (Some(output_path), Some(ref data)) = (&config.output_path, &frame_data) {
            let image_data = BASE64_STANDARD
                .decode(data)
                .map_err(|e| format!("Failed to decode base64: {}", e))?;
            std::fs::write(output_path, image_data)
                .map_err(|e| format!("Failed to save frame: {}", e))?;
        }
        
        Ok(CameraCaptureResult {
            camera_id: config.camera_id,
            output_path: config.output_path,
            frame_data,
            success: true,
        })
    })
    .join()
    .map_err(|_| "Thread panicked".to_string())?
}

// ============================================================================
// BATCH PROCESSING
// ============================================================================

/// Configuration for batch processing
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchProcessConfig {
    pub file_paths: Vec<String>,
    pub operation: String, // "metadata", "motion", "hls", "extract"
    pub parallel: Option<bool>,
    pub options: Option<serde_json::Value>,
}

/// Result of a single batch item
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchItemResult {
    pub file_path: String,
    pub success: bool,
    pub error: Option<String>,
    pub result: Option<serde_json::Value>,
}

/// Result of batch processing
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchProcessResult {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub results: Vec<BatchItemResult>,
}

/// Execute batch processing on multiple files
#[command]
pub async fn execute_batch_pipeline(
    app: tauri::AppHandle,
    config: BatchProcessConfig,
) -> Result<BatchProcessResult, String> {
    use super::events::{PipelineHandle, ACTIVE_PIPELINES};
    use tauri::Emitter;
    use std::collections::HashMap;
    
    let pipeline_id = uuid::Uuid::new_v4().to_string();
    let total = config.file_paths.len();
    
    // Register pipeline for tracking
    {
        let handle = PipelineHandle::new(pipeline_id.clone());
        if let Ok(mut pipelines) = ACTIVE_PIPELINES.lock() {
            let pipelines: &mut HashMap<String, PipelineHandle> = &mut *pipelines;
            pipelines.insert(pipeline_id.clone(), handle);
        }
    }
    
    let results: Vec<BatchItemResult> = if config.parallel.unwrap_or(false) {
        // Parallel processing using threads
        use std::sync::mpsc;
        let (tx, rx) = mpsc::channel();
        
        for (index, file_path) in config.file_paths.iter().enumerate() {
            let tx = tx.clone();
            let file = file_path.clone();
            let op = config.operation.clone();
            let opts = config.options.clone();
            let app_clone = app.clone();
            let pid = pipeline_id.clone();
            
            std::thread::spawn(move || {
                // Check for cancellation
                let cancelled = {
                    if let Ok(pipelines) = ACTIVE_PIPELINES.lock() {
                        let pipelines: &HashMap<String, PipelineHandle> = &*pipelines;
                        pipelines.get(&pid).map(|h: &PipelineHandle| h.is_cancelled()).unwrap_or(false)
                    } else {
                        false
                    }
                };
                
                if cancelled {
                    tx.send(BatchItemResult {
                        file_path: file,
                        success: false,
                        error: Some("Cancelled".to_string()),
                        result: None,
                    }).ok();
                    return;
                }
                
                let result = process_single_file(&file, &op, opts.as_ref());
                
                // Emit progress
                let _ = app_clone.emit("pipeline:progress", serde_json::json!({
                    "pipelineId": pid,
                    "stepName": op,
                    "stepIndex": index,
                    "totalSteps": total,
                    "progressPercent": ((index + 1) as f64 / total as f64) * 100.0,
                    "message": format!("Processing {}", file),
                    "isComplete": false,
                    "hasError": result.is_err(),
                }));
                
                tx.send(match result {
                    Ok(val) => BatchItemResult {
                        file_path: file,
                        success: true,
                        error: None,
                        result: Some(val),
                    },
                    Err(e) => BatchItemResult {
                        file_path: file,
                        success: false,
                        error: Some(e),
                        result: None,
                    },
                }).ok();
            });
        }
        drop(tx);
        
        rx.iter().collect()
    } else {
        // Sequential processing
        let mut results = Vec::with_capacity(total);
        
        for (index, file_path) in config.file_paths.iter().enumerate() {
            // Check for cancellation
            let cancelled = {
                if let Ok(pipelines) = ACTIVE_PIPELINES.lock() {
                    let pipelines: &HashMap<String, PipelineHandle> = &*pipelines;
                    pipelines.get(&pipeline_id).map(|h: &PipelineHandle| h.is_cancelled()).unwrap_or(false)
                } else {
                    false
                }
            };
            
            if cancelled {
                results.push(BatchItemResult {
                    file_path: file_path.clone(),
                    success: false,
                    error: Some("Cancelled".to_string()),
                    result: None,
                });
                continue;
            }
            
            // Emit progress
            let _ = app.emit("pipeline:progress", serde_json::json!({
                "pipelineId": pipeline_id,
                "stepName": config.operation,
                "stepIndex": index,
                "totalSteps": total,
                "progressPercent": (index as f64 / total as f64) * 100.0,
                "message": format!("Processing {}", file_path),
                "isComplete": false,
                "hasError": false,
            }));
            
            let result = process_single_file(file_path, &config.operation, config.options.as_ref());
            
            results.push(match result {
                Ok(val) => BatchItemResult {
                    file_path: file_path.clone(),
                    success: true,
                    error: None,
                    result: Some(val),
                },
                Err(e) => BatchItemResult {
                    file_path: file_path.clone(),
                    success: false,
                    error: Some(e),
                    result: None,
                },
            });
        }
        
        results
    };
    
    // Cleanup pipeline tracking
    if let Ok(mut pipelines) = ACTIVE_PIPELINES.lock() {
        let pipelines: &mut HashMap<String, PipelineHandle> = &mut *pipelines;
        pipelines.remove(&pipeline_id);
    }
    
    let succeeded = results.iter().filter(|r| r.success).count();
    let failed = results.iter().filter(|r| !r.success).count();
    
    // Emit completion
    let _ = app.emit("pipeline:complete", serde_json::json!({
        "pipelineId": pipeline_id,
        "total": total,
        "succeeded": succeeded,
        "failed": failed,
    }));
    
    Ok(BatchProcessResult {
        total,
        succeeded,
        failed,
        results,
    })
}

fn process_single_file(
    file_path: &str,
    operation: &str,
    _options: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let context = MediaContext::from_file(PathBuf::from(file_path));
    
    match operation {
        "metadata" => {
            let pipeline = Pipeline::new().add_node(ExtractMetadata::new(true));
            let result = pipeline.execute(context)
                .map_err(|e| format!("Metadata extraction failed: {}", e))?;
            
            let metadata = result.metadata.ok_or_else(|| "No metadata".to_string())?;
            serde_json::to_value(metadata).map_err(|e| e.to_string())
        }
        "motion" => {
            let pipeline = Pipeline::new().add_node(DetectMotion::new());
            let result = pipeline.execute(context)
                .map_err(|e| format!("Motion detection failed: {}", e))?;
            
            let analysis = result.analysis.ok_or_else(|| "No analysis".to_string())?;
            let motion: Vec<MotionEvent> = analysis.motion_events.into_iter().map(|e| MotionEvent {
                start_frame: e.start_frame,
                end_frame: e.end_frame,
                event_type: e.event_type,
            }).collect();
            serde_json::to_value(motion).map_err(|e| e.to_string())
        }
        _ => Err(format!("Unknown operation: {}", operation)),
    }
}

// ============================================================================
// COMBINED PIPELINES
// ============================================================================

/// Result of full analysis pipeline
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FullAnalysisResult {
    pub metadata: Option<MediaMetadata>,
    pub analysis: Option<AnalysisResult>,
}

/// Run a full analysis pipeline: metadata + motion detection
#[command]
pub async fn execute_full_analysis_pipeline(
    video_path: String,
    include_thumbnail: bool,
    detect_motion: bool,
    motion_threshold: Option<f64>,
) -> Result<FullAnalysisResult, String> {
    std::thread::spawn(move || {
        let context = MediaContext::from_file(PathBuf::from(&video_path));
        
        let mut pipeline = Pipeline::new()
            .add_node(ExtractMetadata::new(include_thumbnail));
        
        if detect_motion {
            let mut motion_step = DetectMotion::new();
            if let Some(threshold) = motion_threshold {
                motion_step = motion_step.threshold(threshold);
            }
            pipeline = pipeline.add_node(motion_step);
        }
        
        let result = pipeline.execute(context)
            .map_err(|e| format!("Full analysis pipeline failed: {}", e))?;
        
        // Convert analysis to serializable type
        let analysis = result.analysis.map(|a| AnalysisResult {
            motion_events: a.motion_events.into_iter().map(|e| MotionEvent {
                start_frame: e.start_frame,
                end_frame: e.end_frame,
                event_type: e.event_type,
            }).collect(),
            similarity_groups: a.similarity_groups.into_iter().map(|g| SimilarityGroup {
                group_name: g.group_name,
                members: g.members,
            }).collect(),
            image_comparison: a.image_comparison.map(|c| ImageComparisonResult {
                image1: c.image1,
                image2: c.image2,
                similarity_score: c.similarity_score,
                is_duplicate: c.is_duplicate,
            }),
        });
        
        Ok(FullAnalysisResult {
            metadata: result.metadata,
            analysis,
        })
    })
    .join()
    .map_err(|_| "Thread panicked".to_string())?
}
