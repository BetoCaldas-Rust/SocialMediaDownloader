use std::path::PathBuf;

use crate::services::log_buffer::LogLevel;
use crate::services::traits::{
    BatchItem, ChannelOrder, HistoryEntry, TranscriptOrder, VideoQuality,
};

#[derive(Debug, Clone)]
pub enum Effect {
    ReloadLocale(String),
    PushLog {
        level: LogLevel,
        source: String,
        message: String,
    },
    FetchMetadata {
        url: String,
    },
    StartDownload {
        url: String,
        quality: VideoQuality,
    },
    CancelActive,
    CancelTranscript,
    FetchTranscript {
        order: TranscriptOrder,
    },
    FetchChannelPreview {
        order: ChannelOrder,
    },
    CancelChannel,
    DownloadBatch {
        items: Vec<BatchItem>,
        include_video: bool,
        include_transcript: bool,
        quality: VideoQuality,
        transcript: TranscriptOrder,
    },
    RevealInFolder(PathBuf),
    RecordHistory(Box<HistoryEntry>),
    RetryEntry(Box<HistoryEntry>),
    ClearHistory,
}
