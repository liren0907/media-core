#!/bin/sh
# Serves data/test.mp4 as a live stream at rtsp://localhost:8554/mystream, the URL
# the ignored tests in tests/test_rtsp_extraction.rs read from.
#
# Needs mediamtx (the RTSP server) and ffmpeg (which pushes the video to it, looping
# until Ctrl-C; stopping the script stops both). With the stream up, run the tests
# from another terminal:
#   cargo test -p media_core --test test_rtsp_extraction -- --ignored --nocapture
#
# Usage: sh script/rtsp_test_server.sh

set -eu

if [ "$#" -ne 0 ]; then
    echo "usage: sh script/rtsp_test_server.sh" >&2
    exit 2
fi

# Everything below is relative to the crate root, wherever this is run from.
cd "$(dirname "$0")/.."

url=rtsp://localhost:8554/mystream

if ! command -v mediamtx >/dev/null 2>&1; then
    echo "error: mediamtx is needed to serve $url" >&2
    echo "  macOS: brew install mediamtx" >&2
    echo "  other: https://github.com/bluenviron/mediamtx/releases" >&2
    exit 1
fi
if ! command -v ffmpeg >/dev/null 2>&1; then
    echo "error: ffmpeg is needed to push the video to $url" >&2
    echo "  macOS:         brew install ffmpeg" >&2
    echo "  Debian/Ubuntu: sudo apt install ffmpeg" >&2
    exit 1
fi

[ -f data/test.mp4 ] || sh script/gen_test_data.sh

# Both programs run in the background and are stopped here on every way out:
# Ctrl-C, kill, or either of them failing. The pusher is killed outright: ffmpeg
# pacing a looped input with -re does not act on a single TERM or INT, and it has
# nothing to save.
server=
pusher=
log=
stop() {
    if [ -n "$pusher" ]; then
        kill -KILL "$pusher" 2>/dev/null || true
        wait "$pusher" 2>/dev/null || true
    fi
    if [ -n "$server" ]; then
        kill "$server" 2>/dev/null || true
        wait "$server" 2>/dev/null || true
    fi
    [ -z "$log" ] || rm -f "$log"
}
trap stop EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

# Configured by environment alone: only the RTSP server, and only the stream the
# tests read. mediamtx warns that it found no configuration file; that is expected.
MTX_RTMP=no MTX_HLS=no MTX_WEBRTC=no MTX_SRT=no MTX_MOQ=no \
MTX_PATHS_MYSTREAM_SOURCE=publisher \
    mediamtx &
server=$!

sleep 1
if ! kill -0 "$server" 2>/dev/null; then
    echo "error: mediamtx did not start (is port 8554 already in use?)" >&2
    exit 1
fi

# ffmpeg's messages are kept aside and shown only if it stops on its own: on Ctrl-C
# the server may close first, and the broken-pipe errors that follow mean nothing.
log=$(mktemp "${TMPDIR:-/tmp}/rtsp_test_server.XXXXXX")
ffmpeg -hide_banner -loglevel error -nostdin \
    -re -stream_loop -1 -i data/test.mp4 \
    -c copy -rtsp_transport tcp -f rtsp "$url" 2>"$log" &
pusher=$!

echo "serving data/test.mp4 at $url (Ctrl-C to stop)"
wait "$pusher" || true
pusher=
echo "error: ffmpeg stopped pushing to $url" >&2
cat "$log" >&2
exit 1
