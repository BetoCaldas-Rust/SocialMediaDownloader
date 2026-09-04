use crate::core::effect::Effect;
use crate::services::log_buffer::LogLevel;

use super::intent::SettingsIntent;
use super::model::SettingsModel;

pub fn apply(model: &mut SettingsModel, intent: &SettingsIntent) -> Vec<Effect> {
    match intent {
        SettingsIntent::ChangeLocale(locale) => {
            model.locale = locale.clone();
            vec![Effect::ReloadLocale(locale.clone())]
        }
        SettingsIntent::PickDownloadDir => vec![Effect::PickDownloadDir],
        SettingsIntent::DownloadDirPicked(dir) => {
            model.download_dir = dir.clone();
            vec![Effect::SaveSettings]
        }
        SettingsIntent::SetDownloadDir(dir) => {
            model.download_dir = dir.clone();
            vec![Effect::SaveSettings]
        }
        SettingsIntent::SetQualityDefault(quality) => {
            model.quality_default = *quality;
            vec![Effect::SaveSettings]
        }
        SettingsIntent::SetOrganize(value) => {
            model.organize_by_channel = *value;
            vec![Effect::SaveSettings]
        }
        SettingsIntent::SetSimultaneous(count) => {
            model.simultaneous = (*count).clamp(1, 5);
            vec![Effect::SaveSettings]
        }
        SettingsIntent::SetTranscriptLang(lang) => {
            let trimmed = lang.trim();
            if !trimmed.is_empty() {
                model.transcript_lang = trimmed.to_string();
                if model
                    .transcript_fallback
                    .as_deref()
                    .is_some_and(|fallback| fallback.eq_ignore_ascii_case(trimmed))
                {
                    model.transcript_fallback = None;
                }
            }
            vec![Effect::SaveSettings]
        }
        SettingsIntent::SetTranscriptFormat(format) => {
            model.transcript_format = *format;
            vec![Effect::SaveSettings]
        }
        SettingsIntent::SetTranscriptFallback(fallback) => {
            model.transcript_fallback =
                normalize_fallback(fallback.clone(), &model.transcript_lang);
            vec![Effect::SaveSettings]
        }
        SettingsIntent::ToggleTranscriptAuto => {
            model.transcript_accept_auto = !model.transcript_accept_auto;
            vec![Effect::SaveSettings]
        }
        SettingsIntent::ToggleTranscriptTimestamps => {
            model.transcript_timestamps = !model.transcript_timestamps;
            vec![Effect::SaveSettings]
        }
        SettingsIntent::SetAutostart(value) => {
            model.start_with_windows = *value;
            vec![Effect::SaveSettings, Effect::SetAutostart(*value)]
        }
        SettingsIntent::AutostartVerified(actual) => {
            model.autostart_actual = *actual;
            Vec::new()
        }
        SettingsIntent::SetMinimizeTray(value) => {
            model.minimize_to_tray = *value;
            vec![
                Effect::SaveSettings,
                Effect::PushLog {
                    level: LogLevel::Info,
                    source: "settings".to_string(),
                    message: "tray icon arrives with packaging (F8)".to_string(),
                },
            ]
        }
        SettingsIntent::SetNotify(value) => {
            model.notify_on_complete = *value;
            vec![Effect::SaveSettings]
        }
        SettingsIntent::SetLogLevel(level) => {
            model.log_level = *level;
            vec![Effect::SaveSettings, Effect::ApplyLogLevel(*level)]
        }
        SettingsIntent::SetSaveLogs(value) => {
            model.save_logs = *value;
            vec![Effect::SaveSettings]
        }
        SettingsIntent::CheckYtDlp => {
            model.ytdlp_checking = true;
            model.ytdlp_error = None;
            vec![Effect::CheckYtDlpVersion]
        }
        SettingsIntent::YtDlpVersionReceived(result) => {
            model.ytdlp_checking = false;
            match result {
                Ok(version) => {
                    model.ytdlp_version = Some(version.clone());
                    model.ytdlp_error = None;
                }
                Err(key) => {
                    model.ytdlp_error = Some(key.clone());
                }
            }
            Vec::new()
        }
        SettingsIntent::UpdateYtDlp => {
            model.ytdlp_checking = true;
            model.ytdlp_error = None;
            vec![Effect::UpdateYtDlp]
        }
        SettingsIntent::YtDlpUpdated(result) => {
            model.ytdlp_checking = false;
            match result {
                Ok(version) => {
                    model.ytdlp_version = Some(version.clone());
                    model.ytdlp_error = None;
                }
                Err(key) => {
                    model.ytdlp_error = Some(key.clone());
                }
            }
            Vec::new()
        }
    }
}

fn normalize_fallback(raw: Option<String>, lang: &str) -> Option<String> {
    let trimmed = raw.as_deref().unwrap_or("").trim();
    if trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("off")
        || trimmed.eq_ignore_ascii_case(lang)
    {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::traits::{TranscriptFormat, VideoQuality};
    use crate::storage::config::{AppConfig, LogLevelSetting};

    fn model() -> SettingsModel {
        SettingsModel::from_config(&AppConfig::default())
    }

    fn saves(effects: &[Effect]) -> bool {
        effects
            .iter()
            .any(|effect| matches!(effect, Effect::SaveSettings))
    }

    #[test]
    fn locale_path_stays_reload_only() {
        let mut state = model();
        let effects = apply(
            &mut state,
            &SettingsIntent::ChangeLocale("pt-BR".to_string()),
        );
        assert_eq!(state.locale, "pt-BR");
        assert!(matches!(
            effects.as_slice(),
            [Effect::ReloadLocale(locale)] if locale == "pt-BR"
        ));
    }

    #[test]
    fn value_intents_mutate_and_persist() {
        let mut state = model();
        let effects = apply(
            &mut state,
            &SettingsIntent::SetQualityDefault(VideoQuality::Capped720),
        );
        assert_eq!(state.quality_default, VideoQuality::Capped720);
        assert!(saves(&effects));

        let effects = apply(&mut state, &SettingsIntent::SetOrganize(false));
        assert!(!state.organize_by_channel);
        assert!(saves(&effects));

        let effects = apply(&mut state, &SettingsIntent::SetSimultaneous(9));
        assert_eq!(state.simultaneous, 5);
        assert!(saves(&effects));

        let effects = apply(
            &mut state,
            &SettingsIntent::SetDownloadDir(Some("/tmp/x".into())),
        );
        assert_eq!(state.download_dir, Some("/tmp/x".into()));
        assert!(saves(&effects));

        let effects = apply(&mut state, &SettingsIntent::DownloadDirPicked(None));
        assert_eq!(state.download_dir, None);
        assert!(saves(&effects));
    }

    #[test]
    fn transcript_intents_mutate_and_persist() {
        let mut state = model();
        let effects = apply(
            &mut state,
            &SettingsIntent::SetTranscriptLang("en".to_string()),
        );
        assert_eq!(state.transcript_lang, "en");
        assert!(saves(&effects));

        let effects = apply(
            &mut state,
            &SettingsIntent::SetTranscriptFormat(TranscriptFormat::Txt),
        );
        assert_eq!(state.transcript_format, TranscriptFormat::Txt);
        assert!(saves(&effects));

        let effects = apply(
            &mut state,
            &SettingsIntent::SetTranscriptFallback(Some("es".to_string())),
        );
        assert_eq!(state.transcript_fallback, Some("es".to_string()));
        assert!(saves(&effects));

        let effects = apply(
            &mut state,
            &SettingsIntent::SetTranscriptFallback(Some("en".to_string())),
        );
        assert_eq!(state.transcript_fallback, None);
        assert!(saves(&effects));

        let before = state.transcript_accept_auto;
        let effects = apply(&mut state, &SettingsIntent::ToggleTranscriptAuto);
        assert_eq!(state.transcript_accept_auto, !before);
        assert!(saves(&effects));

        let before = state.transcript_timestamps;
        let effects = apply(&mut state, &SettingsIntent::ToggleTranscriptTimestamps);
        assert_eq!(state.transcript_timestamps, !before);
        assert!(saves(&effects));
    }

    #[test]
    fn app_intents_mutate_and_persist() {
        let mut state = model();
        let effects = apply(&mut state, &SettingsIntent::SetAutostart(true));
        assert!(state.start_with_windows);
        assert!(saves(&effects));
        assert!(effects
            .iter()
            .any(|effect| matches!(effect, Effect::SetAutostart(true))));

        let effects = apply(&mut state, &SettingsIntent::AutostartVerified(Some(true)));
        assert_eq!(state.autostart_actual, Some(true));
        assert!(!saves(&effects));

        let effects = apply(&mut state, &SettingsIntent::SetMinimizeTray(false));
        assert!(!state.minimize_to_tray);
        assert!(saves(&effects));
        assert!(effects
            .iter()
            .any(|effect| matches!(effect, Effect::PushLog { .. })));

        let effects = apply(&mut state, &SettingsIntent::SetNotify(false));
        assert!(!state.notify_on_complete);
        assert!(saves(&effects));

        let effects = apply(
            &mut state,
            &SettingsIntent::SetLogLevel(LogLevelSetting::Debug),
        );
        assert_eq!(state.log_level, LogLevelSetting::Debug);
        assert!(saves(&effects));
        assert!(effects
            .iter()
            .any(|effect| matches!(effect, Effect::ApplyLogLevel(_))));

        let effects = apply(&mut state, &SettingsIntent::SetSaveLogs(true));
        assert!(state.save_logs);
        assert!(saves(&effects));
    }

    #[test]
    fn ytdlp_check_flow_updates_state() {
        let mut state = model();
        let effects = apply(&mut state, &SettingsIntent::CheckYtDlp);
        assert!(state.ytdlp_checking);
        assert!(effects
            .iter()
            .any(|effect| matches!(effect, Effect::CheckYtDlpVersion)));

        apply(
            &mut state,
            &SettingsIntent::YtDlpVersionReceived(Ok("2026.01.01".to_string())),
        );
        assert!(!state.ytdlp_checking);
        assert_eq!(state.ytdlp_version.as_deref(), Some("2026.01.01"));
        assert_eq!(state.ytdlp_error, None);

        let effects = apply(&mut state, &SettingsIntent::UpdateYtDlp);
        assert!(state.ytdlp_checking);
        assert!(effects
            .iter()
            .any(|effect| matches!(effect, Effect::UpdateYtDlp)));

        apply(
            &mut state,
            &SettingsIntent::YtDlpUpdated(Err("settings_ytdlp_error_update".to_string())),
        );
        assert!(!state.ytdlp_checking);
        assert_eq!(
            state.ytdlp_error.as_deref(),
            Some("settings_ytdlp_error_update")
        );
    }

    #[test]
    fn state_round_trips_through_config() {
        let mut state = model();
        apply(&mut state, &SettingsIntent::SetSimultaneous(5));
        apply(
            &mut state,
            &SettingsIntent::SetTranscriptFallback(Some("es".to_string())),
        );
        let mut config = AppConfig::default();
        state.apply_to_config(&mut config);
        assert_eq!(config.simultaneous, 5);
        assert_eq!(config.transcript_fallback, Some("es".to_string()));
        let restored = SettingsModel::from_config(&config);
        assert_eq!(restored.simultaneous, 5);
        assert_eq!(restored.transcript_fallback, Some("es".to_string()));
    }
}
