use std::sync::Arc;

use egui::Context;

use crate::core::intent::{AppIntent, VideoIntent};
use crate::core::state::Screen;
use crate::core::store::Store;
use crate::features::shared::sidebar;
use crate::features::{channel, console, history, settings, transcript, video};
use crate::hotkey::CaptureHotkey;
use crate::i18n::registry::t;
use crate::services::json_history::JsonHistoryStore;
use crate::services::locale::LocaleService;
use crate::services::log_buffer::LevelHandle;
use crate::services::log_sink::BufferLogSink;
use crate::services::yt_dlp::{
    YtDlpChannelProvider, YtDlpDownloader, YtDlpMetadataProvider, YtDlpTranscriber,
};
use crate::storage::config::AppConfig;
use crate::ui::components::{body_margin, truncate_middle};
use crate::ui::theme::{apply_custom_theme, configure_fonts, SIDEBAR_BG};

pub struct SmdApp {
    store: Store,
    capture: Option<CaptureHotkey>,
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
        let mut store = Store::new(
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
        if crate::i18n::registry::available_locales().is_empty() {
            tracing::error!(target: "startup", "no translation files found");
            store.dispatch(AppIntent::PushNotice {
                key: "notice_locales_missing".to_string(),
                detail: "locales/".to_string(),
            });
        }
        let capture = match CaptureHotkey::register() {
            Some(listener) => {
                tracing::info!(target: "startup", "global hotkey Win+Shift+X armed");
                Some(listener)
            }
            None => {
                tracing::warn!(target: "startup", "global hotkey unavailable");
                None
            }
        };
        tracing::info!(target: "startup", "ui ready");
        Self { store, capture }
    }

    fn capture_url_from_clipboard(&mut self) {
        let text = arboard::Clipboard::new()
            .and_then(|mut clipboard| clipboard.get_text())
            .unwrap_or_default();
        let url = text.trim();
        if url.starts_with("http://") || url.starts_with("https://") {
            self.store.dispatch(AppIntent::Navigate(Screen::Video));
            self.store
                .dispatch(AppIntent::Video(VideoIntent::SetUrl(url.to_string())));
        } else {
            tracing::info!(target: "hotkey", "clipboard holds no URL");
        }
    }

    fn render_notices(&mut self, ui: &mut egui::Ui) {
        let notices = self.store.state().notices.clone();
        for notice in &notices {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    let detail = truncate_middle(&notice.detail, 80);
                    ui.label(format!("{}: {}", t(&notice.message_key), detail));
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
        if self.capture.as_ref().is_some_and(|capture| capture.triggered()) {
            self.capture_url_from_clipboard();
        }
        if self.store.is_busy() {
            ctx.request_repaint();
        }
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .default_width(200.0)
            .frame(
                egui::Frame::side_top_panel(&ctx.style())
                    .fill(SIDEBAR_BG)
                    .inner_margin(egui::Margin::symmetric(10.0, 18.0)),
            )
            .show(ctx, |ui| {
                sidebar::render(ui, &mut self.store);
            });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::central_panel(&ctx.style()).inner_margin(body_margin()),
            )
            .show(ctx, |ui| {
                self.render_notices(ui);
                self.render_screen(ui);
            });
    }
}
