use super::state::Screen;
use crate::services::traits::{
    Container, DownloadProgress, DownloadTicket, VideoMetadata, VideoQuality,
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum TranscriptIntent {
    SetInput(String),
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
    #[allow(dead_code)]
    Transcript(TranscriptIntent),
    #[allow(dead_code)]
    Channel(ChannelIntent),
    #[allow(dead_code)]
    History(HistoryIntent),
    #[allow(dead_code)]
    Console(ConsoleIntent),
    Settings(SettingsIntent),
}
