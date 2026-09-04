use crate::core::effect::Effect;
use crate::core::state::TranscriptStatus;
use crate::services::traits::TranscriptOrder;
use crate::services::yt_dlp::downloader::default_download_dir;

use super::intent::TranscriptIntent;
use super::model::TranscriptModel;

pub const RECENTS_CAP: usize = 20;

pub fn apply(model: &mut TranscriptModel, intent: &TranscriptIntent) -> Vec<Effect> {
    match intent {
        TranscriptIntent::SetInput(input) => {
            model.input = input.clone();
            if !model.status.is_busy() {
                reset_attempt(model);
            }
            Vec::new()
        }
        TranscriptIntent::SetLang(lang) => {
            if !model.status.is_busy() {
                model.lang = lang.clone();
            }
            Vec::new()
        }
        TranscriptIntent::SetFormat(format) => {
            if !model.status.is_busy() {
                model.format = *format;
            }
            Vec::new()
        }
        TranscriptIntent::SetFallback(fallback) => {
            if !model.status.is_busy() {
                model.fallback = fallback.clone();
            }
            Vec::new()
        }
        TranscriptIntent::ToggleAuto => {
            if !model.status.is_busy() {
                model.accept_auto = !model.accept_auto;
            }
            Vec::new()
        }
        TranscriptIntent::ToggleTimestamps => {
            if !model.status.is_busy() {
                model.timestamps = !model.timestamps;
            }
            Vec::new()
        }
        TranscriptIntent::FetchTranscript => start_fetch(model),
        TranscriptIntent::TranscriptFinished(result) => finish_fetch(model, result),
        TranscriptIntent::Retry => retry_fetch(model),
        TranscriptIntent::Dismiss => {
            model.error = None;
            if model.status == TranscriptStatus::Failed {
                model.status = TranscriptStatus::Idle;
            }
            Vec::new()
        }
        TranscriptIntent::RevealRecent(index) => match model.recents.get(*index) {
            Some(entry) => vec![Effect::RevealInFolder(entry.path.clone())],
            None => Vec::new(),
        },
    }
}

pub fn order_from(model: &TranscriptModel) -> TranscriptOrder {
    TranscriptOrder {
        url: model.input.trim().to_string(),
        lang: model.lang.clone(),
        format: model.format,
        fallback: normalized_fallback(model),
        accept_auto: model.accept_auto,
        timestamps: model.timestamps,
        output_dir: default_download_dir(),
    }
}

fn normalized_fallback(model: &TranscriptModel) -> Option<String> {
    let fallback = model.fallback.trim();
    if fallback.is_empty() || fallback.eq_ignore_ascii_case("off") {
        None
    } else {
        Some(fallback.to_string())
    }
}

fn reset_attempt(model: &mut TranscriptModel) {
    model.status = TranscriptStatus::Idle;
    model.result = None;
    model.error = None;
}

fn start_fetch(model: &mut TranscriptModel) -> Vec<Effect> {
    if model.input.trim().is_empty() {
        model.status = TranscriptStatus::Failed;
        model.error = Some("video_error_empty_url".to_string());
        return Vec::new();
    }
    model.error = None;
    if model.status.is_busy() {
        model.status = TranscriptStatus::Downloading;
        let order = order_from(model);
        return vec![Effect::CancelTranscript, Effect::FetchTranscript { order }];
    }
    model.status = TranscriptStatus::Resolving;
    vec![Effect::FetchTranscript {
        order: order_from(model),
    }]
}

fn retry_fetch(model: &mut TranscriptModel) -> Vec<Effect> {
    if model.status.is_busy() {
        return start_fetch(model);
    }
    model.error = None;
    start_fetch(model)
}

fn finish_fetch(
    model: &mut TranscriptModel,
    result: &Result<crate::services::traits::TranscriptResult, String>,
) -> Vec<Effect> {
    match result {
        Ok(ticket) => {
            model.status = TranscriptStatus::Completed;
            model.result = Some(ticket.clone());
            model.error = None;
            model.recents.retain(|entry| entry.path != ticket.path);
            model.recents.insert(0, ticket.clone());
            model.recents.truncate(RECENTS_CAP);
            Vec::new()
        }
        Err(key) => {
            model.status = TranscriptStatus::Failed;
            model.error = Some(key.clone());
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::traits::{TranscriptFormat, TranscriptResult};

    fn model_with_input() -> TranscriptModel {
        TranscriptModel {
            input: "https://example.com/watch?v=abc".to_string(),
            ..Default::default()
        }
    }

    fn sample_result() -> TranscriptResult {
        TranscriptResult {
            path: std::path::PathBuf::from("/tmp/video [abc].srt"),
            lang_used: "pt".to_string(),
            auto_generated: false,
            size_bytes: 128,
        }
    }

    #[test]
    fn fetch_moves_to_resolving_with_effect() {
        let mut model = model_with_input();
        let effects = apply(&mut model, &TranscriptIntent::FetchTranscript);
        assert_eq!(model.status, TranscriptStatus::Resolving);
        assert!(matches!(
            effects.as_slice(),
            [Effect::FetchTranscript { .. }]
        ));
    }

    #[test]
    fn fetch_with_empty_url_fails_without_effect() {
        let mut model = TranscriptModel::default();
        let effects = apply(&mut model, &TranscriptIntent::FetchTranscript);
        assert_eq!(model.status, TranscriptStatus::Failed);
        assert_eq!(model.error.as_deref(), Some("video_error_empty_url"));
        assert!(effects.is_empty());
    }

    #[test]
    fn refetch_while_busy_cancels_then_refetches() {
        let mut model = model_with_input();
        model.status = TranscriptStatus::Resolving;
        let effects = apply(&mut model, &TranscriptIntent::FetchTranscript);
        assert_eq!(model.status, TranscriptStatus::Downloading);
        assert!(matches!(
            effects.as_slice(),
            [Effect::CancelTranscript, Effect::FetchTranscript { .. }]
        ));
    }

    #[test]
    fn finished_ok_completes_and_prepends_recent() {
        let mut model = model_with_input();
        model.status = TranscriptStatus::Resolving;
        let effects = apply(
            &mut model,
            &TranscriptIntent::TranscriptFinished(Ok(sample_result())),
        );
        assert_eq!(model.status, TranscriptStatus::Completed);
        assert!(effects.is_empty());
        assert_eq!(model.recents.len(), 1);
        assert_eq!(model.result.as_ref(), model.recents.first());
    }

    #[test]
    fn recents_dedupe_and_cap() {
        let mut model = model_with_input();
        for index in 0..(RECENTS_CAP + 5) {
            let mut entry = sample_result();
            entry.path = std::path::PathBuf::from(format!("/tmp/v{index}.srt"));
            apply(&mut model, &TranscriptIntent::TranscriptFinished(Ok(entry)));
        }
        assert_eq!(model.recents.len(), RECENTS_CAP);
        let newest = format!("/tmp/v{}.srt", RECENTS_CAP + 4);
        assert!(model
            .recents
            .first()
            .is_some_and(|entry| entry.path.to_string_lossy() == newest));
        let dupe = sample_result();
        apply(&mut model, &TranscriptIntent::TranscriptFinished(Ok(dupe)));
        assert_eq!(model.recents.len(), RECENTS_CAP);
        assert_eq!(model.recents.first(), model.result.as_ref());
    }

    #[test]
    fn finished_err_fails_with_error_key() {
        let mut model = model_with_input();
        model.status = TranscriptStatus::Resolving;
        apply(
            &mut model,
            &TranscriptIntent::TranscriptFinished(Err(
                "transcript_error_no_transcript|https://x".to_string()
            )),
        );
        assert_eq!(model.status, TranscriptStatus::Failed);
        assert_eq!(
            model.error.as_deref(),
            Some("transcript_error_no_transcript|https://x")
        );
    }

    #[test]
    fn retry_clears_error_and_refetches() {
        let mut model = model_with_input();
        model.status = TranscriptStatus::Failed;
        model.error = Some("transcript_error_fetch".to_string());
        let effects = apply(&mut model, &TranscriptIntent::Retry);
        assert_eq!(model.status, TranscriptStatus::Resolving);
        assert_eq!(model.error, None);
        assert!(matches!(
            effects.as_slice(),
            [Effect::FetchTranscript { .. }]
        ));
    }

    #[test]
    fn prefs_locked_while_busy() {
        let mut model = model_with_input();
        model.status = TranscriptStatus::Resolving;
        apply(&mut model, &TranscriptIntent::SetLang("es".to_string()));
        apply(&mut model, &TranscriptIntent::ToggleAuto);
        assert_eq!(model.lang, "pt");
        assert!(model.accept_auto);
    }

    #[test]
    fn order_maps_off_fallback_to_none() {
        let mut model = model_with_input();
        model.fallback = "off".to_string();
        model.format = TranscriptFormat::Txt;
        let order = order_from(&model);
        assert_eq!(order.fallback, None);
        assert_eq!(order.format, TranscriptFormat::Txt);
        assert_eq!(order.url, "https://example.com/watch?v=abc");
    }
}
