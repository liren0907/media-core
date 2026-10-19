# examples

Standalone programs built on this workspace. Each one is its own crate under `examples/`, listed
by name in the root `Cargo.toml` (not as `examples/*`, because this folder also holds local data).

| Example | What it does | Run |
|---|---|---|
| [`extract-frames`](extract-frames/) | Writes every frame of each video into its own folder of JPEGs | `cargo run --release -p extract-frames` |
| [`split-frames`](split-frames/) | Builds an upload-ready copy of those folders, cutting any over 1 GB into parts of at most 500 MB | `cargo run --release -p split-frames` |

## Local data (gitignored)

| Folder | What goes there |
|---|---|
| `videos/` | Input videos for `extract-frames`. Never committed: videos are large, and GitHub rejects files over 100 MB. |
| `frames/` | What `extract-frames` writes: `frames/<video name>/frame_000000.jpg`, … |
| `frames-split/` | What `split-frames` writes: `frames/` again as hard links, with each big folder cut into `<video name>-001/`, `<video name>-002/`, … |

## extract-frames at a glance

Full details are in [`extract-frames/README.md`](extract-frames/README.md).

- **Every frame, one folder per video.** The folder is named after the video file without its
  extension. Frames are `frame_000000.jpg`, `frame_000001.jpg`, …, numbered from 0 in the order
  they are decoded.
- **One decoder, many writers.** One thread reads the video in order; a pool of threads writes the
  JPEGs. The result is byte-identical to a single-threaded run (`--jobs 1`).
- **Resumable.** Each video is written to `<name>.partial/` and renamed to `<name>/` only when
  complete. A rerun skips finished videos and starts an interrupted one over.
- **Keeps going.** A video that fails does not stop the batch; failures are listed at the end and
  the exit code is non-zero.
- **Disk space.** A 1080p frame takes roughly 300 KB, so an hour of 30 fps video needs around 30 GB.

### Running a full extraction

1. Put the videos in `examples/videos/`.
2. Check that there is enough free disk space (see above).
3. Run `cargo run --release -p extract-frames` from anywhere in the repository.
4. If it is interrupted, run the same command again: finished videos are skipped and the
   interrupted one starts over.
5. When it ends, the last line should read `Done: … 0 failed`, and `examples/frames/` should hold
   one folder per video and no `*.partial` folder.

## split-frames at a glance

Full details are in [`split-frames/README.md`](split-frames/README.md).

- **For sharing.** A folder of up to 1 GB is kept whole under its own name; a bigger one is cut,
  in frame order, into `<name>-001/`, `<name>-002/`, … of at most 500 MB each (1 MB =
  1,000,000 bytes). Both sizes are options.
- **Nothing is copied or moved.** Every file is a hard link to the one in `frames/`, so the copy
  takes no extra space and `frames/` stays as `extract-frames` left it. `du` and Finder still
  report the full size of `frames-split/`.
- **Names are kept.** Moving the files of all of a folder's parts back into one folder gives the
  original.
- **Look first.** `cargo run --release -p split-frames -- --dry-run` prints every part's first
  and last frame, count and size without writing anything.

## Adding an example

1. Create `examples/<name>/` with its own `Cargo.toml` (`publish = false`; take shared
   dependencies such as `opencv` with `{ workspace = true }`).
2. Add `"examples/<name>"` to `members` in the root `Cargo.toml`.
3. Keep any sample media in a gitignored folder, and add a row to the table above.
