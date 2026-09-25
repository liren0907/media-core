use media_core::{RTSPCapture, StreamConfig, SavingOption};
use serde_json::json;
use tauri::Emitter;
use std::thread;

#[tauri::command]
pub async fn process_rtsp_config(window: tauri::Window, config_path: &str) -> Result<String, String> {
    let window_clone = window.clone();
    let config_path = config_path.to_string();

    std::thread::spawn(move || {
        match std::fs::read_to_string(&config_path) {
            Ok(config_content) => {
                match serde_json::from_str::<StreamConfig>(&config_content) {
                    Ok(cfg) => {
                        match start_rtsp_capture(cfg) {
                            Ok(msg) => {
                                let _ = window_clone.emit("rtsp-status", json!({
                                    "status": "success",
                                    "message": msg
                                }));
                            }
                            Err(e) => {
                                let _ = window_clone.emit("rtsp-status", json!({
                                    "status": "error",
                                    "message": e
                                }));
                            }
                        }
                    }
                    Err(e) => {
                        let _ = window_clone.emit("rtsp-status", json!({
                            "status": "error",
                            "message": format!("Invalid configuration: {}", e)
                        }));
                    }
                }
            }
            Err(e) => {
                let _ = window_clone.emit("rtsp-status", json!({
                    "status": "error",
                    "message": format!("Failed to read config file: {}", e)
                }));
            }
        }
    });

    Ok("RTSP streams processing initiated".to_string())
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn validate_config(configPath: &str) -> Result<String, String> {
    match std::fs::read_to_string(configPath) {
        Ok(config_content) => {
            match serde_json::from_str::<StreamConfig>(&config_content) {
                Ok(_) => Ok("Configuration is valid".to_string()),
                Err(e) => Err(format!("Invalid configuration: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to read config file: {}", e)),
    }
}

#[tauri::command]
pub fn start_rtsp_capture(payload: StreamConfig) -> Result<String, String> {
    // Determine which URLs to process based on saving_option
    let (urls_to_process, show_preview_for_main) = match payload.saving_option {
        SavingOption::Single => (vec![payload.rtsp_url.clone()], payload.show_preview),
        SavingOption::List => (payload.rtsp_url_list.clone(), false),
        SavingOption::Both => {
            let mut urls = vec![payload.rtsp_url.clone()];
            urls.extend(payload.rtsp_url_list.clone());
            (urls, false)
        }
    };

    for url in urls_to_process {
        let output_dir = payload.output_directory.clone();
        let show_preview = if payload.rtsp_url == url {
            show_preview_for_main
        } else {
            false
        };
        let segment_duration = payload.saved_time_duration;
        let use_fps = payload.use_fps;
        let fps = payload.fps;
        let hls_config = Some(payload.hls.clone());

        thread::spawn(move || {
            match RTSPCapture::new(
                url.clone(),
                output_dir,
                show_preview,
                segment_duration,
                use_fps,
                fps,
                hls_config,
                false, // run_once: false for app usage
            ) {
                Ok(mut capture) => {
                    if let Err(e) = capture.process_stream() {
                        eprintln!("Error processing stream {}: {:?}", url, e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to create RTSP capture for {}: {:?}", url, e);
                }
            }
        });
    }

    Ok("RTSP capture started".to_string())
}

