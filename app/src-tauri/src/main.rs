// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod handlers;

use annotation::VideoAnnotation;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::video::get_video_info,
            commands::video::read_video_file,
            commands::video::read_image_file,
            commands::video::open_file_dialog,
            commands::video::extract_all_frames_ffmpeg,
            commands::video::extract_frames_ffmpeg_interval,
            commands::annotation::read_annotation_file,
            commands::annotation::start_video_annotation,
            commands::rtsp::process_rtsp_config,
            commands::rtsp::validate_config,
            commands::rtsp::start_rtsp_capture,
            commands::hls::get_hls_status,
            commands::hls::start_hls_playback,
            commands::hls::start_direct_playback,
            commands::hls::start_multiple_hls_playback,
            // Analysis commands
            commands::analysis::analyze_similarity,
            commands::analysis::detect_motion,
            // Metadata commands
            commands::metadata::get_media_metadata,
            commands::metadata::check_media_support,
            // Camera commands
            commands::camera::get_available_cameras,
            commands::camera::check_camera_access,
            commands::camera::capture_camera_snapshot,
            // Pipeline executor commands
            commands::pipeline::execute_metadata_pipeline,
            commands::pipeline::execute_motion_pipeline,
            commands::pipeline::execute_similarity_pipeline,
            commands::pipeline::execute_compare_pipeline,
            commands::pipeline::execute_extract_frames_pipeline,
            commands::pipeline::execute_hls_pipeline,
            commands::pipeline::execute_extract_to_disk_pipeline,
            commands::pipeline::execute_camera_pipeline,
            commands::pipeline::execute_full_analysis_pipeline,
            commands::pipeline::execute_batch_pipeline,
            // System metrics commands
            commands::system::get_system_metrics,
            commands::system::get_stream_stats,
            commands::system::get_throughput_history,
            commands::system::record_bytes_processed,
            commands::system::record_frames_processed,
            commands::system::reset_metrics,
            // Event system commands
            commands::events::start_metrics_stream,
            commands::events::stop_metrics_stream,
            commands::events::is_metrics_streaming,
            commands::events::start_pipeline_tracking,
            commands::events::cancel_pipeline,
            commands::events::is_pipeline_cancelled,
            commands::events::finish_pipeline_tracking,
            commands::events::get_active_pipelines,
            commands::events::emit_progress,
        ])
        .on_window_event(|_app_handle, event| {
            if let tauri::WindowEvent::Destroyed = event {
                if let Err(e) = VideoAnnotation::cleanup_tmp_files() {
                    eprintln!("Failed to cleanup temporary files: {}", e);
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
