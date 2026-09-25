#!/bin/bash
# ============================================================================
# Generate Dedup Test Data
#
# Takes the source files in data/dedup-source-data/ and creates two test
# directories with different tree structures, filenames, exact copies,
# and perceptually similar variants.
#
# Requirements: sips (macOS built-in), ffmpeg
# ============================================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
SOURCE_DIR="$PROJECT_DIR/data/dedup-source-data"
TEST_A="$PROJECT_DIR/data/dedup-test-a"
TEST_B="$PROJECT_DIR/data/dedup-test-b"

# Source files
SRC_JPG1="$SOURCE_DIR/test.jpg"
SRC_JPG2="$SOURCE_DIR/zidane.jpg"
SRC_MP4_SHORT="$SOURCE_DIR/test_short.mp4"
SRC_MP4_LONG="$SOURCE_DIR/test1.mp4"

# ─── Verify source files exist ───
for f in "$SRC_JPG1" "$SRC_JPG2" "$SRC_MP4_SHORT" "$SRC_MP4_LONG"; do
  if [ ! -f "$f" ]; then
    echo "ERROR: Source file not found: $f"
    exit 1
  fi
done

echo "=== Cleaning previous test data ==="
rm -rf "$TEST_A" "$TEST_B"

# ══════════════════════════════════════════════════════════════════════════════
# TEST-A: Simulates a "Camera Roll" source
# ══════════════════════════════════════════════════════════════════════════════
echo "=== Creating dedup-test-a (Camera Roll) ==="

mkdir -p "$TEST_A/photos/vacation"
mkdir -p "$TEST_A/photos/portraits"
mkdir -p "$TEST_A/photos/random"
mkdir -p "$TEST_A/videos/trips"
mkdir -p "$TEST_A/videos/misc"

# Exact copies with different names
cp "$SRC_JPG1"       "$TEST_A/photos/vacation/beach_sunset.jpg"
cp "$SRC_JPG1"       "$TEST_A/photos/vacation/ocean_view.jpg"
cp "$SRC_JPG2"       "$TEST_A/photos/portraits/family_group.jpg"
cp "$SRC_MP4_SHORT"  "$TEST_A/videos/trips/trip_clip.mp4"
cp "$SRC_MP4_LONG"   "$TEST_A/videos/misc/full_recording.mp4"

# Similar variant: resized to 1024px wide (pHash should be close, BLAKE3 different)
sips --resampleWidth 1024 "$TEST_A/photos/vacation/beach_sunset.jpg" --out "$TEST_A/photos/random/beach_resized.jpg" > /dev/null 2>&1

# Similar variant: lower JPEG quality
sips -s formatOptions 30 "$SRC_JPG2" --out "$TEST_A/photos/random/portrait_compressed.jpg" > /dev/null 2>&1

# Unique image: generate a solid color image (no match to anything)
sips -z 720 1280 "$SRC_JPG1" --out "$TEST_A/photos/random/unique_gradient.jpg" > /dev/null 2>&1
# Overwrite with a generated image via ffmpeg (solid blue)
ffmpeg -y -f lavfi -i "color=c=blue:s=1280x720:d=1" -frames:v 1 "$TEST_A/photos/random/unique_blue.jpg" > /dev/null 2>&1

echo "  Created $(find "$TEST_A" -type f | wc -l | tr -d ' ') files in dedup-test-a"

# ══════════════════════════════════════════════════════════════════════════════
# TEST-B: Simulates a "Backup Drive" source
# ══════════════════════════════════════════════════════════════════════════════
echo "=== Creating dedup-test-b (Backup Drive) ==="

mkdir -p "$TEST_B/backup/camera_roll"
mkdir -p "$TEST_B/backup/screenshots"
mkdir -p "$TEST_B/backup/edited"
mkdir -p "$TEST_B/media/videos"
mkdir -p "$TEST_B/media/clips"

# ── Exact matches (same content as test-a, different names + tree) ──
cp "$SRC_JPG1"       "$TEST_B/backup/camera_roll/IMG_2024_001.jpg"
cp "$SRC_JPG2"       "$TEST_B/backup/camera_roll/IMG_2024_002.jpg"
cp "$SRC_MP4_SHORT"  "$TEST_B/media/clips/holiday_clip.mp4"

# ── Similar matches (perceptually similar, different BLAKE3) ──

# Slight resize of test.jpg
sips --resampleWidth 1200 "$SRC_JPG1" --out "$TEST_B/backup/edited/photo_edited.jpg" > /dev/null 2>&1

# Heavy JPEG compression of test.jpg
sips -s formatOptions 15 "$SRC_JPG1" --out "$TEST_B/backup/edited/photo_lowq.jpg" > /dev/null 2>&1

# Crop zidane.jpg (resize to slightly different aspect)
sips -c 700 1280 "$SRC_JPG2" --out "$TEST_B/backup/screenshots/screen_capture.jpg" > /dev/null 2>&1

# Video: re-encode test_short.mp4 at lower bitrate (same content, different hash)
ffmpeg -y -i "$SRC_MP4_SHORT" -b:v 500k -s 640x360 "$TEST_B/media/clips/trip_lowres.mp4" > /dev/null 2>&1

# ── No match (unique files) ──

# Unique generated images
ffmpeg -y -f lavfi -i "color=c=red:s=1280x720:d=1" -frames:v 1 "$TEST_B/backup/screenshots/unique_red.jpg" > /dev/null 2>&1
ffmpeg -y -f lavfi -i "color=c=green:s=1280x720:d=1" -frames:v 1 "$TEST_B/backup/screenshots/unique_green.jpg" > /dev/null 2>&1

# Copy the long video as-is (different from test-a since test-a also has it — this IS an exact match)
cp "$SRC_MP4_LONG" "$TEST_B/media/videos/archive_recording.mp4"

echo "  Created $(find "$TEST_B" -type f | wc -l | tr -d ' ') files in dedup-test-b"

# ══════════════════════════════════════════════════════════════════════════════
# Summary
# ══════════════════════════════════════════════════════════════════════════════
echo ""
echo "=== Test Data Generated ==="
echo ""
echo "dedup-test-a/ (Camera Roll):"
find "$TEST_A" -type f | sort | sed "s|$TEST_A/|  |"
echo ""
echo "dedup-test-b/ (Backup Drive):"
find "$TEST_B" -type f | sort | sed "s|$TEST_B/|  |"
echo ""
echo "=== Expected Matches ==="
echo ""
echo "EXACT matches (same BLAKE3):"
echo "  test-a/photos/vacation/beach_sunset.jpg   ↔ test-b/backup/camera_roll/IMG_2024_001.jpg"
echo "  test-a/photos/vacation/ocean_view.jpg      ↔ test-b/backup/camera_roll/IMG_2024_001.jpg"
echo "  test-a/photos/portraits/family_group.jpg   ↔ test-b/backup/camera_roll/IMG_2024_002.jpg"
echo "  test-a/videos/trips/trip_clip.mp4          ↔ test-b/media/clips/holiday_clip.mp4"
echo "  test-a/videos/misc/full_recording.mp4      ↔ test-b/media/videos/archive_recording.mp4"
echo ""
echo "SIMILAR matches (close pHash, different BLAKE3):"
echo "  test-a/photos/vacation/beach_sunset.jpg    ~ test-b/backup/edited/photo_edited.jpg (resized)"
echo "  test-a/photos/vacation/beach_sunset.jpg    ~ test-b/backup/edited/photo_lowq.jpg (compressed)"
echo "  test-a/photos/portraits/family_group.jpg   ~ test-b/backup/screenshots/screen_capture.jpg (cropped)"
echo "  test-a/photos/random/beach_resized.jpg     ~ test-b/backup/edited/photo_edited.jpg (both resized)"
echo ""
echo "NO match (unique files):"
echo "  test-a/photos/random/unique_blue.jpg"
echo "  test-b/backup/screenshots/unique_red.jpg"
echo "  test-b/backup/screenshots/unique_green.jpg"
echo ""
echo "Done! Use these paths in the Dedup tab:"
echo "  Source A: $TEST_A"
echo "  Source B: $TEST_B"
