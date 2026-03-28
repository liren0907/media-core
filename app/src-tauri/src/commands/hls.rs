use lazy_static::lazy_static;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tiny_http::{Response, Server};

lazy_static! {
    static ref HLS_SERVER: Arc<Mutex<Option<Arc<Server>>>> = Arc::new(Mutex::new(None));
}

fn start_http_server_if_needed(hls_root_path: PathBuf) -> Result<String, String> {
    let mut server_guard = HLS_SERVER.lock().unwrap();

    if server_guard.is_none() {
        let server_addr = "127.0.0.1:1521";
        let server = Server::http(server_addr).map_err(|e| format!("Failed to start HTTP server: {}", e))?;
        let server_arc = Arc::new(server);
        *server_guard = Some(server_arc.clone());

        let hls_path = Arc::new(hls_root_path);

        thread::spawn(move || {
            // Canonicalize the HLS root once for path traversal checks
            let root_canonical = match hls_path.canonicalize() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to canonicalize HLS root path: {}", e);
                    return;
                }
            };

            for request in server_arc.incoming_requests() {
                let cors_header = tiny_http::Header::from_bytes(
                    &b"Access-Control-Allow-Origin"[..],
                    &b"http://localhost:1420"[..],
                ).unwrap();

                let url = request.url();
                let file_path = hls_path.join(url.trim_start_matches('/'));

                // Path traversal protection: ensure resolved path is within HLS root
                let canonical = match file_path.canonicalize() {
                    Ok(p) => p,
                    Err(_) => {
                        let response = Response::from_string("404 Not Found")
                            .with_status_code(404)
                            .with_header(cors_header);
                        let _ = request.respond(response);
                        continue;
                    }
                };
                if !canonical.starts_with(&root_canonical) {
                    let response = Response::from_string("403 Forbidden")
                        .with_status_code(403)
                        .with_header(cors_header);
                    let _ = request.respond(response);
                    continue;
                }

                if let Ok(file) = fs::File::open(&canonical) {
                    let response = Response::from_file(file).with_header(cors_header);
                    let _ = request.respond(response);
                } else {
                    let response = Response::from_string("404 Not Found")
                        .with_status_code(404)
                        .with_header(cors_header);
                    let _ = request.respond(response);
                }
            }
        });
        
        println!("Started HLS HTTP server at {}", server_addr);
    }
    
    Ok("http://127.0.0.1:1521".to_string())
}

#[tauri::command]
#[allow(non_snake_case)]
pub fn get_hls_status(outputDir: &str) -> Result<String, String> {
    let hls_dir = Path::new(outputDir);
    let playlist_path = hls_dir.join("playlist.m3u8");
    
    if playlist_path.exists() {
        match std::fs::read_to_string(&playlist_path) {
            Ok(content) => {
                let segment_count = content.lines()
                    .filter(|line| line.ends_with(".ts"))
                    .count();
                
                Ok(json!({
                    "status": "active",
                    "playlist_exists": true,
                    "segment_count": segment_count,
                    "playlist_path": playlist_path.to_string_lossy()
                }).to_string())
            }
            Err(e) => Err(format!("Failed to read playlist: {}", e)),
        }
    } else {
        Ok(json!({
            "status": "inactive",
            "playlist_exists": false,
            "segment_count": 0,
            "playlist_path": playlist_path.to_string_lossy()
        }).to_string())
    }
}

#[tauri::command]
pub async fn start_hls_playback(url: String) -> Result<String, String> {
    // Use hls_output as the default directory (configurable via frontend settings)
    let dev_path = std::env::current_dir().map_err(|e| e.to_string())?;
    let hls_root_dir = dev_path.join("hls_output");
    let stream_dir = hls_root_dir.join("stream_0");

    // Clean up previous HLS files in the specific stream directory
    if stream_dir.exists() {
        fs::remove_dir_all(&stream_dir).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&stream_dir).map_err(|e| e.to_string())?;

    let hls_config = media_core::HLSConfig {
        enabled: true,
        output_directory: stream_dir.to_str().unwrap().to_string(),
        segment_duration: 2,
        playlist_size: 5,
    };

    let playlist_path = stream_dir.join("playlist.m3u8");

    // Spawn a background thread to run the HLS transcoding process.
    std::thread::spawn(move || {
        let mut capture = media_core::RTSPCapture::new(
            url,
            String::new(), // Not used for HLS-only playback
            false,
            0,
            false,
            0.0,
            Some(hls_config),
            false, // run_once
        )
        .expect("Failed to create RTSPCapture instance for HLS playback.");

        if let Err(e) = capture.process_stream() {
            eprintln!("Error processing HLS stream: {}", e);
        }
    });

    // Poll for a few seconds to wait for the playlist to be created by ffmpeg.
    let start_time = Instant::now();
    while !playlist_path.exists() {
        if start_time.elapsed() > Duration::from_secs(10) {
            return Err("HLS playlist generation timed out.".to_string());
        }
        std::thread::sleep(Duration::from_millis(200));
    }

    // Ensure the HTTP server is running and get its base URL.
    let server_base_url = start_http_server_if_needed(hls_root_dir)?;
    
    // The playlist URL is now relative to the server root.
    let playlist_url = format!("{}/stream_0/playlist.m3u8", server_base_url);

    Ok(playlist_url)
}

#[tauri::command]
pub fn start_direct_playback(url: String) -> Result<(), String> {
    std::thread::spawn(move || {
        let result = Command::new("ffplay")
            .args([
                "-i",
                &url,
                "-window_title",
                &format!("RTSP Stream: {}", url),
                "-exitonkeydown",
                "-exitonmousedown",
            ])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        if let Err(e) = result {
            eprintln!("Failed to start direct playback for {}: {}", url, e);
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn start_multiple_hls_playback(urls: Vec<String>) -> Result<Vec<String>, String> {
    let mut handles = vec![];
    let mut playlist_paths = vec![];

    // Use hls_output as the default directory (configurable via frontend settings)
    let dev_path = env::current_dir().map_err(|e| e.to_string())?;
    let base_hls_dir = dev_path.join("hls_output");

    for (index, url) in urls.into_iter().enumerate() {
        // Create a unique directory for each stream's HLS segments.
        let hls_output_dir = base_hls_dir.join(format!("stream_{}", index));
        
        // Clean up previous HLS files in the directory before starting a new stream.
        if hls_output_dir.exists() {
            fs::remove_dir_all(&hls_output_dir).map_err(|e| e.to_string())?;
        }
        fs::create_dir_all(&hls_output_dir).map_err(|e| e.to_string())?;

        let hls_config = media_core::HLSConfig {
            enabled: true,
            output_directory: hls_output_dir.to_str().unwrap().to_string(),
            segment_duration: 2,
            playlist_size: 5,
        };

        let playlist_path = hls_output_dir.join("playlist.m3u8");
        playlist_paths.push(playlist_path.to_str().unwrap().to_string());

        let handle = std::thread::spawn(move || {
        let mut capture = media_core::RTSPCapture::new(
            url,
            String::new(), // Not used
            false,
            0,
            false,
            0.0,
            Some(hls_config),
            false, // run_once
        )
            .expect("Failed to create RTSPCapture for multi-stream.");

            if let Err(e) = capture.process_stream() {
                eprintln!("Error processing HLS stream: {}", e);
            }
        });
        handles.push(handle);
    }

    // Wait for a short period to allow all ffmpeg processes to start and create playlists.
    // A more robust solution might involve polling each path individually.
    std::thread::sleep(Duration::from_secs(5));

    // Return the canonical, absolute paths.
    let canonical_paths = playlist_paths
        .iter()
        .map(|p| fs::canonicalize(p).unwrap().to_str().unwrap().to_string())
        .collect();

    Ok(canonical_paths)
}

