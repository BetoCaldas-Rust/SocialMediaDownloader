use egui::Ui;

use crate::core::intent::{AppIntent, TranscriptIntent};
use crate::core::state::{TranscriptState, TranscriptStatus};
use crate::core::store::Store;
use crate::i18n::registry::t;
use crate::services::traits::TranscriptFormat;
use crate::services::yt_dlp::transcript::{
    parse_no_transcript_error, transcript_error_display_key,
};

const LANG_OPTIONS: [&str; 3] = ["pt", "en", "es"];
const FALLBACK_OPTIONS: [&str; 4] = ["off", "en", "es", "pt"];

pub fn render(ui: &mut Ui, store: &mut Store) {
    let state = store.state().transcript.clone();
    ui.heading(t("transcript_title"));
    ui.label(t("transcript_subtitle"));
    ui.add_space(8.0);
    render_url_card(ui, store, &state);
    ui.add_space(8.0);
    render_options(ui, store, &state);
    ui.add_space(8.0);
    render_actions(ui, store, &state);
    render_error(ui, store, &state);
    render_recents(ui, store, &state);
}

fn dispatch(store: &mut Store, intent: TranscriptIntent) {
    store.dispatch(AppIntent::Transcript(intent));
}

fn render_url_card(ui: &mut Ui, store: &mut Store, state: &TranscriptState) {
    ui.group(|ui| {
        ui.label(t("transcript_url_label"));
        ui.horizontal(|ui| {
            let mut input = state.input.clone();
            let field =
                ui.add(egui::TextEdit::singleline(&mut input).hint_text(t("transcript_url_hint")));
            if field.changed() {
                dispatch(store, TranscriptIntent::SetInput(input));
            }
            let busy = state.status.is_busy();
            let label = if busy {
                t("transcript_fetching")
            } else {
                t("transcript_fetch_button")
            };
            let ready = !busy && !state.input.trim().is_empty();
            if ui.add_enabled(ready, egui::Button::new(label)).clicked() {
                dispatch(store, TranscriptIntent::FetchTranscript);
            }
        });
    });
}

fn render_options(ui: &mut Ui, store: &mut Store, state: &TranscriptState) {
    let locked = state.status.is_busy();
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!locked, |ui| {
                let mut lang = state.lang.clone();
                egui::ComboBox::from_label(t("transcript_lang_label"))
                    .selected_text(lang.clone())
                    .show_ui(ui, |ui| {
                        for option in LANG_OPTIONS {
                            ui.selectable_value(&mut lang, option.to_string(), option);
                        }
                    });
                if lang != state.lang {
                    dispatch(store, TranscriptIntent::SetLang(lang));
                }
                let mut fallback = state.fallback.clone();
                egui::ComboBox::from_label(t("transcript_fallback_label"))
                    .selected_text(fallback.clone())
                    .show_ui(ui, |ui| {
                        for option in FALLBACK_OPTIONS {
                            ui.selectable_value(
                                &mut fallback,
                                option.to_string(),
                                t_fallback(option),
                            );
                        }
                    });
                if fallback != state.fallback {
                    dispatch(store, TranscriptIntent::SetFallback(fallback));
                }
            });
        });
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!locked, |ui| {
                let mut format = state.format;
                egui::ComboBox::from_label(t("transcript_format_label"))
                    .selected_text(t(format.locale_key()))
                    .show_ui(ui, |ui| {
                        for option in TranscriptFormat::ordered() {
                            ui.selectable_value(&mut format, option, t(option.locale_key()));
                        }
                    });
                if format != state.format {
                    dispatch(store, TranscriptIntent::SetFormat(format));
                }
                let mut auto = state.accept_auto;
                if ui.checkbox(&mut auto, t("transcript_auto_label")).changed() {
                    dispatch(store, TranscriptIntent::ToggleAuto);
                }
                let mut timestamps = state.timestamps;
                if ui
                    .checkbox(&mut timestamps, t("transcript_timestamps_label"))
                    .changed()
                {
                    dispatch(store, TranscriptIntent::ToggleTimestamps);
                }
            });
        });
        ui.label(t("transcript_format_hint"));
    });
}

fn render_actions(ui: &mut Ui, store: &mut Store, state: &TranscriptState) {
    ui.horizontal(|ui| {
        if state.status.is_busy() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(t("transcript_fetching"));
            });
            return;
        }
        if state.status == TranscriptStatus::Completed {
            ui.label(t("transcript_completed"));
        }
        let has_input = !state.input.trim().is_empty();
        let label = match state.status {
            TranscriptStatus::Failed => t("transcript_retry"),
            _ => t("transcript_fetch_button"),
        };
        let intent = match state.status {
            TranscriptStatus::Failed => TranscriptIntent::Retry,
            _ => TranscriptIntent::FetchTranscript,
        };
        if ui
            .add_enabled(has_input, egui::Button::new(label))
            .clicked()
        {
            dispatch(store, intent);
        }
    });
}

fn render_error(ui: &mut Ui, store: &mut Store, state: &TranscriptState) {
    if state.status != TranscriptStatus::Failed {
        return;
    }
    let message = error_message(state);
    ui.group(|ui| {
        ui.colored_label(egui::Color32::LIGHT_RED, message);
        ui.horizontal(|ui| {
            if ui.button(t("transcript_retry")).clicked() {
                dispatch(store, TranscriptIntent::Retry);
            }
            if ui.button(t("transcript_dismiss")).clicked() {
                dispatch(store, TranscriptIntent::Dismiss);
            }
        });
    });
}

fn render_recents(ui: &mut Ui, store: &mut Store, state: &TranscriptState) {
    ui.group(|ui| {
        ui.label(t("transcript_recents_title"));
        if state.recents.is_empty() {
            ui.label(t("transcript_recents_empty"));
            return;
        }
        for (index, entry) in state.recents.iter().enumerate() {
            ui.horizontal(|ui| {
                let name = entry
                    .path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("?");
                ui.label(name);
                ui.label(format!("[{}]", entry.lang_used));
                if entry.auto_generated {
                    ui.label(t("transcript_auto_tag"));
                }
                ui.label(format_size(entry.size_bytes));
                if ui.button(t("transcript_open_file")).clicked() {
                    dispatch(store, TranscriptIntent::RevealRecent(index));
                }
            });
        }
    });
}

fn error_message(state: &TranscriptState) -> String {
    let raw = state.error.as_deref().unwrap_or("transcript_error_fetch");
    if let Some(url) = parse_no_transcript_error(raw) {
        return format!("{}: {url}", t(transcript_error_display_key(raw)));
    }
    t(transcript_error_display_key(raw))
}

fn t_fallback(option: &str) -> String {
    if option == "off" {
        t("transcript_fallback_off")
    } else {
        option.to_string()
    }
}

fn format_size(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    let size = bytes as f64;
    if size >= MIB {
        format!("{:.1} MiB", size / MIB)
    } else if size >= KIB {
        format!("{:.0} KiB", size / KIB)
    } else {
        format!("{bytes} B")
    }
}
