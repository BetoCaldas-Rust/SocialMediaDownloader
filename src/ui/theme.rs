use egui::{Color32, Rounding, Stroke, Visuals};

// Cores do tema preto e amarelo
pub const BG_PRIMARY: Color32 = Color32::from_rgb(26, 26, 26); // #1a1a1a
pub const BG_SECONDARY: Color32 = Color32::from_rgb(45, 45, 45); // #2d2d2d
pub const BG_DARK: Color32 = Color32::from_rgb(0, 0, 0); // #000000

pub const ACCENT_PRIMARY: Color32 = Color32::from_rgb(255, 215, 0); // #ffd700 - Gold
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(255, 237, 78); // #ffed4e
pub const ACCENT_DARK: Color32 = Color32::from_rgb(255, 183, 0); // #ffb700

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(255, 255, 255);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(200, 200, 200);
pub const TEXT_DISABLED: Color32 = Color32::from_rgb(120, 120, 120);

pub const ERROR_COLOR: Color32 = Color32::from_rgb(255, 69, 58);
pub const SUCCESS_COLOR: Color32 = Color32::from_rgb(52, 199, 89);

pub fn configure_fonts(ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();

    // Adiciona fontes customizadas se necessário
    // fonts.font_data.insert(...)

    ctx.set_fonts(fonts);
}

pub fn apply_custom_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    // Cores de fundo
    visuals.panel_fill = BG_PRIMARY;
    visuals.window_fill = BG_PRIMARY;
    visuals.faint_bg_color = BG_SECONDARY;
    visuals.extreme_bg_color = BG_DARK;

    // Widgets
    visuals.widgets.noninteractive.bg_fill = BG_SECONDARY;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);

    visuals.widgets.inactive.bg_fill = BG_SECONDARY;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_SECONDARY);
    visuals.widgets.inactive.weak_bg_fill = BG_SECONDARY;

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 60, 60);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, ACCENT_LIGHT);
    visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(50, 50, 50);

    visuals.widgets.active.bg_fill = ACCENT_DARK;
    visuals.widgets.active.fg_stroke = Stroke::new(2.0, BG_DARK);
    visuals.widgets.active.weak_bg_fill = ACCENT_DARK;

    // Seleção
    visuals.selection.bg_fill = ACCENT_PRIMARY.linear_multiply(0.3);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT_PRIMARY);

    // Hyperlinks
    visuals.hyperlink_color = ACCENT_PRIMARY;

    // Arredondamentos
    visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
    visuals.widgets.inactive.rounding = Rounding::same(4.0);
    visuals.widgets.hovered.rounding = Rounding::same(4.0);
    visuals.widgets.active.rounding = Rounding::same(4.0);

    ctx.set_visuals(visuals);
}
