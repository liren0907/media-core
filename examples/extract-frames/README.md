# extract-frames

Writes every frame of every video in a folder as a JPEG, one folder per video.

```
examples/videos/clip.mp4   ->   examples/frames/clip/frame_000000.jpg
                                                     frame_000001.jpg
                                                     ...
```

## Run

From anywhere in the repository:

```bash
cargo run --release -p extract-frames
```

With no arguments it reads `examples/videos/` and writes to `examples/frames/`. Both folders
are gitignored: put your own videos in `examples/videos/` first.

```
extract-frames [INPUT] [OUTPUT] [--jobs N] [--force]

  INPUT     a video file, or a folder of videos (default: examples/videos)
  OUTPUT    where the per-video folders go (default: examples/frames)
  --jobs N  JPEG writer threads (default: CPU count - 1)
  --force   redo videos that already have a folder
```

Arguments go after `--` when using `cargo run`, e.g.
`cargo run --release -p extract-frames -- path/to/clip.mp4 --jobs 4`.

## How it works

- **One decoder, many writers.** One thread reads each video from start to end without seeking,
  so a frame's number is exactly its position in the video. A pool of threads encodes the JPEGs,
  which is the slow part. The output is byte-identical to a `--jobs 1` run.
- **Bounded memory.** Frames wait in a queue of `2 × jobs` slots; a 1080p frame is about 6 MB.
- **Safe to interrupt.** Each video is written to `<name>.partial/` and renamed to `<name>/`
  only when every frame is on disk. Rerunning skips finished videos and starts an interrupted
  one over.
- **One bad video does not stop the batch.** Failures are listed at the end and the exit code
  is non-zero.

## Performance

Decoding has to happen in order on one thread, so it sets the ceiling. On a 10-core laptop with
1080p H.264, throughput stopped improving at about four writer threads, at roughly 250–300
frames/s. More threads did not help, but did not slow it down either, so the default stays at
CPU count - 1. Debug and release builds ran at about the same speed because the work happens
inside OpenCV; `--release` only gains a little.

## Disk space

Every frame is kept, at JPEG quality 95 (`JPEG_QUALITY` in `src/main.rs`). A 1080p frame takes
roughly 300 KB, so an hour of 30 fps 1080p video needs around 30 GB. Check free space before a
large run.
