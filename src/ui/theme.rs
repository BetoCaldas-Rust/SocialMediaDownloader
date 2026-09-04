use egui::{Color32, Rounding, Stroke, Visuals};

pub const BG_PRIMARY: Color32 = Color32::from_rgb(26, 26, 26);
pub const BG_SECONDARY: Color32 = Color32::from_rgb(45, 45, 45);
pub const BG_DARK: Color32 = Color32::from_rgb(0, 0, 0);

pub const ACCENT_PRIMARY: Color32 = Color32::from_rgb(255, 215, 0);
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(255, 237, 78);
pub const ACCENT_DARK: Color32 = Color32::from_rgb(255, 183, 0);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(200, 200, 200);
#[allow(dead_code)]
pub const TEXT_DISABLED: Color32 = Color32::from_rgb(120, 120, 120);

#[allow(dead_code)]
pub const ERROR_COLOR: Color32 = Color32::from_rgb(255, 69, 58);
#[allow(dead_code)]
pub const SUCCESS_COLOR: Color32 = Color32::from_rgb(52, 199, 89);

pub const LOG_ERROR: Color32 = Color32::from_rgb(255, 107, 107);
pub const LOG_WARN: Color32 = Color32::from_rgb(255, 184, 0);
pub const LOG_INFO: Color32 = Color32::from_rgb(74, 222, 128);
pub const LOG_DEBUG: Color32 = Color32::from_rgb(142, 142, 147);
pub const LOG_SOURCE: Color32 = Color32::from_rgb(122, 183, 255);
#[allow(dead_code)]
pub const LOG_PATH: Color32 = Color32::from_rgb(196, 165, 255);

pub fn configure_fonts(ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();
    ctx.set_fonts(fonts);
}

pub fn apply_custom_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.panel_fill = BG_PRIMARY;
    visuals.window_fill = BG_PRIMARY;
    visuals.faint_bg_color = BG_SECONDARY;
    visuals.extreme_bg_color = BG_DARK;

    visuals.widgets.noninteractive.bg_fill = BG_SECONDARY;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, TEXT_PRIMARY);

    visuals.widgets.inactive.bg_fill = BG_SECONDARY;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, TEXT_SECONDARY);
    visuals.widgets.inactive.weak_bg_fill = BG_SECONDARY;

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 60);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5_f32, ACCENT_LIGHT);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(50, 50, 50);

    visuals.widgets.active.bg_fill = ACCENT_DARK;
    visuals.widgets.active.fg_stroke = Stroke::new(2.0_f32, BG_DARK);
    visuals.widgets.active.weak_bg_fill = ACCENT_DARK;

    visuals.selection.bg_fill = ACCENT_PRIMARY.linear_multiply(0.3);
    visuals.selection.stroke = Stroke::new(1.0_f32, ACCENT_PRIMARY);

    visuals.hyperlink_color = ACCENT_PRIMARY;

    visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
    visuals.widgets.inactive.rounding = Rounding::same(4.0);
    visuals.widgets.hovered.rounding = Rounding::same(4.0);
    visuals.widgets.active.rounding = Rounding::same(4.0);

    ctx.set_visuals(visuals);
}
