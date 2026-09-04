use chrono::Local;

use crate::core::effect::Effect;
use crate::core::state::{
    BatchItemState, BatchItemStatus, ChannelInclude, ChannelStatus, DatePreset,
};
use crate::services::log_buffer::LogLevel;
use crate::services::traits::{
    file_name, file_size, history_source, new_history_id, now_ms, EntryKind, EntryStatus,
    HistoryEntry,
};
use crate::services::traits::{
    BatchItem, ChannelOrder, ChannelPreview, DownloadProgress, DownloadTicket, TranscriptOrder,
    TranscriptResult, VideoKind,
};
use crate::services::yt_dlp::channel::format_br_date;
use crate::services::yt_dlp::transcript::parse_no_transcript_error;

use super::intent::ChannelIntent;
use super::model::{parse_br_date, preset_range, ChannelModel};

pub fn apply(model: &mut ChannelModel, intent: &ChannelIntent) -> Vec<Effect> {
    match intent {
        ChannelIntent::SetInput(input) => {
            model.input = input.clone();
            if !model.status.is_busy() {
                reset_preview(model);
            }
            Vec::new()
        }
        ChannelIntent::SetPreset(preset) => {
            model.preset = *preset;
            if *preset != DatePreset::Custom {
                let today = Local::now().date_naive();
                let (from, to) = preset_range(*preset, today);
                model.from = from;
                model.to = to;
                model.from_text = from.map(format_br_date).unwrap_or_default();
                model.to_text = to.map(format_br_date).unwrap_or_default();
            }
            Vec::new()
        }
        ChannelIntent::SetFromText(text) => {
            model.from_text = text.clone();
            model.from = parse_br_date(text);
            model.preset = DatePreset::Custom;
            Vec::new()
        }
        ChannelIntent::SetToText(text) => {
            model.to_text = text.clone();
            model.to = parse_br_date(text);
            model.preset = DatePreset::Custom;
            Vec::new()
        }
        ChannelIntent::ToggleKind(kind) => {
            if model.status.is_busy() {
                return Vec::new();
            }
            match kind {
                VideoKind::Video => model.show_video = !model.show_video,
                VideoKind::Short => model.show_shorts = !model.show_shorts,
                VideoKind::Live => model.show_lives = !model.show_lives,
            }
            Vec::new()
        }
        ChannelIntent::ToggleInclude(include) => {
            if model.status == ChannelStatus::Downloading {
                return Vec::new();
            }
            match include {
                ChannelInclude::Video => model.include_video = !model.include_video,
                ChannelInclude::Transcript => {
                    model.include_transcript = !model.include_transcript;
                }
            }
            Vec::new()
        }
        ChannelIntent::ToggleSelect(index) => {
            if model.status.is_busy() {
                return Vec::new();
            }
            if let Some(flag) = model.selected.get_mut(*index) {
                *flag = !*flag;
            }
            Vec::new()
        }
        ChannelIntent::SelectAll => {
            if !model.status.is_busy() {
                model.selected.fill(true);
            }
            Vec::new()
        }
        ChannelIntent::SelectNone => {
            if !model.status.is_busy() {
                model.selected.fill(false);
            }
            Vec::new()
        }
        ChannelIntent::SetQuality(quality) => {
            if !model.status.is_busy() {
                model.quality = *quality;
            }
            Vec::new()
        }
        ChannelIntent::FetchPreview => start_preview(model),
        ChannelIntent::PreviewReceived(result) => finish_preview(model, result),
        ChannelIntent::DownloadBatch => start_batch(model, None),
        ChannelIntent::BatchTick { index, tick } => tick_item(model, *index, tick),
        ChannelIntent::BatchItemFinished {
            index,
            video_result,
            transcript_result,
        } => finish_item(model, *index, video_result, transcript_result),
        ChannelIntent::CancelBatch => cancel_batch(model),
        ChannelIntent::Retry => retry(model),
        ChannelIntent::Dismiss => {
            model.error_key = None;
            if model.status == ChannelStatus::Failed {
                model.status = reset_status(model);
            }
            Vec::new()
        }
    }
}

fn reset_preview(model: &mut ChannelModel) {
    model.status = ChannelStatus::Idle;
    model.preview = None;
    model.selected.clear();
    model.items.clear();
    model.error_key = None;
}

fn reset_status(model: &ChannelModel) -> ChannelStatus {
    if model.preview.is_some() {
        ChannelStatus::Ready
    } else {
        ChannelStatus::Idle
    }
}

fn fail(model: &mut ChannelModel, key: &str) -> Vec<Effect> {
    model.status = ChannelStatus::Failed;
    model.error_key = Some(key.to_string());
    Vec::new()
}

fn active_kinds(model: &ChannelModel) -> Vec<VideoKind> {
    let mut kinds = Vec::new();
    if model.show_video {
        kinds.push(VideoKind::Video);
    }
    if model.show_shorts {
        kinds.push(VideoKind::Short);
    }
    if model.show_lives {
        kinds.push(VideoKind::Live);
    }
    kinds
}

fn preview_order(model: &ChannelModel) -> ChannelOrder {
    let (from, to) = match model.preset {
        DatePreset::Custom => (model.from, model.to),
        preset => preset_range(preset, Local::now().date_naive()),
    };
    ChannelOrder {
        url: model.input.trim().to_string(),
        from,
        to,
        kinds: active_kinds(model),
    }
}

fn start_preview(model: &mut ChannelModel) -> Vec<Effect> {
    if model.status.is_busy() {
        return Vec::new();
    }
    if model.input.trim().is_empty() {
        return fail(model, "channel_error_empty_url");
    }
    model.status = ChannelStatus::Loading;
    model.error_key = None;
    vec![Effect::FetchChannelPreview {
        order: preview_order(model),
    }]
}

fn finish_preview(
    model: &mut ChannelModel,
    result: &Result<ChannelPreview, String>,
) -> Vec<Effect> {
    match result {
        Ok(preview) => {
            model.status = ChannelStatus::Ready;
            model.selected = vec![true; preview.videos.len()];
            model.items.clear();
            model.preview = Some(preview.clone());
            model.error_key = None;
            Vec::new()
        }
        Err(key) => fail(model, key),
    }
}

fn picked_items(model: &ChannelModel) -> Vec<BatchItem> {
    let Some(preview) = &model.preview else {
        return Vec::new();
    };
    preview
        .videos
        .iter()
        .zip(model.selected.iter())
        .filter(|(_, flag)| **flag)
        .map(|(video, _)| BatchItem {
            url: video.url.clone(),
            title: video.title.clone(),
        })
        .collect()
}

fn batch_effect(model: &ChannelModel, items: Vec<BatchItem>) -> Vec<Effect> {
    vec![Effect::DownloadBatch {
        items,
        include_video: model.include_video,
        include_transcript: model.include_transcript,
        quality: model.quality,
        transcript: TranscriptOrder::default(),
    }]
}

fn start_batch(model: &mut ChannelModel, forced: Option<Vec<BatchItem>>) -> Vec<Effect> {
    if !matches!(
        model.status,
        ChannelStatus::Ready | ChannelStatus::Completed
    ) {
        return Vec::new();
    }
    if !model.include_video && !model.include_transcript {
        return Vec::new();
    }
    let picked = forced.unwrap_or_else(|| picked_items(model));
    if picked.is_empty() {
        model.error_key = Some("channel_no_selection".to_string());
        return Vec::new();
    }
    model.items = picked
        .iter()
        .map(|item| BatchItemState {
            url: item.url.clone(),
            title: item.title.clone(),
            ..Default::default()
        })
        .collect();
    model.status = ChannelStatus::Downloading;
    model.error_key = None;
    batch_effect(model, picked)
}

fn tick_item(model: &mut ChannelModel, index: usize, tick: &DownloadProgress) -> Vec<Effect> {
    if model.status != ChannelStatus::Downloading {
        return Vec::new();
    }
    if let Some(item) = model.items.get_mut(index) {
        item.status = BatchItemStatus::Active;
        item.progress = tick.percent.clamp(0.0, 100.0);
        item.speed.clone_from(&tick.speed);
    }
    Vec::new()
}

fn first_error(
    video: &Option<Result<DownloadTicket, String>>,
    transcript: &Option<Result<TranscriptResult, String>>,
) -> Option<String> {
    if let Some(Err(key)) = video {
        return Some(key.clone());
    }
    if let Some(Err(key)) = transcript {
        return Some(key.clone());
    }
    None
}

fn transcript_order_for(model: &ChannelModel, url: &str) -> TranscriptOrder {
    model
        .last_transcript_order
        .clone()
        .map(|order| TranscriptOrder {
            url: url.to_string(),
            ..order
        })
        .unwrap_or(TranscriptOrder {
            url: url.to_string(),
            ..Default::default()
        })
}

fn transcript_detail(result: &TranscriptResult) -> String {
    if result.auto_generated {
        format!("[{}] (auto)", result.lang_used)
    } else {
        format!("[{}]", result.lang_used)
    }
}

fn video_entry(
    model: &ChannelModel,
    url: &str,
    title: &str,
    result: &Result<DownloadTicket, String>,
) -> HistoryEntry {
    let channel = model.preview.as_ref().map(|preview| preview.name.as_str());
    let source = history_source(channel, url);
    match result {
        Ok(ticket) => {
            let name = match (title.trim().is_empty(), file_name(&ticket.path)) {
                (false, _) => title.to_string(),
                (true, Some(file)) => file,
                (true, None) => url.to_string(),
            };
            HistoryEntry {
                id: new_history_id(),
                kind: EntryKind::Video,
                name,
                source,
                url: url.to_string(),
                detail: None,
                finished_at_ms: now_ms(),
                size_bytes: file_size(&ticket.path),
                status: EntryStatus::Completed,
                error: None,
                path: Some(ticket.path.clone()),
                transcript_order: None,
            }
        }
        Err(key) => HistoryEntry {
            id: new_history_id(),
            kind: EntryKind::Video,
            name: if title.trim().is_empty() {
                url.to_string()
            } else {
                title.to_string()
            },
            source,
            url: url.to_string(),
            detail: None,
            finished_at_ms: now_ms(),
            size_bytes: None,
            status: EntryStatus::Failed,
            error: Some(key.clone()),
            path: None,
            transcript_order: None,
        },
    }
}

fn transcript_entry(
    model: &ChannelModel,
    url: &str,
    title: &str,
    result: &Result<TranscriptResult, String>,
) -> HistoryEntry {
    let channel = model.preview.as_ref().map(|preview| preview.name.as_str());
    let source = history_source(channel, url);
    let order = transcript_order_for(model, url);
    match result {
        Ok(ticket) => {
            let name = match (title.trim().is_empty(), file_name(&ticket.path)) {
                (false, _) => title.to_string(),
                (true, Some(file)) => file,
                (true, None) => url.to_string(),
            };
            HistoryEntry {
                id: new_history_id(),
                kind: EntryKind::Transcript,
                name,
                source,
                url: url.to_string(),
                detail: Some(transcript_detail(ticket)),
                finished_at_ms: now_ms(),
                size_bytes: Some(ticket.size_bytes),
                status: EntryStatus::Completed,
                error: None,
                path: Some(ticket.path.clone()),
                transcript_order: Some(order),
            }
        }
        Err(key) => {
            let item_url = parse_no_transcript_error(key).unwrap_or_else(|| url.to_string());
            HistoryEntry {
                id: new_history_id(),
                kind: EntryKind::Transcript,
                name: if title.trim().is_empty() {
                    item_url.clone()
                } else {
                    title.to_string()
                },
                source,
                url: item_url,
                detail: None,
                finished_at_ms: now_ms(),
                size_bytes: None,
                status: EntryStatus::Failed,
                error: Some(key.clone()),
                path: None,
                transcript_order: Some(order),
            }
        }
    }
}

fn finish_item(
    model: &mut ChannelModel,
    index: usize,
    video: &Option<Result<DownloadTicket, String>>,
    transcript: &Option<Result<TranscriptResult, String>>,
) -> Vec<Effect> {
    let Some((url, title)) = model
        .items
        .get(index)
        .map(|item| (item.url.clone(), item.title.clone()))
    else {
        return Vec::new();
    };
    let mut effects = Vec::new();
    if let Some(result) = video {
        effects.push(Effect::RecordHistory(Box::new(video_entry(
            model, &url, &title, result,
        ))));
    }
    if let Some(result) = transcript {
        effects.push(Effect::RecordHistory(Box::new(transcript_entry(
            model, &url, &title, result,
        ))));
    }
    let Some(item) = model.items.get_mut(index) else {
        return effects;
    };
    let video_ok = video.as_ref().is_none_or(Result::is_ok);
    let transcript_ok = transcript.as_ref().is_none_or(Result::is_ok);
    if video_ok && transcript_ok {
        item.status = BatchItemStatus::Done;
        item.error_key = None;
        item.progress = 100.0;
    } else {
        item.status = BatchItemStatus::Failed;
        item.error_key = first_error(video, transcript);
    }
    let settled = model.items.iter().all(|entry| {
        matches!(
            entry.status,
            BatchItemStatus::Done | BatchItemStatus::Failed
        )
    });
    if settled {
        model.status = ChannelStatus::Completed;
    }
    effects
}

fn cancel_batch(model: &mut ChannelModel) -> Vec<Effect> {
    if !model.status.is_busy() {
        return Vec::new();
    }
    for item in &mut model.items {
        if item.status == BatchItemStatus::Active {
            item.status = BatchItemStatus::Queued;
        }
    }
    model.status = reset_status(model);
    vec![
        Effect::CancelChannel,
        Effect::PushLog {
            level: LogLevel::Info,
            source: "channel".to_string(),
            message: "channel batch cancelled".to_string(),
        },
    ]
}

fn retry(model: &mut ChannelModel) -> Vec<Effect> {
    match model.status {
        ChannelStatus::Failed => start_preview(model),
        ChannelStatus::Completed => {
            let failed: Vec<BatchItem> = model
                .items
                .iter()
                .filter(|item| item.status == BatchItemStatus::Failed)
                .map(|item| BatchItem {
                    url: item.url.clone(),
                    title: item.title.clone(),
                })
                .collect();
            if failed.is_empty() {
                return Vec::new();
            }
            for item in &mut model.items {
                if item.status == BatchItemStatus::Failed {
                    item.status = BatchItemStatus::Queued;
                    item.error_key = None;
                    item.progress = 0.0;
                    item.speed = None;
                }
            }
            model.status = ChannelStatus::Downloading;
            batch_effect(model, failed)
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::traits::{ChannelVideo, VideoKind};
    use chrono::Days;

    fn preview_with(count: usize) -> ChannelPreview {
        ChannelPreview {
            name: "Demo".to_string(),
            handle: None,
            subs: None,
            videos: (0..count)
                .map(|index| ChannelVideo {
                    id: format!("id{index}"),
                    title: format!("Video {index}"),
                    url: format!("https://www.youtube.com/watch?v=id{index}"),
                    ..Default::default()
                })
                .collect(),
        }
    }

    fn ready_model() -> ChannelModel {
        let mut model = ChannelModel {
            input: "https://www.youtube.com/@demo/videos".to_string(),
            ..Default::default()
        };
        apply(
            &mut model,
            &ChannelIntent::PreviewReceived(Ok(preview_with(3))),
        );
        model
    }

    #[test]
    fn preset_change_recomputes_range() {
        let mut model = ChannelModel::default();
        apply(&mut model, &ChannelIntent::SetPreset(DatePreset::Last7));
        let today = Local::now().date_naive();
        assert_eq!(model.from, today.checked_sub_days(Days::new(6)));
        assert_eq!(model.to, Some(today));
        assert!(!model.from_text.is_empty());
    }

    #[test]
    fn custom_text_sets_custom_preset() {
        let mut model = ChannelModel::default();
        apply(
            &mut model,
            &ChannelIntent::SetFromText("01/02/2024".to_string()),
        );
        assert_eq!(model.preset, DatePreset::Custom);
        assert!(model.from.is_some());
    }

    #[test]
    fn new_preview_resets_selection() {
        let mut model = ready_model();
        apply(&mut model, &ChannelIntent::ToggleSelect(0));
        assert_eq!(model.selected, vec![false, true, true]);
        apply(
            &mut model,
            &ChannelIntent::PreviewReceived(Ok(preview_with(2))),
        );
        assert_eq!(model.selected, vec![true, true]);
        assert!(model.items.is_empty());
    }

    #[test]
    fn select_all_and_none() {
        let mut model = ready_model();
        apply(&mut model, &ChannelIntent::SelectNone);
        assert_eq!(model.selected, vec![false, false, false]);
        apply(&mut model, &ChannelIntent::SelectAll);
        assert_eq!(model.selected, vec![true, true, true]);
    }

    #[test]
    fn batch_tick_updates_only_that_index() {
        let mut model = ready_model();
        apply(&mut model, &ChannelIntent::DownloadBatch);
        assert_eq!(model.status, ChannelStatus::Downloading);
        apply(
            &mut model,
            &ChannelIntent::BatchTick {
                index: 1,
                tick: DownloadProgress {
                    percent: 40.0,
                    speed: Some("1 MiB/s".to_string()),
                    ..Default::default()
                },
            },
        );
        assert_eq!(model.items[0].progress, 0.0);
        assert_eq!(model.items[1].progress, 40.0);
        assert_eq!(model.items[1].status, BatchItemStatus::Active);
    }

    #[test]
    fn item_finished_marks_per_item_status() {
        let mut model = ready_model();
        apply(&mut model, &ChannelIntent::SelectNone);
        apply(&mut model, &ChannelIntent::ToggleSelect(0));
        apply(&mut model, &ChannelIntent::ToggleSelect(1));
        apply(&mut model, &ChannelIntent::DownloadBatch);
        assert_eq!(model.items.len(), 2);
        apply(
            &mut model,
            &ChannelIntent::BatchItemFinished {
                index: 0,
                video_result: Some(Err("video_error_download".to_string())),
                transcript_result: None,
            },
        );
        assert_eq!(model.items[0].status, BatchItemStatus::Failed);
        assert_eq!(model.status, ChannelStatus::Downloading);
        apply(
            &mut model,
            &ChannelIntent::BatchItemFinished {
                index: 1,
                video_result: None,
                transcript_result: Some(Ok(TranscriptResult {
                    lang_used: "pt".to_string(),
                    ..Default::default()
                })),
            },
        );
        assert_eq!(model.items[1].status, BatchItemStatus::Done);
        assert_eq!(model.status, ChannelStatus::Completed);
    }

    #[test]
    fn empty_url_fails_preview_without_effect() {
        let mut model = ChannelModel::default();
        let effects = apply(&mut model, &ChannelIntent::FetchPreview);
        assert_eq!(model.status, ChannelStatus::Failed);
        assert_eq!(model.error_key.as_deref(), Some("channel_error_empty_url"));
        assert!(effects.is_empty());
    }

    #[test]
    fn download_without_selection_sets_error() {
        let mut model = ready_model();
        apply(&mut model, &ChannelIntent::SelectNone);
        let effects = apply(&mut model, &ChannelIntent::DownloadBatch);
        assert!(effects.is_empty());
        assert_eq!(model.error_key.as_deref(), Some("channel_no_selection"));
    }

    #[test]
    fn kind_toggle_flips_matching_flag() {
        let mut model = ready_model();
        apply(&mut model, &ChannelIntent::ToggleKind(VideoKind::Short));
        assert!(!model.show_shorts);
        assert!(model.show_video);
    }
}
