//! Whisper.cpp transcription engine: model management (HuggingFace download
//! with progress events) and audio-file transcription with live segment
//! events. Cancellation works through a process-wide flag checked by the
//! download loop, the decode loop, and whisper's abort callback.

use serde::Serialize;
use std::fs::{self, File};
use std::io::{BufWriter, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};

use symphonia::core::audio::{AudioBufferRef, SampleBuffer};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::{get_codecs, get_probe};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const TARGET_SAMPLE_RATE: u32 = 16_000;

/// One ASR operation (download or transcription) runs at a time; the UI
/// enforces that, this flag lets `cancel_asr` interrupt whichever is active.
static CANCELLED: AtomicBool = AtomicBool::new(false);

pub fn request_cancel() {
    CANCELLED.store(true, Ordering::SeqCst);
}

fn reset_cancel() {
    CANCELLED.store(false, Ordering::SeqCst);
}

fn is_cancelled() -> bool {
    CANCELLED.load(Ordering::SeqCst)
}

// ---------------------------------------------------------------------------
// Model registry
// ---------------------------------------------------------------------------

struct ModelSpec {
    name: &'static str,
    file: &'static str,
    url: &'static str,
    approx_mb: u32,
}

const MODELS: &[ModelSpec] = &[
    ModelSpec { name: "tiny", file: "ggml-tiny.bin", url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin", approx_mb: 78 },
    ModelSpec { name: "tiny.en", file: "ggml-tiny.en.bin", url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin", approx_mb: 78 },
    ModelSpec { name: "base", file: "ggml-base.bin", url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin", approx_mb: 148 },
    ModelSpec { name: "base.en", file: "ggml-base.en.bin", url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin", approx_mb: 148 },
    ModelSpec { name: "small", file: "ggml-small.bin", url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin", approx_mb: 488 },
    ModelSpec { name: "small.en", file: "ggml-small.en.bin", url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin", approx_mb: 488 },
    ModelSpec { name: "medium", file: "ggml-medium.bin", url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin", approx_mb: 1530 },
    ModelSpec { name: "medium.en", file: "ggml-medium.en.bin", url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin", approx_mb: 1530 },
    ModelSpec { name: "large-v3", file: "ggml-large-v3.bin", url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3.bin", approx_mb: 3100 },
    ModelSpec { name: "large-v3-turbo", file: "ggml-large-v3-turbo.bin", url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin", approx_mb: 1620 },
];

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsrModelInfo {
    pub name: String,
    pub file: String,
    pub approx_mb: u32,
    pub downloaded: bool,
    pub size_bytes: Option<u64>,
}

fn resolve_model(model: &str) -> Result<&'static ModelSpec, String> {
    let normalized = model
        .split('/')
        .next_back()
        .unwrap_or(model)
        .trim()
        .to_lowercase();
    MODELS
        .iter()
        .find(|spec| spec.name == normalized)
        .ok_or_else(|| format!("Unsupported model: {}", model))
}

fn models_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(base.join("whisper").join("models"))
}

fn model_info(spec: &ModelSpec, dir: &Path) -> AsrModelInfo {
    let path = dir.join(spec.file);
    let size_bytes = fs::metadata(&path).ok().map(|m| m.len());
    AsrModelInfo {
        name: spec.name.to_string(),
        file: spec.file.to_string(),
        approx_mb: spec.approx_mb,
        downloaded: size_bytes.is_some(),
        size_bytes,
    }
}

pub fn list_models(app: &AppHandle) -> Result<Vec<AsrModelInfo>, String> {
    let dir = models_dir(app)?;
    Ok(MODELS.iter().map(|spec| model_info(spec, &dir)).collect())
}

// ---------------------------------------------------------------------------
// Model download
// ---------------------------------------------------------------------------

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AsrModelProgressEvent {
    model: String,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    percent: Option<i32>,
    done: bool,
}

pub async fn download_model(app: AppHandle, model: String) -> Result<AsrModelInfo, String> {
    reset_cancel();
    let spec = resolve_model(&model)?;
    let dir = models_dir(&app)?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let model_path = dir.join(spec.file);
    if model_path.exists() {
        return Ok(model_info(spec, &dir));
    }

    let download_path = model_path.with_extension("download");
    let app_for_task = app.clone();
    let download_path_for_task = download_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        download_file_with_progress(&app_for_task, spec, &download_path_for_task)
    })
    .await
    .map_err(|e| e.to_string())??;

    fs::rename(&download_path, &model_path).map_err(|e| e.to_string())?;

    let _ = app.emit(
        "asr:model-progress",
        AsrModelProgressEvent {
            model: spec.name.to_string(),
            downloaded_bytes: fs::metadata(&model_path).map(|m| m.len()).unwrap_or(0),
            total_bytes: None,
            percent: Some(100),
            done: true,
        },
    );

    Ok(model_info(spec, &dir))
}

fn download_file_with_progress(
    app: &AppHandle,
    spec: &ModelSpec,
    path: &Path,
) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(30))
        // model files run into the GB range — no overall request timeout,
        // cancellation covers stalled downloads
        .timeout(None::<std::time::Duration>)
        .build()
        .map_err(|e| e.to_string())?;

    let mut response = client.get(spec.url).send().map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("Failed to download model: {}", response.status()));
    }

    let total_bytes = response.content_length();
    let mut file = BufWriter::new(File::create(path).map_err(|e| e.to_string())?);
    let mut buffer = [0u8; 256 * 1024];
    let mut downloaded: u64 = 0;
    let mut last_percent: i32 = -1;
    let mut last_emit_bytes: u64 = 0;

    loop {
        if is_cancelled() {
            drop(file);
            let _ = fs::remove_file(path);
            return Err("Download cancelled".to_string());
        }

        let n = response.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
        downloaded += n as u64;

        // Throttle events: every whole percent, or every 16 MB when the
        // server didn't send a content length.
        let percent = total_bytes.map(|total| ((downloaded as f64 / total as f64) * 100.0) as i32);
        let should_emit = match percent {
            Some(p) => p != last_percent,
            None => downloaded - last_emit_bytes >= 16 * 1024 * 1024,
        };
        if should_emit {
            last_percent = percent.unwrap_or(-1);
            last_emit_bytes = downloaded;
            let _ = app.emit(
                "asr:model-progress",
                AsrModelProgressEvent {
                    model: spec.name.to_string(),
                    downloaded_bytes: downloaded,
                    total_bytes,
                    percent,
                    done: false,
                },
            );
        }
    }

    file.flush().map_err(|e| e.to_string())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Transcription
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionSegment {
    pub start_seconds: f64,
    pub end_seconds: f64,
    pub text: String,
}

#[derive(Serialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub srt: String,
    pub json: serde_json::Value,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AsrProgressEvent {
    kind: String,
    progress: Option<i32>,
    message: Option<String>,
    segment_index: Option<i32>,
    start_seconds: Option<f64>,
    end_seconds: Option<f64>,
    text: Option<String>,
}

pub async fn transcribe_file(
    app: AppHandle,
    audio_path: String,
    model: String,
    language: Option<String>,
) -> Result<TranscriptionResult, String> {
    reset_cancel();
    let spec = resolve_model(&model)?;
    let model_path = models_dir(&app)?.join(spec.file);
    if !model_path.exists() {
        return Err(format!("Model '{}' is not downloaded yet", spec.name));
    }
    let model_path_string = model_path
        .to_str()
        .ok_or_else(|| "Invalid model path".to_string())?
        .to_string();

    let app_for_task = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let bytes =
            fs::read(&audio_path).map_err(|e| format!("Failed to read audio file: {}", e))?;
        let samples = decode_audio_bytes(bytes, &audio_path)?;
        if is_cancelled() {
            return Err("Transcription cancelled".to_string());
        }
        run_transcription(&app_for_task, &model_path_string, samples, language.as_deref())
    })
    .await
    .map_err(|e| e.to_string())?
}

fn run_transcription(
    app: &AppHandle,
    model_path: &str,
    samples: Vec<f32>,
    language: Option<&str>,
) -> Result<TranscriptionResult, String> {
    let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
        .map_err(|e| e.to_string())?;
    let mut state = ctx.create_state().map_err(|e| e.to_string())?;

    let mut params = FullParams::new(SamplingStrategy::BeamSearch {
        beam_size: 5,
        patience: -1.0,
    });
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(4);
    params.set_n_threads(n_threads);
    params.set_print_progress(false);
    params.set_print_realtime(false);

    params.set_abort_callback_safe(is_cancelled);

    let app_for_progress = app.clone();
    params.set_progress_callback_safe(move |percent| {
        let payload = AsrProgressEvent {
            kind: "progress".to_string(),
            progress: Some(percent),
            message: Some(format!("Progress {}%", percent)),
            segment_index: None,
            start_seconds: None,
            end_seconds: None,
            text: None,
        };
        let _ = app_for_progress.emit("asr:progress", payload);
    });

    let app_for_segments = app.clone();
    params.set_segment_callback_safe_lossy(move |data: whisper_rs::SegmentCallbackData| {
        let payload = AsrProgressEvent {
            kind: "segment".to_string(),
            progress: None,
            message: Some(format!("Segment {}", data.segment)),
            segment_index: Some(data.segment),
            start_seconds: Some(data.start_timestamp as f64 / 100.0),
            end_seconds: Some(data.end_timestamp as f64 / 100.0),
            text: Some(data.text),
        };
        let _ = app_for_segments.emit("asr:progress", payload);
    });

    match language {
        Some(lang) if !lang.is_empty() && lang != "auto" => params.set_language(Some(lang)),
        _ => params.set_detect_language(true),
    }

    state.full(params, &samples).map_err(|e| {
        if is_cancelled() {
            "Transcription cancelled".to_string()
        } else {
            e.to_string()
        }
    })?;

    let mut segments = Vec::new();
    let mut full_text = String::new();
    for segment in state.as_iter() {
        let text = segment.to_str_lossy().map_err(|e| e.to_string())?;
        let trimmed = text.trim().to_string();
        let start_seconds = segment.start_timestamp() as f64 / 100.0;
        let end_seconds = segment.end_timestamp() as f64 / 100.0;
        if !trimmed.is_empty() {
            if !full_text.is_empty() {
                full_text.push(' ');
            }
            full_text.push_str(&trimmed);
        }
        segments.push(TranscriptionSegment {
            start_seconds,
            end_seconds,
            text: trimmed,
        });
    }

    let srt = build_srt(&segments);
    let json = serde_json::json!({
        "text": full_text,
        "segments": segments,
    });

    let _ = app.emit(
        "asr:progress",
        AsrProgressEvent {
            kind: "complete".to_string(),
            progress: Some(100),
            message: Some("Complete".to_string()),
            segment_index: None,
            start_seconds: None,
            end_seconds: None,
            text: None,
        },
    );

    Ok(TranscriptionResult {
        text: full_text,
        srt,
        json,
    })
}

fn build_srt(segments: &[TranscriptionSegment]) -> String {
    let mut output = String::new();
    for (idx, segment) in segments.iter().enumerate() {
        let start = format_srt_timestamp(segment.start_seconds);
        let end = format_srt_timestamp(segment.end_seconds);
        output.push_str(&(idx + 1).to_string());
        output.push('\n');
        output.push_str(&start);
        output.push_str(" --> ");
        output.push_str(&end);
        output.push('\n');
        output.push_str(&segment.text);
        output.push_str("\n\n");
    }
    output
}

fn format_srt_timestamp(seconds: f64) -> String {
    let total_ms = (seconds * 1000.0).round() as u64;
    let ms = total_ms % 1000;
    let total_seconds = total_ms / 1000;
    let sec = total_seconds % 60;
    let total_minutes = total_seconds / 60;
    let min = total_minutes % 60;
    let hr = total_minutes / 60;
    format!("{:02}:{:02}:{:02},{:03}", hr, min, sec, ms)
}

// ---------------------------------------------------------------------------
// Audio decoding (symphonia → 16 kHz mono f32)
// ---------------------------------------------------------------------------

fn decode_audio_bytes(bytes: Vec<u8>, path: &str) -> Result<Vec<f32>, String> {
    let cursor = Cursor::new(bytes);
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = Path::new(path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let probed = get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| e.to_string())?;

    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or_else(|| "No supported audio tracks found".to_string())?;
    let track_id = track.id;
    let input_rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| "Missing sample rate".to_string())?;
    let mut decoder = get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;

    let mut sample_buf: Option<SampleBuffer<f32>> = None;
    let mut mono_samples: Vec<f32> = Vec::new();

    loop {
        if is_cancelled() {
            return Err("Transcription cancelled".to_string());
        }

        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::ResetRequired) => return Err("Decoder reset required".to_string()),
            Err(SymphoniaError::IoError(_)) => break,
            Err(err) => return Err(err.to_string()),
        };

        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::IoError(_)) => break,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(err) => return Err(err.to_string()),
        };

        match decoded {
            AudioBufferRef::F32(buf) => {
                if sample_buf.is_none() {
                    sample_buf = Some(SampleBuffer::new(buf.capacity() as u64, *buf.spec()));
                }
                if let Some(sample_buf) = sample_buf.as_mut() {
                    let channel_count = buf.spec().channels.count();
                    sample_buf.copy_interleaved_ref(AudioBufferRef::F32(buf));
                    append_mono_samples(sample_buf.samples(), channel_count, &mut mono_samples);
                }
            }
            _ => {
                let mut sample_buf_ref =
                    SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
                let channel_count = decoded.spec().channels.count();
                sample_buf_ref.copy_interleaved_ref(decoded.clone());
                append_mono_samples(sample_buf_ref.samples(), channel_count, &mut mono_samples);
            }
        }
    }

    if mono_samples.is_empty() {
        return Err("No audio samples decoded — the file may use an unsupported codec".to_string());
    }

    Ok(resample_linear(&mono_samples, input_rate, TARGET_SAMPLE_RATE))
}

fn append_mono_samples(interleaved: &[f32], channels: usize, output: &mut Vec<f32>) {
    if channels == 0 {
        return;
    }
    if channels == 1 {
        output.extend_from_slice(interleaved);
        return;
    }
    for frame in interleaved.chunks(channels) {
        let sum: f32 = frame.iter().sum();
        output.push(sum / channels as f32);
    }
}

fn resample_linear(input: &[f32], input_rate: u32, output_rate: u32) -> Vec<f32> {
    if input.is_empty() || input_rate == output_rate {
        return input.to_vec();
    }
    let ratio = input_rate as f64 / output_rate as f64;
    let output_len = ((input.len() as f64) / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);
    for i in 0..output_len {
        let pos = i as f64 * ratio;
        let idx = pos.floor() as usize;
        let frac = (pos - idx as f64) as f32;
        let s0 = input.get(idx).copied().unwrap_or(0.0);
        let s1 = input.get(idx + 1).copied().unwrap_or(s0);
        output.push(s0 + (s1 - s0) * frac);
    }
    output
}
