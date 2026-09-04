use egui::{Align, Layout, RichText, Ui};

use crate::core::intent::{AppIntent, SettingsIntent};
use crate::core::state::SettingsState;
use crate::core::store::Store;
use crate::i18n::registry::t;
use crate::services::traits::{TranscriptFormat, VideoQuality};
use crate::services::yt_dlp::binary::resolve_binary;
use crate::storage::config::{logs_dir, LogLevelSetting};

const LANG_OPTIONS: [&str; 3] = ["pt", "en", "es"];
const FALLBACK_OPTIONS: [&str; 4] = ["off", "en", "es", "pt"];

fn dispatch(store: &mut Store, intent: SettingsIntent) {
    store.dispatch(AppIntent::Settings(intent));
}

fn display_name(locale: &str) -> String {
    match locale {
        "en-US" => t("lang_en_US"),
        "pt-BR" => t("lang_pt_BR"),
        _ => locale.to_string(),
    }
}

pub fn render(ui: &mut Ui, store: &mut Store) {
    ui.heading(t("settings_title"));
    ui.label(t("settings_subtitle"));
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
        render_downloads(ui, store);
        ui.add_space(8.0);
        render_transcript(ui, store);
        ui.add_space(8.0);
        render_app(ui, store);
        ui.add_space(8.0);
        render_language(ui, store);
        ui.add_space(8.0);
        render_system(ui, store);
    });
}

fn group(ui: &mut Ui, title_key: &str, desc_key: &str, body: impl FnOnce(&mut Ui)) {
    ui.group(|ui| {
        ui.label(RichText::new(t(title_key)).strong());
        ui.label(RichText::new(t(desc_key)).small().weak());
        ui.separator();
        body(ui);
    });
}

fn row(ui: &mut Ui, title_key: &str, desc_key: &str, control: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(t(title_key)).strong());
            ui.label(RichText::new(t(desc_key)).small().weak());
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            control(ui);
        });
    });
    ui.separator();
}

fn toggle(
    ui: &mut Ui,
    store: &mut Store,
    current: bool,
    intent: impl FnOnce(bool) -> SettingsIntent,
) {
    let mut value = current;
    if ui.checkbox(&mut value, "").changed() {
        dispatch(store, intent(value));
    }
}

fn render_downloads(ui: &mut Ui, store: &mut Store) {
    let state = store.state().settings.clone();
    group(
        ui,
        "settings_group_downloads",
        "settings_group_downloads_desc",
        |ui| {
            row(
                ui,
                "settings_download_dir_title",
                "settings_download_dir_desc",
                |ui| {
                    ui.horizontal(|ui| {
                        let label = state
                            .download_dir
                            .as_ref()
                            .map(|path| path.display().to_string())
                            .unwrap_or_else(|| t("settings_download_dir_default"));
                        ui.label(RichText::new(label).monospace().small());
                        if ui.button(t("settings_download_dir_change")).clicked() {
                            dispatch(store, SettingsIntent::PickDownloadDir);
                        }
                        if state.download_dir.is_some()
                            && ui.button(t("settings_download_dir_reset")).clicked()
                        {
                            dispatch(store, SettingsIntent::SetDownloadDir(None));
                        }
                    });
                },
            );
            row(
                ui,
                "settings_quality_title",
                "settings_quality_desc",
                |ui| {
                    let mut quality = state.quality_default;
                    egui::ComboBox::from_id_source("settings_quality")
                        .selected_text(t(quality.locale_key()))
                        .show_ui(ui, |ui| {
                            for option in VideoQuality::ordered() {
                                ui.selectable_value(&mut quality, option, t(option.locale_key()));
                            }
                        });
                    if quality != state.quality_default {
                        dispatch(store, SettingsIntent::SetQualityDefault(quality));
                    }
                },
            );
            row(
                ui,
                "settings_simultaneous_title",
                "settings_simultaneous_desc",
                |ui| {
                    let mut count = state.simultaneous;
                    egui::ComboBox::from_id_source("settings_simultaneous")
                        .selected_text(count.to_string())
                        .show_ui(ui, |ui| {
                            for option in 1..=5u8 {
                                ui.selectable_value(&mut count, option, option.to_string());
                            }
                        });
                    if count != state.simultaneous {
                        dispatch(store, SettingsIntent::SetSimultaneous(count));
                    }
                },
            );
            row(
                ui,
                "settings_organize_title",
                "settings_organize_desc",
                |ui| {
                    toggle(
                        ui,
                        store,
                        state.organize_by_channel,
                        SettingsIntent::SetOrganize,
                    );
                },
            );
        },
    );
}

fn render_transcript(ui: &mut Ui, store: &mut Store) {
    let state: SettingsState = store.state().settings.clone();
    group(
        ui,
        "settings_group_transcript",
        "settings_group_transcript_desc",
        |ui| {
            row(ui, "settings_tlang_title", "settings_tlang_desc", |ui| {
                let mut lang = state.transcript_lang.clone();
                egui::ComboBox::from_id_source("settings_tlang")
                    .selected_text(lang.clone())
                    .show_ui(ui, |ui| {
                        for option in LANG_OPTIONS {
                            ui.selectable_value(&mut lang, option.to_string(), option);
                        }
                    });
                if lang != state.transcript_lang {
                    dispatch(store, SettingsIntent::SetTranscriptLang(lang));
                }
            });
            row(
                ui,
                "settings_tformat_title",
                "settings_tformat_desc",
                |ui| {
                    let mut format = state.transcript_format;
                    egui::ComboBox::from_id_source("settings_tformat")
                        .selected_text(t(format.locale_key()))
                        .show_ui(ui, |ui| {
                            for option in TranscriptFormat::ordered() {
                                ui.selectable_value(&mut format, option, t(option.locale_key()));
                            }
                        });
                    if format != state.transcript_format {
                        dispatch(store, SettingsIntent::SetTranscriptFormat(format));
                    }
                },
            );
            row(
                ui,
                "settings_tfallback_title",
                "settings_tfallback_desc",
                |ui| {
                    let mut fallback = state
                        .transcript_fallback
                        .clone()
                        .unwrap_or_else(|| "off".to_string());
                    egui::ComboBox::from_id_source("settings_tfallback")
                        .selected_text(fallback_label(&fallback))
                        .show_ui(ui, |ui| {
                            for option in FALLBACK_OPTIONS {
                                ui.selectable_value(
                                    &mut fallback,
                                    option.to_string(),
                                    fallback_label(option),
                                );
                            }
                        });
                    let current = state
                        .transcript_fallback
                        .clone()
                        .unwrap_or_else(|| "off".to_string());
                    if fallback != current {
                        let value = if fallback == "off" {
                            None
                        } else {
                            Some(fallback)
                        };
                        dispatch(store, SettingsIntent::SetTranscriptFallback(value));
                    }
                },
            );
            row(ui, "settings_tauto_title", "settings_tauto_desc", |ui| {
                toggle(ui, store, state.transcript_accept_auto, |_| {
                    SettingsIntent::ToggleTranscriptAuto
                });
            });
            row(
                ui,
                "settings_ttimestamps_title",
                "settings_ttimestamps_desc",
                |ui| {
                    toggle(ui, store, state.transcript_timestamps, |_| {
                        SettingsIntent::ToggleTranscriptTimestamps
                    });
                },
            );
        },
    );
}

fn fallback_label(option: &str) -> String {
    if option == "off" {
        t("transcript_fallback_off")
    } else {
        option.to_string()
    }
}

fn render_app(ui: &mut Ui, store: &mut Store) {
    let state = store.state().settings.clone();
    group(ui, "settings_group_app", "settings_group_app_desc", |ui| {
        row(ui, "settings_hotkey_title", "settings_hotkey_desc", |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Win+Shift+X").monospace().strong());
                ui.label(RichText::new(t("settings_hotkey_note")).small().weak());
            });
        });
        row(
            ui,
            "settings_autostart_title",
            "settings_autostart_desc",
            |ui| {
                ui.horizontal(|ui| {
                    toggle(
                        ui,
                        store,
                        state.start_with_windows,
                        SettingsIntent::SetAutostart,
                    );
                    ui.label(RichText::new(autostart_state(&state)).small().weak());
                });
            },
        );
        #[cfg(not(windows))]
        {
            ui.label(
                RichText::new(t("settings_autostart_unsupported"))
                    .small()
                    .weak(),
            );
            ui.separator();
        }
        row(ui, "settings_tray_title", "settings_tray_desc", |ui| {
            ui.horizontal(|ui| {
                toggle(
                    ui,
                    store,
                    state.minimize_to_tray,
                    SettingsIntent::SetMinimizeTray,
                );
                ui.label(RichText::new(t("settings_tray_note")).small().weak());
            });
        });
        row(ui, "settings_notify_title", "settings_notify_desc", |ui| {
            toggle(
                ui,
                store,
                state.notify_on_complete,
                SettingsIntent::SetNotify,
            );
        });
    });
}

fn autostart_state(state: &SettingsState) -> String {
    match state.autostart_actual {
        Some(true) => t("settings_autostart_actual_on"),
        Some(false) => t("settings_autostart_actual_off"),
        None => t("settings_autostart_unknown"),
    }
}

fn render_language(ui: &mut Ui, store: &mut Store) {
    let current = store.state().locale.clone();
    group(
        ui,
        "settings_group_language",
        "settings_group_language_desc",
        |ui| {
            let mut selected = current.clone();
            egui::ComboBox::from_label(t("settings_language_label"))
                .selected_text(display_name(selected.as_str()))
                .show_ui(ui, |ui| {
                    for locale in store.available_locales() {
                        ui.selectable_value(
                            &mut selected,
                            locale.clone(),
                            display_name(locale.as_str()),
                        );
                    }
                });
            if selected != current {
                dispatch(store, SettingsIntent::ChangeLocale(selected));
            }
        },
    );
}

fn render_system(ui: &mut Ui, store: &mut Store) {
    let state = store.state().settings.clone();
    group(
        ui,
        "settings_group_system",
        "settings_group_system_desc",
        |ui| {
            row(
                ui,
                "settings_loglevel_title",
                "settings_loglevel_desc",
                |ui| {
                    let mut level = state.log_level;
                    egui::ComboBox::from_id_source("settings_loglevel")
                        .selected_text(t(level.locale_key()))
                        .show_ui(ui, |ui| {
                            for option in LogLevelSetting::ordered() {
                                ui.selectable_value(&mut level, option, t(option.locale_key()));
                            }
                        });
                    if level != state.log_level {
                        dispatch(store, SettingsIntent::SetLogLevel(level));
                    }
                },
            );
            row(
                ui,
                "settings_savelogs_title",
                "settings_savelogs_desc",
                |ui| {
                    ui.horizontal(|ui| {
                        toggle(ui, store, state.save_logs, SettingsIntent::SetSaveLogs);
                        if let Some(dir) = logs_dir() {
                            ui.label(RichText::new(dir.display().to_string()).monospace().small());
                        }
                    });
                },
            );
            row(ui, "settings_ytdlp_title", "settings_ytdlp_desc", |ui| {
                ui.horizontal(|ui| {
                    if state.ytdlp_checking {
                        ui.spinner();
                        ui.label(t("settings_ytdlp_checking"));
                    } else if let Some(version) = &state.ytdlp_version {
                        ui.label(format!("{} {version}", t("settings_ytdlp_version")));
                    } else {
                        ui.label(t("settings_ytdlp_unknown"));
                    }
                    if ui.button(t("settings_ytdlp_check")).clicked() {
                        dispatch(store, SettingsIntent::CheckYtDlp);
                    }
                    if ui.button(t("settings_ytdlp_update")).clicked() {
                        dispatch(store, SettingsIntent::UpdateYtDlp);
                    }
                });
            });
            if let Some(key) = &state.ytdlp_error {
                ui.colored_label(egui::Color32::LIGHT_RED, t(key));
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new(t("settings_sidecar_title")).strong());
                match resolve_binary() {
                    Ok(path) => {
                        ui.label(
                            RichText::new(path.display().to_string())
                                .monospace()
                                .small(),
                        );
                    }
                    Err(key) => {
                        ui.colored_label(egui::Color32::LIGHT_RED, t(&key));
                    }
                }
            });
            ui.label(RichText::new(t("settings_sidecar_desc")).small().weak());
        },
    );
}
