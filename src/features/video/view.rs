use egui::Ui;

use crate::core::store::Store;
use crate::i18n::registry::t;

pub fn render(ui: &mut Ui, _store: &mut Store) {
    ui.heading(t("video_title"));
    ui.label(t("video_subtitle"));
}
