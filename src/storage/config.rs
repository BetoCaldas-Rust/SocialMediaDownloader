use std::path::PathBuf;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::services::log_buffer::LogLevel;
use crate::services::traits::{TranscriptFormat, VideoQuality};
use crate::services::yt_dlp::downloader::default_download_dir;

fn default_locale() -> String {
    "en-US".to_string()
}

fn default_true() -> bool {
    true
}

fn default_simultaneous() -> u8 {
    3
}

fn default_transcript_lang() -> String {
    "pt".to_string()
}

fn default_fallback() -> Option<String> {
    Some("en".to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LogLevelSetting {
    Error,
    Warn,
    #[default]
    Info,
    Debug,
}

impl LogLevelSetting {
    pub fn locale_key(self) -> &'static str {
        match self {
            LogLevelSetting::Error => "settings_loglevel_error",
            LogLevelSetting::Warn => "settings_loglevel_warn",
            LogLevelSetting::Info => "settings_loglevel_info",
            LogLevelSetting::Debug => "settings_loglevel_debug",
        }
    }

    pub fn ordered() -> [LogLevelSetting; 4] {
        [
            LogLevelSetting::Error,
            LogLevelSetting::Warn,
            LogLevelSetting::Info,
            LogLevelSetting::Debug,
        ]
    }

    pub fn tracing_filter(self) -> tracing_subscriber::filter::LevelFilter {
        use tracing_subscriber::filter::LevelFilter;
        match self {
            LogLevelSetting::Error => LevelFilter::ERROR,
            LogLevelSetting::Warn => LevelFilter::WARN,
            LogLevelSetting::Info => LevelFilter::INFO,
            LogLevelSetting::Debug => LevelFilter::DEBUG,
        }
    }

    /// Hierarchy helper kept for tests and future filters; the live
    /// tracing backend is driven by `tracing_filter` instead.
    #[allow(dead_code)]
    pub fn allows(self, level: LogLevel) -> bool {
        rank(level) <= rank_setting(self)
    }
}

#[allow(dead_code)]
fn rank(level: LogLevel) -> u8 {
    match level {
        LogLevel::Error => 0,
        LogLevel::Warn => 1,
        LogLevel::Info => 2,
        LogLevel::Debug => 3,
    }
}

#[allow(dead_code)]
fn rank_setting(setting: LogLevelSetting) -> u8 {
    match setting {
        LogLevelSetting::Error => 0,
        LogLevelSetting::Warn => 1,
        LogLevelSetting::Info => 2,
        LogLevelSetting::Debug => 3,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    #[serde(default = "default_locale")]
    pub locale: String,
    pub download_dir: Option<PathBuf>,
    pub quality_default: VideoQuality,
    #[serde(default = "default_true")]
    pub organize_by_channel: bool,
    #[serde(default = "default_simultaneous")]
    pub simultaneous: u8,
    #[serde(default = "default_transcript_lang")]
    pub transcript_lang: String,
    pub transcript_format: TranscriptFormat,
    #[serde(default = "default_fallback")]
    pub transcript_fallback: Option<String>,
    #[serde(default = "default_true")]
    pub transcript_accept_auto: bool,
    #[serde(default = "default_true")]
    pub transcript_timestamps: bool,
    pub start_with_windows: bool,
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
    #[serde(default = "default_true")]
    pub notify_on_complete: bool,
    pub log_level: LogLevelSetting,
    pub save_logs: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            locale: default_locale(),
            download_dir: None,
            quality_default: VideoQuality::default(),
            organize_by_channel: true,
            simultaneous: default_simultaneous(),
            transcript_lang: default_transcript_lang(),
            transcript_format: TranscriptFormat::default(),
            transcript_fallback: default_fallback(),
            transcript_accept_auto: true,
            transcript_timestamps: true,
            start_with_windows: false,
            minimize_to_tray: true,
            notify_on_complete: true,
            log_level: LogLevelSetting::default(),
            save_logs: false,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|base| base.config_dir().join("SMD").join("config.json"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        let mut config: Self = serde_json::from_str(&content).unwrap_or_default();
        config.validate();
        config
    }

    pub fn save(&self) {
        let Some(path) = Self::config_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            if std::fs::create_dir_all(parent).is_err() {
                return;
            }
        }
        let Ok(content) = serde_json::to_string_pretty(self) else {
            return;
        };
        if std::fs::write(&path, content).is_err() {
            tracing::warn!(target: "config", "unable to save config");
        }
    }

    pub fn validate(&mut self) {
        self.simultaneous = self.simultaneous.clamp(1, 5);
        let lang = self.transcript_lang.trim().to_string();
        if lang.is_empty() {
            self.transcript_lang = default_transcript_lang();
        } else {
            self.transcript_lang = lang;
        }
        self.transcript_fallback =
            normalize_fallback(self.transcript_fallback.as_deref(), &self.transcript_lang);
    }
}

fn normalize_fallback(raw: Option<&str>, lang: &str) -> Option<String> {
    let fallback = raw.unwrap_or("").trim();
    if fallback.is_empty()
        || fallback.eq_ignore_ascii_case("off")
        || fallback.eq_ignore_ascii_case(lang)
    {
        None
    } else {
        Some(fallback.to_string())
    }
}

pub fn resolve_download_dir(config: &AppConfig) -> PathBuf {
    config
        .download_dir
        .clone()
        .unwrap_or_else(default_download_dir)
}

pub fn resolve_output_dir(config: &AppConfig, channel: Option<&str>) -> PathBuf {
    let base = resolve_download_dir(config);
    if !config.organize_by_channel {
        return base;
    }
    let Some(name) = channel.map(str::trim).filter(|name| !name.is_empty()) else {
        return base;
    };
    base.join(sanitize_dirname(name))
}

pub fn sanitize_dirname(name: &str) -> String {
    const ILLEGAL: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut cleaned: String = name
        .chars()
        .map(|ch| {
            if ILLEGAL.contains(&ch) || ch.is_control() {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>()
        .trim()
        .trim_matches(['.', ' '])
        .to_string();
    while cleaned.contains("  ") {
        cleaned = cleaned.replace("  ", " ");
    }
    if cleaned.chars().count() > 100 {
        cleaned = cleaned.chars().take(100).collect::<String>();
        cleaned = cleaned.trim_end_matches(['.', ' ']).to_string();
    }
    if cleaned.is_empty() {
        return "unknown".to_string();
    }
    cleaned
}

pub fn logs_dir() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|base| base.config_dir().join("SMD").join("logs"))
}

pub fn log_file_for(dir: &std::path::Path, day: NaiveDate) -> PathBuf {
    dir.join(format!("{}.log", day.format("%Y-%m-%d")))
}

pub fn prune_old_logs(dir: &std::path::Path, today: NaiveDate, keep_days: i64) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    let mut removed = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let is_log = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("log"));
        if !is_log {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or_default()
            .to_string();
        let Ok(date) = NaiveDate::parse_from_str(&stem, "%Y-%m-%d") else {
            continue;
        };
        if (today - date).num_days() > keep_days && std::fs::remove_file(&path).is_ok() {
            removed += 1;
        }
    }
    removed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_fields_are_ignored() {
        let parsed: AppConfig = serde_json::from_str(
            r#"{"locale":"pt-BR","some_future_field":{"nested":true},"locale2":1}"#,
        )
        .unwrap_or_default();
        assert_eq!(parsed.locale, "pt-BR");
    }

    #[test]
    fn simultaneous_clamps_on_validate() {
        let mut low = AppConfig {
            simultaneous: 0,
            ..Default::default()
        };
        low.validate();
        assert_eq!(low.simultaneous, 1);
        let mut high = AppConfig {
            simultaneous: 9,
            ..Default::default()
        };
        high.validate();
        assert_eq!(high.simultaneous, 5);
    }

    #[test]
    fn simultaneous_clamps_from_json() {
        let parsed: AppConfig = serde_json::from_str(r#"{"simultaneous": 42}"#).unwrap_or_default();
        let mut parsed = parsed;
        parsed.validate();
        assert_eq!(parsed.simultaneous, 5);
    }

    #[test]
    fn fallback_equal_to_lang_becomes_none() {
        let mut config = AppConfig {
            transcript_lang: "pt".to_string(),
            transcript_fallback: Some("pt".to_string()),
            ..Default::default()
        };
        config.validate();
        assert_eq!(config.transcript_fallback, None);
    }

    #[test]
    fn fallback_off_and_empty_become_none() {
        for raw in [Some("off"), Some("OFF"), Some("  "), None] {
            assert_eq!(normalize_fallback(raw, "pt"), None);
        }
        assert_eq!(normalize_fallback(Some("en"), "pt"), Some("en".to_string()));
    }

    #[test]
    fn defaults_match_canvas() {
        let config = AppConfig::default();
        assert!(config.organize_by_channel);
        assert_eq!(config.simultaneous, 3);
        assert_eq!(config.transcript_lang, "pt");
        assert_eq!(config.transcript_fallback, Some("en".to_string()));
        assert!(config.transcript_accept_auto);
        assert!(config.transcript_timestamps);
        assert!(config.minimize_to_tray);
        assert!(config.notify_on_complete);
        assert!(!config.save_logs);
        assert_eq!(config.log_level, LogLevelSetting::Info);
    }

    #[test]
    fn resolve_download_dir_prefers_config() {
        let custom = AppConfig {
            download_dir: Some(PathBuf::from("/tmp/smd-custom")),
            ..Default::default()
        };
        assert_eq!(
            resolve_download_dir(&custom),
            PathBuf::from("/tmp/smd-custom")
        );
        let fallback = AppConfig::default();
        assert_eq!(resolve_download_dir(&fallback), default_download_dir());
    }

    #[test]
    fn sanitize_replaces_illegal_chars() {
        assert_eq!(sanitize_dirname("a/b:c*d?e"), "a_b_c_d_e");
        assert_eq!(sanitize_dirname("  name...  "), "name");
        assert_eq!(sanitize_dirname(""), "unknown");
        assert_eq!(sanitize_dirname("..."), "unknown");
        assert_eq!(sanitize_dirname("<>:\"/\\|?*"), "_________");
    }

    #[test]
    fn organize_flag_controls_subdir() {
        let config = AppConfig {
            download_dir: Some(PathBuf::from("/tmp/smd")),
            organize_by_channel: true,
            ..Default::default()
        };
        assert_eq!(
            resolve_output_dir(&config, Some("My Channel")),
            PathBuf::from("/tmp/smd/My Channel")
        );
        let flat = AppConfig {
            organize_by_channel: false,
            ..Default::default()
        };
        assert_eq!(
            resolve_output_dir(&flat, Some("My Channel")),
            resolve_download_dir(&flat)
        );
        assert_eq!(resolve_output_dir(&config, None), PathBuf::from("/tmp/smd"));
    }

    #[test]
    fn prune_removes_files_older_than_keep() {
        let dir = std::env::temp_dir().join(format!(
            "smd-prune-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|span| span.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::create_dir_all(&dir);
        let today = NaiveDate::from_ymd_opt(2026, 9, 3).expect("fixed date");
        for name in [
            "2026-08-20.log",
            "2026-08-26.log",
            "2026-09-02.log",
            "2026-09-03.log",
            "notes.txt",
            "not-a-date.log",
        ] {
            let _ = std::fs::write(dir.join(name), "x");
        }
        let removed = prune_old_logs(&dir, today, 7);
        assert_eq!(removed, 2);
        assert!(!dir.join("2026-08-20.log").exists());
        assert!(!dir.join("2026-08-26.log").exists());
        assert!(dir.join("2026-09-02.log").exists());
        assert!(dir.join("notes.txt").exists());
        assert!(dir.join("not-a-date.log").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn log_level_allows_hierarchy() {
        assert!(LogLevelSetting::Info.allows(LogLevel::Error));
        assert!(LogLevelSetting::Info.allows(LogLevel::Info));
        assert!(!LogLevelSetting::Info.allows(LogLevel::Debug));
        assert!(LogLevelSetting::Debug.allows(LogLevel::Debug));
        assert!(!LogLevelSetting::Error.allows(LogLevel::Warn));
    }
}
