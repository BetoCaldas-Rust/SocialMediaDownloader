use std::path::PathBuf;

use tokio::sync::mpsc::UnboundedSender;

use crate::services::log_buffer::LogLevel;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VideoMetadata {
    pub title: String,
    pub channel: String,
    pub duration_secs: u64,
    pub view_count: Option<u64>,
    pub size_label: Option<String>,
}

/// Quality chosen in the UI. The yt-dlp `-f` mapping lives in
/// [`crate::services::yt_dlp::downloader::format_arg`] so the UI,
/// the order DTO and the sidecar call share one definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoQuality {
    #[default]
    Best,
    Capped1080,
    Capped720,
}

impl VideoQuality {
    pub fn locale_key(self) -> &'static str {
        match self {
            VideoQuality::Best => "video_quality_best",
            VideoQuality::Capped1080 => "video_quality_1080",
            VideoQuality::Capped720 => "video_quality_720",
        }
    }

    pub fn ordered() -> [VideoQuality; 3] {
        [
            VideoQuality::Best,
            VideoQuality::Capped1080,
            VideoQuality::Capped720,
        ]
    }
}

/// Output container. Only MP4 (merged video+audio) in F2; audio-only
/// extraction is an honest F4+ placeholder in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Container {
    #[default]
    Mp4,
}

impl Container {
    pub fn locale_key(self) -> &'static str {
        match self {
            Container::Mp4 => "video_format_mp4",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DownloadOrder {
    pub url: String,
    pub quality: VideoQuality,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, Default)]
pub struct DownloadTicket {
    /// Reserved for history (F6+); the view uses `path` today.
    #[allow(dead_code)]
    pub id: String,
    pub path: PathBuf,
}

/// One parsed progress sample. The downloader streams these over the
/// channel; the store forwards each as a `VideoIntent::ProgressTick`.
#[derive(Debug, Clone, Default)]
pub struct DownloadProgress {
    /// 0.0 - 100.0 scale, matching yt-dlp's own percent readout.
    pub percent: f32,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub filename: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TranscriptOrder {
    pub url: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TranscriptDraft {
    pub text: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ChannelOrder {
    pub url: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ChannelSummary {
    pub name: String,
}

#[async_trait::async_trait]
pub trait MetadataProvider: Send + Sync {
    async fn fetch_metadata(&self, url: &str) -> Result<VideoMetadata, String>;
}

#[async_trait::async_trait]
pub trait Downloader: Send + Sync {
    async fn download(
        &self,
        order: DownloadOrder,
        progress: UnboundedSender<DownloadProgress>,
    ) -> Result<DownloadTicket, String>;
}

#[allow(dead_code)]
#[async_trait::async_trait]
pub trait Transcriber: Send + Sync {
    async fn transcribe(&self, order: TranscriptOrder) -> Result<TranscriptDraft, String>;
}

#[allow(dead_code)]
#[async_trait::async_trait]
pub trait ChannelProvider: Send + Sync {
    async fn describe(&self, order: ChannelOrder) -> Result<ChannelSummary, String>;
}

#[allow(dead_code)]
pub trait HistoryStore: Send + Sync {
    fn entries(&self) -> Vec<String>;
    fn record(&self, entry: String);
}

pub trait LocaleProvider: Send + Sync {
    fn available(&self) -> Vec<String>;
    fn current(&self) -> String;
    fn set(&self, locale: &str) -> bool;
    #[allow(dead_code)]
    fn translate(&self, key: &str) -> String;
}

pub trait LogSink: Send + Sync {
    fn push(&self, level: LogLevel, source: &str, message: &str);
}
