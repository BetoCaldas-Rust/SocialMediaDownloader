use egui::{Button, RichText, Ui};

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
    let mut label = format!("{}  {}", screen_icon(screen), t(screen.nav_key()));
    if screen == Screen::History {
        let count = store.history_count();
        if count > 0 {
            label = format!("{label}  {count}");
        }
    }
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
    let response = ui.add_sized(
        [width, 34.0],
        Button::new(RichText::new(label).size(13.0).color(text_color))
            .fill(fill)
            .rounding(4.0),
    );
    if response.clicked() {
        store.dispatch(AppIntent::Navigate(screen));
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
