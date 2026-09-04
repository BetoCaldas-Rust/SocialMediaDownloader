use egui::{RichText, Ui};

use crate::core::intent::AppIntent;
use crate::core::state::Screen;
use crate::core::store::Store;
use crate::i18n::registry::t;
use crate::ui::theme::{accent_dim, ACCENT_PRIMARY, TEXT_DISABLED, TEXT_SECONDARY};

fn screen_icon(screen: Screen) -> &'static str {
    match screen {
        Screen::Video => "🎬",
        Screen::Transcript => "📄",
        Screen::Channel => "📺",
        Screen::History => "🕓",
        Screen::Console => "▣",
        Screen::Settings => "⚙",
    }
}

pub fn render(ui: &mut Ui, store: &mut Store) {
    let current = store.state().screen;
    ui.add_space(4.0);
    ui.label(
        RichText::new("📥 SMD")
            .size(15.0)
            .strong()
            .color(ACCENT_PRIMARY),
    );
    ui.add_space(12.0);
    for screen in Screen::ordered() {
        render_nav_item(ui, store, screen, current == screen);
        ui.add_space(2.0);
    }
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        ui.add_space(8.0);
        render_footer(ui);
    });
}

fn render_nav_item(ui: &mut Ui, store: &mut Store, screen: Screen, selected: bool) {
    let badge = if screen == Screen::History {
        let count = store.history_count();
        if count > 0 {
            Some(count.to_string())
        } else {
            None
        }
    } else {
        None
    };
    let text_color = if selected {
        ACCENT_PRIMARY
    } else {
        TEXT_SECONDARY
    };
    let fill = if selected {
        accent_dim()
    } else {
        egui::Color32::TRANSPARENT
    };
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(
        egui::Vec2::new(width, 34.0),
        egui::Sense::click(),
    );
    if response.clicked() {
        store.dispatch(AppIntent::Navigate(screen));
    }
    if response.hovered() && !selected {
        ui.painter().rect_filled(
            rect,
            4.0,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 10),
        );
    } else {
        ui.painter().rect_filled(rect, 4.0, fill);
    }
    let font = egui::FontId::proportional(13.0);
    ui.painter().text(
        egui::Pos2::new(rect.min.x + 10.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        format!("{}  {}", screen_icon(screen), t(screen.nav_key())),
        font,
        text_color,
    );
    if let Some(count) = badge {
        let badge_font = egui::FontId::proportional(10.0);
        let galley = ui.fonts(|fonts| {
            fonts.layout_no_wrap(count.clone(), badge_font.clone(), TEXT_SECONDARY)
        });
        let pill = egui::Rect::from_min_size(
            egui::Pos2::new(
                rect.max.x - 10.0 - galley.size().x - 12.0,
                rect.center().y - 9.0,
            ),
            egui::Vec2::new(galley.size().x + 12.0, 18.0),
        );
        ui.painter().rect_filled(
            pill,
            9.0,
            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 20),
        );
        ui.painter().text(
            pill.center(),
            egui::Align2::CENTER_CENTER,
            count,
            badge_font,
            TEXT_SECONDARY,
        );
    }
}

fn render_footer(ui: &mut Ui) {
    let ready = crate::services::yt_dlp::binary::resolve_binary().is_ok();
    let (dot, key) = if ready {
        ("●", "sidebar_online")
    } else {
        ("●", "sidebar_sidecar_missing")
    };
    let color = if ready {
        ACCENT_PRIMARY
    } else {
        TEXT_DISABLED
    };
    ui.horizontal(|ui| {
        ui.label(RichText::new(dot).color(color).small());
        ui.label(RichText::new(t(key)).color(TEXT_SECONDARY).small());
    });
}
