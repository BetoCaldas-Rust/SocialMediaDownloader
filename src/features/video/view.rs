use egui::Ui;

use crate::core::intent::{AppIntent, VideoIntent};
use crate::core::state::{VideoState, VideoStatus};
use crate::core::store::Store;
use crate::i18n::registry::t;
use crate::services::traits::{Container, VideoQuality};
use crate::ui::components::{
    card, check_row, field_label, full_primary, input_black, micro_label, page_header,
    section_title,
};
use crate::ui::theme::TEXT_SECONDARY;

pub fn render(ui: &mut Ui, store: &mut Store) {
    let state = store.state().video.clone();
    page_header(ui, "video_title", "video_subtitle");
    render_url_card(ui, store, &state);
    ui.add_space(10.0);
    card(ui, |ui| {
        render_options(ui, store, &state);
    });
    ui.add_space(10.0);
    card(ui, |ui| {
        render_preview(ui, &state);
    });
    ui.add_space(10.0);
    render_actions(ui, store, &state);
    render_progress(ui, store, &state);
    render_error(ui, store, &state);
}

fn dispatch(store: &mut Store, intent: VideoIntent) {
    store.dispatch(AppIntent::Video(intent));
}

fn render_url_card(ui: &mut Ui, store: &mut Store, state: &VideoState) {
    card(ui, |ui| {
        field_label(ui, t("video_url_label"));
        ui.horizontal(|ui| {
            let mut url = state.url.clone();
            let field = input_black(ui, &mut url, t("video_url_hint"));
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
        ui.vertical(|ui| {
            micro_label(ui, t("video_quality_label"));
            let mut quality = state.quality;
            egui::ComboBox::from_id_source("video_quality")
                .selected_text(t(quality.locale_key()))
                .show_ui(ui, |ui| {
                    for option in VideoQuality::ordered() {
                        ui.selectable_value(&mut quality, option, t(option.locale_key()));
                    }
                });
            if quality != state.quality {
                dispatch(store, VideoIntent::SetQuality(quality));
            }
        });
        ui.add_space(12.0);
        ui.vertical(|ui| {
            micro_label(ui, t("video_format_label"));
            let mut container = state.container;
            egui::ComboBox::from_id_source("video_format")
                .selected_text(t(container.locale_key()))
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
    });
    ui.add_space(8.0);
    let transcript = state.include_transcript;
    check_row(
        ui,
        transcript,
        true,
        t("video_transcript_check"),
        t("video_transcript_desc"),
        || dispatch(store, VideoIntent::SetTranscript(!transcript)),
    );
    ui.add_space(6.0);
    check_row(
        ui,
        false,
        false,
        t("video_audio_check"),
        t("video_audio_desc"),
        || {},
    );
}

fn render_preview(ui: &mut Ui, state: &VideoState) {
    section_title(ui, t("video_preview_title"));
    match (&state.status, &state.metadata) {
        (VideoStatus::Resolving, _) => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(t("video_fetching"));
            });
        }
        (_, Some(meta)) => {
            ui.horizontal(|ui| {
                render_thumb(ui, &meta.title, &format_duration(meta.duration_secs));
                ui.vertical(|ui| {
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
                });
            });
        }
        _ => {
            ui.label(
                egui::RichText::new(t("video_preview_empty")).color(TEXT_SECONDARY),
            );
        }
    }
}

fn render_thumb(ui: &mut Ui, title: &str, duration: &str) {
    let letter = title.chars().next().unwrap_or('▶').to_string();
    let (rect, _) = ui.allocate_exact_size(egui::Vec2::new(140.0, 80.0), egui::Sense::hover());
    ui.painter().rect_filled(
        rect,
        3.0,
        egui::Color32::from_rgb(42, 42, 42),
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        letter,
        egui::FontId::proportional(28.0),
        TEXT_SECONDARY,
    );
    let badge_font = egui::FontId::proportional(10.0);
    let galley = ui.fonts(|fonts| {
        fonts.layout_no_wrap(
            duration.to_string(),
            badge_font.clone(),
            egui::Color32::WHITE,
        )
    });
    let badge = egui::Rect::from_min_size(
        egui::Pos2::new(
            rect.max.x - 4.0 - galley.size().x - 8.0,
            rect.max.y - 4.0 - 16.0,
        ),
        egui::Vec2::new(galley.size().x + 8.0, 16.0),
    );
    ui.painter()
        .rect_filled(badge, 2.0, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 217));
    ui.painter().text(
        badge.center(),
        egui::Align2::CENTER_CENTER,
        duration,
        badge_font,
        egui::Color32::WHITE,
    );
}

fn render_actions(ui: &mut Ui, store: &mut Store, state: &VideoState) {
    let has_url = !state.url.trim().is_empty();
    match state.status {
        VideoStatus::Downloading => {
            ui.horizontal(|ui| {
                let label = format!("{} {:.0}%", t("video_downloading"), state.progress);
                ui.add_enabled(false, egui::Button::new(label));
                if ui.button(t("video_cancel")).clicked() {
                    dispatch(store, VideoIntent::CancelDownload);
                }
            });
        }
        VideoStatus::Resolving => {
            ui.add_enabled_ui(false, |ui| {
                full_primary(ui, t("video_fetching"));
            });
        }
        VideoStatus::Failed => {
            if full_primary(ui, t("video_retry")).clicked() {
                dispatch(store, VideoIntent::StartDownload);
            }
        }
        _ => {
            ui.add_enabled_ui(has_url, |ui| {
                if full_primary(ui, t("video_download_button")).clicked() {
                    dispatch(store, VideoIntent::StartDownload);
                }
            });
        }
    }
    ui.add_space(4.0);
}

fn render_progress(ui: &mut Ui, store: &mut Store, state: &VideoState) {
    let active = state.status == VideoStatus::Downloading;
    let done = state.status == VideoStatus::Completed && state.output_path.is_some();
    if !active && !done {
        return;
    }
    card(ui, |ui| {
        if active {
            section_title(ui, t("video_downloading"));
        } else {
            section_title(ui, t("video_completed"));
        }
        ui.horizontal(|ui| {
            crate::ui::components::video_chip(ui, t("video_chip_video"));
            ui.label(state.filename.as_deref().unwrap_or("…"));
        });
        ui.add(
            egui::ProgressBar::new((state.progress / 100.0).clamp(0.0, 1.0))
                .show_percentage()
                .fill(crate::ui::theme::ACCENT_PRIMARY),
        );
        if let Some(summary) = transfer_summary(state) {
            ui.label(
                egui::RichText::new(summary)
                    .small()
                    .color(crate::ui::theme::TEXT_SECONDARY),
            );
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
    card(ui, |ui| {
        ui.colored_label(crate::ui::theme::LOG_ERROR, message);
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
