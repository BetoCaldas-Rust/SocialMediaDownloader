use egui::{Button, Frame, InnerResponse, Margin, Response, RichText, Ui};

use super::theme::{ACCENT_PRIMARY, BG_DARK, BG_SECONDARY, TEXT_ON_ACCENT, TEXT_SECONDARY};
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
    ui.label(RichText::new(t(title_key)).size(22.0).strong());
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
    ui.label(RichText::new(text).size(14.0));
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

pub fn input_black(ui: &mut Ui, text: &mut String, hint: String) -> egui::Response {
    ui.style_mut().visuals.extreme_bg_color = BG_DARK;
    ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(RichText::new(hint).color(TEXT_SECONDARY))
            .desired_width(f32::INFINITY),
    )
}
