use egui::Ui;

use crate::core::store::Store;
use crate::i18n::registry::t;

pub fn render(ui: &mut Ui, _store: &mut Store) {
    ui.heading(t("history_title"));
    ui.label(t("history_subtitle"));
}
