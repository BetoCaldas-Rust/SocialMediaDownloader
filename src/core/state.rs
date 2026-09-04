use std::path::PathBuf;

use chrono::NaiveDate;

use crate::services::log_buffer::LogEntry;
use crate::services::traits::{
    ChannelPreview, Container, TranscriptFormat, TranscriptOrder, TranscriptResult, VideoMetadata,
    VideoQuality,
};
use crate::storage::config::{AppConfig, LogLevelSetting};

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
    pub last_transcript_order: Option<TranscriptOrder>,
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
            last_transcript_order: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistoryFilter {
    #[default]
    All,
    Videos,
    Transcripts,
    Failures,
}

impl HistoryFilter {
    pub fn locale_key(self) -> &'static str {
        match self {
            HistoryFilter::All => "history_filter_all",
            HistoryFilter::Videos => "history_filter_videos",
            HistoryFilter::Transcripts => "history_filter_transcripts",
            HistoryFilter::Failures => "history_filter_failures",
        }
    }

    pub fn ordered() -> [HistoryFilter; 4] {
        [
            HistoryFilter::All,
            HistoryFilter::Videos,
            HistoryFilter::Transcripts,
            HistoryFilter::Failures,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HistorySort {
    #[default]
    Recent,
    Largest,
    Name,
}

impl HistorySort {
    pub fn locale_key(self) -> &'static str {
        match self {
            HistorySort::Recent => "history_sort_recent",
            HistorySort::Largest => "history_sort_largest",
            HistorySort::Name => "history_sort_name",
        }
    }

    pub fn ordered() -> [HistorySort; 3] {
        [HistorySort::Recent, HistorySort::Largest, HistorySort::Name]
    }
}

#[derive(Debug, Clone, Default)]
pub struct HistoryState {
    pub filter: HistoryFilter,
    pub query: String,
    pub sort: HistorySort,
    pub confirm_clear: bool,
}

#[derive(Debug, Clone)]
pub struct ConsoleState {
    pub show_error: bool,
    pub show_warn: bool,
    pub show_info: bool,
    pub show_debug: bool,
    pub source: String,
    pub query: String,
    pub paused: bool,
    pub autoscroll: bool,
    pub paused_len: usize,
    pub frozen: Option<Vec<LogEntry>>,
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self {
            show_error: true,
            show_warn: true,
            show_info: true,
            show_debug: false,
            source: "all".to_string(),
            query: String::new(),
            paused: false,
            autoscroll: true,
            paused_len: 0,
            frozen: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsState {
    pub locale: String,
    pub download_dir: Option<PathBuf>,
    pub quality_default: VideoQuality,
    pub organize_by_channel: bool,
    pub simultaneous: u8,
    pub transcript_lang: String,
    pub transcript_format: TranscriptFormat,
    pub transcript_fallback: Option<String>,
    pub transcript_accept_auto: bool,
    pub transcript_timestamps: bool,
    pub start_with_windows: bool,
    pub minimize_to_tray: bool,
    pub notify_on_complete: bool,
    pub log_level: LogLevelSetting,
    pub save_logs: bool,
    pub ytdlp_version: Option<String>,
    pub ytdlp_error: Option<String>,
    pub ytdlp_checking: bool,
    pub autostart_actual: Option<bool>,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self::from_config(&AppConfig::default())
    }
}

impl SettingsState {
    pub fn from_config(config: &AppConfig) -> Self {
        Self {
            locale: config.locale.clone(),
            download_dir: config.download_dir.clone(),
            quality_default: config.quality_default,
            organize_by_channel: config.organize_by_channel,
            simultaneous: config.simultaneous,
            transcript_lang: config.transcript_lang.clone(),
            transcript_format: config.transcript_format,
            transcript_fallback: config.transcript_fallback.clone(),
            transcript_accept_auto: config.transcript_accept_auto,
            transcript_timestamps: config.transcript_timestamps,
            start_with_windows: config.start_with_windows,
            minimize_to_tray: config.minimize_to_tray,
            notify_on_complete: config.notify_on_complete,
            log_level: config.log_level,
            save_logs: config.save_logs,
            ytdlp_version: None,
            ytdlp_error: None,
            ytdlp_checking: false,
            autostart_actual: None,
        }
    }

    pub fn apply_to_config(&self, config: &mut AppConfig) {
        config.locale = self.locale.clone();
        config.download_dir = self.download_dir.clone();
        config.quality_default = self.quality_default;
        config.organize_by_channel = self.organize_by_channel;
        config.simultaneous = self.simultaneous;
        config.transcript_lang = self.transcript_lang.clone();
        config.transcript_format = self.transcript_format;
        config.transcript_fallback = self.transcript_fallback.clone();
        config.transcript_accept_auto = self.transcript_accept_auto;
        config.transcript_timestamps = self.transcript_timestamps;
        config.start_with_windows = self.start_with_windows;
        config.minimize_to_tray = self.minimize_to_tray;
        config.notify_on_complete = self.notify_on_complete;
        config.log_level = self.log_level;
        config.save_logs = self.save_logs;
        config.validate();
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub id: String,
    pub message_key: String,
    pub detail: String,
}

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
    pub notices: Vec<Notice>,
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
            notices: Vec::new(),
        }
    }
}

/// Seeds tab-local session state from settings at startup. Settings own
/// the persisted defaults; the video/channel/transcript tabs may diverge
/// afterwards without writing back to the config.
pub fn apply_config_defaults(state: &mut AppState, config: &AppConfig) {
    state.video.quality = config.quality_default;
    state.channel.quality = config.quality_default;
    state.transcript.lang = config.transcript_lang.clone();
    state.transcript.format = config.transcript_format;
    state.transcript.fallback = config
        .transcript_fallback
        .clone()
        .unwrap_or_else(|| "off".to_string());
    state.transcript.accept_auto = config.transcript_accept_auto;
    state.transcript.timestamps = config.transcript_timestamps;
}
