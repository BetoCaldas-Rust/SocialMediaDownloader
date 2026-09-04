use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TranscriptFormat {
    #[default]
    Srt,
    Vtt,
    Txt,
}

impl TranscriptFormat {
    pub fn locale_key(self) -> &'static str {
        match self {
            TranscriptFormat::Srt => "transcript_format_srt",
            TranscriptFormat::Vtt => "transcript_format_vtt",
            TranscriptFormat::Txt => "transcript_format_txt",
        }
    }

    pub fn ordered() -> [TranscriptFormat; 3] {
        [
            TranscriptFormat::Srt,
            TranscriptFormat::Vtt,
            TranscriptFormat::Txt,
        ]
    }

    pub fn extension(self) -> &'static str {
        match self {
            TranscriptFormat::Srt => "srt",
            TranscriptFormat::Vtt => "vtt",
            TranscriptFormat::Txt => "txt",
        }
    }

    pub fn convert_target(self) -> &'static str {
        match self {
            TranscriptFormat::Srt => "srt",
            TranscriptFormat::Vtt => "vtt",
            TranscriptFormat::Txt => "srt",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptOrder {
    pub url: String,
    pub lang: String,
    pub format: TranscriptFormat,
    pub fallback: Option<String>,
    pub accept_auto: bool,
    pub timestamps: bool,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TranscriptResult {
    pub path: PathBuf,
    pub lang_used: String,
    pub auto_generated: bool,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoKind {
    #[default]
    Video,
    Short,
    Live,
}

impl VideoKind {
    pub fn locale_key(self) -> &'static str {
        match self {
            VideoKind::Video => "channel_kind_video",
            VideoKind::Short => "channel_kind_shorts",
            VideoKind::Live => "channel_kind_live",
        }
    }

    pub fn ordered() -> [VideoKind; 3] {
        [VideoKind::Video, VideoKind::Short, VideoKind::Live]
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChannelOrder {
    pub url: String,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub kinds: Vec<VideoKind>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChannelVideo {
    pub id: String,
    pub title: String,
    pub date_label: String,
    pub duration_secs: u64,
    pub url: String,
    pub kind: VideoKind,
}

#[derive(Debug, Clone, Default)]
pub struct ChannelPreview {
    pub name: String,
    pub handle: Option<String>,
    pub subs: Option<String>,
    pub videos: Vec<ChannelVideo>,
}

#[derive(Debug, Clone, Default)]
pub struct BatchItem {
    pub url: String,
    pub title: String,
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

#[async_trait::async_trait]
pub trait Transcriber: Send + Sync {
    async fn transcribe(&self, order: &TranscriptOrder) -> Result<TranscriptResult, String>;
}

#[async_trait::async_trait]
pub trait ChannelProvider: Send + Sync {
    async fn preview(&self, order: &ChannelOrder) -> Result<ChannelPreview, String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EntryKind {
    #[default]
    Video,
    Transcript,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EntryStatus {
    #[default]
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub kind: EntryKind,
    pub name: String,
    pub source: String,
    pub url: String,
    pub detail: Option<String>,
    pub finished_at_ms: i64,
    pub size_bytes: Option<u64>,
    pub status: EntryStatus,
    pub error: Option<String>,
    pub path: Option<PathBuf>,
    pub transcript_order: Option<TranscriptOrder>,
}

#[derive(Debug, Clone)]
pub enum RetryPlan {
    Video { url: String },
    Transcript { order: TranscriptOrder },
}

pub fn retry_plan(entry: &HistoryEntry) -> RetryPlan {
    match entry.kind {
        EntryKind::Video => RetryPlan::Video {
            url: entry.url.clone(),
        },
        EntryKind::Transcript => RetryPlan::Transcript {
            order: entry.transcript_order.clone().unwrap_or(TranscriptOrder {
                url: entry.url.clone(),
                ..Default::default()
            }),
        },
    }
}

static HISTORY_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn new_history_id() -> String {
    let millis = now_ms();
    let counter = HISTORY_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{millis}-{counter}")
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub fn file_size(path: &Path) -> Option<u64> {
    std::fs::metadata(path).ok().map(|meta| meta.len())
}

pub fn file_name(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
}

/// Prefers metadata channel/title context when available, else the URL
/// host, else a dash placeholder.
pub fn history_source(channel: Option<&str>, url: &str) -> String {
    if let Some(context) = channel {
        if !context.trim().is_empty() {
            return context.trim().to_string();
        }
    }
    url_host(url).unwrap_or_else(|| "—".to_string())
}

pub fn url_host(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return None;
    }
    let without_scheme = match trimmed.split_once("://") {
        Some((_, rest)) => rest,
        None => trimmed,
    };
    let mut host = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .to_string();
    if let Some((_, after)) = host.rsplit_once('@') {
        host = after.to_string();
    }
    if let Some((bare, _)) = host.split_once(':') {
        host = bare.to_string();
    }
    if host.trim().is_empty() {
        return None;
    }
    Some(host)
}

pub trait HistoryStore: Send + Sync {
    fn entries(&self) -> Vec<HistoryEntry>;
    fn record(&self, entry: HistoryEntry);
    fn clear(&self);
    fn len(&self) -> usize {
        self.entries().len()
    }
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
