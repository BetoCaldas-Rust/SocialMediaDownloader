use egui::Ui;

use crate::core::intent::AppIntent;
use crate::core::state::Screen;
use crate::core::store::Store;
use crate::i18n::registry::t;

pub fn render(ui: &mut Ui, store: &mut Store) {
    let current = store.state().screen;
    ui.heading(t("app_title"));
    ui.separator();
    for screen in Screen::ordered() {
        render_nav_item(ui, store, screen, current == screen);
    }
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        ui.separator();
        ui.label(t("sidebar_footer"));
    });
}

fn render_nav_item(ui: &mut Ui, store: &mut Store, screen: Screen, selected: bool) {
    if ui.selectable_label(selected, t(screen.nav_key())).clicked() {
        store.dispatch(AppIntent::Navigate(screen));
    }
}
