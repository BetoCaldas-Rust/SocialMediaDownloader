#![allow(dead_code)]

use super::traits::{
    ChannelOrder, ChannelProvider, ChannelSummary, DownloadOrder, DownloadTicket, Downloader,
    HistoryStore, MetadataProvider, Transcriber, TranscriptDraft, TranscriptOrder, VideoMetadata,
};

const PENDING: &str = "not implemented yet (F2+)";

pub struct StubMetadataProvider;

pub struct StubDownloader;

pub struct StubTranscriber;

pub struct StubChannelProvider;

pub struct StubHistoryStore;

#[async_trait::async_trait]
impl MetadataProvider for StubMetadataProvider {
    async fn fetch_metadata(&self, _url: &str) -> Result<VideoMetadata, String> {
        Err(PENDING.to_string())
    }
}

#[async_trait::async_trait]
impl Downloader for StubDownloader {
    async fn download(&self, _order: DownloadOrder) -> Result<DownloadTicket, String> {
        Err(PENDING.to_string())
    }
}

#[async_trait::async_trait]
impl Transcriber for StubTranscriber {
    async fn transcribe(&self, _order: TranscriptOrder) -> Result<TranscriptDraft, String> {
        Err(PENDING.to_string())
    }
}

#[async_trait::async_trait]
impl ChannelProvider for StubChannelProvider {
    async fn describe(&self, _order: ChannelOrder) -> Result<ChannelSummary, String> {
        Err(PENDING.to_string())
    }
}

impl HistoryStore for StubHistoryStore {
    fn entries(&self) -> Vec<String> {
        Vec::new()
    }

    fn record(&self, _entry: String) {}
}
