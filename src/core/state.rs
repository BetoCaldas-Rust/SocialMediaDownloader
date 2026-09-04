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

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct VideoState {
    pub url: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TranscriptState {
    pub input: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct ChannelState {
    pub input: String,
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
