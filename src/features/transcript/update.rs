use crate::core::effect::Effect;
use crate::core::state::TranscriptStatus;
use crate::services::traits::{
    file_name, history_source, new_history_id, now_ms, EntryKind, EntryStatus, HistoryEntry,
    TranscriptOrder,
};
use crate::services::yt_dlp::downloader::default_download_dir;
use crate::services::yt_dlp::transcript::parse_no_transcript_error;

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
    order_from_in(model, &default_download_dir())
}

pub fn order_from_in(model: &TranscriptModel, output_dir: &std::path::Path) -> TranscriptOrder {
    TranscriptOrder {
        url: model.input.trim().to_string(),
        lang: model.lang.clone(),
        format: model.format,
        fallback: normalized_fallback(model),
        accept_auto: model.accept_auto,
        timestamps: model.timestamps,
        output_dir: output_dir.to_path_buf(),
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

fn transcript_detail(lang_used: &str, auto_generated: bool) -> String {
    if auto_generated {
        format!("[{lang_used}] (auto)")
    } else {
        format!("[{lang_used}]")
    }
}

fn finish_fetch(
    model: &mut TranscriptModel,
    result: &Result<crate::services::traits::TranscriptResult, String>,
) -> Vec<Effect> {
    let order = order_from(model);
    match result {
        Ok(ticket) => {
            model.status = TranscriptStatus::Completed;
            model.result = Some(ticket.clone());
            model.error = None;
            model.recents.retain(|entry| entry.path != ticket.path);
            model.recents.insert(0, ticket.clone());
            model.recents.truncate(RECENTS_CAP);
            let url = model.input.trim().to_string();
            let name = file_name(&ticket.path).unwrap_or_else(|| url.clone());
            vec![Effect::RecordHistory(Box::new(HistoryEntry {
                id: new_history_id(),
                kind: EntryKind::Transcript,
                name,
                source: history_source(None, &url),
                url,
                detail: Some(transcript_detail(&ticket.lang_used, ticket.auto_generated)),
                finished_at_ms: now_ms(),
                size_bytes: Some(ticket.size_bytes),
                status: EntryStatus::Completed,
                error: None,
                path: Some(ticket.path.clone()),
                transcript_order: Some(order),
            }))]
        }
        Err(key) => {
            model.status = TranscriptStatus::Failed;
            model.error = Some(key.clone());
            let url =
                parse_no_transcript_error(key).unwrap_or_else(|| model.input.trim().to_string());
            vec![Effect::RecordHistory(Box::new(HistoryEntry {
                id: new_history_id(),
                kind: EntryKind::Transcript,
                name: url.clone(),
                source: history_source(None, &url),
                url,
                detail: None,
                finished_at_ms: now_ms(),
                size_bytes: None,
                status: EntryStatus::Failed,
                error: Some(key.clone()),
                path: None,
                transcript_order: Some(order),
            }))]
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
    fn finished_ok_completes_prepends_recent_and_records() {
        let mut model = model_with_input();
        model.status = TranscriptStatus::Resolving;
        let effects = apply(
            &mut model,
            &TranscriptIntent::TranscriptFinished(Ok(sample_result())),
        );
        assert_eq!(model.status, TranscriptStatus::Completed);
        assert!(matches!(
            effects.as_slice(),
            [Effect::RecordHistory(entry)]
            if entry.status == crate::services::traits::EntryStatus::Completed
                && entry.detail.as_deref() == Some("[pt]")
                && entry.size_bytes == Some(128)
        ));
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
        let effects = apply(
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
        assert!(matches!(
            effects.as_slice(),
            [Effect::RecordHistory(entry)]
            if entry.status == crate::services::traits::EntryStatus::Failed
                && entry.url == "https://x"
                && entry.error.as_deref() == Some("transcript_error_no_transcript|https://x")
                && entry.transcript_order.is_some()
        ));
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
    fn order_uses_given_output_dir() {
        let model = model_with_input();
        let order = order_from_in(&model, std::path::Path::new("/tmp/smd-custom"));
        assert_eq!(
            order.output_dir,
            std::path::PathBuf::from("/tmp/smd-custom")
        );
        assert_eq!(order.lang, "pt");
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
