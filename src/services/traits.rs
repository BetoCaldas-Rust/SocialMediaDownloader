#![allow(dead_code)]

use crate::services::log_buffer::LogLevel;

#[derive(Debug, Clone, Default)]
pub struct VideoMetadata {
    pub title: String,
    pub duration_secs: u64,
}

#[derive(Debug, Clone, Default)]
pub struct DownloadOrder {
    pub url: String,
}

#[derive(Debug, Clone, Default)]
pub struct DownloadTicket {
    pub id: String,
}

#[derive(Debug, Clone, Default)]
pub struct TranscriptOrder {
    pub url: String,
}

#[derive(Debug, Clone, Default)]
pub struct TranscriptDraft {
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct ChannelOrder {
    pub url: String,
}

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
    async fn download(&self, order: DownloadOrder) -> Result<DownloadTicket, String>;
}

#[async_trait::async_trait]
pub trait Transcriber: Send + Sync {
    async fn transcribe(&self, order: TranscriptOrder) -> Result<TranscriptDraft, String>;
}

#[async_trait::async_trait]
pub trait ChannelProvider: Send + Sync {
    async fn describe(&self, order: ChannelOrder) -> Result<ChannelSummary, String>;
}

pub trait HistoryStore: Send + Sync {
    fn entries(&self) -> Vec<String>;
    fn record(&self, entry: String);
}

pub trait LocaleProvider: Send + Sync {
    fn available(&self) -> Vec<String>;
    fn current(&self) -> String;
    fn set(&self, locale: &str) -> bool;
    fn translate(&self, key: &str) -> String;
}

pub trait LogSink: Send + Sync {
    fn push(&self, level: LogLevel, source: &str, message: &str);
}
