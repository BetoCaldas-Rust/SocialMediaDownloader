use chrono::Local;
use egui::Ui;

use crate::core::intent::{AppIntent, HistoryIntent};
use crate::core::state::{HistoryFilter, HistorySort, HistoryState};
use crate::core::store::Store;
use crate::features::history::update::{apply_filter, apply_sort, format_size, format_when};
use crate::i18n::registry::t;
use crate::services::traits::{EntryKind, HistoryEntry};
use crate::services::yt_dlp::transcript::{
    parse_no_transcript_error, transcript_error_display_key,
};

pub fn render(ui: &mut Ui, store: &mut Store) {
    let entries = store.history_entries();
    let state = store.state().history.clone();
    ui.heading(t("history_title"));
    ui.label(t("history_subtitle"));
    ui.add_space(8.0);
    render_toolbar(ui, store, &state, &entries);
    ui.add_space(8.0);
    render_list(ui, store, &state, &entries);
    ui.add_space(8.0);
    render_statusbar(ui, store);
}

fn dispatch(store: &mut Store, intent: HistoryIntent) {
    store.dispatch(AppIntent::History(intent));
}

fn render_toolbar(ui: &mut Ui, store: &mut Store, state: &HistoryState, entries: &[HistoryEntry]) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            for filter in HistoryFilter::ordered() {
                let count = apply_filter(entries, filter, "").len();
                let label = format!("{} ({count})", t(filter.locale_key()));
                if ui.selectable_label(state.filter == filter, label).clicked() {
                    dispatch(store, HistoryIntent::SetFilter(filter));
                }
            }
        });
        ui.horizontal(|ui| {
            let mut query = state.query.clone();
            let field =
                ui.add(egui::TextEdit::singleline(&mut query).hint_text(t("history_search_hint")));
            if field.changed() {
                dispatch(store, HistoryIntent::SetQuery(query));
            }
            let mut sort = state.sort;
            egui::ComboBox::from_label(t("history_sort_label"))
                .selected_text(t(sort.locale_key()))
                .show_ui(ui, |ui| {
                    for option in HistorySort::ordered() {
                        ui.selectable_value(&mut sort, option, t(option.locale_key()));
                    }
                });
            if sort != state.sort {
                dispatch(store, HistoryIntent::SetSort(sort));
            }
        });
        ui.horizontal(|ui| {
            if ui.button(t("history_open_folder")).clicked() {
                dispatch(store, HistoryIntent::OpenDownloadsFolder);
            }
            if state.confirm_clear {
                if ui.button(t("history_clear_confirm")).clicked() {
                    dispatch(store, HistoryIntent::ClearHistory);
                }
                if ui.button(t("history_clear_cancel")).clicked() {
                    dispatch(store, HistoryIntent::CancelClear);
                }
            } else if ui.button(t("history_clear")).clicked() {
                dispatch(store, HistoryIntent::RequestClear);
            }
        });
    });
}

fn render_list(ui: &mut Ui, store: &mut Store, state: &HistoryState, entries: &[HistoryEntry]) {
    let filtered = apply_filter(entries, state.filter, &state.query);
    let sorted = apply_sort(filtered, state.sort);
    if sorted.is_empty() {
        ui.group(|ui| {
            ui.label(t("history_empty"));
        });
        return;
    }
    let now_ms = Local::now().timestamp_millis();
    ui.group(|ui| {
        egui::ScrollArea::vertical()
            .max_height(380.0)
            .show(ui, |ui| {
                for entry in &sorted {
                    render_row(ui, store, entry, now_ms);
                    ui.separator();
                }
            });
    });
}

fn render_row(ui: &mut Ui, store: &mut Store, entry: &HistoryEntry, now_ms: i64) {
    ui.horizontal(|ui| {
        ui.label(match entry.kind {
            EntryKind::Video => "🎬",
            EntryKind::Transcript => "📄",
        });
        ui.vertical(|ui| {
            ui.label(entry.name.clone());
            if let Some(error) = &entry.error {
                ui.colored_label(egui::Color32::LIGHT_RED, display_error(error));
            }
            ui.label(egui::RichText::new(entry.source.clone()).small().weak());
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("↻").on_hover_text(t("history_retry")).clicked() {
                dispatch(store, HistoryIntent::RetryEntry(entry.id.clone()));
            }
            if entry.path.is_some() && ui.button("📂").on_hover_text(t("history_reveal")).clicked()
            {
                dispatch(store, HistoryIntent::RevealEntry(entry.id.clone()));
            }
            ui.label(
                egui::RichText::new(format_when(
                    entry.finished_at_ms,
                    now_ms,
                    &t("history_today"),
                    &t("history_yesterday"),
                ))
                .small()
                .weak(),
            );
            ui.label(egui::RichText::new(size_label(entry)).monospace().small());
        });
    });
}

fn display_error(error: &str) -> String {
    if let Some(url) = parse_no_transcript_error(error) {
        return format!("{}: {url}", t(transcript_error_display_key(error)));
    }
    t(error)
}

fn size_label(entry: &HistoryEntry) -> String {
    match entry.size_bytes {
        Some(bytes) => format_size(bytes),
        None => "—".to_string(),
    }
}

fn render_statusbar(ui: &mut Ui, store: &mut Store) {
    let summary = store.history_totals();
    let folder = store.download_dir().display().to_string();
    ui.group(|ui| {
        ui.label(format!(
            "{} {} · {} {} · {} {} · {}",
            summary.count,
            t("history_status_items"),
            format_size(summary.bytes_completed),
            t("history_status_ondisk"),
            summary.failures,
            t("history_status_failures"),
            folder,
        ));
    });
}
