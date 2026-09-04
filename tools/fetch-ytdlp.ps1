# Downloads the yt-dlp sidecar binary into resources/bin/.
# The binary itself is git-ignored; run this before dev or packaging.
# Usage: powershell -ExecutionPolicy Bypass -File tools/fetch-ytdlp.ps1 [-Force]
param([switch]$Force)

$ErrorActionPreference = "Stop"
$dest = Join-Path $PSScriptRoot "..\resources\bin\yt-dlp.exe"
$dest = [System.IO.Path]::GetFullPath($dest)

if ((Test-Path $dest) -and (-not $Force)) {
    Write-Host "yt-dlp already present at $dest (use -Force to re-download)."
    exit 0
}

Write-Host "Resolving latest yt-dlp release..."
$release = Invoke-RestMethod "https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest"
$asset = $release.assets | Where-Object { $_.name -eq "yt-dlp.exe" } | Select-Object -First 1
if ($null -eq $asset) {
    Write-Error "yt-dlp.exe asset not found in latest release."
    exit 1
}

New-Item -ItemType Directory -Path (Split-Path $dest) -Force | Out-Null
Write-Host "Downloading $($asset.browser_download_url) ..."
Invoke-WebRequest $asset.browser_download_url -OutFile $dest
Write-Host "Saved to $dest"
& $dest --version
