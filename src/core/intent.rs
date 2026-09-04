use std::path::PathBuf;

use super::state::{ChannelInclude, DatePreset, HistoryFilter, HistorySort, Screen};
use crate::services::log_buffer::LogLevel;
use crate::services::traits::{
    ChannelPreview, Container, DownloadProgress, DownloadTicket, TranscriptFormat,
    TranscriptResult, VideoKind, VideoMetadata, VideoQuality,
};
use crate::storage::config::LogLevelSetting;

#[derive(Debug, Clone)]
pub enum VideoIntent {
    SetUrl(String),
    SetQuality(VideoQuality),
    SetContainer(Container),
    SetTranscript(bool),
    FetchMetadata,
    StartDownload,
    CancelDownload,
    MetadataReceived(Result<VideoMetadata, String>),
    ProgressTick(DownloadProgress),
    DownloadFinished(Result<DownloadTicket, String>),
    RevealOutput,
    DismissError,
}

#[derive(Debug, Clone)]
pub enum TranscriptIntent {
    SetInput(String),
    SetLang(String),
    SetFormat(TranscriptFormat),
    SetFallback(String),
    ToggleAuto,
    ToggleTimestamps,
    FetchTranscript,
    TranscriptFinished(Result<TranscriptResult, String>),
    Retry,
    Dismiss,
    RevealRecent(usize),
}

#[derive(Debug, Clone)]
pub enum ChannelIntent {
    SetInput(String),
    SetPreset(DatePreset),
    SetFromText(String),
    SetToText(String),
    ToggleKind(VideoKind),
    ToggleInclude(ChannelInclude),
    ToggleSelect(usize),
    SelectAll,
    SelectNone,
    SetQuality(VideoQuality),
    FetchPreview,
    PreviewReceived(Result<ChannelPreview, String>),
    DownloadBatch,
    BatchTick {
        index: usize,
        tick: DownloadProgress,
    },
    BatchItemFinished {
        index: usize,
        video_result: Option<Result<DownloadTicket, String>>,
        transcript_result: Option<Result<TranscriptResult, String>>,
    },
    CancelBatch,
    Retry,
    Dismiss,
}

#[derive(Debug, Clone)]
pub enum HistoryIntent {
    SetFilter(HistoryFilter),
    SetQuery(String),
    SetSort(HistorySort),
    RequestClear,
    CancelClear,
    ClearHistory,
    OpenDownloadsFolder,
    RevealEntry(String),
    RetryEntry(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ConsoleIntent {
    ToggleLevel(LogLevel),
    SetSource(String),
    SetQuery(String),
    TogglePause,
    ToggleAutoscroll,
    CopyAll,
    ExportLogs,
    ExportPathChosen(Option<PathBuf>),
    Clear,
    JumpToBottom,
}

#[derive(Debug, Clone)]
pub enum SettingsIntent {
    ChangeLocale(String),
    PickDownloadDir,
    DownloadDirPicked(Option<PathBuf>),
    SetDownloadDir(Option<PathBuf>),
    SetQualityDefault(VideoQuality),
    SetOrganize(bool),
    SetSimultaneous(u8),
    SetTranscriptLang(String),
    SetTranscriptFormat(TranscriptFormat),
    SetTranscriptFallback(Option<String>),
    ToggleTranscriptAuto,
    ToggleTranscriptTimestamps,
    SetAutostart(bool),
    AutostartVerified(Option<bool>),
    SetMinimizeTray(bool),
    SetNotify(bool),
    SetLogLevel(LogLevelSetting),
    SetSaveLogs(bool),
    CheckYtDlp,
    YtDlpVersionReceived(Result<String, String>),
    UpdateYtDlp,
    YtDlpUpdated(Result<String, String>),
}

#[derive(Debug, Clone)]
pub enum AppIntent {
    Navigate(Screen),
    Video(VideoIntent),
    Transcript(TranscriptIntent),
    Channel(ChannelIntent),
    #[allow(dead_code)]
    History(HistoryIntent),
    #[allow(dead_code)]
    Console(ConsoleIntent),
    Settings(SettingsIntent),
    DismissNotice(String),
}
