//! Benchmark Commands
//!
//! This module provides Tauri commands for running performance benchmarks
//! on video processing operations using the media_core benchmark infrastructure.

use media_core::benchmark::{
    BenchmarkContext, BenchmarkFrameExtraction, BenchmarkMetadataExtraction,
    BenchmarkPipelineResult,
};
use media_core::pipeline::{MediaContext, Pipeline};

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::command;

// ============================================================================
// CONFIG TYPES
// ============================================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkConfig {
    pub video_path: String,
    pub operations: Vec<String>,
    pub runs: Option<usize>,
}

// ============================================================================
// RESPONSE TYPES
// ============================================================================

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkResultItem {
    pub name: String,
    pub average_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub std_dev_ms: f64,
    pub runs: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkResponse {
    pub results: Vec<BenchmarkResultItem>,
    pub fastest: String,
    pub slowest: String,
    pub total_ms: f64,
}

// ============================================================================
// COMMANDS
// ============================================================================

/// Run benchmarks on selected operations
#[command]
pub async fn run_benchmark(config: BenchmarkConfig) -> Result<BenchmarkResponse, String> {
    let video_path = config.video_path.clone();
    let runs = config.runs.unwrap_or(3).max(1).min(20);

    // Run in a separate thread since benchmarks are CPU-intensive
    let result = std::thread::spawn(move || -> Result<BenchmarkResponse, String> {
        let context = MediaContext::from_file(PathBuf::from(&video_path));

        // Build pipeline with requested benchmark steps (builder pattern takes ownership)
        let mut pipeline: Pipeline<MediaContext> = Pipeline::new();
        for op in &config.operations {
            pipeline = match op.as_str() {
                "context_init" => pipeline.add_boxed_node(Box::new(
                    BenchmarkContext::new("Context Init").runs(runs),
                )),
                "metadata" => pipeline.add_boxed_node(Box::new(
                    BenchmarkMetadataExtraction::new("Metadata Extraction").runs(runs),
                )),
                "frame_extraction" => pipeline.add_boxed_node(Box::new(
                    BenchmarkFrameExtraction::new("Frame Extraction")
                        .runs(runs)
                        .frame_count(10),
                )),
                _ => {
                    return Err(format!("Unknown benchmark operation: {}", op));
                }
            };
        }

        // Execute the pipeline (takes ownership and returns context)
        let context = pipeline
            .execute(context)
            .map_err(|e| format!("Benchmark pipeline failed: {:?}", e))?;

        // Extract results
        let bench_result = context
            .benchmark_result
            .unwrap_or_else(BenchmarkPipelineResult::new);

        let results: Vec<BenchmarkResultItem> = bench_result
            .results
            .iter()
            .map(|r| BenchmarkResultItem {
                name: r.name.clone(),
                average_ms: r.average.as_secs_f64() * 1000.0,
                min_ms: r.min.as_secs_f64() * 1000.0,
                max_ms: r.max.as_secs_f64() * 1000.0,
                std_dev_ms: r.std_dev.as_secs_f64() * 1000.0,
                runs: r.runs,
            })
            .collect();

        let fastest = results
            .iter()
            .min_by(|a, b| a.average_ms.partial_cmp(&b.average_ms).unwrap())
            .map(|r| r.name.clone())
            .unwrap_or_default();

        let slowest = results
            .iter()
            .max_by(|a, b| a.average_ms.partial_cmp(&b.average_ms).unwrap())
            .map(|r| r.name.clone())
            .unwrap_or_default();

        let total_ms = bench_result.total_time.as_secs_f64() * 1000.0;

        Ok(BenchmarkResponse {
            results,
            fastest,
            slowest,
            total_ms,
        })
    })
    .join()
    .map_err(|_| "Benchmark thread panicked".to_string())??;

    Ok(result)
}

/// Run a full benchmark suite (all operations)
#[command]
pub async fn run_benchmark_suite(
    video_path: String,
    runs: Option<usize>,
) -> Result<BenchmarkResponse, String> {
    let config = BenchmarkConfig {
        video_path,
        operations: vec![
            "context_init".to_string(),
            "metadata".to_string(),
            "frame_extraction".to_string(),
        ],
        runs,
    };
    run_benchmark(config).await
}
