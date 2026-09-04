mod core;
mod features;
mod hotkey;
mod i18n;
mod services;
mod storage;
mod ui;

use ui::app::SmdApp;

fn main() -> Result<(), eframe::Error> {
    let config = storage::config::AppConfig::load();
    let level_handle = services::log_buffer::init_tracing(config.log_level.tracing_filter());
    crate::core::store::prune_startup_logs();
    let locales_dir = i18n::loader::resolve_locales_dir();
    i18n::registry::init_i18n(locales_dir);
    i18n::registry::set_locale(&config.locale);
    let title = i18n::registry::t("app_title");
    tracing::info!(target: "startup", "starting {title}");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 780.0])
            .with_min_inner_size([1000.0, 650.0])
            .with_title("Social Media Downloader"),
        ..Default::default()
    };
    eframe::run_native(
        "Social Media Downloader",
        options,
        Box::new(move |creation| Ok(Box::new(SmdApp::new(creation, config, Some(level_handle))))),
    )
}
