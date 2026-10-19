//! Writes every frame of every video in a folder as JPEGs, one folder per video.
//!
//! Decoding has to happen in order, so one thread reads each video from start to
//! end and never seeks: the frame number is simply how many frames came before.
//! Encoding JPEGs is the slow part and has no order, so a pool of writer threads
//! takes frames off a bounded queue. The file name carries the frame number, which
//! makes the result identical to a single-threaded run.
//!
//! Videos are done one at a time, each into `<name>.partial/` and renamed to
//! `<name>/` only once every frame is written. A rerun skips finished videos and
//! starts an interrupted one over.

use std::collections::HashSet;
use std::fs;
use std::io::{IsTerminal, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use opencv::core::{Mat, Vector};
use opencv::imgcodecs;
use opencv::prelude::*;
use opencv::videoio::{self, VideoCapture};

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Same as OpenCV's default; kept here so it is easy to find and change.
const JPEG_QUALITY: i32 = 95;

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mov", "mkv", "avi", "m4v", "webm"];

const USAGE: &str = "\
Usage: extract-frames [INPUT] [OUTPUT] [--jobs N] [--force]

  INPUT     a video file, or a folder of videos (default: examples/videos)
  OUTPUT    where the per-video folders go (default: examples/frames)
  --jobs N  JPEG writer threads (default: CPU count - 1)
  --force   redo videos that already have a folder";

struct Args {
    input: PathBuf,
    output: PathBuf,
    jobs: usize,
    force: bool,
}

/// A frame and its position in the video, counted from 0.
type Frame = (usize, Mat);

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(Some(args)) => args,
        Ok(None) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("error: {message}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };

    match run(&args) {
        Ok(0) => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Returns how many videos failed.
fn run(args: &Args) -> Result<usize, Error> {
    let videos = find_videos(&args.input)?;
    if videos.is_empty() {
        return Err(format!("no videos found in {}", args.input.display()).into());
    }

    // Two videos with the same name and different extensions would fight over one folder.
    let mut names = HashSet::new();
    for video in &videos {
        if !names.insert(name_of(video)) {
            return Err(format!("more than one video is named {}", name_of(video)).into());
        }
    }

    fs::create_dir_all(&args.output)?;
    println!(
        "{} video(s) -> {} ({} writer thread(s))",
        videos.len(),
        args.output.display(),
        args.jobs
    );

    let started = Instant::now();
    let (mut done, mut skipped, mut failed, mut frames) = (0, 0, Vec::new(), 0);

    for (i, video) in videos.iter().enumerate() {
        let name = name_of(video);
        let label = format!("[{}/{}] {name}", i + 1, videos.len());
        let final_dir = args.output.join(&name);

        if final_dir.exists() && !args.force {
            println!("{label}: already done, skipped");
            skipped += 1;
            continue;
        }

        // Built by hand: the names contain dots, which `with_extension` would eat.
        let partial_dir = args.output.join(format!("{name}.partial"));
        match extract_video(video, &partial_dir, args.jobs, &label)
            .and_then(|stats| finish(&partial_dir, &final_dir).map(|()| stats))
        {
            Ok(stats) => {
                let secs = stats.elapsed.as_secs_f64();
                let mut line = format!(
                    "{}{label}: {} frames in {} ({:.0} frames/s)",
                    clear_line(),
                    stats.frames,
                    format_duration(stats.elapsed),
                    stats.frames as f64 / secs.max(f64::EPSILON)
                );
                if stats.frames != stats.expected {
                    line += &format!(", the container says {}", stats.expected);
                }
                println!("{line}");
                done += 1;
                frames += stats.frames;
            }
            Err(e) => {
                println!("{}{label}: failed: {e}", clear_line());
                failed.push(name);
            }
        }
    }

    println!(
        "Done: {done} extracted, {skipped} skipped, {} failed; {frames} frames in {}",
        failed.len(),
        format_duration(started.elapsed())
    );
    for name in &failed {
        println!("  failed: {name} (rerun to try it again)");
    }
    Ok(failed.len())
}

struct Stats {
    frames: usize,
    /// The frame count the container reports. Only for progress and a sanity note:
    /// the frames actually decoded are what gets written.
    expected: usize,
    elapsed: Duration,
}

fn extract_video(video: &Path, dir: &Path, jobs: usize, label: &str) -> Result<Stats, Error> {
    // Opened first, so a file that is not a video leaves no folder behind.
    let path = video.to_str().ok_or("the video path is not valid UTF-8")?;
    let mut capture = VideoCapture::from_file(path, videoio::CAP_ANY)?;
    if !capture.is_opened()? {
        return Err(format!("could not open {}", video.display()).into());
    }
    let expected = capture.get(videoio::CAP_PROP_FRAME_COUNT)?.max(0.0) as usize;

    // Left by an interrupted run: it holds an unknown subset of the frames.
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    fs::create_dir_all(dir)?;

    let started = Instant::now();
    let stop = AtomicBool::new(false);
    // Bounded, so a decoder that outruns the writers cannot fill memory
    // (a 1080p frame is about 6 MB).
    let (sender, receiver) = mpsc::sync_channel::<Frame>(jobs * 2);
    let receiver = Arc::new(Mutex::new(receiver));

    let (decoded, written) = thread::scope(|s| {
        let writers: Vec<_> = (0..jobs)
            .map(|_| {
                let receiver = Arc::clone(&receiver);
                let stop = &stop;
                s.spawn(move || write_frames(receiver, dir, stop))
            })
            .collect();
        // Only the writers may hold the receiver: if they all stop, `send` fails
        // instead of blocking forever on a full queue.
        drop(receiver);

        let decoded = decode_frames(&mut capture, sender, &stop, expected, label);
        if decoded.is_err() {
            stop.store(true, Ordering::Relaxed);
        }

        let mut written = Ok(0);
        for writer in writers {
            let result = writer
                .join()
                .unwrap_or_else(|_| Err("a writer thread panicked".into()));
            written = match (written, result) {
                (Ok(total), Ok(n)) => Ok(total + n),
                (Err(e), _) | (Ok(_), Err(e)) => Err(e),
            };
        }
        (decoded, written)
    });

    let (decoded, written) = (decoded?, written?);
    if written != decoded {
        return Err(format!("decoded {decoded} frames but wrote {written}").into());
    }
    if decoded == 0 {
        return Err("no frames could be decoded".into());
    }
    Ok(Stats {
        frames: decoded,
        expected,
        elapsed: started.elapsed(),
    })
}

/// Reads every frame in order and queues it. Takes the sender by value so the
/// queue closes as soon as decoding ends, letting the writers drain it and exit.
fn decode_frames(
    capture: &mut VideoCapture,
    sender: SyncSender<Frame>,
    stop: &AtomicBool,
    expected: usize,
    label: &str,
) -> Result<usize, Error> {
    let live = std::io::stdout().is_terminal();
    let mut index = 0;
    let mut last_report = Instant::now();
    while !stop.load(Ordering::Relaxed) {
        // A new Mat every time: the previous one now belongs to a writer.
        let mut frame = Mat::default();
        if !capture.read(&mut frame)? || frame.empty() {
            break;
        }
        if sender.send((index, frame)).is_err() {
            break; // every writer has stopped; the failure comes back from them
        }
        index += 1;

        if live && last_report.elapsed() >= Duration::from_secs(1) {
            print!("{}{label}: {index}/{expected} frames", clear_line());
            let _ = std::io::stdout().flush();
            last_report = Instant::now();
        }
    }
    Ok(index)
}

/// Takes the receiver by value so it is dropped when the writer exits.
fn write_frames(
    receiver: Arc<Mutex<Receiver<Frame>>>,
    dir: &Path,
    stop: &AtomicBool,
) -> Result<usize, Error> {
    let params = Vector::from_slice(&[imgcodecs::IMWRITE_JPEG_QUALITY, JPEG_QUALITY]);
    let mut written = 0;
    loop {
        // The lock is held while taking the next frame, not while encoding it.
        let next = receiver
            .lock()
            .map_err(|_| "the frame queue lock is poisoned")?
            .recv();
        let Ok((index, frame)) = next else {
            return Ok(written); // the queue is closed and empty
        };
        if stop.load(Ordering::Relaxed) {
            return Ok(written);
        }

        let path = dir.join(format!("frame_{index:06}.jpg"));
        let written_ok = path
            .to_str()
            .ok_or_else(|| Error::from("the output path is not valid UTF-8"))
            .and_then(|p| Ok(imgcodecs::imwrite(p, &frame, &params)?));
        match written_ok {
            Ok(true) => written += 1,
            Ok(false) => {
                stop.store(true, Ordering::Relaxed);
                return Err(format!("could not write {}", path.display()).into());
            }
            Err(e) => {
                stop.store(true, Ordering::Relaxed);
                return Err(e);
            }
        }
    }
}

/// Moves a finished video's frames into place, replacing an older copy (`--force`).
fn finish(partial_dir: &Path, final_dir: &Path) -> Result<(), Error> {
    if final_dir.exists() {
        fs::remove_dir_all(final_dir)?;
    }
    fs::rename(partial_dir, final_dir)?;
    Ok(())
}

fn find_videos(input: &Path) -> Result<Vec<PathBuf>, Error> {
    if input.is_file() {
        return Ok(vec![input.to_path_buf()]);
    }
    let entries =
        fs::read_dir(input).map_err(|e| format!("cannot read {}: {e}", input.display()))?;

    let mut videos = Vec::new();
    for entry in entries {
        let path = entry?.path();
        let hidden = path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with('.'));
        let is_video = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| VIDEO_EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()));
        if path.is_file() && is_video && !hidden {
            videos.push(path);
        }
    }
    videos.sort();
    Ok(videos)
}

/// The video's file name without its extension: the name of its output folder.
fn name_of(video: &Path) -> String {
    video
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "video".to_string())
}

/// On a terminal, returns to the start of the line and clears it, so a result
/// replaces the progress counter. Redirected output gets no progress counter,
/// so it needs nothing.
fn clear_line() -> &'static str {
    if std::io::stdout().is_terminal() { "\r\x1b[K" } else { "" }
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs_f64();
    if secs < 60.0 {
        format!("{secs:.1}s")
    } else {
        let whole = d.as_secs();
        format!("{}m {:02}s", whole / 60, whole % 60)
    }
}

fn default_jobs() -> usize {
    thread::available_parallelism()
        .map(|n| n.get().saturating_sub(1).max(1))
        .unwrap_or(1)
}

/// `Ok(None)` means `--help`.
fn parse_args() -> Result<Option<Args>, String> {
    // Defaults sit next to this crate, so they work from any working directory.
    let examples = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("this crate lives inside examples/");

    let mut paths = Vec::new();
    let mut jobs = None;
    let mut force = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "-j" | "--jobs" => {
                let value = args.next().ok_or("--jobs needs a number")?;
                let n: usize = value
                    .parse()
                    .map_err(|_| format!("--jobs: `{value}` is not a number"))?;
                if n == 0 {
                    return Err("--jobs must be at least 1".to_string());
                }
                jobs = Some(n);
            }
            "--force" => force = true,
            other if other.starts_with('-') => return Err(format!("unknown option `{other}`")),
            _ => paths.push(PathBuf::from(arg)),
        }
    }
    if paths.len() > 2 {
        return Err("expected at most INPUT and OUTPUT".to_string());
    }

    let mut paths = paths.into_iter();
    Ok(Some(Args {
        input: paths.next().unwrap_or_else(|| examples.join("videos")),
        output: paths.next().unwrap_or_else(|| examples.join("frames")),
        jobs: jobs.unwrap_or_else(default_jobs),
        force,
    }))
}
