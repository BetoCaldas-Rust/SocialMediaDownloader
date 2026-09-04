use std::path::PathBuf;

use crate::services::log_buffer::LogLevel;
use crate::services::traits::{TranscriptOrder, VideoQuality};

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
    RevealInFolder(PathBuf),
}
