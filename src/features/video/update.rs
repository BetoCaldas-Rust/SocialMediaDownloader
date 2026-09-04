use crate::core::effect::Effect;
use crate::core::state::VideoStatus;
use crate::services::log_buffer::LogLevel;

use super::intent::VideoIntent;
use super::model::VideoModel;

pub fn apply(model: &mut VideoModel, intent: &VideoIntent) -> Vec<Effect> {
    match intent {
        VideoIntent::SetUrl(url) => {
            model.url = url.clone();
            if !is_busy(model) {
                reset_attempt(model);
            }
            Vec::new()
        }
        VideoIntent::SetQuality(quality) => {
            if !is_busy(model) {
                model.quality = *quality;
            }
            Vec::new()
        }
        VideoIntent::SetContainer(container) => {
            if !is_busy(model) {
                model.container = *container;
            }
            Vec::new()
        }
        VideoIntent::SetTranscript(include) => {
            model.include_transcript = *include;
            Vec::new()
        }
        VideoIntent::FetchMetadata => start_resolve(model),
        VideoIntent::StartDownload => start_download(model),
        VideoIntent::CancelDownload => cancel_active(model),
        VideoIntent::MetadataReceived(result) => finish_resolve(model, result),
        VideoIntent::ProgressTick(tick) => {
            if model.status == VideoStatus::Downloading {
                model.progress = tick.percent.clamp(0.0, 100.0);
                model.speed.clone_from(&tick.speed);
                model.eta.clone_from(&tick.eta);
                if tick.filename.is_some() {
                    model.filename.clone_from(&tick.filename);
                }
            }
            Vec::new()
        }
        VideoIntent::DownloadFinished(result) => finish_download(model, result),
        VideoIntent::RevealOutput => reveal_output(model),
        VideoIntent::DismissError => {
            model.error_key = None;
            if model.status == VideoStatus::Failed {
                model.status = VideoStatus::Idle;
            }
            Vec::new()
        }
    }
}

fn is_busy(model: &VideoModel) -> bool {
    matches!(
        model.status,
        VideoStatus::Resolving | VideoStatus::Downloading
    )
}

fn reset_attempt(model: &mut VideoModel) {
    model.status = VideoStatus::Idle;
    model.metadata = None;
    model.progress = 0.0;
    model.speed = None;
    model.eta = None;
    model.filename = None;
    model.output_path = None;
    model.error_key = None;
}

fn fail(model: &mut VideoModel, key: &str) -> Vec<Effect> {
    model.status = VideoStatus::Failed;
    model.error_key = Some(key.to_string());
    Vec::new()
}

fn start_resolve(model: &mut VideoModel) -> Vec<Effect> {
    if is_busy(model) {
        return Vec::new();
    }
    if model.url.trim().is_empty() {
        return fail(model, "video_error_empty_url");
    }
    model.status = VideoStatus::Resolving;
    model.error_key = None;
    vec![Effect::FetchMetadata {
        url: model.url.trim().to_string(),
    }]
}

fn start_download(model: &mut VideoModel) -> Vec<Effect> {
    if is_busy(model) {
        return Vec::new();
    }
    if model.url.trim().is_empty() {
        return fail(model, "video_error_empty_url");
    }
    model.status = VideoStatus::Downloading;
    model.progress = 0.0;
    model.speed = None;
    model.eta = None;
    model.error_key = None;
    vec![Effect::StartDownload {
        url: model.url.trim().to_string(),
        quality: model.quality,
    }]
}

fn cancel_active(model: &mut VideoModel) -> Vec<Effect> {
    if !is_busy(model) {
        return Vec::new();
    }
    model.status = VideoStatus::Idle;
    vec![
        Effect::CancelActive,
        Effect::PushLog {
            level: LogLevel::Info,
            source: "download".to_string(),
            message: "download cancelled".to_string(),
        },
    ]
}

fn finish_resolve(
    model: &mut VideoModel,
    result: &Result<crate::services::traits::VideoMetadata, String>,
) -> Vec<Effect> {
    match result {
        Ok(metadata) => {
            model.status = VideoStatus::Ready;
            model.metadata = Some(metadata.clone());
            model.error_key = None;
            Vec::new()
        }
        Err(key) => fail(model, key),
    }
}

fn finish_download(
    model: &mut VideoModel,
    result: &Result<crate::services::traits::DownloadTicket, String>,
) -> Vec<Effect> {
    match result {
        Ok(ticket) => {
            model.status = VideoStatus::Completed;
            model.progress = 100.0;
            model.output_path = Some(ticket.path.clone());
            model.error_key = None;
            if model.include_transcript {
                return vec![Effect::PushLog {
                    level: LogLevel::Info,
                    source: "transcript".to_string(),
                    message: "transcript requested — arrives in F3".to_string(),
                }];
            }
            Vec::new()
        }
        Err(key) => fail(model, key),
    }
}

fn reveal_output(model: &VideoModel) -> Vec<Effect> {
    match &model.output_path {
        Some(path) => vec![Effect::RevealInFolder(path.clone())],
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::traits::{DownloadProgress, DownloadTicket, VideoMetadata};

    fn model_with_url() -> VideoModel {
        VideoModel {
            url: "https://example.com/watch?v=abc".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn set_url_resets_stale_attempt() {
        let mut model = model_with_url();
        model.status = VideoStatus::Failed;
        model.error_key = Some("video_error_fetch".to_string());
        apply(
            &mut model,
            &VideoIntent::SetUrl("https://other.example/v".to_string()),
        );
        assert_eq!(model.status, VideoStatus::Idle);
        assert_eq!(model.error_key, None);
    }

    #[test]
    fn fetch_with_empty_url_fails_without_effect() {
        let mut model = VideoModel::default();
        let effects = apply(&mut model, &VideoIntent::FetchMetadata);
        assert_eq!(model.status, VideoStatus::Failed);
        assert_eq!(model.error_key.as_deref(), Some("video_error_empty_url"));
        assert!(effects.is_empty());
    }

    #[test]
    fn fetch_emits_effect_and_marks_resolving() {
        let mut model = model_with_url();
        let effects = apply(&mut model, &VideoIntent::FetchMetadata);
        assert_eq!(model.status, VideoStatus::Resolving);
        assert!(matches!(effects.as_slice(), [Effect::FetchMetadata { .. }]));
    }

    #[test]
    fn metadata_received_ready() {
        let mut model = model_with_url();
        model.status = VideoStatus::Resolving;
        let effects = apply(
            &mut model,
            &VideoIntent::MetadataReceived(Ok(VideoMetadata {
                title: "T".to_string(),
                ..Default::default()
            })),
        );
        assert_eq!(model.status, VideoStatus::Ready);
        assert_eq!(model.metadata.as_ref().map(|m| m.title.as_str()), Some("T"));
        assert!(effects.is_empty());
    }

    #[test]
    fn metadata_error_fails() {
        let mut model = model_with_url();
        model.status = VideoStatus::Resolving;
        apply(
            &mut model,
            &VideoIntent::MetadataReceived(Err("video_error_fetch".to_string())),
        );
        assert_eq!(model.status, VideoStatus::Failed);
        assert_eq!(model.error_key.as_deref(), Some("video_error_fetch"));
    }

    #[test]
    fn start_download_snapshots_quality() {
        let mut model = model_with_url();
        let effects = apply(&mut model, &VideoIntent::StartDownload);
        assert_eq!(model.status, VideoStatus::Downloading);
        assert!(matches!(effects.as_slice(), [Effect::StartDownload { .. }]));
    }

    #[test]
    fn progress_tick_clamps_and_stores_details() {
        let mut model = model_with_url();
        model.status = VideoStatus::Downloading;
        apply(
            &mut model,
            &VideoIntent::ProgressTick(DownloadProgress {
                percent: 240.0,
                speed: Some("3.2 MiB/s".to_string()),
                eta: Some("00:21".to_string()),
                filename: Some("vid.mp4".to_string()),
            }),
        );
        assert_eq!(model.progress, 100.0);
        assert_eq!(model.speed.as_deref(), Some("3.2 MiB/s"));
        assert_eq!(model.filename.as_deref(), Some("vid.mp4"));
    }

    #[test]
    fn progress_ignored_when_not_downloading() {
        let mut model = model_with_url();
        apply(
            &mut model,
            &VideoIntent::ProgressTick(DownloadProgress {
                percent: 50.0,
                ..Default::default()
            }),
        );
        assert_eq!(model.progress, 0.0);
    }

    #[test]
    fn download_finished_logs_transcript_note_when_checked() {
        let mut model = model_with_url();
        model.status = VideoStatus::Downloading;
        model.include_transcript = true;
        let effects = apply(
            &mut model,
            &VideoIntent::DownloadFinished(Ok(DownloadTicket {
                id: "vid".to_string(),
                path: std::path::PathBuf::from("/tmp/vid.mp4"),
            })),
        );
        assert_eq!(model.status, VideoStatus::Completed);
        assert_eq!(model.progress, 100.0);
        assert!(matches!(
            effects.as_slice(),
            [Effect::PushLog { source, .. }] if source == "transcript"
        ));
    }
}
