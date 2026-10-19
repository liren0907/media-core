#!/bin/sh
# Generates data/test.mp4, the video the integration tests use.
#
# The video is synthetic (a moving test pattern with a 1 kHz tone), so this recipe
# is versioned instead of the file. The tests run this script themselves when the
# file is missing; run it by hand to create or recreate it.
#
# Usage: sh script/gen_test_data.sh [--force]
#   --force  regenerate even if data/test.mp4 already exists

set -eu

case "${1:-}" in
    "") force=0 ;;
    --force) force=1 ;;
    *)
        echo "usage: sh script/gen_test_data.sh [--force]" >&2
        exit 2
        ;;
esac

# Everything below is relative to the crate root, wherever this is run from.
cd "$(dirname "$0")/.."

out=data/test.mp4

if [ "$force" -eq 0 ] && [ -f "$out" ]; then
    echo "$out already exists (use --force to regenerate)"
    exit 0
fi

if ! command -v ffmpeg >/dev/null 2>&1; then
    echo "error: ffmpeg is needed to generate $out" >&2
    echo "  macOS:         brew install ffmpeg" >&2
    echo "  Debian/Ubuntu: sudo apt install ffmpeg" >&2
    exit 1
fi

mkdir -p data

# Written under a name of its own and renamed only once complete: an interrupted
# run leaves no half-written video, and two runs at once never share a file.
tmp="$out.tmp.$$"
trap 'rm -f "$tmp"' EXIT

ffmpeg -hide_banner -loglevel error -y \
    -f lavfi -i testsrc2=size=640x360:rate=30:duration=10 \
    -f lavfi -i sine=frequency=1000:sample_rate=48000:duration=10 \
    -c:v libx264 -pix_fmt yuv420p -c:a aac -shortest \
    -f mp4 "$tmp"
mv "$tmp" "$out"

echo "generated $out"
