# split-frames

Builds an upload-ready copy of the frame folders that `extract-frames` writes. A folder of up to
1 GB goes in whole; a bigger one is cut into numbered parts of at most 500 MB each.

```
examples/frames/                     examples/frames-split/
  short/   (300 MB)          ->        short/
  long/    (2.2 GB)          ->        long-001/   frame_000000.jpg … frame_001190.jpg   (≤ 500 MB)
                                       long-002/   frame_001191.jpg … frame_002379.jpg   (≤ 500 MB)
                                       ...
                                       long-005/   … the last frames                      (smaller)
```

## Run

From anywhere in the repository:

```bash
cargo run --release -p split-frames -- --dry-run
cargo run --release -p split-frames
```

The first command only prints how each folder would be split. With no paths it reads
`examples/frames/` and writes to `examples/frames-split/`; both are gitignored.

```
split-frames [INPUT] [OUTPUT] [--split-over MB] [--part-size MB] [--dry-run] [--force]

  INPUT            a folder of frame folders (default: examples/frames)
  OUTPUT           where the upload-ready folders go (default: examples/frames-split)
  --split-over MB  split a folder whose files add up to more than this (default: 1000)
  --part-size MB   the most one part may hold (default: 500)
  --dry-run        print how each folder would be split, and write nothing
  --force          rebuild folders that are already in OUTPUT
```

A MB here is 1,000,000 bytes, counted from the files' lengths, so a 500 MB part is also under
500 MiB. Arguments go after `--` when using `cargo run`.

## How it works

- **In order.** Files are taken in name order, which for `frame_000000.jpg`, … is frame order. A
  part closes as soon as the next frame would not fit, so every part except the last is nearly
  full, and the frames in each part are consecutive.
- **Names are kept.** To put a split folder back together, move the files of all its parts into
  one folder; no two parts share a file name.
- **Hard links, not copies.** Each file in OUTPUT is another name for the same file in INPUT. The
  copy takes no extra disk space, and INPUT is only ever read. Linking is quick but not free: about
  90,000 frames took under two minutes on a laptop SSD. Deleting
  OUTPUT leaves INPUT as it was, and the other way round. OUTPUT must be on the same disk as
  INPUT.
- **Safe to interrupt.** Each folder is built in `<name>.partial/` inside OUTPUT, and its parts
  are moved out only once every file is linked. Rerunning skips folders already in OUTPUT and
  starts an interrupted one over. Use `--force` after changing the sizes, or after
  re-extracting frames.
- **One bad folder does not stop the batch.** A folder that holds anything but plain files, or a
  file bigger than a part, is reported at the end and the exit code is non-zero.

## Disk space

Because of the hard links, `df` shows the same free space before and after a run. Tools that
add up file sizes do not know about them: `du -sh examples/frames-split` on its own, or Finder's
Get Info, reports the full size of the frames. `du -sh examples/frames examples/frames-split`
counts each file once.

An upload or a copy to another disk turns the links into real files, so that side needs the
full size.
