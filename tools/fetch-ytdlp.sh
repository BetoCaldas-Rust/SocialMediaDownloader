#!/bin/bash
# Downloads the yt-dlp sidecar binary into resources/bin/.
# The binary itself is git-ignored; run this before dev or packaging.
# Usage: ./tools/fetch-ytdlp.sh [--force]
set -euo pipefail
FORCE=0
[ "${1:-}" = "--force" ] && FORCE=1
DEST="$(cd "$(dirname "$0")/../resources/bin" && pwd)/yt-dlp"
if [ -f "$DEST" ] && [ "$FORCE" -eq 0 ]; then
    echo "yt-dlp already present at $DEST (use --force to re-download)."
    exit 0
fi
echo "Resolving latest yt-dlp release..."
URL=$(curl -fsSL https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest \
    | grep -o '"browser_download_url": *"[^"]*yt-dlp_linux"' | head -1 | cut -d'"' -f4)
if [ -z "$URL" ]; then
    echo "yt-dlp_linux asset not found in latest release." >&2
    exit 1
fi
mkdir -p "$(dirname "$DEST")"
echo "Downloading $URL ..."
curl -fsSL "$URL" -o "$DEST"
chmod +x "$DEST"
"$DEST" --version
