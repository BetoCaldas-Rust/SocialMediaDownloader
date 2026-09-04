use super::state::Screen;
use crate::services::traits::{
    Container, DownloadProgress, DownloadTicket, TranscriptFormat, TranscriptResult, VideoMetadata,
    VideoQuality,
};

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

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ChannelIntent {
    SetInput(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum HistoryIntent {
    SetFilter(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ConsoleIntent {
    Clear,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum SettingsIntent {
    ChangeLocale(String),
}

#[derive(Debug, Clone)]
pub enum AppIntent {
    Navigate(Screen),
    Video(VideoIntent),
    Transcript(TranscriptIntent),
    #[allow(dead_code)]
    Channel(ChannelIntent),
    #[allow(dead_code)]
    History(HistoryIntent),
    #[allow(dead_code)]
    Console(ConsoleIntent),
    Settings(SettingsIntent),
}
