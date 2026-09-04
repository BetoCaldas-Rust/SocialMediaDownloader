#![allow(dead_code)]

use super::state::Screen;

#[derive(Debug, Clone)]
pub enum VideoIntent {
    SetUrl(String),
}

#[derive(Debug, Clone)]
pub enum TranscriptIntent {
    SetInput(String),
}

#[derive(Debug, Clone)]
pub enum ChannelIntent {
    SetInput(String),
}

#[derive(Debug, Clone)]
pub enum HistoryIntent {
    SetFilter(String),
}

#[derive(Debug, Clone)]
pub enum ConsoleIntent {
    Clear,
}

#[derive(Debug, Clone)]
pub enum SettingsIntent {
    ChangeLocale(String),
}

#[derive(Debug, Clone)]
pub enum AppIntent {
    Navigate(Screen),
    Video(VideoIntent),
    Transcript(TranscriptIntent),
    Channel(ChannelIntent),
    History(HistoryIntent),
    Console(ConsoleIntent),
    Settings(SettingsIntent),
}
