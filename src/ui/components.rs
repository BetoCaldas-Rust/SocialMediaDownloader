use egui::{Button, Frame, InnerResponse, Margin, Response, RichText, Ui};

use super::theme::{
    ACCENT_PRIMARY, BG_DARK, BG_SECONDARY, TEXT_ON_ACCENT, TEXT_PRIMARY, TEXT_SECONDARY,
};
use crate::i18n::registry::t;

pub fn body_margin() -> Margin {
    Margin {
        left: 22.0,
        right: 22.0,
        top: 18.0,
        bottom: 18.0,
    }
}

pub fn page_header(ui: &mut Ui, title_key: &str, subtitle_key: &str) {
    ui.label(
        RichText::new(t(title_key))
            .size(22.0)
            .strong()
            .color(TEXT_PRIMARY),
    );
    ui.label(RichText::new(t(subtitle_key)).size(13.0).color(TEXT_SECONDARY));
    ui.add_space(14.0);
}

pub fn section_title(ui: &mut Ui, text: String) {
    ui.label(
        RichText::new(text)
            .size(17.0)
            .strong()
            .color(ACCENT_PRIMARY),
    );
    ui.add_space(6.0);
}

pub fn card<R>(
    ui: &mut Ui,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    Frame::none()
        .fill(BG_SECONDARY)
        .rounding(4.0)
        .inner_margin(Margin::symmetric(14.0_f32, 12.0_f32))
        .stroke(egui::Stroke::new(
            1.0_f32,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8),
        ))
        .show(ui, add_contents)
}

pub fn field_label(ui: &mut Ui, text: String) {
    ui.label(RichText::new(text).size(14.0).color(TEXT_PRIMARY));
    ui.add_space(4.0);
}

pub fn micro_label(ui: &mut Ui, text: String) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(11.0)
            .color(TEXT_SECONDARY),
    );
    ui.add_space(4.0);
}

pub fn full_primary(ui: &mut Ui, text: String) -> Response {
    let width = ui.available_width();
    ui.add_sized(
        [width, 44.0],
        Button::new(
            RichText::new(text)
                .size(16.0)
                .strong()
                .color(TEXT_ON_ACCENT),
        )
        .fill(ACCENT_PRIMARY),
    )
}

pub fn input_row(
    ui: &mut Ui,
    text: &mut String,
    hint: String,
    action_label: String,
    action_enabled: bool,
) -> (bool, bool) {
    let mut changed = false;
    let mut clicked = false;
    ui.horizontal(|ui| {
        let input_width = (ui.available_width() - 140.0).max(80.0);
        ui.style_mut().visuals.extreme_bg_color = BG_DARK;
        let field = ui.add(
            egui::TextEdit::singleline(text)
                .hint_text(RichText::new(hint).color(TEXT_SECONDARY))
                .desired_width(input_width),
        );
        if field.changed() {
            changed = true;
        }
        ui.add_enabled_ui(action_enabled, |ui| {
            if ui
                .add_sized([132.0, 28.0], egui::Button::new(action_label))
                .clicked()
            {
                clicked = true;
            }
        });
    });
    (changed, clicked)
}

pub fn search_box(ui: &mut Ui, text: &mut String, hint: String, width: f32) -> bool {
    ui.style_mut().visuals.extreme_bg_color = BG_DARK;
    let field = ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(RichText::new(hint).color(TEXT_SECONDARY))
            .desired_width(width),
    );
    field.changed()
}

pub fn truncate_middle(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars || max_chars < 5 {
        return text.to_string();
    }
    let tail = max_chars / 3;
    let head = max_chars - tail - 1;
    let start: String = text.chars().take(head).collect();
    let end: String = text
        .chars()
        .rev()
        .take(tail)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{start}…{end}")
}

pub fn chip(ui: &mut Ui, text: String, bg: egui::Color32, fg: egui::Color32) {
    egui::Frame::none()
        .fill(bg)
        .rounding(2.0)
        .inner_margin(Margin::symmetric(6.0_f32, 2.0_f32))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .size(10.0)
                    .strong()
                    .color(fg),
            );
        });
}

pub fn video_chip(ui: &mut Ui, text: String) {
    chip(
        ui,
        text,
        egui::Color32::from_rgba_unmultiplied(255, 215, 0, 38),
        ACCENT_PRIMARY,
    );
}

pub fn transcript_chip(ui: &mut Ui, text: String) {
    chip(
        ui,
        text,
        egui::Color32::from_rgba_unmultiplied(52, 199, 89, 38),
        egui::Color32::from_rgb(52, 199, 89),
    );
}

pub fn check_row(
    ui: &mut Ui,
    selected: bool,
    enabled: bool,
    title: String,
    desc: String,
    on_toggle: impl FnOnce(),
) {
    let stroke = if selected {
        egui::Stroke::new(1.0_f32, ACCENT_PRIMARY)
    } else {
        egui::Stroke::new(1.0_f32, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10))
    };
    let fill = if selected {
        egui::Color32::from_rgba_unmultiplied(255, 215, 0, 15)
    } else {
        BG_SECONDARY
    };
    egui::Frame::none()
        .fill(fill)
        .rounding(4.0)
        .stroke(stroke)
        .inner_margin(Margin::symmetric(10.0_f32, 8.0_f32))
        .show(ui, |ui| {
            ui.add_enabled_ui(enabled, |ui| {
                let size = egui::Vec2::new(ui.available_width(), 44.0);
                let (rect, response) =
                    ui.allocate_exact_size(size, egui::Sense::click());
                if response.clicked() {
                    on_toggle();
                }
                let box_top = rect.min.y + 6.0;
                let box_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(rect.min.x, box_top),
                    egui::Vec2::new(16.0, 16.0),
                );
                let box_fill = if selected {
                    ACCENT_PRIMARY
                } else {
                    BG_DARK
                };
                ui.painter().rect_filled(box_rect, 3.0, box_fill);
                if selected {
                    ui.painter().text(
                        box_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "✓",
                        egui::FontId::proportional(11.0),
                        TEXT_ON_ACCENT,
                    );
                }
                let text_x = box_rect.max.x + 8.0;
                ui.painter().text(
                    egui::Pos2::new(text_x, rect.min.y + 4.0),
                    egui::Align2::LEFT_TOP,
                    title,
                    egui::FontId::proportional(13.0),
                    super::theme::TEXT_PRIMARY,
                );
                ui.painter().text(
                    egui::Pos2::new(text_x, rect.min.y + 22.0),
                    egui::Align2::LEFT_TOP,
                    desc,
                    egui::FontId::proportional(11.0),
                    TEXT_SECONDARY,
                );
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_passes_through() {
        assert_eq!(truncate_middle("abc", 10), "abc");
        assert_eq!(truncate_middle("abcde", 5), "abcde");
    }

    #[test]
    fn long_text_keeps_head_and_tail() {
        let text = "Massimo - Did you know about zinc fireworks [12345].mp4";
        let out = truncate_middle(text, 30);
        assert_eq!(out.chars().count(), 30);
        assert!(out.starts_with("Massimo - Did you k"));
        assert!(out.ends_with("12345].mp4"));
        assert!(out.contains('…'));
    }
}
