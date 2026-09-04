use std::path::PathBuf;

use chrono::NaiveDate;

use crate::services::traits::{
    ChannelPreview, Container, TranscriptFormat, TranscriptResult, VideoMetadata, VideoQuality,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Video,
    Transcript,
    Channel,
    History,
    Console,
    Settings,
}

impl Screen {
    pub fn ordered() -> [Screen; 6] {
        [
            Screen::Video,
            Screen::Transcript,
            Screen::Channel,
            Screen::History,
            Screen::Console,
            Screen::Settings,
        ]
    }

    pub fn nav_key(self) -> &'static str {
        match self {
            Screen::Video => "nav_video",
            Screen::Transcript => "nav_transcript",
            Screen::Channel => "nav_channel",
            Screen::History => "nav_history",
            Screen::Console => "nav_console",
            Screen::Settings => "nav_settings",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoStatus {
    #[default]
    Idle,
    Resolving,
    Ready,
    Downloading,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Default)]
pub struct VideoState {
    pub url: String,
    pub status: VideoStatus,
    pub metadata: Option<VideoMetadata>,
    /// 0.0 - 100.0 download progress.
    pub progress: f32,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub filename: Option<String>,
    pub output_path: Option<PathBuf>,
    pub quality: VideoQuality,
    pub container: Container,
    pub include_transcript: bool,
    /// Locale key for the failure message; the view translates it.
    pub error_key: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TranscriptStatus {
    #[default]
    Idle,
    Resolving,
    Downloading,
    Completed,
    Failed,
}

impl TranscriptStatus {
    pub fn is_busy(self) -> bool {
        matches!(
            self,
            TranscriptStatus::Resolving | TranscriptStatus::Downloading
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptState {
    pub input: String,
    pub lang: String,
    pub format: TranscriptFormat,
    pub fallback: String,
    pub accept_auto: bool,
    pub timestamps: bool,
    pub status: TranscriptStatus,
    pub result: Option<TranscriptResult>,
    pub error: Option<String>,
    pub recents: Vec<TranscriptResult>,
}

impl Default for TranscriptState {
    fn default() -> Self {
        Self {
            input: String::new(),
            lang: "pt".to_string(),
            format: TranscriptFormat::Srt,
            fallback: "en".to_string(),
            accept_auto: true,
            timestamps: true,
            status: TranscriptStatus::Idle,
            result: None,
            error: None,
            recents: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DatePreset {
    Last7,
    #[default]
    Last30,
    Last60,
    Year,
    All,
    Custom,
}

impl DatePreset {
    pub fn locale_key(self) -> &'static str {
        match self {
            DatePreset::Last7 => "channel_preset_last7",
            DatePreset::Last30 => "channel_preset_last30",
            DatePreset::Last60 => "channel_preset_last60",
            DatePreset::Year => "channel_preset_year",
            DatePreset::All => "channel_preset_all",
            DatePreset::Custom => "channel_preset_custom",
        }
    }

    pub fn ordered() -> [DatePreset; 6] {
        [
            DatePreset::Last7,
            DatePreset::Last30,
            DatePreset::Last60,
            DatePreset::Year,
            DatePreset::All,
            DatePreset::Custom,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChannelStatus {
    #[default]
    Idle,
    Loading,
    Ready,
    Downloading,
    Completed,
    Failed,
}

impl ChannelStatus {
    pub fn is_busy(self) -> bool {
        matches!(self, ChannelStatus::Loading | ChannelStatus::Downloading)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChannelInclude {
    #[default]
    Video,
    Transcript,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BatchItemStatus {
    #[default]
    Queued,
    Active,
    Done,
    Failed,
}

#[derive(Debug, Clone, Default)]
pub struct BatchItemState {
    pub url: String,
    pub title: String,
    pub status: BatchItemStatus,
    pub progress: f32,
    pub speed: Option<String>,
    pub error_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChannelState {
    pub input: String,
    pub preset: DatePreset,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
    pub from_text: String,
    pub to_text: String,
    pub show_video: bool,
    pub show_shorts: bool,
    pub show_lives: bool,
    pub include_video: bool,
    pub include_transcript: bool,
    pub quality: VideoQuality,
    pub status: ChannelStatus,
    pub preview: Option<ChannelPreview>,
    pub selected: Vec<bool>,
    pub items: Vec<BatchItemState>,
    pub error_key: Option<String>,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            input: String::new(),
            preset: DatePreset::Last30,
            from: None,
            to: None,
            from_text: String::new(),
            to_text: String::new(),
            show_video: true,
            show_shorts: true,
            show_lives: true,
            include_video: true,
            include_transcript: true,
            quality: VideoQuality::default(),
            status: ChannelStatus::Idle,
            preview: None,
            selected: Vec::new(),
            items: Vec::new(),
            error_key: None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct HistoryState {
    pub filter: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ConsoleState {}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct SettingsState {}

#[derive(Debug, Clone)]
pub struct AppState {
    pub screen: Screen,
    pub locale: String,
    pub video: VideoState,
    pub transcript: TranscriptState,
    pub channel: ChannelState,
    pub history: HistoryState,
    pub console: ConsoleState,
    pub settings: SettingsState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            screen: Screen::Video,
            locale: "en-US".to_string(),
            video: VideoState::default(),
            transcript: TranscriptState::default(),
            channel: ChannelState::default(),
            history: HistoryState::default(),
            console: ConsoleState::default(),
            settings: SettingsState::default(),
        }
    }
}
