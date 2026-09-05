use egui::Ui;

use crate::core::intent::{AppIntent, ChannelIntent};
use crate::core::state::{
    BatchItemStatus, ChannelInclude, ChannelState, ChannelStatus, DatePreset,
};
use crate::core::store::Store;
use crate::features::channel::model::{format_duration, selected_count, total_duration_secs};
use crate::i18n::registry::t;
use crate::services::traits::{ChannelPreview, VideoKind, VideoQuality};
use crate::ui::components::{
    card, field_label, full_primary, input_row, micro_label, page_header, section_title,
};
use crate::ui::theme::{ACCENT_PRIMARY, LOG_ERROR, TEXT_SECONDARY};

pub fn render(ui: &mut Ui, store: &mut Store) {
    let state = store.state().channel.clone();
    page_header(ui, "channel_title", "channel_subtitle");
    render_input_card(ui, store, &state);
    ui.add_space(10.0);
    card(ui, |ui| {
        render_period(ui, store, &state);
    });
    ui.add_space(10.0);
    card(ui, |ui| {
        render_options(ui, store, &state);
    });
    ui.add_space(10.0);
    render_preview(ui, store, &state);
    render_batch(ui, &state);
    render_error(ui, store, &state);
}

fn dispatch(store: &mut Store, intent: ChannelIntent) {
    store.dispatch(AppIntent::Channel(intent));
}

fn render_input_card(ui: &mut Ui, store: &mut Store, state: &ChannelState) {
    card(ui, |ui| {
        field_label(ui, t("channel_input_label"));
        let mut input = state.input.clone();
        let loading = state.status == ChannelStatus::Loading;
        let label = if loading {
            t("channel_fetching")
        } else {
            t("channel_fetch_button")
        };
        let ready = !state.status.is_busy() && !state.input.trim().is_empty();
        let (changed, clicked) = input_row(ui, &mut input, t("channel_input_hint"), label, ready);
        if changed {
            dispatch(store, ChannelIntent::SetInput(input));
        }
        if clicked {
            dispatch(store, ChannelIntent::FetchPreview);
        }
    });
}

fn render_period(ui: &mut Ui, store: &mut Store, state: &ChannelState) {
    section_title(ui, t("channel_period_label"));
    ui.add_enabled_ui(!state.status.is_busy(), |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                micro_label(ui, t("channel_from_label"));
                let mut from = state.from_text.clone();
                let from_field = ui.add(
                    egui::TextEdit::singleline(&mut from)
                        .desired_width(90.0)
                        .hint_text(t("channel_date_hint")),
                );
                if from_field.changed() {
                    dispatch(store, ChannelIntent::SetFromText(from));
                }
            });
            ui.vertical(|ui| {
                micro_label(ui, t("channel_to_label"));
                let mut to = state.to_text.clone();
                let to_field = ui.add(
                    egui::TextEdit::singleline(&mut to)
                        .desired_width(90.0)
                        .hint_text(t("channel_date_hint")),
                );
                if to_field.changed() {
                    dispatch(store, ChannelIntent::SetToText(to));
                }
            });
        });
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            for preset in DatePreset::ordered() {
                let selected = state.preset == preset;
                let text = egui::RichText::new(t(preset.locale_key())).small();
                let response = if selected {
                    ui.add(
                        egui::Button::new(text.color(crate::ui::theme::TEXT_ON_ACCENT))
                            .fill(ACCENT_PRIMARY)
                            .rounding(10.0),
                    )
                } else {
                    ui.add(egui::Button::new(text).rounding(10.0))
                };
                if response.clicked() {
                    dispatch(store, ChannelIntent::SetPreset(preset));
                }
            }
        });
    });
}

fn render_options(ui: &mut Ui, store: &mut Store, state: &ChannelState) {
    section_title(ui, t("channel_what_label"));
    ui.add_enabled_ui(state.status != ChannelStatus::Downloading, |ui| {
        ui.horizontal_wrapped(|ui| {
            let mut video = state.include_video;
            if ui
                .checkbox(&mut video, t("channel_include_video"))
                .changed()
            {
                dispatch(store, ChannelIntent::ToggleInclude(ChannelInclude::Video));
            }
            let mut transcript = state.include_transcript;
            if ui
                .checkbox(&mut transcript, t("channel_include_transcript"))
                .changed()
            {
                dispatch(
                    store,
                    ChannelIntent::ToggleInclude(ChannelInclude::Transcript),
                );
            }
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            micro_label(ui, t("video_quality_label"));
            let mut quality = state.quality;
            egui::ComboBox::from_id_source("channel_quality")
                .selected_text(t(quality.locale_key()))
                .show_ui(ui, |ui| {
                    for option in VideoQuality::ordered() {
                        ui.selectable_value(&mut quality, option, t(option.locale_key()));
                    }
                });
            if quality != state.quality {
                dispatch(store, ChannelIntent::SetQuality(quality));
            }
        });
    });
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        ui.label(
            egui::RichText::new(t("channel_kinds_label"))
                .small()
                .color(TEXT_SECONDARY),
        );
        ui.add_enabled_ui(!state.status.is_busy(), |ui| {
            for kind in VideoKind::ordered() {
                let active = kind_active(state, kind);
                if ui.selectable_label(active, t(kind.locale_key())).clicked() {
                    dispatch(store, ChannelIntent::ToggleKind(kind));
                }
            }
        });
    });
}

fn kind_active(state: &ChannelState, kind: VideoKind) -> bool {
    match kind {
        VideoKind::Video => state.show_video,
        VideoKind::Short => state.show_shorts,
        VideoKind::Live => state.show_lives,
    }
}

fn render_preview(ui: &mut Ui, store: &mut Store, state: &ChannelState) {
    if state.status == ChannelStatus::Loading {
        card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(t("channel_fetching"));
            });
        });
        ui.add_space(10.0);
        return;
    }
    let Some(preview) = &state.preview else {
        card(ui, |ui| {
            ui.label(
                egui::RichText::new(t("channel_empty")).color(TEXT_SECONDARY),
            );
        });
        ui.add_space(10.0);
        return;
    };
    render_channel_card(ui, preview);
    ui.add_space(10.0);
    render_table(ui, store, state, preview);
    ui.add_space(10.0);
    render_summary(ui, store, state, preview);
}

fn render_channel_card(ui: &mut Ui, preview: &ChannelPreview) {
    card(ui, |ui| {
        ui.horizontal(|ui| {
            let initial = preview.name.chars().next().unwrap_or('?');
            let (rect, _) =
                ui.allocate_exact_size(egui::Vec2::new(54.0, 54.0), egui::Sense::hover());
            ui.painter().circle_filled(
                rect.center(),
                27.0,
                egui::Color32::from_rgb(170, 34, 34),
            );
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                initial.to_string().to_uppercase(),
                egui::FontId::proportional(20.0),
                egui::Color32::WHITE,
            );
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(&preview.name).size(15.0).strong());
                let mut meta = Vec::new();
                if let Some(handle) = &preview.handle {
                    meta.push(handle.clone());
                }
                if let Some(subs) = &preview.subs {
                    meta.push(subs.clone());
                }
                if !meta.is_empty() {
                    ui.label(
                        egui::RichText::new(meta.join(" · "))
                            .small()
                            .color(TEXT_SECONDARY),
                    );
                }
            });
        });
    });
}

fn render_table(ui: &mut Ui, store: &mut Store, state: &ChannelState, preview: &ChannelPreview) {
    let locked = state.status.is_busy();
    card(ui, |ui| {
        section_title(ui, t("channel_preview_title"));
        ui.horizontal(|ui| {
            let all = !state.selected.is_empty() && state.selected.iter().all(|flag| *flag);
            let mut toggle = all;
            ui.add_enabled_ui(!locked, |ui| {
                if ui.checkbox(&mut toggle, t("channel_select_all")).changed() {
                    dispatch(
                        store,
                        if all {
                            ChannelIntent::SelectNone
                        } else {
                            ChannelIntent::SelectAll
                        },
                    );
                }
                if ui.button(t("channel_select_none")).clicked() {
                    dispatch(store, ChannelIntent::SelectNone);
                }
            });
        });
        egui::ScrollArea::vertical()
            .max_height(320.0)
            .show(ui, |ui| {
                for (index, video) in preview.videos.iter().enumerate() {
                    let checked = state.selected.get(index).copied().unwrap_or(false);
                    ui.horizontal(|ui| {
                        let mut flag = checked;
                        ui.add_enabled_ui(!locked, |ui| {
                            if ui.checkbox(&mut flag, "").changed() {
                                dispatch(store, ChannelIntent::ToggleSelect(index));
                            }
                        });
                        ui.label(
                            egui::RichText::new(format_duration(video.duration_secs))
                                .monospace()
                                .small(),
                        );
                        ui.vertical(|ui| {
                            ui.label(crate::ui::components::truncate_middle(&video.title, 70));
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} · {}",
                                    t(video.kind.locale_key()),
                                    video.date_label
                                ))
                                .small()
                                .weak(),
                            );
                        });
                    });
                    ui.separator();
                }
            });
    });
}

fn render_summary(ui: &mut Ui, store: &mut Store, state: &ChannelState, preview: &ChannelPreview) {
    let Some(videos) = state.preview.as_ref().map(|_| &preview.videos) else {
        return;
    };
    let picked = selected_count(&state.selected);
    let total = videos.len();
    let duration = total_duration_secs(videos, &state.selected);
    card(ui, |ui| {
        ui.label(
            egui::RichText::new(format!(
                "{picked} {} {total} {} · {}: {}",
                t("channel_summary_of"),
                t("channel_summary_selected"),
                t("channel_summary_duration"),
                format_duration(duration)
            ))
            .strong()
            .color(ACCENT_PRIMARY),
        );
        ui.add_space(6.0);
        ui.horizontal(|ui| match state.status {
            ChannelStatus::Downloading => {
                ui.add_enabled(false, egui::Button::new(t("channel_downloading")));
                if ui.button(t("channel_cancel")).clicked() {
                    dispatch(store, ChannelIntent::CancelBatch);
                }
            }
            ChannelStatus::Loading => {
                ui.add_enabled(false, egui::Button::new(t("channel_fetching")));
                if ui.button(t("channel_cancel")).clicked() {
                    dispatch(store, ChannelIntent::CancelBatch);
                }
            }
            ChannelStatus::Completed => {
                ui.label(t("channel_completed"));
                if ui.button(t("channel_retry")).clicked() {
                    dispatch(store, ChannelIntent::Retry);
                }
            }
            _ => {
                let ready = picked > 0 && (state.include_video || state.include_transcript);
                let label = format!("{} ({picked})", t("channel_download_button"));
                ui.add_enabled_ui(ready, |ui| {
                    if full_primary(ui, label).clicked() {
                        dispatch(store, ChannelIntent::DownloadBatch);
                    }
                });
            }
        });
    });
}

fn render_batch(ui: &mut Ui, state: &ChannelState) {
    if state.items.is_empty() {
        return;
    }
    if !matches!(
        state.status,
        ChannelStatus::Downloading | ChannelStatus::Completed
    ) {
        return;
    }
    ui.add_space(10.0);
    card(ui, |ui| {
        section_title(ui, t("channel_batch_title"));
        for item in &state.items {
            ui.horizontal(|ui| {
                ui.label(batch_marker(item.status));
                ui.vertical(|ui| {
                    ui.label(crate::ui::components::truncate_middle(&item.title, 70));
                    ui.add(
                        egui::ProgressBar::new((item.progress / 100.0).clamp(0.0, 1.0))
                            .show_percentage(),
                    );
                    if let Some(speed) = &item.speed {
                        ui.label(speed.clone());
                    }
                    if let Some(key) = &item.error_key {
                        ui.colored_label(LOG_ERROR, t(key));
                    }
                });
            });
            ui.separator();
        }
    });
}

fn batch_marker(status: BatchItemStatus) -> &'static str {
    match status {
        BatchItemStatus::Queued => "○",
        BatchItemStatus::Active => "●",
        BatchItemStatus::Done => "✓",
        BatchItemStatus::Failed => "✗",
    }
}

fn render_error(ui: &mut Ui, store: &mut Store, state: &ChannelState) {
    let Some(key) = &state.error_key else {
        return;
    };
    ui.add_space(10.0);
    card(ui, |ui| {
        ui.colored_label(LOG_ERROR, t(key));
        ui.horizontal(|ui| {
            if ui.button(t("channel_retry")).clicked() {
                dispatch(store, ChannelIntent::Retry);
            }
            if ui.button(t("channel_dismiss")).clicked() {
                dispatch(store, ChannelIntent::Dismiss);
            }
        });
    });
}
