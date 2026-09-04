use std::sync::RwLock;

use tokio::sync::mpsc::UnboundedSender;

use super::json_history::HISTORY_CAP;
use super::traits::{
    ChannelOrder, ChannelPreview, ChannelProvider, DownloadOrder, DownloadProgress, DownloadTicket,
    Downloader, HistoryEntry, HistoryStore, MetadataProvider, Transcriber, TranscriptOrder,
    TranscriptResult, VideoMetadata,
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
#[derive(Default)]
pub struct StubHistoryStore {
    entries: RwLock<Vec<HistoryEntry>>,
}

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
    fn entries(&self) -> Vec<HistoryEntry> {
        self.entries
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    fn record(&self, entry: HistoryEntry) {
        if let Ok(mut guard) = self.entries.write() {
            guard.push(entry);
            if guard.len() > HISTORY_CAP {
                let excess = guard.len() - HISTORY_CAP;
                guard.drain(..excess);
            }
        }
    }

    fn clear(&self) {
        if let Ok(mut guard) = self.entries.write() {
            guard.clear();
        }
    }

    fn len(&self) -> usize {
        self.entries.read().map(|guard| guard.len()).unwrap_or(0)
    }
}
