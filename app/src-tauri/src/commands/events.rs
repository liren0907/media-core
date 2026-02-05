//! Event System Commands
//!
//! This module provides Tauri commands and utilities for emitting events
//! to the frontend. Events enable real-time updates without polling.

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{command, AppHandle, Emitter};

// ============================================================================
// EVENT TYPES
// ============================================================================

/// Pipeline progress event payload
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineProgressEvent {
    pub pipeline_id: String,
    pub step_name: String,
    pub step_index: usize,
    pub total_steps: usize,
    pub progress_percent: f64,
    pub message: String,
    pub is_complete: bool,
    pub has_error: bool,
    pub error_message: Option<String>,
}

/// Stream status event payload
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamStatusEvent {
    pub stream_id: String,
    pub status: String, // "starting", "active", "paused", "error", "stopped"
    pub message: Option<String>,
    pub timestamp: u64,
}

/// System metrics event payload
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMetricsEvent {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub disk_percent: f64,
    pub active_streams: usize,
    pub timestamp: u64,
}

// ============================================================================
// EVENT NAMES (constants for consistency)
// ============================================================================

pub const EVENT_PIPELINE_PROGRESS: &str = "pipeline:progress";
pub const EVENT_PIPELINE_COMPLETE: &str = "pipeline:complete";
pub const EVENT_STREAM_STATUS: &str = "stream:status";
pub const EVENT_SYSTEM_METRICS: &str = "system:metrics";

// ============================================================================
// EVENT EMISSION HELPERS
// ============================================================================

/// Emit a pipeline progress event
pub fn emit_pipeline_progress(app: &AppHandle, event: PipelineProgressEvent) {
    let _ = app.emit(EVENT_PIPELINE_PROGRESS, &event);
}

/// Emit a stream status event
pub fn emit_stream_status(app: &AppHandle, event: StreamStatusEvent) {
    let _ = app.emit(EVENT_STREAM_STATUS, &event);
}

/// Emit a system metrics event
pub fn emit_system_metrics(app: &AppHandle, event: SystemMetricsEvent) {
    let _ = app.emit(EVENT_SYSTEM_METRICS, &event);
}

// ============================================================================
// BACKGROUND METRICS EMITTER
// ============================================================================

// Global flag to control metrics emission
static METRICS_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Start emitting system metrics at regular intervals
#[command]
pub async fn start_metrics_stream(app: AppHandle, interval_ms: u64) -> Result<(), String> {
    if METRICS_ACTIVE.load(Ordering::Relaxed) {
        return Ok(()); // Already running
    }
    
    METRICS_ACTIVE.store(true, Ordering::Relaxed);
    
    let interval = Duration::from_millis(interval_ms.max(500)); // Min 500ms
    
    std::thread::spawn(move || {
        while METRICS_ACTIVE.load(Ordering::Relaxed) {
            let metrics = collect_system_metrics();
            emit_system_metrics(&app, metrics);
            std::thread::sleep(interval);
        }
    });
    
    Ok(())
}

/// Stop emitting system metrics
#[command]
pub fn stop_metrics_stream() {
    METRICS_ACTIVE.store(false, Ordering::Relaxed);
}

/// Check if metrics streaming is active
#[command]
pub fn is_metrics_streaming() -> bool {
    METRICS_ACTIVE.load(Ordering::Relaxed)
}

fn collect_system_metrics() -> SystemMetricsEvent {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Get CPU usage (simplified)
    let cpu_percent = get_cpu_usage();
    
    // Get memory usage
    let memory_percent = get_memory_usage();
    
    // Get disk usage
    let disk_percent = get_disk_usage();
    
    // Count active streams
    let active_streams = count_active_streams();
    
    SystemMetricsEvent {
        cpu_percent,
        memory_percent,
        disk_percent,
        active_streams,
        timestamp,
    }
}

#[cfg(target_os = "macos")]
fn get_cpu_usage() -> f64 {
    let core_count = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1);
    
    if let Ok(output) = std::process::Command::new("sysctl")
        .args(["-n", "vm.loadavg"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = output_str.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(load) = parts[1].trim_matches(|c| c == '{' || c == '}').parse::<f64>() {
                return (load / core_count as f64 * 100.0).min(100.0);
            }
        }
    }
    45.0
}

#[cfg(not(target_os = "macos"))]
fn get_cpu_usage() -> f64 {
    45.0
}

#[cfg(target_os = "macos")]
fn get_memory_usage() -> f64 {
    if let Ok(output) = std::process::Command::new("vm_stat").output() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let page_size = 16384u64;
        
        let mut free_pages = 0u64;
        let mut inactive_pages = 0u64;
        let mut active_pages = 0u64;
        let mut wired_pages = 0u64;
        
        for line in output_str.lines() {
            if line.contains("Pages free:") {
                free_pages = parse_vm_stat_value(line);
            } else if line.contains("Pages inactive:") {
                inactive_pages = parse_vm_stat_value(line);
            } else if line.contains("Pages active:") {
                active_pages = parse_vm_stat_value(line);
            } else if line.contains("Pages wired down:") {
                wired_pages = parse_vm_stat_value(line);
            }
        }
        
        let total = free_pages + inactive_pages + active_pages + wired_pages;
        let used = active_pages + wired_pages;
        
        if total > 0 {
            return (used as f64 / total as f64 * 100.0).min(100.0);
        }
    }
    60.0
}

#[cfg(target_os = "macos")]
fn parse_vm_stat_value(line: &str) -> u64 {
    line.split(':')
        .nth(1)
        .and_then(|v| v.trim().trim_end_matches('.').parse().ok())
        .unwrap_or(0)
}

#[cfg(not(target_os = "macos"))]
fn get_memory_usage() -> f64 {
    60.0
}

fn get_disk_usage() -> f64 {
    #[cfg(target_os = "macos")]
    {
        let workspace = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
        if let Ok(output) = std::process::Command::new("df")
            .args(["-g", workspace.to_str().unwrap_or("/")])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();
            if lines.len() >= 2 {
                let parts: Vec<&str> = lines[1].split_whitespace().collect();
                if parts.len() >= 5 {
                    // The 5th column is usually percentage
                    if let Some(pct_str) = parts.get(4) {
                        if let Ok(pct) = pct_str.trim_end_matches('%').parse::<f64>() {
                            return pct;
                        }
                    }
                }
            }
        }
    }
    60.0
}

fn count_active_streams() -> usize {
    let dev_path = std::env::current_dir().unwrap_or_default();
    
    // Check multiple possible HLS output directories
    let possible_dirs = [
        dev_path.join("hls_output"),
        dev_path.join("hls_media_dev"),
        dev_path.join("output").join("hls"),
    ];
    
    let mut count = 0;
    for hls_dir in possible_dirs.iter() {
        if hls_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(hls_dir) {
                count += entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        let path = e.path();
                        path.is_dir() && path.join("playlist.m3u8").exists()
                    })
                    .count();
            }
        }
    }
    count
}

// ============================================================================
// PIPELINE PROGRESS TRACKING
// ============================================================================

/// Pipeline execution handle for progress tracking and cancellation
#[derive(Clone)]
pub struct PipelineHandle {
    pub id: String,
    pub cancelled: Arc<AtomicBool>,
}

impl PipelineHandle {
    pub fn new(id: String) -> Self {
        Self {
            id,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
    
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
    
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

// Store for active pipeline handles
lazy_static::lazy_static! {
    pub static ref ACTIVE_PIPELINES: std::sync::Mutex<std::collections::HashMap<String, PipelineHandle>> = 
        std::sync::Mutex::new(std::collections::HashMap::new());
}

/// Start tracking a pipeline execution
#[command]
pub fn start_pipeline_tracking(pipeline_id: String) -> String {
    let handle = PipelineHandle::new(pipeline_id.clone());
    if let Ok(mut pipelines) = ACTIVE_PIPELINES.lock() {
        pipelines.insert(pipeline_id.clone(), handle);
    }
    pipeline_id
}

/// Cancel a running pipeline
#[command]
pub fn cancel_pipeline(pipeline_id: String) -> bool {
    if let Ok(pipelines) = ACTIVE_PIPELINES.lock() {
        if let Some(handle) = pipelines.get(&pipeline_id) {
            handle.cancel();
            return true;
        }
    }
    false
}

/// Check if a pipeline is cancelled
#[command]
pub fn is_pipeline_cancelled(pipeline_id: String) -> bool {
    if let Ok(pipelines) = ACTIVE_PIPELINES.lock() {
        if let Some(handle) = pipelines.get(&pipeline_id) {
            return handle.is_cancelled();
        }
    }
    false
}

/// Remove a pipeline from tracking
#[command]
pub fn finish_pipeline_tracking(pipeline_id: String) {
    if let Ok(mut pipelines) = ACTIVE_PIPELINES.lock() {
        pipelines.remove(&pipeline_id);
    }
}

/// Get list of active pipeline IDs
#[command]
pub fn get_active_pipelines() -> Vec<String> {
    if let Ok(pipelines) = ACTIVE_PIPELINES.lock() {
        pipelines.keys().cloned().collect()
    } else {
        Vec::new()
    }
}

/// Emit progress for a specific pipeline step
#[command]
pub fn emit_progress(
    app: AppHandle,
    pipeline_id: String,
    step_name: String,
    step_index: usize,
    total_steps: usize,
    progress_percent: f64,
    message: String,
) {
    let event = PipelineProgressEvent {
        pipeline_id,
        step_name,
        step_index,
        total_steps,
        progress_percent,
        message,
        is_complete: false,
        has_error: false,
        error_message: None,
    };
    emit_pipeline_progress(&app, event);
}
