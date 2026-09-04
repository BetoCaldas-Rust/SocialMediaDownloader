use egui::Ui;

use crate::core::intent::{AppIntent, VideoIntent};
use crate::core::state::{VideoState, VideoStatus};
use crate::core::store::Store;
use crate::i18n::registry::t;
use crate::services::traits::{Container, VideoQuality};

pub fn render(ui: &mut Ui, store: &mut Store) {
    let state = store.state().video.clone();
    ui.heading(t("video_title"));
    ui.label(t("video_subtitle"));
    ui.add_space(8.0);
    render_url_card(ui, store, &state);
    ui.add_space(8.0);
    render_options(ui, store, &state);
    ui.add_space(8.0);
    render_preview(ui, &state);
    ui.add_space(8.0);
    render_actions(ui, store, &state);
    render_progress(ui, store, &state);
    render_error(ui, store, &state);
}

fn dispatch(store: &mut Store, intent: VideoIntent) {
    store.dispatch(AppIntent::Video(intent));
}

fn render_url_card(ui: &mut Ui, store: &mut Store, state: &VideoState) {
    ui.group(|ui| {
        ui.label(t("video_url_label"));
        ui.horizontal(|ui| {
            let mut url = state.url.clone();
            let field = ui.add(egui::TextEdit::singleline(&mut url).hint_text(t("video_url_hint")));
            if field.changed() {
                dispatch(store, VideoIntent::SetUrl(url));
            }
            let resolving = state.status == VideoStatus::Resolving;
            let label = if resolving {
                t("video_fetching")
            } else {
                t("video_fetch_button")
            };
            let ready = !resolving && !state.url.trim().is_empty();
            if ui.add_enabled(ready, egui::Button::new(label)).clicked() {
                dispatch(store, VideoIntent::FetchMetadata);
            }
        });
    });
}

fn render_options(ui: &mut Ui, store: &mut Store, state: &VideoState) {
    ui.horizontal(|ui| {
        let mut quality = state.quality;
        egui::ComboBox::from_label(t("video_quality_label"))
            .selected_text(t(quality.locale_key()))
            .show_ui(ui, |ui| {
                for option in VideoQuality::ordered() {
                    ui.selectable_value(&mut quality, option, t(option.locale_key()));
                }
            });
        if quality != state.quality {
            dispatch(store, VideoIntent::SetQuality(quality));
        }
        let mut container = state.container;
        let container_label = t(container.locale_key());
        egui::ComboBox::from_label(t("video_format_label"))
            .selected_text(container_label)
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut container,
                    Container::Mp4,
                    t(Container::Mp4.locale_key()),
                );
            });
        if container != state.container {
            dispatch(store, VideoIntent::SetContainer(container));
        }
    });
    ui.horizontal(|ui| {
        let mut transcript = state.include_transcript;
        if ui
            .checkbox(&mut transcript, t("video_transcript_check"))
            .changed()
        {
            dispatch(store, VideoIntent::SetTranscript(transcript));
        }
        let mut audio = false;
        ui.add_enabled(
            false,
            egui::Checkbox::new(&mut audio, t("video_audio_check")),
        )
        .on_disabled_hover_text(t("video_audio_tooltip"));
    });
}

fn render_preview(ui: &mut Ui, state: &VideoState) {
    ui.group(|ui| {
        ui.label(t("video_preview_title"));
        match (&state.status, &state.metadata) {
            (VideoStatus::Resolving, _) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(t("video_fetching"));
                });
            }
            (_, Some(meta)) => {
                ui.label(egui::RichText::new(&meta.title).strong());
                if !meta.channel.is_empty() {
                    ui.label(detail_line("video_preview_channel", &meta.channel));
                }
                ui.label(detail_line(
                    "video_preview_duration",
                    &format_duration(meta.duration_secs),
                ));
                if let Some(views) = meta.view_count {
                    ui.label(detail_line("video_preview_views", &views.to_string()));
                }
                if let Some(size) = &meta.size_label {
                    ui.label(detail_line("video_preview_size", size));
                }
            }
            _ => {
                ui.label(t("video_preview_empty"));
            }
        }
    });
}

fn render_actions(ui: &mut Ui, store: &mut Store, state: &VideoState) {
    ui.horizontal(|ui| {
        let has_url = !state.url.trim().is_empty();
        match state.status {
            VideoStatus::Downloading => {
                let label = format!("{} {:.0}%", t("video_downloading"), state.progress);
                ui.add_enabled(false, egui::Button::new(label));
                if ui.button(t("video_cancel")).clicked() {
                    dispatch(store, VideoIntent::CancelDownload);
                }
            }
            VideoStatus::Resolving => {
                ui.add_enabled(false, egui::Button::new(t("video_fetching")));
            }
            VideoStatus::Failed => {
                if ui.button(t("video_retry")).clicked() {
                    dispatch(store, VideoIntent::StartDownload);
                }
            }
            _ => {
                if ui
                    .add_enabled(has_url, egui::Button::new(t("video_download_button")))
                    .clicked()
                {
                    dispatch(store, VideoIntent::StartDownload);
                }
            }
        }
    });
}

fn render_progress(ui: &mut Ui, store: &mut Store, state: &VideoState) {
    let active = state.status == VideoStatus::Downloading;
    let done = state.status == VideoStatus::Completed && state.output_path.is_some();
    if !active && !done {
        return;
    }
    ui.group(|ui| {
        if active {
            ui.label(t("video_downloading"));
        } else {
            ui.label(t("video_completed"));
        }
        ui.add(egui::ProgressBar::new((state.progress / 100.0).clamp(0.0, 1.0)).show_percentage());
        if let Some(summary) = transfer_summary(state) {
            ui.label(summary);
        }
        if let Some(name) = &state.filename {
            ui.label(name);
        }
        if state.output_path.is_some() && ui.button(t("video_open_file")).clicked() {
            dispatch(store, VideoIntent::RevealOutput);
        }
    });
}

fn render_error(ui: &mut Ui, store: &mut Store, state: &VideoState) {
    if state.status != VideoStatus::Failed {
        return;
    }
    let message = state.error_key.as_deref().map(t).unwrap_or_default();
    ui.group(|ui| {
        ui.colored_label(egui::Color32::LIGHT_RED, message);
        ui.horizontal(|ui| {
            if ui.button(t("video_retry")).clicked() {
                dispatch(store, VideoIntent::StartDownload);
            }
            if ui.button(t("video_dismiss")).clicked() {
                dispatch(store, VideoIntent::DismissError);
            }
        });
    });
}

fn detail_line(key: &str, value: &str) -> String {
    format!("{}: {value}", t(key))
}

fn transfer_summary(state: &VideoState) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(speed) = &state.speed {
        parts.push(speed.clone());
    }
    if let Some(eta) = &state.eta {
        parts.push(format!("ETA {eta}"));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

fn format_duration(total_secs: u64) -> String {
    let hours = total_secs / 3600;
    let minutes = total_secs % 3600 / 60;
    let secs = total_secs % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{secs:02}")
    } else {
        format!("{minutes:02}:{secs:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_durations() {
        assert_eq!(format_duration(0), "00:00");
        assert_eq!(format_duration(65), "01:05");
        assert_eq!(format_duration(3725), "1:02:05");
    }
}
