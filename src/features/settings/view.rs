use egui::Ui;

use crate::core::intent::{AppIntent, SettingsIntent};
use crate::core::store::Store;
use crate::i18n::registry::t;

fn display_name(locale: &str) -> String {
    match locale {
        "en-US" => t("lang_en_US"),
        "pt-BR" => t("lang_pt_BR"),
        _ => locale.to_string(),
    }
}

pub fn render(ui: &mut Ui, store: &mut Store) {
    ui.heading(t("settings_title"));
    ui.label(t("settings_subtitle"));
    ui.separator();
    let current = store.state().locale.clone();
    let mut selected = current.clone();
    egui::ComboBox::from_label(t("settings_language_label"))
        .selected_text(display_name(selected.as_str()))
        .show_ui(ui, |ui| {
            for locale in store.available_locales() {
                ui.selectable_value(&mut selected, locale.clone(), display_name(locale.as_str()));
            }
        });
    if selected != current {
        store.dispatch(AppIntent::Settings(SettingsIntent::ChangeLocale(selected)));
    }
}
