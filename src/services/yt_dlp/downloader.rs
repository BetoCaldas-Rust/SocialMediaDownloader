use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::UnboundedSender;

use crate::services::log_buffer::LogLevel;
use crate::services::traits::{
    DownloadOrder, DownloadProgress, DownloadTicket, Downloader, LogSink, VideoQuality,
};
use crate::services::yt_dlp::binary::{hidden_command, resolve_binary};
use crate::services::yt_dlp::progress::{parse_progress_line, ProgressTick};

/// Single definition of the quality -> `-f` mapping.
/// Every tier merges to MP4 so the output plays everywhere:
/// - Best: best video + best audio, no cap
/// - 1080p/720p: best streams at or below the height cap, with
///   fallbacks so single-file formats still match.
pub fn format_arg(quality: VideoQuality) -> &'static str {
    match quality {
        VideoQuality::Best => "bv*+ba/b",
        VideoQuality::Capped1080 => "bv*[height<=1080]+ba/b[height<=1080]/b[height<=1080]/b",
        VideoQuality::Capped720 => "bv*[height<=720]+ba/b[height<=720]/b[height<=720]/b",
    }
}

/// `~/Downloads/SocialMediaDownloader` until F6 owns the location.
pub fn default_download_dir() -> PathBuf {
    let base = directories::UserDirs::new()
        .and_then(|dirs| dirs.download_dir().map(Path::to_path_buf))
        .unwrap_or_else(std::env::temp_dir);
    base.join("SocialMediaDownloader")
}

pub struct YtDlpDownloader {
    log: Arc<dyn LogSink>,
}

impl YtDlpDownloader {
    pub fn new(log: Arc<dyn LogSink>) -> Self {
        Self { log }
    }

    fn note(&self, level: LogLevel, message: &str) {
        self.log.push(level, "yt-dlp", message);
    }
}

#[async_trait::async_trait]
impl Downloader for YtDlpDownloader {
    async fn download(
        &self,
        order: DownloadOrder,
        progress: UnboundedSender<DownloadProgress>,
    ) -> Result<DownloadTicket, String> {
        let url = order.url.trim().to_string();
        if url.is_empty() {
            return Err("video_error_empty_url".to_string());
        }
        let binary = resolve_binary()?;
        let dir = ensure_dir(&order.output_dir, &*self.log)?;
        let mut child = spawn_download(&binary, &url, order.quality, &dir, &*self.log)?;
        let stdout = take_pipe(child.stdout.take(), "stdout")?;
        drain_stderr(child.stderr.take(), self.log.clone());
        let filename = pump_stdout(stdout, &progress).await;
        let status = child
            .wait()
            .await
            .map_err(|_| "video_error_download".to_string())?;
        if !status.success() {
            self.note(LogLevel::Warn, "download exited non-zero");
            return Err("video_error_download".to_string());
        }
        Ok(ticket_for(&dir, filename))
    }
}

/// Dropping the task that owns the child kills yt-dlp
/// (`kill_on_drop`), which is how Cancel works from the store.
fn spawn_download(
    binary: &PathBuf,
    url: &str,
    quality: VideoQuality,
    dir: &Path,
    log: &dyn LogSink,
) -> Result<tokio::process::Child, String> {
    hidden_command(binary)
        .arg("--newline")
        .arg("--no-playlist")
        .arg("--no-warnings")
        .arg("-f")
        .arg(format_arg(quality))
        .arg("--merge-output-format")
        .arg("mp4")
        .arg("-P")
        .arg(dir)
        .arg("-o")
        .arg("%(title).150B [%(id)s].%(ext)s")
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| {
            log.push(LogLevel::Error, "download", "failed to spawn yt-dlp");
            "video_error_download".to_string()
        })
}

fn ensure_dir(dir: &Path, log: &dyn LogSink) -> Result<PathBuf, String> {
    if std::fs::create_dir_all(dir).is_err() {
        log.push(LogLevel::Error, "download", "unable to create output dir");
        return Err("video_error_download".to_string());
    }
    Ok(dir.to_path_buf())
}

fn take_pipe<T>(pipe: Option<T>, what: &str) -> Result<T, String> {
    pipe.ok_or_else(|| {
        tracing::warn!(target: "download", "missing child {what}");
        "video_error_download".to_string()
    })
}

fn drain_stderr(pipe: Option<tokio::process::ChildStderr>, log: Arc<dyn LogSink>) {
    let Some(stderr) = pipe else { return };
    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if !line.trim().is_empty() {
                log.push(LogLevel::Debug, "yt-dlp", line.trim());
            }
        }
    });
}

async fn pump_stdout(
    stdout: tokio::process::ChildStdout,
    progress: &UnboundedSender<DownloadProgress>,
) -> Option<String> {
    let mut lines = BufReader::new(stdout).lines();
    let mut filename = None;
    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(tick) = parse_progress_line(&line) {
            if tick.filename.is_some() {
                filename = tick.filename.clone();
            }
            if tick.percent.unwrap_or(0.0) > 0.0 || tick.filename.is_some() {
                let _ = progress.send(tick_as_progress(tick));
            }
        }
    }
    filename
}

fn tick_as_progress(tick: ProgressTick) -> DownloadProgress {
    DownloadProgress {
        percent: tick.percent.unwrap_or(0.0),
        speed: tick.speed,
        eta: tick.eta,
        filename: tick.filename,
    }
}

fn ticket_for(dir: &Path, filename: Option<String>) -> DownloadTicket {
    match filename {
        Some(name) => {
            let path = dir.join(&name);
            DownloadTicket { id: name, path }
        }
        None => DownloadTicket {
            id: "download".to_string(),
            path: dir.to_path_buf(),
        },
    }
}
