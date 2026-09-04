use egui::{Color32, RichText, ScrollArea, Ui};

use crate::core::store::Store;
use crate::i18n::registry::t;
use crate::services::log_buffer::{log_entries, LogLevel};
use crate::ui::theme::{
    LOG_DEBUG, LOG_ERROR, LOG_INFO, LOG_SOURCE, LOG_WARN, TEXT_PRIMARY, TEXT_SECONDARY,
};

fn level_color(level: LogLevel) -> Color32 {
    match level {
        LogLevel::Error => LOG_ERROR,
        LogLevel::Warn => LOG_WARN,
        LogLevel::Info => LOG_INFO,
        LogLevel::Debug => LOG_DEBUG,
    }
}

pub fn render(ui: &mut Ui, store: &mut Store) {
    ui.heading(t("console_title"));
    ui.label(t("console_subtitle"));
    ui.separator();
    let threshold = store.state().settings.log_level;
    let entries = log_entries();
    ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
        for entry in entries.iter().filter(|entry| threshold.allows(entry.level)) {
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
    });
}
