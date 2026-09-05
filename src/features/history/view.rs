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
use crate::ui::components::{card, page_header, search_box};
use crate::ui::theme::{ACCENT_PRIMARY, LOG_ERROR, TEXT_SECONDARY};

pub fn render(ui: &mut Ui, store: &mut Store) {
    let entries = store.history_entries();
    let state = store.state().history.clone();
    page_header(ui, "history_title", "history_subtitle");
    card(ui, |ui| {
        render_toolbar(ui, store, &state, &entries);
    });
    ui.add_space(10.0);
    render_list(ui, store, &state, &entries);
    ui.add_space(10.0);
    render_statusbar(ui, store);
}

fn dispatch(store: &mut Store, intent: HistoryIntent) {
    store.dispatch(AppIntent::History(intent));
}

fn render_toolbar(ui: &mut Ui, store: &mut Store, state: &HistoryState, entries: &[HistoryEntry]) {
    ui.horizontal_wrapped(|ui| {
        for filter in HistoryFilter::ordered() {
            let count = apply_filter(entries, filter, "").len();
            let label = format!("{} {count}", t(filter.locale_key()));
            let selected = state.filter == filter;
            let response = if selected {
                ui.add(
                    egui::Button::new(
                        egui::RichText::new(label)
                            .small()
                            .color(crate::ui::theme::TEXT_ON_ACCENT),
                    )
                    .fill(ACCENT_PRIMARY)
                    .rounding(10.0),
                )
            } else {
                ui.add(egui::Button::new(egui::RichText::new(label).small()).rounding(10.0))
            };
            if response.clicked() {
                dispatch(store, HistoryIntent::SetFilter(filter));
            }
        }
    });
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        let mut query = state.query.clone();
        if search_box(ui, &mut query, t("history_search_hint"), 260.0) {
            dispatch(store, HistoryIntent::SetQuery(query));
        }
        let mut sort = state.sort;
        egui::ComboBox::from_id_source("history_sort")
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
    ui.add_space(6.0);
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
}

fn render_list(ui: &mut Ui, store: &mut Store, state: &HistoryState, entries: &[HistoryEntry]) {
    let filtered = apply_filter(entries, state.filter, &state.query);
    let sorted = apply_sort(filtered, state.sort);
    if sorted.is_empty() {
        card(ui, |ui| {
            ui.label(
                egui::RichText::new(t("history_empty")).color(TEXT_SECONDARY),
            );
        });
        return;
    }
    let now_ms = Local::now().timestamp_millis();
    card(ui, |ui| {
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
                ui.colored_label(LOG_ERROR, display_error(error));
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
    ui.label(
        egui::RichText::new(format!(
            "{} {} · {} {} · {} {} · {}",
            summary.count,
            t("history_status_items"),
            format_size(summary.bytes_completed),
            t("history_status_ondisk"),
            summary.failures,
            t("history_status_failures"),
            folder,
        ))
        .small()
        .color(TEXT_SECONDARY),
    );
}
