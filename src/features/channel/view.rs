use egui::Ui;

use crate::core::intent::{AppIntent, ChannelIntent};
use crate::core::state::{
    BatchItemStatus, ChannelInclude, ChannelState, ChannelStatus, DatePreset,
};
use crate::core::store::Store;
use crate::features::channel::model::{format_duration, selected_count, total_duration_secs};
use crate::i18n::registry::t;
use crate::services::traits::{ChannelPreview, VideoKind, VideoQuality};

pub fn render(ui: &mut Ui, store: &mut Store) {
    let state = store.state().channel.clone();
    ui.heading(t("channel_title"));
    ui.label(t("channel_subtitle"));
    ui.add_space(8.0);
    render_input_card(ui, store, &state);
    ui.add_space(8.0);
    render_period(ui, store, &state);
    ui.add_space(8.0);
    render_options(ui, store, &state);
    ui.add_space(8.0);
    render_preview(ui, store, &state);
    render_batch(ui, &state);
    render_error(ui, store, &state);
}

fn dispatch(store: &mut Store, intent: ChannelIntent) {
    store.dispatch(AppIntent::Channel(intent));
}

fn render_input_card(ui: &mut Ui, store: &mut Store, state: &ChannelState) {
    ui.group(|ui| {
        ui.label(t("channel_input_label"));
        ui.horizontal(|ui| {
            let mut input = state.input.clone();
            let field =
                ui.add(egui::TextEdit::singleline(&mut input).hint_text(t("channel_input_hint")));
            if field.changed() {
                dispatch(store, ChannelIntent::SetInput(input));
            }
            let loading = state.status == ChannelStatus::Loading;
            let label = if loading {
                t("channel_fetching")
            } else {
                t("channel_fetch_button")
            };
            let ready = !state.status.is_busy() && !state.input.trim().is_empty();
            if ui.add_enabled(ready, egui::Button::new(label)).clicked() {
                dispatch(store, ChannelIntent::FetchPreview);
            }
        });
    });
}

fn render_period(ui: &mut Ui, store: &mut Store, state: &ChannelState) {
    ui.group(|ui| {
        ui.label(t("channel_period_label"));
        ui.add_enabled_ui(!state.status.is_busy(), |ui| {
            ui.horizontal(|ui| {
                ui.label(t("channel_from_label"));
                let mut from = state.from_text.clone();
                let from_field = ui.add(
                    egui::TextEdit::singleline(&mut from)
                        .desired_width(90.0)
                        .hint_text(t("channel_date_hint")),
                );
                if from_field.changed() {
                    dispatch(store, ChannelIntent::SetFromText(from));
                }
                ui.label(t("channel_to_label"));
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
            ui.horizontal(|ui| {
                for preset in DatePreset::ordered() {
                    let selected = state.preset == preset;
                    if ui
                        .selectable_label(selected, t(preset.locale_key()))
                        .clicked()
                    {
                        dispatch(store, ChannelIntent::SetPreset(preset));
                    }
                }
            });
        });
    });
}

fn render_options(ui: &mut Ui, store: &mut Store, state: &ChannelState) {
    ui.group(|ui| {
        ui.label(t("channel_what_label"));
        ui.add_enabled_ui(state.status != ChannelStatus::Downloading, |ui| {
            ui.horizontal(|ui| {
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
                let mut quality = state.quality;
                egui::ComboBox::from_label(t("video_quality_label"))
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
        ui.horizontal(|ui| {
            ui.label(t("channel_kinds_label"));
            ui.add_enabled_ui(!state.status.is_busy(), |ui| {
                for kind in VideoKind::ordered() {
                    let active = kind_active(state, kind);
                    if ui.selectable_label(active, t(kind.locale_key())).clicked() {
                        dispatch(store, ChannelIntent::ToggleKind(kind));
                    }
                }
            });
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
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(t("channel_fetching"));
            });
        });
        ui.add_space(8.0);
        return;
    }
    let Some(preview) = &state.preview else {
        ui.group(|ui| {
            ui.label(t("channel_empty"));
        });
        ui.add_space(8.0);
        return;
    };
    render_channel_card(ui, preview);
    ui.add_space(8.0);
    render_table(ui, store, state, preview);
    ui.add_space(8.0);
    render_summary(ui, store, state, preview);
}

fn render_channel_card(ui: &mut Ui, preview: &ChannelPreview) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            let initial = preview.name.chars().next().unwrap_or('?');
            ui.label(egui::RichText::new(initial.to_string().to_uppercase()).heading());
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(&preview.name).strong());
                let mut meta = Vec::new();
                if let Some(handle) = &preview.handle {
                    meta.push(handle.clone());
                }
                if let Some(subs) = &preview.subs {
                    meta.push(subs.clone());
                }
                if !meta.is_empty() {
                    ui.label(meta.join(" · "));
                }
            });
        });
    });
}

fn render_table(ui: &mut Ui, store: &mut Store, state: &ChannelState, preview: &ChannelPreview) {
    let locked = state.status.is_busy();
    ui.group(|ui| {
        ui.label(t("channel_preview_title"));
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
                            ui.label(video.title.clone());
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
    ui.group(|ui| {
        ui.label(format!(
            "{picked} {} {total} {} · {}: {}",
            t("channel_summary_of"),
            t("channel_summary_selected"),
            t("channel_summary_duration"),
            format_duration(duration)
        ));
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
                if ui.add_enabled(ready, egui::Button::new(label)).clicked() {
                    dispatch(store, ChannelIntent::DownloadBatch);
                }
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
    ui.add_space(8.0);
    ui.group(|ui| {
        ui.label(t("channel_batch_title"));
        for item in &state.items {
            ui.horizontal(|ui| {
                ui.label(batch_marker(item.status));
                ui.vertical(|ui| {
                    ui.label(item.title.clone());
                    ui.add(
                        egui::ProgressBar::new((item.progress / 100.0).clamp(0.0, 1.0))
                            .show_percentage(),
                    );
                    if let Some(speed) = &item.speed {
                        ui.label(speed.clone());
                    }
                    if let Some(key) = &item.error_key {
                        ui.colored_label(egui::Color32::LIGHT_RED, t(key));
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
    ui.add_space(8.0);
    ui.group(|ui| {
        ui.colored_label(egui::Color32::LIGHT_RED, t(key));
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
