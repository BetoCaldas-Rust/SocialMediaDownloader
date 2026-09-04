use egui::{Color32, RichText, ScrollArea, Ui};

use crate::core::intent::{AppIntent, ConsoleIntent};
use crate::core::state::ConsoleState;
use crate::core::store::Store;
use crate::features::console::update::{
    available_sources, level_counts, matches_filters, tail_jump_count,
};
use crate::i18n::registry::t;
use crate::services::log_buffer::{log_entries, log_len, LogEntry, LogLevel};
use crate::ui::components::{card, page_header};
use crate::ui::theme::{
    LOG_DEBUG, LOG_ERROR, LOG_ERROR_BG, LOG_INFO, LOG_SOURCE, LOG_WARN, SUCCESS_COLOR,
    TEXT_DISABLED, TEXT_PRIMARY, TEXT_SECONDARY,
};

fn level_color(level: LogLevel) -> Color32 {
    match level {
        LogLevel::Error => LOG_ERROR,
        LogLevel::Warn => LOG_WARN,
        LogLevel::Info => LOG_INFO,
        LogLevel::Debug => LOG_DEBUG,
    }
}

fn dispatch(store: &mut Store, intent: ConsoleIntent) {
    store.dispatch(AppIntent::Console(intent));
}

pub fn render(ui: &mut Ui, store: &mut Store) {
    let state = store.state().console.clone();
    page_header(ui, "console_title", "console_subtitle");
    card(ui, |ui| {
        render_toolbar(ui, store, &state);
    });
    ui.add_space(10.0);
    let live_len = log_len();
    let base: Vec<LogEntry> = if state.paused {
        state.frozen.clone().unwrap_or_default()
    } else {
        log_entries()
    };
    let visible: Vec<&LogEntry> = base
        .iter()
        .filter(|entry| matches_filters(entry, &state))
        .collect();
    render_log(ui, &state, &visible);
    render_tail_jump(ui, store, tail_jump_count(&state, live_len));
    ui.add_space(8.0);
    render_statusbar(ui, &state, live_len, &visible);
}

fn render_toolbar(ui: &mut Ui, store: &mut Store, state: &ConsoleState) {
    let entries = log_entries();
    let counts = level_counts(&entries);
    let sources = available_sources(&entries);
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new(t("console_levels"))
                .small()
                .color(TEXT_SECONDARY),
        );
        render_chip(ui, store, LogLevel::Error, state.show_error, counts[0]);
        render_chip(ui, store, LogLevel::Warn, state.show_warn, counts[1]);
        render_chip(ui, store, LogLevel::Info, state.show_info, counts[2]);
        render_chip(ui, store, LogLevel::Debug, state.show_debug, counts[3]);
        ui.separator();
        render_source_picker(ui, store, state, &sources);
        render_search(ui, store, state);
        ui.separator();
        render_buttons(ui, store, state);
    });
}

fn render_chip(ui: &mut Ui, store: &mut Store, level: LogLevel, active: bool, count: usize) {
    let text = format!("● {} {count}", level.label());
    if active {
        let response = ui.add(
            egui::Button::new(
                RichText::new(text)
                    .color(level_color(level))
                    .monospace()
                    .small(),
            )
            .rounding(10.0),
        );
        if response.clicked() {
            dispatch(store, ConsoleIntent::ToggleLevel(level));
        }
    } else if ui
        .selectable_label(
            false,
            RichText::new(text).color(TEXT_DISABLED).monospace().small(),
        )
        .clicked()
    {
        dispatch(store, ConsoleIntent::ToggleLevel(level));
    }
}

fn render_source_picker(ui: &mut Ui, store: &mut Store, state: &ConsoleState, sources: &[String]) {
    let mut current = state.source.clone();
    let selected = if current == "all" {
        t("console_source_all")
    } else {
        current.clone()
    };
    egui::ComboBox::from_id_source("console-source")
        .selected_text(selected)
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut current, "all".to_string(), t("console_source_all"));
            for source in sources {
                ui.selectable_value(&mut current, source.clone(), source);
            }
        });
    if current != state.source {
        dispatch(store, ConsoleIntent::SetSource(current));
    }
}

fn render_search(ui: &mut Ui, store: &mut Store, state: &ConsoleState) {
    let mut query = state.query.clone();
    let field = ui.add(
        egui::TextEdit::singleline(&mut query)
            .hint_text(t("console_search_hint"))
            .desired_width(220.0),
    );
    if field.changed() {
        dispatch(store, ConsoleIntent::SetQuery(query));
    }
}

fn render_buttons(ui: &mut Ui, store: &mut Store, state: &ConsoleState) {
    if ui
        .selectable_label(state.autoscroll, "⤓")
        .on_hover_text(t("console_tooltip_autoscroll"))
        .clicked()
    {
        dispatch(store, ConsoleIntent::ToggleAutoscroll);
    }
    let (glyph, tip) = if state.paused {
        ("▶", t("console_tooltip_resume"))
    } else {
        ("⏸", t("console_tooltip_pause"))
    };
    if ui.button(glyph).on_hover_text(tip).clicked() {
        dispatch(store, ConsoleIntent::TogglePause);
    }
    if ui
        .button("⧉")
        .on_hover_text(t("console_tooltip_copy"))
        .clicked()
    {
        dispatch(store, ConsoleIntent::CopyAll);
    }
    if ui
        .button("↓")
        .on_hover_text(t("console_tooltip_export"))
        .clicked()
    {
        dispatch(store, ConsoleIntent::ExportLogs);
    }
    if ui
        .button("🗑")
        .on_hover_text(t("console_tooltip_clear"))
        .clicked()
    {
        dispatch(store, ConsoleIntent::Clear);
    }
}

fn render_log(ui: &mut Ui, state: &ConsoleState, visible: &[&LogEntry]) {
    ui.group(|ui| {
        ScrollArea::vertical()
            .max_height(380.0)
            .stick_to_bottom(state.autoscroll && !state.paused)
            .show(ui, |ui| {
                for entry in visible {
                    if entry.level == LogLevel::Error {
                        egui::Frame::none().fill(LOG_ERROR_BG).show(ui, |ui| {
                            render_row(ui, entry);
                        });
                    } else {
                        render_row(ui, entry);
                    }
                }
            });
    });
}

fn render_row(ui: &mut Ui, entry: &LogEntry) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.label(
            RichText::new(entry.timestamp.as_str())
                .monospace()
                .color(TEXT_SECONDARY),
        );
        ui.label(
            RichText::new(entry.level.label())
                .monospace()
                .color(level_color(entry.level)),
        );
        ui.label(
            RichText::new(entry.source.as_str())
                .monospace()
                .color(LOG_SOURCE),
        );
        ui.label(
            RichText::new(entry.message.as_str())
                .monospace()
                .color(TEXT_PRIMARY),
        );
    });
}

fn render_tail_jump(ui: &mut Ui, store: &mut Store, pending: usize) {
    if pending == 0 {
        return;
    }
    ui.vertical_centered(|ui| {
        let label = t("console_tail_jump").replace("{n}", &pending.to_string());
        if ui.button(label).clicked() {
            dispatch(store, ConsoleIntent::JumpToBottom);
        }
    });
}

fn render_statusbar(ui: &mut Ui, state: &ConsoleState, live_len: usize, visible: &[&LogEntry]) {
    let visible_label = t("console_status_visible")
        .replace("{v}", &visible.len().to_string())
        .replace("{t}", &live_len.to_string());
    let last_label = match visible.last() {
        Some(entry) => t("console_status_last").replace("{ts}", &entry.timestamp),
        None => t("console_status_last").replace("{ts}", "—"),
    };
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            if state.paused {
                ui.colored_label(LOG_WARN, t("console_status_paused"));
            } else {
                ui.colored_label(SUCCESS_COLOR, t("console_status_live"));
            }
            ui.label("·");
            ui.label(visible_label);
            ui.label("·");
            ui.label(t("console_status_buffer"));
            ui.label("·");
            ui.label(last_label);
        });
    });
}
