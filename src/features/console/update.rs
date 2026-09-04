use crate::core::effect::Effect;
use crate::services::log_buffer::{log_entries, LogEntry, LogLevel};

use super::intent::ConsoleIntent;
use super::model::ConsoleModel;

pub fn apply(model: &mut ConsoleModel, intent: &ConsoleIntent) -> Vec<Effect> {
    match intent {
        ConsoleIntent::ToggleLevel(level) => {
            match level {
                LogLevel::Error => model.show_error = !model.show_error,
                LogLevel::Warn => model.show_warn = !model.show_warn,
                LogLevel::Info => model.show_info = !model.show_info,
                LogLevel::Debug => model.show_debug = !model.show_debug,
            }
            Vec::new()
        }
        ConsoleIntent::SetSource(source) => {
            model.source = source.clone();
            Vec::new()
        }
        ConsoleIntent::SetQuery(query) => {
            model.query = query.clone();
            Vec::new()
        }
        ConsoleIntent::TogglePause => {
            if model.paused {
                resume(model);
            } else {
                let snapshot = log_entries();
                model.paused_len = snapshot.len();
                model.frozen = Some(snapshot);
                model.paused = true;
            }
            Vec::new()
        }
        ConsoleIntent::ToggleAutoscroll => {
            model.autoscroll = !model.autoscroll;
            Vec::new()
        }
        ConsoleIntent::CopyAll => vec![Effect::CopyFiltered],
        ConsoleIntent::ExportLogs => vec![Effect::RequestExportPath],
        ConsoleIntent::ExportPathChosen(path) => match path {
            Some(chosen) => vec![Effect::WriteExportFile {
                path: chosen.clone(),
            }],
            None => Vec::new(),
        },
        ConsoleIntent::Clear => {
            crate::services::log_buffer::clear_logs();
            model.paused_len = 0;
            if model.paused {
                model.frozen = Some(Vec::new());
            }
            vec![Effect::PushLog {
                level: LogLevel::Info,
                source: "console".to_string(),
                message: "console cleared".to_string(),
            }]
        }
        ConsoleIntent::JumpToBottom => {
            resume(model);
            model.autoscroll = true;
            Vec::new()
        }
    }
}

fn resume(model: &mut ConsoleModel) {
    model.paused = false;
    model.frozen = None;
    model.paused_len = 0;
}

fn level_enabled(state: &ConsoleModel, level: LogLevel) -> bool {
    match level {
        LogLevel::Error => state.show_error,
        LogLevel::Warn => state.show_warn,
        LogLevel::Info => state.show_info,
        LogLevel::Debug => state.show_debug,
    }
}

pub fn matches_query(entry: &LogEntry, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let raw = query.as_bytes();
    if raw.len() >= 2 && raw[0] == b'/' && raw[raw.len() - 1] == b'/' {
        let inner = &query[1..query.len() - 1];
        if inner.is_empty() {
            return true;
        }
        let haystack = format!("{} {}", entry.source, entry.message);
        return haystack.contains(inner);
    }
    let needle = query.to_lowercase();
    let haystack = format!("{} {}", entry.source, entry.message).to_lowercase();
    haystack.contains(&needle)
}

pub fn matches_filters(entry: &LogEntry, state: &ConsoleModel) -> bool {
    if !level_enabled(state, entry.level) {
        return false;
    }
    if !(state.source.is_empty() || state.source == "all" || entry.source == state.source) {
        return false;
    }
    matches_query(entry, &state.query)
}

pub fn available_sources(entries: &[LogEntry]) -> Vec<String> {
    let mut sources: Vec<String> = entries.iter().map(|entry| entry.source.clone()).collect();
    sources.sort();
    sources.dedup();
    sources
}

pub fn level_counts(entries: &[LogEntry]) -> [usize; 4] {
    let mut counts = [0_usize; 4];
    for entry in entries {
        counts[level_index(entry.level)] += 1;
    }
    counts
}

fn level_index(level: LogLevel) -> usize {
    match level {
        LogLevel::Error => 0,
        LogLevel::Warn => 1,
        LogLevel::Info => 2,
        LogLevel::Debug => 3,
    }
}

pub fn tail_jump_count(state: &ConsoleModel, current_len: usize) -> usize {
    if !state.paused {
        return 0;
    }
    let frozen_len = state.frozen.as_ref().map_or(state.paused_len, Vec::len);
    current_len.saturating_sub(frozen_len)
}

pub fn format_log_line(entry: &LogEntry) -> String {
    format!(
        "{} {:<5} {} {}",
        entry.timestamp,
        entry.level.label(),
        entry.source,
        entry.message
    )
}

pub fn filtered_lines(entries: &[LogEntry], state: &ConsoleModel) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| matches_filters(entry, state))
        .map(format_log_line)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(level: LogLevel, source: &str, message: &str) -> LogEntry {
        LogEntry {
            timestamp: "2026-01-01 10:00:00".to_string(),
            level,
            source: source.to_string(),
            message: message.to_string(),
        }
    }

    fn state() -> ConsoleModel {
        ConsoleModel::default()
    }

    #[test]
    fn defaults_match_canvas() {
        let model = state();
        assert!(model.show_error);
        assert!(model.show_warn);
        assert!(model.show_info);
        assert!(!model.show_debug);
        assert_eq!(model.source, "all");
        assert!(model.query.is_empty());
        assert!(!model.paused);
        assert!(model.autoscroll);
        assert_eq!(model.paused_len, 0);
        assert!(model.frozen.is_none());
    }

    #[test]
    fn level_toggle_filters_matrix() {
        let entries = [
            entry(LogLevel::Error, "a", "e"),
            entry(LogLevel::Warn, "a", "w"),
            entry(LogLevel::Info, "a", "i"),
            entry(LogLevel::Debug, "a", "d"),
        ];
        let model = state();
        let visible: Vec<_> = entries
            .iter()
            .filter(|e| matches_filters(e, &model))
            .collect();
        assert_eq!(visible.len(), 3);
        let mut toggled = state();
        apply(&mut toggled, &ConsoleIntent::ToggleLevel(LogLevel::Debug));
        assert!(toggled.show_debug);
        let visible: Vec<_> = entries
            .iter()
            .filter(|e| matches_filters(e, &toggled))
            .collect();
        assert_eq!(visible.len(), 4);
        apply(&mut toggled, &ConsoleIntent::ToggleLevel(LogLevel::Error));
        let visible: Vec<_> = entries
            .iter()
            .filter(|e| matches_filters(e, &toggled))
            .collect();
        assert_eq!(visible.len(), 3);
        assert!(visible.iter().all(|e| e.level != LogLevel::Error));
    }

    #[test]
    fn source_and_query_combine() {
        let entries = [
            entry(LogLevel::Info, "net", "download started"),
            entry(LogLevel::Info, "disk", "download saved"),
            entry(LogLevel::Error, "net", "connection reset"),
        ];
        let mut model = state();
        apply(&mut model, &ConsoleIntent::SetSource("net".to_string()));
        apply(&mut model, &ConsoleIntent::SetQuery("download".to_string()));
        let visible: Vec<_> = entries
            .iter()
            .filter(|e| matches_filters(e, &model))
            .collect();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].message, "download started");
    }

    #[test]
    fn query_plain_is_case_insensitive_over_source_and_message() {
        let from_source = entry(LogLevel::Info, "DownLoader", "idle");
        let from_message = entry(LogLevel::Info, "net", "DOWNLOAD started");
        assert!(matches_query(&from_source, "downloader"));
        assert!(matches_query(&from_message, "download STARTED"));
        assert!(!matches_query(&from_message, "missing"));
        assert!(matches_query(&from_message, ""));
    }

    #[test]
    fn query_slash_wrapped_is_case_sensitive() {
        let found = entry(LogLevel::Info, "net", "Download Started");
        assert!(matches_query(&found, "/Download/"));
        assert!(!matches_query(&found, "/download/"));
        assert!(matches_query(&found, "//"));
    }

    #[test]
    fn query_unclosed_slash_falls_back_to_plain() {
        let found = entry(LogLevel::Info, "net", "path /Download here");
        assert!(matches_query(&found, "/download"));
        assert!(matches_query(&found, "/DOWNLOAD"));
        let plain = entry(LogLevel::Info, "net", "Download Started");
        assert!(!matches_query(&plain, "/download"));
    }

    #[test]
    fn sources_sorted_and_deduped() {
        let entries = vec![
            entry(LogLevel::Info, "zeta", "1"),
            entry(LogLevel::Info, "alpha", "2"),
            entry(LogLevel::Info, "zeta", "3"),
        ];
        assert_eq!(available_sources(&entries), vec!["alpha", "zeta"]);
        assert!(available_sources(&[]).is_empty());
    }

    #[test]
    fn counts_follow_error_warn_info_debug_order() {
        let entries = vec![
            entry(LogLevel::Error, "a", "1"),
            entry(LogLevel::Error, "a", "2"),
            entry(LogLevel::Warn, "a", "3"),
            entry(LogLevel::Debug, "a", "4"),
        ];
        assert_eq!(level_counts(&entries), [2, 1, 0, 1]);
    }

    #[test]
    fn tail_jump_math_uses_frozen_len() {
        let mut model = state();
        assert_eq!(tail_jump_count(&model, 100), 0);
        model.paused = true;
        model.paused_len = 10;
        model.frozen = Some(vec![entry(LogLevel::Info, "a", "x"); 10]);
        assert_eq!(tail_jump_count(&model, 13), 3);
        assert_eq!(tail_jump_count(&model, 8), 0);
    }

    #[test]
    fn format_line_exact_string() {
        let line = format_log_line(&entry(LogLevel::Info, "net", "hello"));
        assert_eq!(line, "2026-01-01 10:00:00 INFO  net hello");
        let warn = format_log_line(&entry(LogLevel::Warn, "net", "hello"));
        assert_eq!(warn, "2026-01-01 10:00:00 WARN  net hello");
    }

    #[test]
    fn pause_engage_freezes_and_resume_clears() {
        crate::services::log_buffer::clear_logs();
        crate::services::log_buffer::push_log(LogLevel::Info, "net", "one");
        crate::services::log_buffer::push_log(LogLevel::Info, "net", "two");
        let mut model = state();
        apply(&mut model, &ConsoleIntent::TogglePause);
        assert!(model.paused);
        assert_eq!(model.paused_len, 2);
        assert_eq!(model.frozen.as_ref().map(Vec::len), Some(2));
        crate::services::log_buffer::push_log(LogLevel::Info, "net", "three");
        let frozen_len = model.frozen.as_ref().map_or(0, Vec::len);
        assert_eq!(frozen_len, 2);
        assert_eq!(tail_jump_count(&model, 3), 1);
        apply(&mut model, &ConsoleIntent::TogglePause);
        assert!(!model.paused);
        assert!(model.frozen.is_none());
        assert_eq!(model.paused_len, 0);
        crate::services::log_buffer::clear_logs();
    }

    #[test]
    fn jump_to_bottom_resumes_and_enables_autoscroll() {
        let mut model = state();
        model.paused = true;
        model.autoscroll = false;
        model.paused_len = 5;
        model.frozen = Some(Vec::new());
        apply(&mut model, &ConsoleIntent::JumpToBottom);
        assert!(!model.paused);
        assert!(model.autoscroll);
        assert!(model.frozen.is_none());
    }

    #[test]
    fn copy_export_intents_emit_effects_without_executing() {
        let mut model = state();
        let effects = apply(&mut model, &ConsoleIntent::CopyAll);
        assert!(matches!(effects.as_slice(), [Effect::CopyFiltered]));
        let effects = apply(&mut model, &ConsoleIntent::ExportLogs);
        assert!(matches!(effects.as_slice(), [Effect::RequestExportPath]));
        let effects = apply(&mut model, &ConsoleIntent::ExportPathChosen(None));
        assert!(effects.is_empty());
        let effects = apply(
            &mut model,
            &ConsoleIntent::ExportPathChosen(Some("/tmp/out.log".into())),
        );
        assert!(matches!(
            effects.as_slice(),
            [Effect::WriteExportFile { .. }]
        ));
    }

    #[test]
    fn clear_keeps_behavior_and_resets_pause_bookkeeping() {
        crate::services::log_buffer::clear_logs();
        let mut model = state();
        let effects = apply(&mut model, &ConsoleIntent::Clear);
        assert!(matches!(
            effects.as_slice(),
            [Effect::PushLog { message, .. }] if message == "console cleared"
        ));
    }

    #[test]
    fn filtered_lines_share_format_with_export() {
        let entries = vec![
            entry(LogLevel::Info, "net", "hello"),
            entry(LogLevel::Debug, "net", "hidden"),
        ];
        let lines = filtered_lines(&entries, &state());
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], format_log_line(&entries[0]));
    }

    #[test]
    fn autoscroll_toggle_flips() {
        let mut model = state();
        apply(&mut model, &ConsoleIntent::ToggleAutoscroll);
        assert!(!model.autoscroll);
    }
}
