use tokio::sync::mpsc::UnboundedSender;

use super::traits::{
    ChannelOrder, ChannelPreview, ChannelProvider, DownloadOrder, DownloadProgress, DownloadTicket,
    Downloader, HistoryStore, MetadataProvider, Transcriber, TranscriptOrder, TranscriptResult,
    VideoMetadata,
};

#[allow(dead_code)]
pub struct StubMetadataProvider;

#[allow(dead_code)]
pub struct StubDownloader;

#[allow(dead_code)]
pub struct StubTranscriber;

#[allow(dead_code)]
pub struct StubChannelProvider;

#[allow(dead_code)]
pub struct StubHistoryStore;

#[async_trait::async_trait]
impl MetadataProvider for StubMetadataProvider {
    async fn fetch_metadata(&self, _url: &str) -> Result<VideoMetadata, String> {
        Err("not implemented yet (F2+)".to_string())
    }
}

#[async_trait::async_trait]
impl Downloader for StubDownloader {
    async fn download(
        &self,
        _order: DownloadOrder,
        _progress: UnboundedSender<DownloadProgress>,
    ) -> Result<DownloadTicket, String> {
        Err("not implemented yet (F2+)".to_string())
    }
}

#[async_trait::async_trait]
impl Transcriber for StubTranscriber {
    async fn transcribe(&self, _order: &TranscriptOrder) -> Result<TranscriptResult, String> {
        Err("transcript_error_fetch".to_string())
    }
}

#[async_trait::async_trait]
impl ChannelProvider for StubChannelProvider {
    async fn preview(&self, _order: &ChannelOrder) -> Result<ChannelPreview, String> {
        Err("channel_error_fetch".to_string())
    }
}

impl HistoryStore for StubHistoryStore {
    fn entries(&self) -> Vec<String> {
        Vec::new()
    }

    fn record(&self, _entry: String) {}
}
