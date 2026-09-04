# yt-dlp sidecar (dev drop zone)

The app looks for the yt-dlp binary in this order:

1. `resources/bin/yt-dlp.exe` next to the running executable
   (during development: this very folder, `<repo>/resources/bin/`);
2. `%APPDATA%\SMD\updates\yt-dlp.exe` (self-update drop zone, used from F8);
3. `yt-dlp` / `yt-dlp.exe` found on `PATH`.

To run downloads locally before F8 bundles the binary, download
`yt-dlp.exe` from https://github.com/yt-dlp/yt-dlp/releases and drop
it into this folder. The file itself is intentionally NOT committed.
