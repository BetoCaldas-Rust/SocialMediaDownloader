use std::sync::Arc;

use egui::Context;

use crate::core::intent::AppIntent;
use crate::core::state::Screen;
use crate::core::store::Store;
use crate::features::shared::sidebar;
use crate::features::{channel, console, history, settings, transcript, video};
use crate::i18n::registry::t;
use crate::services::json_history::JsonHistoryStore;
use crate::services::locale::LocaleService;
use crate::services::log_buffer::LevelHandle;
use crate::services::log_sink::BufferLogSink;
use crate::services::yt_dlp::{
    YtDlpChannelProvider, YtDlpDownloader, YtDlpMetadataProvider, YtDlpTranscriber,
};
use crate::storage::config::AppConfig;
use crate::ui::theme::{apply_custom_theme, configure_fonts};

pub struct SmdApp {
    store: Store,
}

impl SmdApp {
    pub fn new(
        creation: &eframe::CreationContext<'_>,
        config: AppConfig,
        level_handle: Option<LevelHandle>,
    ) -> Self {
        apply_custom_theme(&creation.egui_ctx);
        configure_fonts(&creation.egui_ctx);
        let log_sink = Arc::new(BufferLogSink);
        let store = Store::new(
            Arc::new(LocaleService),
            log_sink.clone(),
            Arc::new(YtDlpMetadataProvider::new(log_sink.clone())),
            Arc::new(YtDlpDownloader::new(log_sink.clone())),
            Arc::new(YtDlpTranscriber::new(log_sink.clone())),
            Arc::new(YtDlpChannelProvider::new(log_sink)),
            Arc::new(JsonHistoryStore::load()),
            config,
            level_handle,
        );
        tracing::info!(target: "startup", "ui ready");
        Self { store }
    }

    fn render_notices(&mut self, ui: &mut egui::Ui) {
        let notices = self.store.state().notices.clone();
        for notice in &notices {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("{}: {}", t(&notice.message_key), notice.detail));
                    if ui.button(t("notice_dismiss")).clicked() {
                        self.store
                            .dispatch(AppIntent::DismissNotice(notice.id.clone()));
                    }
                });
            });
        }
    }

    fn render_screen(&mut self, ui: &mut egui::Ui) {
        let screen = self.store.state().screen;
        match screen {
            Screen::Video => video::view::render(ui, &mut self.store),
            Screen::Transcript => transcript::view::render(ui, &mut self.store),
            Screen::Channel => channel::view::render(ui, &mut self.store),
            Screen::History => history::view::render(ui, &mut self.store),
            Screen::Console => console::view::render(ui, &mut self.store),
            Screen::Settings => settings::view::render(ui, &mut self.store),
        }
    }
}

impl eframe::App for SmdApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.store.drain_pending();
        if self.store.is_busy() {
            ctx.request_repaint();
        }
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .default_width(220.0)
            .show(ctx, |ui| {
                sidebar::render(ui, &mut self.store);
            });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_notices(ui);
            self.render_screen(ui);
        });
    }
}
