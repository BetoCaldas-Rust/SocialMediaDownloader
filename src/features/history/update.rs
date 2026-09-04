use chrono::{DateTime, Local};

use crate::core::effect::Effect;
use crate::core::state::{HistoryFilter, HistorySort};
use crate::services::log_buffer::LogLevel;
use crate::services::traits::{EntryKind, EntryStatus, HistoryEntry};

use super::intent::HistoryIntent;
use super::model::HistoryModel;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HistoryTotals {
    pub count: usize,
    pub bytes_completed: u64,
    pub failures: usize,
}

pub fn apply_in(
    model: &mut HistoryModel,
    intent: &HistoryIntent,
    entries: &[HistoryEntry],
    download_dir: &std::path::Path,
) -> Vec<Effect> {
    match intent {
        HistoryIntent::SetFilter(filter) => {
            model.filter = *filter;
            Vec::new()
        }
        HistoryIntent::SetQuery(query) => {
            model.query = query.clone();
            Vec::new()
        }
        HistoryIntent::SetSort(sort) => {
            model.sort = *sort;
            Vec::new()
        }
        HistoryIntent::RequestClear => {
            model.confirm_clear = true;
            Vec::new()
        }
        HistoryIntent::CancelClear => {
            model.confirm_clear = false;
            Vec::new()
        }
        HistoryIntent::ClearHistory => {
            if model.confirm_clear {
                model.confirm_clear = false;
                vec![Effect::ClearHistory]
            } else {
                Vec::new()
            }
        }
        HistoryIntent::OpenDownloadsFolder => {
            vec![Effect::RevealInFolder(download_dir.to_path_buf())]
        }
        HistoryIntent::RevealEntry(id) => match entries.iter().find(|entry| entry.id == *id) {
            Some(entry) => match &entry.path {
                Some(path) => vec![Effect::RevealInFolder(path.clone())],
                None => Vec::new(),
            },
            None => vec![missing_entry_effect(id)],
        },
        HistoryIntent::RetryEntry(id) => match entries.iter().find(|entry| entry.id == *id) {
            Some(entry) => vec![Effect::RetryEntry(Box::new(entry.clone()))],
            None => vec![missing_entry_effect(id)],
        },
    }
}

fn missing_entry_effect(id: &str) -> Effect {
    Effect::PushLog {
        level: LogLevel::Warn,
        source: "history".to_string(),
        message: format!("history entry not found: {id}"),
    }
}

pub fn apply_filter(
    entries: &[HistoryEntry],
    filter: HistoryFilter,
    query: &str,
) -> Vec<HistoryEntry> {
    let needle = query.trim().to_lowercase();
    entries
        .iter()
        .filter(|entry| match filter {
            HistoryFilter::All => true,
            HistoryFilter::Videos => entry.kind == EntryKind::Video,
            HistoryFilter::Transcripts => entry.kind == EntryKind::Transcript,
            HistoryFilter::Failures => entry.status == EntryStatus::Failed,
        })
        .filter(|entry| {
            if needle.is_empty() {
                return true;
            }
            [
                entry.name.as_str(),
                entry.source.as_str(),
                entry.url.as_str(),
            ]
            .iter()
            .any(|field| field.to_lowercase().contains(&needle))
        })
        .cloned()
        .collect()
}

pub fn apply_sort(mut entries: Vec<HistoryEntry>, sort: HistorySort) -> Vec<HistoryEntry> {
    match sort {
        HistorySort::Recent => {
            entries.sort_by_key(|entry| std::cmp::Reverse(entry.finished_at_ms));
        }
        HistorySort::Largest => {
            entries.sort_by_key(|entry| {
                (
                    std::cmp::Reverse(entry.size_bytes.unwrap_or(0)),
                    std::cmp::Reverse(entry.finished_at_ms),
                )
            });
        }
        HistorySort::Name => {
            entries.sort_by_key(|entry| {
                (
                    entry.name.to_lowercase(),
                    std::cmp::Reverse(entry.finished_at_ms),
                )
            });
        }
    }
    entries
}

pub fn format_size(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    let size = bytes as f64;
    if size >= GIB {
        format!("{:.1} GB", size / GIB)
    } else if size >= MIB {
        format!("{:.1} MB", size / MIB)
    } else if size >= KIB {
        format!("{:.1} KB", size / KIB)
    } else {
        format!("{bytes} B")
    }
}

pub fn format_when(
    finished_at_ms: i64,
    now_ms: i64,
    today_label: &str,
    yesterday_label: &str,
) -> String {
    let Some(entry_local) = DateTime::from_timestamp_millis(finished_at_ms)
        .map(|instant| instant.with_timezone(&Local))
    else {
        return "—".to_string();
    };
    let Some(now_local) =
        DateTime::from_timestamp_millis(now_ms).map(|instant| instant.with_timezone(&Local))
    else {
        return "—".to_string();
    };
    let entry_date = entry_local.date_naive();
    let now_date = now_local.date_naive();
    let time = entry_local.format("%H:%M").to_string();
    if entry_date == now_date {
        format!("{today_label}, {time}")
    } else if now_date.pred_opt() == Some(entry_date) {
        format!("{yesterday_label}, {time}")
    } else {
        entry_local.format("%d %b, %H:%M").to_string()
    }
}

pub fn totals(entries: &[HistoryEntry]) -> HistoryTotals {
    HistoryTotals {
        count: entries.len(),
        bytes_completed: entries
            .iter()
            .filter(|entry| entry.status == EntryStatus::Completed)
            .map(|entry| entry.size_bytes.unwrap_or(0))
            .sum(),
        failures: entries
            .iter()
            .filter(|entry| entry.status == EntryStatus::Failed)
            .count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::traits::{EntryStatus, TranscriptOrder};
    use chrono::TimeZone;

    fn test_dir() -> &'static std::path::Path {
        std::path::Path::new("/tmp")
    }

    fn entry(
        id: &str,
        kind: EntryKind,
        status: EntryStatus,
        name: &str,
        origin: (&str, &str),
        size: Option<u64>,
        finished_at_ms: i64,
    ) -> HistoryEntry {
        let (source, url) = origin;
        HistoryEntry {
            id: id.to_string(),
            kind,
            name: name.to_string(),
            source: source.to_string(),
            url: url.to_string(),
            detail: None,
            finished_at_ms,
            size_bytes: size,
            status,
            error: if status == EntryStatus::Failed {
                Some("video_error_download".to_string())
            } else {
                None
            },
            path: None,
            transcript_order: if kind == EntryKind::Transcript {
                Some(TranscriptOrder {
                    url: url.to_string(),
                    ..Default::default()
                })
            } else {
                None
            },
        }
    }

    fn matrix() -> Vec<HistoryEntry> {
        vec![
            entry(
                "v1",
                EntryKind::Video,
                EntryStatus::Completed,
                "Cats Compilation",
                ("Cat Channel", "https://www.youtube.com/watch?v=cats"),
                Some(50_000_000),
                3,
            ),
            entry(
                "v2",
                EntryKind::Video,
                EntryStatus::Failed,
                "Dogs Running",
                ("Dog Channel", "https://www.youtube.com/watch?v=dogs"),
                None,
                2,
            ),
            entry(
                "t1",
                EntryKind::Transcript,
                EntryStatus::Completed,
                "Cats Interview",
                ("News", "https://www.youtube.com/watch?v=news"),
                Some(2048),
                4,
            ),
            entry(
                "t2",
                EntryKind::Transcript,
                EntryStatus::Failed,
                "Cooking Show",
                ("Food", "https://vimeo.com/123"),
                None,
                1,
            ),
        ]
    }

    fn ids(entries: &[HistoryEntry]) -> Vec<&str> {
        entries.iter().map(|entry| entry.id.as_str()).collect()
    }

    #[test]
    fn filter_matrix_kind_and_status() {
        let entries = matrix();
        assert_eq!(
            ids(&apply_filter(&entries, HistoryFilter::All, "")).len(),
            4
        );
        assert_eq!(
            ids(&apply_filter(&entries, HistoryFilter::Videos, "")),
            vec!["v1", "v2"]
        );
        assert_eq!(
            ids(&apply_filter(&entries, HistoryFilter::Transcripts, "")),
            vec!["t1", "t2"]
        );
        assert_eq!(
            ids(&apply_filter(&entries, HistoryFilter::Failures, "")),
            vec!["v2", "t2"]
        );
    }

    #[test]
    fn query_matches_case_insensitive_across_fields() {
        let entries = matrix();
        assert_eq!(
            ids(&apply_filter(&entries, HistoryFilter::All, "cats")),
            vec!["v1", "t1"]
        );
        assert_eq!(
            ids(&apply_filter(&entries, HistoryFilter::All, "CATS")),
            vec!["v1", "t1"]
        );
        assert_eq!(
            ids(&apply_filter(&entries, HistoryFilter::All, "vimeo.com/123")),
            vec!["t2"]
        );
        assert_eq!(
            ids(&apply_filter(&entries, HistoryFilter::All, "dog channel")),
            vec!["v2"]
        );
        assert!(apply_filter(&entries, HistoryFilter::All, "nope").is_empty());
    }

    #[test]
    fn query_combines_with_kind_filter() {
        let entries = matrix();
        assert_eq!(
            ids(&apply_filter(&entries, HistoryFilter::Videos, "cats")),
            vec!["v1"]
        );
    }

    #[test]
    fn sort_orders() {
        let entries = matrix();
        assert_eq!(
            ids(&apply_sort(entries.clone(), HistorySort::Recent)),
            vec!["t1", "v1", "v2", "t2"]
        );
        assert_eq!(
            ids(&apply_sort(entries.clone(), HistorySort::Largest)),
            vec!["v1", "t1", "v2", "t2"]
        );
        assert_eq!(
            ids(&apply_sort(entries.clone(), HistorySort::Name)),
            vec!["v1", "t1", "t2", "v2"]
        );
    }

    #[test]
    fn size_boundaries() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(2 * 1024 * 1024 * 1024), "2.0 GB");
    }

    #[test]
    fn when_labels() {
        let now = Local
            .with_ymd_and_hms(2026, 8, 30, 12, 0, 0)
            .single()
            .map(|date| date.timestamp_millis())
            .unwrap_or_default();
        let today = Local
            .with_ymd_and_hms(2026, 8, 30, 9, 5, 0)
            .single()
            .map(|date| date.timestamp_millis())
            .unwrap_or_default();
        let yesterday = Local
            .with_ymd_and_hms(2026, 8, 29, 22, 10, 0)
            .single()
            .map(|date| date.timestamp_millis())
            .unwrap_or_default();
        let older = Local
            .with_ymd_and_hms(2026, 1, 5, 8, 30, 0)
            .single()
            .map(|date| date.timestamp_millis())
            .unwrap_or_default();
        assert_eq!(format_when(today, now, "Hoje", "Ontem"), "Hoje, 09:05");
        assert_eq!(format_when(yesterday, now, "Hoje", "Ontem"), "Ontem, 22:10");
        assert_eq!(format_when(older, now, "Hoje", "Ontem"), "05 Jan, 08:30");
    }

    #[test]
    fn totals_count_bytes_and_failures() {
        let summary = totals(&matrix());
        assert_eq!(summary.count, 4);
        assert_eq!(summary.bytes_completed, 50_002_048);
        assert_eq!(summary.failures, 2);
    }

    #[test]
    fn clear_requires_confirm_flag() {
        let mut model = HistoryModel::default();
        let effects = apply_in(&mut model, &HistoryIntent::ClearHistory, &[], test_dir());
        assert!(effects.is_empty());
        apply_in(&mut model, &HistoryIntent::RequestClear, &[], test_dir());
        assert!(model.confirm_clear);
        let effects = apply_in(&mut model, &HistoryIntent::ClearHistory, &[], test_dir());
        assert!(matches!(effects.as_slice(), [Effect::ClearHistory]));
        assert!(!model.confirm_clear);
    }

    #[test]
    fn cancel_clear_resets_flag() {
        let mut model = HistoryModel {
            confirm_clear: true,
            ..Default::default()
        };
        apply_in(&mut model, &HistoryIntent::CancelClear, &[], test_dir());
        assert!(!model.confirm_clear);
    }

    #[test]
    fn retry_emits_effect_with_entry_data() {
        let entries = matrix();
        let mut model = HistoryModel::default();
        let effects = apply_in(
            &mut model,
            &HistoryIntent::RetryEntry("t1".to_string()),
            &entries,
            test_dir(),
        );
        assert!(matches!(
            effects.as_slice(),
            [Effect::RetryEntry(entry)] if entry.id == "t1" && entry.transcript_order.is_some()
        ));
    }

    #[test]
    fn retry_missing_id_emits_warning() {
        let mut model = HistoryModel::default();
        let effects = apply_in(
            &mut model,
            &HistoryIntent::RetryEntry("nope".to_string()),
            &matrix(),
            test_dir(),
        );
        assert!(matches!(effects.as_slice(), [Effect::PushLog { .. }]));
    }

    #[test]
    fn retry_plan_shapes() {
        let entries = matrix();
        match crate::services::traits::retry_plan(&entries[0]) {
            crate::services::traits::RetryPlan::Video { url } => {
                assert_eq!(url, "https://www.youtube.com/watch?v=cats");
            }
            crate::services::traits::RetryPlan::Transcript { .. } => {
                panic!("expected video plan");
            }
        }
        match crate::services::traits::retry_plan(&entries[2]) {
            crate::services::traits::RetryPlan::Transcript { order } => {
                assert_eq!(order.url, "https://www.youtube.com/watch?v=news");
            }
            crate::services::traits::RetryPlan::Video { .. } => {
                panic!("expected transcript plan");
            }
        }
    }

    #[test]
    fn open_folder_uses_settings_dir() {
        let mut model = HistoryModel::default();
        let custom = std::path::Path::new("/tmp/smd-custom");
        let effects = apply_in(&mut model, &HistoryIntent::OpenDownloadsFolder, &[], custom);
        assert!(matches!(
            effects.as_slice(),
            [Effect::RevealInFolder(path)] if path == custom
        ));
    }

    #[test]
    fn filter_and_sort_intents_update_model() {
        let mut model = HistoryModel::default();
        apply_in(
            &mut model,
            &HistoryIntent::SetFilter(HistoryFilter::Failures),
            &[],
            test_dir(),
        );
        apply_in(
            &mut model,
            &HistoryIntent::SetQuery("cats".to_string()),
            &[],
            test_dir(),
        );
        apply_in(
            &mut model,
            &HistoryIntent::SetSort(HistorySort::Name),
            &[],
            test_dir(),
        );
        assert_eq!(model.filter, HistoryFilter::Failures);
        assert_eq!(model.query, "cats");
        assert_eq!(model.sort, HistorySort::Name);
    }
}
