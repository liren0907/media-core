//! Builds an upload-ready copy of a folder of frame folders, splitting the big
//! ones into numbered parts.
//!
//! `extract-frames` writes one folder per video, and the biggest hold many GB of
//! JPEGs: too much to upload or download in one go. This program leaves its
//! input alone and builds a second folder next to it. A frame folder whose files
//! add up to at most `--split-over` goes in whole, under its own name; a bigger
//! one is cut, in file name order, into `<name>-001/`, `<name>-002/`, … each
//! holding at most `--part-size`. File names are kept, so moving every part's
//! files back into one folder gives the original.
//!
//! Every file is a hard link, not a copy: the result takes no extra disk space,
//! and the input is only ever read. Each folder is built inside `<name>.partial/`
//! and its parts are moved out only once all are linked. A rerun skips finished
//! folders and starts an interrupted one over.

use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::ops::Range;
use std::path::{Component, Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Sizes on the command line are in MB of this many bytes, the unit cloud
/// storage usually shows. A part of at most 500 of these is also under 500 MiB.
const MB: u64 = 1_000_000;

/// Parts are numbered `-001` to `-999`.
const PART_DIGITS: usize = 3;
const MAX_PARTS: usize = 999;

const USAGE: &str = "\
Usage: split-frames [INPUT] [OUTPUT] [--split-over MB] [--part-size MB] [--dry-run] [--force]

  INPUT            a folder of frame folders (default: examples/frames)
  OUTPUT           where the upload-ready folders go (default: examples/frames-split)
  --split-over MB  split a folder whose files add up to more than this (default: 1000)
  --part-size MB   the most one part may hold (default: 500)
  --dry-run        print how each folder would be split, and write nothing
  --force          rebuild folders that are already in OUTPUT

A MB is 1,000,000 bytes, counted from the files' lengths. Every file in OUTPUT
is a hard link to the one in INPUT, so OUTPUT must be on the same disk.";

struct Args {
    input: PathBuf,
    output: PathBuf,
    split_over: u64,
    part_size: u64,
    dry_run: bool,
    force: bool,
}

/// One frame folder of INPUT and how it is laid out in OUTPUT.
struct Folder {
    name: String,
    /// The folder's files in name order, which for `frame_000000.jpg`, … is frame order.
    files: Vec<OsString>,
    sizes: Vec<u64>,
    split: bool,
    /// Each output folder's name and the files it takes. A folder kept whole has
    /// one part, named after the folder itself.
    parts: Vec<(String, Range<usize>)>,
}

enum Built {
    Done(Duration),
    Skipped,
}

#[derive(Default)]
struct Tally {
    split: usize,
    parts: usize,
    whole: usize,
    skipped: usize,
    failed: Vec<String>,
}

impl Tally {
    fn count(&mut self, folder: &Folder) {
        if folder.split {
            self.split += 1;
            self.parts += folder.parts.len();
        } else {
            self.whole += 1;
        }
    }
}

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

/// Returns how many folders failed.
fn run(args: &Args) -> Result<usize, Error> {
    let input = resolve(&args.input)?;
    let output = resolve(&args.output)?;
    if !input.is_dir() {
        return Err(format!("{} is not a folder", input.display()).into());
    }
    // `--force` and the cleanup after an interrupted run delete OUTPUT folders by
    // name, so OUTPUT must never reach into INPUT.
    if output.starts_with(&input) || input.starts_with(&output) {
        return Err("INPUT and OUTPUT must not be the same folder, or one inside the other".into());
    }

    let names = find_folders(&input)?;
    if names.is_empty() {
        return Err(format!("no frame folders found in {}", input.display()).into());
    }
    check_names(&names)?;

    println!(
        "{} folder(s): {} -> {}{}",
        names.len(),
        input.display(),
        output.display(),
        if args.dry_run { " (dry run)" } else { "" }
    );
    println!(
        "Splitting folders over {} into parts of at most {}",
        format_size(args.split_over),
        format_size(args.part_size)
    );

    let started = Instant::now();
    let mut tally = Tally::default();

    for (i, name) in names.iter().enumerate() {
        let label = format!("[{}/{}] {name}", i + 1, names.len());
        let folder = match plan_folder(&input.join(name), name, args) {
            Ok(folder) => folder,
            Err(e) => {
                println!("{label}: failed: {e}");
                tally.failed.push(name.clone());
                continue;
            }
        };

        if args.dry_run {
            let note = dry_run_note(&output, name, args.force)?;
            println!("{label}: {}{note}", describe(&folder));
            if folder.split {
                for (part, range) in &folder.parts {
                    println!("    {}", describe_part(&folder, part, range));
                }
            }
            tally.count(&folder);
            continue;
        }

        match build(&folder, &input, &output, args.force) {
            Ok(Built::Done(elapsed)) => {
                println!("{label}: {} ({})", describe(&folder), format_duration(elapsed));
                tally.count(&folder);
            }
            Ok(Built::Skipped) => {
                println!("{label}: already in OUTPUT, skipped");
                tally.skipped += 1;
            }
            Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
                return Err(
                    "OUTPUT is on a different disk from INPUT; hard links need both on the same disk"
                        .into(),
                );
            }
            Err(e) => {
                println!("{label}: failed: {e}");
                tally.failed.push(name.clone());
            }
        }
    }

    if args.dry_run {
        println!(
            "Dry run: {} folder(s) would be split into {} parts, {} kept whole, {} failed; nothing was written",
            tally.split,
            tally.parts,
            tally.whole,
            tally.failed.len()
        );
    } else {
        println!(
            "Done in {}: {} folder(s) split into {} parts, {} kept whole, {} skipped, {} failed",
            format_duration(started.elapsed()),
            tally.split,
            tally.parts,
            tally.whole,
            tally.skipped,
            tally.failed.len()
        );
    }
    for name in &tally.failed {
        println!("  failed: {name}");
    }
    Ok(tally.failed.len())
}

/// Reads one frame folder and decides its parts. Writes nothing.
fn plan_folder(dir: &Path, name: &str, args: &Args) -> Result<Folder, Error> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))? {
        let entry = entry?;
        let file_name = entry.file_name();
        if file_name.as_encoded_bytes().starts_with(b".") {
            continue;
        }
        if !entry.file_type()?.is_file() {
            return Err(format!(
                "{} is not a plain file; a frame folder should hold only frames",
                file_name.to_string_lossy()
            )
            .into());
        }
        entries.push((file_name, entry.metadata()?.len()));
    }
    entries.sort();
    let (files, sizes): (Vec<_>, Vec<_>) = entries.into_iter().unzip();
    let total: u64 = sizes.iter().sum();

    if total <= args.split_over {
        let parts = vec![(name.to_string(), 0..files.len())];
        return Ok(Folder { name: name.to_string(), files, sizes, split: false, parts });
    }

    let ranges = plan_parts(&sizes, args.part_size).map_err(|i| {
        format!(
            "{} alone is {}, more than a part may hold",
            files[i].to_string_lossy(),
            format_size(sizes[i])
        )
    })?;
    if ranges.len() > MAX_PARTS {
        return Err(format!(
            "would need {} parts, more than {MAX_PARTS}; raise --part-size",
            ranges.len()
        )
        .into());
    }
    let parts = ranges
        .into_iter()
        .enumerate()
        .map(|(i, range)| (format!("{name}-{:0width$}", i + 1, width = PART_DIGITS), range))
        .collect();
    Ok(Folder { name: name.to_string(), files, sizes, split: true, parts })
}

/// Cuts files, in order, into runs whose sizes add up to at most `max`: a run
/// closes as soon as the next file would not fit. Fails with the index of a file
/// that is bigger than `max` on its own.
fn plan_parts(sizes: &[u64], max: u64) -> Result<Vec<Range<usize>>, usize> {
    let mut parts = Vec::new();
    let (mut start, mut used) = (0, 0);
    for (i, &size) in sizes.iter().enumerate() {
        if size > max {
            return Err(i);
        }
        if used + size > max {
            parts.push(start..i);
            (start, used) = (i, 0);
        }
        used += size;
    }
    if start < sizes.len() {
        parts.push(start..sizes.len());
    }
    Ok(parts)
}

/// Links one folder's parts into OUTPUT, or skips it if it is already there.
fn build(folder: &Folder, input: &Path, output: &Path, force: bool) -> io::Result<Built> {
    let started = Instant::now();
    // Built by hand: the names contain dots, which `with_extension` would eat.
    let staging = output.join(format!("{}.partial", folder.name));
    let existing = existing_outputs(output, &folder.name)?;

    if staging.exists() {
        // Interrupted last time: the parts that made it out may not be all of them.
        fs::remove_dir_all(&staging).map_err(at(&staging))?;
        remove_all(&existing)?;
    } else if !existing.is_empty() {
        if !force {
            return Ok(Built::Skipped);
        }
        remove_all(&existing)?;
    }

    let source = input.join(&folder.name);
    fs::create_dir_all(&staging).map_err(at(&staging))?;
    for (part, range) in &folder.parts {
        let dir = staging.join(part);
        fs::create_dir(&dir).map_err(at(&dir))?;
        for file in &folder.files[range.clone()] {
            let from = source.join(file);
            if let Err(e) = fs::hard_link(&from, dir.join(file)) {
                if e.kind() == io::ErrorKind::CrossesDevices {
                    // Nothing can be linked into this OUTPUT; leave nothing behind in it.
                    let _ = fs::remove_dir_all(&staging);
                }
                return Err(at(&from)(e));
            }
        }
    }

    // If this is cut short, the `.partial` left behind tells the next run to start over.
    for (part, _) in &folder.parts {
        let to = output.join(part);
        fs::rename(staging.join(part), &to).map_err(at(&to))?;
    }
    fs::remove_dir(&staging).map_err(at(&staging))?;
    Ok(Built::Done(started.elapsed()))
}

fn remove_all(dirs: &[PathBuf]) -> io::Result<()> {
    for dir in dirs {
        // Removes the links only: the frames in INPUT keep their data.
        fs::remove_dir_all(dir).map_err(at(dir))?;
    }
    Ok(())
}

/// The folders in OUTPUT that belong to the frame folder `name`: `<name>/` or `<name>-NNN/`.
fn existing_outputs(output: &Path, name: &str) -> io::Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    let entries = match fs::read_dir(output) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(found),
        Err(e) => return Err(at(output)(e)),
    };
    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(entry_name) = file_name.to_str() else { continue };
        let ours = entry_name == name || part_base(entry_name) == Some(name);
        if ours && entry.file_type()?.is_dir() {
            found.push(entry.path());
        }
    }
    Ok(found)
}

/// What a real run would do differently from the plan just printed.
fn dry_run_note(output: &Path, name: &str, force: bool) -> io::Result<&'static str> {
    if output.join(format!("{name}.partial")).exists() {
        return Ok("  [interrupted before; a real run redoes it]");
    }
    if !force && !existing_outputs(output, name)?.is_empty() {
        return Ok("  [already in OUTPUT; a real run skips it]");
    }
    Ok("")
}

/// The names of the frame folders in INPUT, sorted. Hidden entries and the
/// unfinished `*.partial` folders of `extract-frames` are left out.
fn find_folders(input: &Path) -> Result<Vec<String>, Error> {
    let entries =
        fs::read_dir(input).map_err(|e| format!("cannot read {}: {e}", input.display()))?;

    let mut names = Vec::new();
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|n| format!("folder name is not UTF-8: {}", n.to_string_lossy()))?;
        if !name.starts_with('.') && !name.ends_with(".partial") {
            names.push(name);
        }
    }
    names.sort();
    Ok(names)
}

/// Refuses names whose outputs would mix: a folder `x-001` next to a folder `x`
/// would look like a part of `x`, and rebuilding `x` would delete it.
fn check_names(names: &[String]) -> Result<(), Error> {
    let all: HashSet<&str> = names.iter().map(String::as_str).collect();
    for name in names {
        if let Some(base) = part_base(name)
            && all.contains(base)
        {
            return Err(format!("{name} would be taken for a part of {base}; rename one of them").into());
        }
    }
    Ok(())
}

/// For a part folder name `<base>-NNN`, returns `<base>`.
fn part_base(name: &str) -> Option<&str> {
    let (base, number) = name.rsplit_once('-')?;
    let is_number = number.len() == PART_DIGITS && number.bytes().all(|b| b.is_ascii_digit());
    is_number.then_some(base)
}

/// The absolute form of `path` with symlinks and `..` resolved, whether or not
/// it exists yet, so that INPUT and OUTPUT can be compared.
fn resolve(path: &Path) -> io::Result<PathBuf> {
    let mut resolved = PathBuf::new();
    for component in std::path::absolute(path)?.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            other => {
                resolved.push(other);
                // Resolved one step at a time, so a later `..` goes up from where
                // a symlink points rather than from the link.
                if resolved.exists() {
                    resolved = resolved.canonicalize()?;
                }
            }
        }
    }
    Ok(resolved)
}

/// Adds the path to an I/O error, keeping its kind.
fn at(path: &Path) -> impl Fn(io::Error) -> io::Error + '_ {
    move |e| io::Error::new(e.kind(), format!("{}: {e}", path.display()))
}

fn describe(folder: &Folder) -> String {
    let what = if folder.split {
        format!("split into {} parts", folder.parts.len())
    } else {
        "kept whole".to_string()
    };
    format!(
        "{}, {}, {what}",
        count_files(folder.files.len()),
        format_size(folder.sizes.iter().sum())
    )
}

fn describe_part(folder: &Folder, part: &str, range: &Range<usize>) -> String {
    format!(
        "{part}  {} … {}  {}  {}",
        folder.files[range.start].to_string_lossy(),
        folder.files[range.end - 1].to_string_lossy(),
        count_files(range.len()),
        format_size(folder.sizes[range.clone()].iter().sum())
    )
}

fn count_files(n: usize) -> String {
    format!("{} file{}", group(n), if n == 1 { "" } else { "s" })
}

/// Rounded down, so a part shown as 499.9 MB really is under 500 MB.
fn format_size(bytes: u64) -> String {
    const GB: u64 = 1000 * MB;
    if bytes >= GB {
        format!("{}.{:02} GB", bytes / GB, bytes % GB / (GB / 100))
    } else {
        format!("{}.{} MB", bytes / MB, bytes % MB / (MB / 10))
    }
}

/// `31558` → `31,558`.
fn group(n: usize) -> String {
    let digits = n.to_string();
    let mut grouped = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
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

/// `Ok(None)` means `--help`.
fn parse_args() -> Result<Option<Args>, String> {
    // Defaults sit next to this crate, so they work from any working directory.
    let examples = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("this crate lives inside examples/");

    let mut paths = Vec::new();
    let (mut split_over, mut part_size) = (1000 * MB, 500 * MB);
    let (mut dry_run, mut force) = (false, false);

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--split-over" => split_over = megabytes(&arg, args.next())?,
            "--part-size" => part_size = megabytes(&arg, args.next())?,
            "--dry-run" => dry_run = true,
            "--force" => force = true,
            other if other.starts_with('-') => return Err(format!("unknown option `{other}`")),
            _ => paths.push(PathBuf::from(arg)),
        }
    }
    if part_size == 0 {
        return Err("--part-size must be at least 1".to_string());
    }
    if paths.len() > 2 {
        return Err("expected at most INPUT and OUTPUT".to_string());
    }

    let mut paths = paths.into_iter();
    Ok(Some(Args {
        input: paths.next().unwrap_or_else(|| examples.join("frames")),
        output: paths.next().unwrap_or_else(|| examples.join("frames-split")),
        split_over,
        part_size,
        dry_run,
        force,
    }))
}

fn megabytes(option: &str, value: Option<String>) -> Result<u64, String> {
    let value = value.ok_or_else(|| format!("{option} needs a size in MB"))?;
    value
        .parse::<u64>()
        .ok()
        .and_then(|mb| mb.checked_mul(MB))
        .ok_or_else(|| format!("{option}: `{value}` is not a whole number of MB"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_files_make_no_parts() {
        assert_eq!(plan_parts(&[], 10), Ok(vec![]));
    }

    #[test]
    fn a_part_may_be_exactly_full() {
        assert_eq!(plan_parts(&[5, 5, 5], 10), Ok(vec![0..2, 2..3]));
    }

    #[test]
    fn a_part_closes_when_the_next_file_does_not_fit() {
        assert_eq!(plan_parts(&[4, 4, 4, 4, 4], 10), Ok(vec![0..2, 2..4, 4..5]));
    }

    #[test]
    fn a_file_bigger_than_a_part_is_an_error() {
        assert_eq!(plan_parts(&[1, 11, 1], 10), Err(1));
    }

    #[test]
    fn parts_cover_every_file_in_order_and_each_is_as_full_as_it_can_be() {
        // xorshift64: varied sizes without a dependency.
        let mut state: u64 = 0x2545_F491_4F6C_DD1D;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        for _ in 0..1000 {
            let max = 1 + next() % 50;
            let len = (next() % 40) as usize;
            let sizes: Vec<u64> = (0..len).map(|_| next() % (max + 1)).collect();
            let parts = plan_parts(&sizes, max).unwrap();

            let mut end = 0;
            for (i, part) in parts.iter().enumerate() {
                assert_eq!(part.start, end, "parts must follow each other");
                assert!(!part.is_empty());
                let used: u64 = sizes[part.clone()].iter().sum();
                assert!(used <= max);
                if i + 1 < parts.len() {
                    assert!(used + sizes[part.end] > max, "a part closed while the next file fit");
                }
                end = part.end;
            }
            assert_eq!(end, sizes.len(), "every file must be in a part");
        }
    }

    #[test]
    fn part_names_are_three_digits_after_a_dash() {
        assert_eq!(part_base("sample_01.12.30-001"), Some("sample_01.12.30"));
        assert_eq!(part_base("sample-a-999"), Some("sample-a"));
        assert_eq!(part_base("sample-01"), None);
        assert_eq!(part_base("sample-0001"), None);
        assert_eq!(part_base("sample-00x"), None);
        assert_eq!(part_base("sample_mod"), None);
    }

    #[test]
    fn sizes_round_down() {
        assert_eq!(format_size(499_999_999), "499.9 MB");
        assert_eq!(format_size(500_000_000), "500.0 MB");
        assert_eq!(format_size(12_929_999_999), "12.92 GB");
        assert_eq!(format_size(0), "0.0 MB");
    }

    #[test]
    fn counts_are_grouped_by_thousands() {
        assert_eq!(group(0), "0");
        assert_eq!(group(958), "958");
        assert_eq!(group(1191), "1,191");
        assert_eq!(group(31558), "31,558");
        assert_eq!(group(1234567), "1,234,567");
    }
}
