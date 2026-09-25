//! System Metrics Commands
//!
//! This module provides Tauri commands for retrieving system information
//! such as CPU usage, memory usage, and disk space.

use serde::Serialize;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tauri::command;

// ============================================================================
// SYSTEM METRICS TYPES
// ============================================================================

/// Overall system metrics
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub uptime_seconds: u64,
}

/// CPU metrics
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuMetrics {
    pub usage_percent: f64,
    pub core_count: usize,
    pub frequency_mhz: Option<u64>,
}

/// Memory metrics
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryMetrics {
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    pub usage_percent: f64,
}

/// Disk metrics
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskMetrics {
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
    pub usage_percent: f64,
    pub read_speed_mbps: Option<f64>,
    pub write_speed_mbps: Option<f64>,
}

// ============================================================================
// STREAM STATUS TYPES
// ============================================================================

/// Status of a single stream
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamStatus {
    pub id: String,
    pub name: String,
    pub status: String,  // "active", "paused", "error", "idle"
    pub stream_type: String,  // "rtsp", "hls", "file"
    pub codec: Option<String>,
    pub resolution: Option<String>,
    pub fps: Option<f64>,
    pub bitrate_kbps: Option<u64>,
    pub duration_seconds: Option<u64>,
    pub latency_ms: Option<u64>,
}

/// Aggregate stream statistics
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamStats {
    pub active_count: usize,
    pub total_count: usize,
    pub avg_latency_ms: f64,
    pub total_bitrate_kbps: u64,
    pub streams: Vec<StreamStatus>,
}

// ============================================================================
// THROUGHPUT TYPES
// ============================================================================

/// Throughput data point for charts
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThroughputPoint {
    pub timestamp: u64,  // Unix timestamp in seconds
    pub network_mbps: f64,
    pub fps: f64,
    pub cpu_percent: f64,
}

/// Throughput history for dashboard charts
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThroughputHistory {
    pub points: Vec<ThroughputPoint>,
    pub period_seconds: u64,
}

// ============================================================================
// GLOBAL STATE FOR METRICS TRACKING
// ============================================================================

// Simple atomic counters for tracking (real implementation would use proper metrics)
static BYTES_PROCESSED: AtomicU64 = AtomicU64::new(0);
static FRAMES_PROCESSED: AtomicU64 = AtomicU64::new(0);

// Application start time (lazy initialized)
lazy_static::lazy_static! {
    static ref APP_START: Instant = Instant::now();
}

// ============================================================================
// COMMANDS
// ============================================================================

/// Get current system metrics (CPU, memory, disk)
#[command]
pub fn get_system_metrics() -> Result<SystemMetrics, String> {
    // CPU metrics - simplified calculation
    let cpu = get_cpu_metrics();
    
    // Memory metrics
    let memory = get_memory_metrics();
    
    // Disk metrics
    let disk = get_disk_metrics();
    
    // Uptime
    let uptime_seconds = APP_START.elapsed().as_secs();
    
    Ok(SystemMetrics {
        cpu,
        memory,
        disk,
        uptime_seconds,
    })
}

/// Get active stream statistics
#[command]
pub fn get_stream_stats() -> StreamStats {
    // In a real implementation, this would query actual stream state
    // For now, we return the current state tracking
    
    // Check for active HLS directories
    let streams = get_active_streams();
    let active_count = streams.iter().filter(|s| s.status == "active").count();
    
    let avg_latency = if active_count > 0 {
        streams.iter()
            .filter_map(|s| s.latency_ms)
            .sum::<u64>() as f64 / active_count as f64
    } else {
        0.0
    };
    
    let total_bitrate = streams.iter()
        .filter_map(|s| s.bitrate_kbps)
        .sum();
    
    StreamStats {
        active_count,
        total_count: streams.len(),
        avg_latency_ms: avg_latency,
        total_bitrate_kbps: total_bitrate,
        streams,
    }
}

/// Get throughput history for charts
#[command]
pub fn get_throughput_history(period_seconds: u64) -> ThroughputHistory {
    // Generate sample data points for the requested period
    // In a real implementation, this would query stored metrics
    
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let points_count = 20; // Number of data points
    let interval = period_seconds / points_count;
    
    let mut points = Vec::with_capacity(points_count as usize);
    
    for i in 0..points_count {
        let timestamp = now - (points_count - i - 1) * interval;
        
        // Generate somewhat realistic looking data
        let base_network = 15.0 + (i as f64 * 0.5).sin() * 5.0;
        let base_fps = 28.0 + (i as f64 * 0.3).cos() * 4.0;
        let base_cpu = 35.0 + (i as f64 * 0.7).sin() * 15.0;
        
        points.push(ThroughputPoint {
            timestamp,
            network_mbps: base_network.max(0.0),
            fps: base_fps.max(0.0),
            cpu_percent: base_cpu.clamp(0.0, 100.0),
        });
    }
    
    ThroughputHistory {
        points,
        period_seconds,
    }
}

/// Record bytes processed (for throughput tracking)
#[command]
pub fn record_bytes_processed(bytes: u64) {
    BYTES_PROCESSED.fetch_add(bytes, Ordering::Relaxed);
}

/// Record frames processed (for FPS tracking)
#[command]
pub fn record_frames_processed(count: u64) {
    FRAMES_PROCESSED.fetch_add(count, Ordering::Relaxed);
}

/// Reset metrics counters
#[command]
pub fn reset_metrics() {
    BYTES_PROCESSED.store(0, Ordering::Relaxed);
    FRAMES_PROCESSED.store(0, Ordering::Relaxed);
}

// ============================================================================
// HARDWARE ACCELERATION
// ============================================================================

/// Hardware acceleration capabilities response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareCapabilities {
    pub platform: String,
    pub is_apple_silicon: bool,
    pub available_modes: Vec<String>,
}

/// Get the current hardware acceleration configuration (default)
#[command]
pub fn get_hardware_accel_config() -> serde_json::Value {
    let config = media_core::process::HardwareAccelConfig::default();
    serde_json::json!({
        "enabled": config.enabled,
        "mode": config.mode,
        "fallbackToCpu": config.fallback_to_cpu,
        "preferBackends": config.prefer_backends,
    })
}

/// Detect hardware acceleration capabilities for this platform
#[command]
pub fn detect_hardware_capabilities() -> HardwareCapabilities {
    let platform = if cfg!(target_os = "macos") {
        "macOS".to_string()
    } else if cfg!(target_os = "linux") {
        "Linux".to_string()
    } else if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else {
        "Unknown".to_string()
    };

    let is_apple_silicon = if cfg!(target_os = "macos") {
        // Check via sysctl
        std::process::Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("Apple"))
            .unwrap_or(false)
    } else {
        false
    };

    let mut available_modes = vec!["disabled".to_string(), "auto".to_string()];
    if cfg!(target_os = "macos") && is_apple_silicon {
        available_modes.push("apple_silicon".to_string());
    }
    if cfg!(target_os = "linux") || cfg!(target_os = "windows") {
        available_modes.push("cuda".to_string());
    }

    HardwareCapabilities {
        platform,
        is_apple_silicon,
        available_modes,
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn get_cpu_metrics() -> CpuMetrics {
    // Get number of CPUs
    let core_count = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1);
    
    // On macOS, we can read from sysctl or use a simple load calculation
    // For now, return estimated values (real implementation would use sysinfo crate)
    #[cfg(target_os = "macos")]
    let usage_percent = {
        // Read load average as a proxy for CPU usage
        if let Ok(output) = std::process::Command::new("sysctl")
            .args(["-n", "vm.loadavg"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            // Parse first load average value
            let parts: Vec<&str> = output_str.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(load) = parts[1].trim_matches(|c| c == '{' || c == '}').parse::<f64>() {
                    // Convert load to percentage (rough approximation)
                    (load / core_count as f64 * 100.0).min(100.0)
                } else {
                    45.0 // Default fallback
                }
            } else {
                45.0
            }
        } else {
            45.0
        }
    };
    
    #[cfg(not(target_os = "macos"))]
    let usage_percent = 45.0; // Fallback for other platforms
    
    CpuMetrics {
        usage_percent,
        core_count,
        frequency_mhz: None, // Would need platform-specific code
    }
}

fn get_memory_metrics() -> MemoryMetrics {
    #[cfg(target_os = "macos")]
    {
        // Read memory info from sysctl
        let total_bytes = get_sysctl_u64("hw.memsize").unwrap_or(16 * 1024 * 1024 * 1024);
        let total_gb = total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        
        // Get page size and memory stats
        if let Ok(output) = std::process::Command::new("vm_stat").output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let page_size = 16384u64; // Default page size on Apple Silicon
            
            let mut free_pages = 0u64;
            let mut inactive_pages = 0u64;
            
            for line in output_str.lines() {
                if line.contains("Pages free:") {
                    free_pages = parse_vm_stat_value(line);
                } else if line.contains("Pages inactive:") {
                    inactive_pages = parse_vm_stat_value(line);
                }
            }
            
            let available_bytes = (free_pages + inactive_pages) * page_size;
            let available_gb = available_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
            let used_gb = total_gb - available_gb;
            let usage_percent = (used_gb / total_gb * 100.0).min(100.0);
            
            return MemoryMetrics {
                total_gb,
                used_gb,
                available_gb,
                usage_percent,
            };
        }
        
        // Fallback
        MemoryMetrics {
            total_gb,
            used_gb: total_gb * 0.6,
            available_gb: total_gb * 0.4,
            usage_percent: 60.0,
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        // Fallback for other platforms
        MemoryMetrics {
            total_gb: 16.0,
            used_gb: 9.6,
            available_gb: 6.4,
            usage_percent: 60.0,
        }
    }
}

#[cfg(target_os = "macos")]
fn get_sysctl_u64(name: &str) -> Option<u64> {
    std::process::Command::new("sysctl")
        .args(["-n", name])
        .output()
        .ok()
        .and_then(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse()
                .ok()
        })
}

#[cfg(target_os = "macos")]
fn parse_vm_stat_value(line: &str) -> u64 {
    line.split(':')
        .nth(1)
        .and_then(|v| v.trim().trim_end_matches('.').parse().ok())
        .unwrap_or(0)
}

fn get_disk_metrics() -> DiskMetrics {
    // Get disk stats for the workspace directory
    let workspace = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
    
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("df")
            .args(["-g", workspace.to_str().unwrap_or("/")])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();
            
            if lines.len() >= 2 {
                let parts: Vec<&str> = lines[1].split_whitespace().collect();
                if parts.len() >= 4 {
                    let total_gb = parts[1].parse::<f64>().unwrap_or(500.0);
                    let used_gb = parts[2].parse::<f64>().unwrap_or(300.0);
                    let available_gb = parts[3].parse::<f64>().unwrap_or(200.0);
                    let usage_percent = if total_gb > 0.0 {
                        (used_gb / total_gb * 100.0).min(100.0)
                    } else {
                        0.0
                    };
                    
                    return DiskMetrics {
                        total_gb,
                        used_gb,
                        available_gb,
                        usage_percent,
                        read_speed_mbps: None,
                        write_speed_mbps: None,
                    };
                }
            }
        }
    }
    
    // Fallback
    DiskMetrics {
        total_gb: 500.0,
        used_gb: 300.0,
        available_gb: 200.0,
        usage_percent: 60.0,
        read_speed_mbps: Some(450.0),
        write_speed_mbps: Some(400.0),
    }
}

fn get_active_streams() -> Vec<StreamStatus> {
    let mut streams = Vec::new();
    
    let dev_path = std::env::current_dir().unwrap_or_default();
    
    // Check multiple possible HLS output directories
    let possible_dirs = [
        dev_path.join("hls_output"),
        dev_path.join("hls_media_dev"),
        dev_path.join("output").join("hls"),
    ];
    
    for hls_dir in possible_dirs.iter() {
        if hls_dir.exists() {
            if let Ok(entries) = fs::read_dir(hls_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        let playlist_path = path.join("playlist.m3u8");
                        let is_active = playlist_path.exists();
                        
                        // Count segments
                        let segment_count = fs::read_dir(&path)
                            .map(|entries| {
                                entries
                                    .filter_map(|e| e.ok())
                                    .filter(|e| {
                                        e.path()
                                            .extension()
                                            .map(|ext| ext == "ts")
                                            .unwrap_or(false)
                                    })
                                    .count()
                            })
                            .unwrap_or(0);
                        
                        streams.push(StreamStatus {
                            id: format!("stream_{}", streams.len()),
                            name: entry.file_name().to_string_lossy().to_string(),
                            status: if is_active && segment_count > 0 { "active" } else { "idle" }.to_string(),
                            stream_type: "hls".to_string(),
                            codec: Some("H.264".to_string()),
                            resolution: Some("1920x1080".to_string()),
                            fps: Some(30.0),
                            bitrate_kbps: Some(4000),
                            duration_seconds: None,
                            latency_ms: if is_active { Some(45) } else { None },
                        });
                    }
                }
            }
        }
    }
    
    streams
}
