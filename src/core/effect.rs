use std::path::PathBuf;

use crate::services::log_buffer::LogLevel;
use crate::services::traits::VideoQuality;

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
    RevealInFolder(PathBuf),
}
