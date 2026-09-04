use egui::Ui;

use crate::core::store::Store;
use crate::i18n::registry::t;

pub fn render(ui: &mut Ui, _store: &mut Store) {
    ui.heading(t("transcript_title"));
    ui.label(t("transcript_subtitle"));
}
