use crate::api::client::ApiClient;
use crate::api::models::DownloadStatus;
use crate::ui::theme::*;
use egui::{Align, Button, FontId, Layout, ProgressBar, RichText, Ui, Vec2};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct SettingsWindow {
    pub show: bool,
    pub backend_url: String,
    pub download_path: String,
}

impl Default for SettingsWindow {
    fn default() -> Self {
        Self {
            show: false,
            backend_url: "http://localhost:8000".to_string(),
            download_path: String::new(),
        }
    }
}

pub struct DownloaderApp {
    // API Client
    api_client: Arc<ApiClient>,
    runtime: Arc<tokio::runtime::Runtime>,

    // UI State
    url_input: String,
    status_message: String,
    is_downloading: bool,
    backend_connected: bool,

    // Downloads
    downloads: Arc<Mutex<HashMap<String, DownloadStatus>>>,
    active_download_id: Option<String>,

    // Settings
    settings: SettingsWindow,
}

impl DownloaderApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Aplica tema customizado
        apply_custom_theme(&cc.egui_ctx);
        configure_fonts(&cc.egui_ctx);

        // Cria runtime tokio
        let runtime = Arc::new(
            tokio::runtime::Runtime::new().expect("Falha ao criar runtime tokio"),
        );

        // Cria API client
        let api_client = Arc::new(ApiClient::new("http://localhost:8000".to_string()));

        // Testa conexão com backend
        let client_clone = api_client.clone();
        let runtime_clone = runtime.clone();
        let backend_connected = runtime_clone.block_on(async move {
            client_clone.health_check().await.unwrap_or(false)
        });

        Self {
            api_client,
            runtime,
            url_input: String::new(),
            status_message: if backend_connected {
                "✅ Conectado ao backend".to_string()
            } else {
                "⏳ Tentando conectar ao backend...".to_string()
            },
            is_downloading: false,
            backend_connected,
            downloads: Arc::new(Mutex::new(HashMap::new())),
            active_download_id: None,
            settings: SettingsWindow::default(),
        }
    }

    fn start_download(&mut self) {
        if self.url_input.is_empty() {
            self.status_message = "⚠️ Digite uma URL".to_string();
            return;
        }

        let url = self.url_input.clone();
        let api_client = self.api_client.clone();
        let downloads = self.downloads.clone();

        self.is_downloading = true;
        self.status_message = "⏳ Iniciando download...".to_string();

        let runtime = self.runtime.clone();
        runtime.spawn(async move {
            match api_client.start_download(url.clone(), None).await {
                Ok(response) => {
                    let download_id = response.id.clone();
                    
                    // Polling de status
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                        match api_client.get_download_status(&download_id).await {
                            Ok(status) => {
                                let is_complete = status.status == "completed" || status.status == "failed";
                                
                                {
                                    let mut downloads = downloads.lock().unwrap();
                                    downloads.insert(download_id.clone(), status);
                                }

                                if is_complete {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Erro ao iniciar download: {}", e);
                }
            }
        });
    }

    fn render_header(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(
                RichText::new("📥 Social Media Downloader")
                    .size(28.0)
                    .color(ACCENT_PRIMARY),
            );
            ui.add_space(5.0);
            ui.label(
                RichText::new("Baixe vídeos de YouTube, Instagram, TikTok e mais!")
                    .size(14.0)
                    .color(TEXT_SECONDARY),
            );
            ui.add_space(20.0);
        });
    }

    fn render_url_input(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.vertical(|ui| {
                ui.label(RichText::new("URL do Vídeo").color(TEXT_PRIMARY).size(16.0));
                ui.add_space(5.0);

                let text_edit = egui::TextEdit::singleline(&mut self.url_input)
                    .hint_text("Cole a URL aqui ou use Win+Shift+X")
                    .desired_width(ui.available_width())
                    .font(FontId::proportional(14.0));

                ui.add(text_edit);
            });
        });
    }

    fn render_download_button(&mut self, ui: &mut Ui) {
        ui.add_space(10.0);

        let button_text = if self.is_downloading {
            "⏳ Baixando..."
        } else {
            "⬇️ Download"
        };

        let button = Button::new(RichText::new(button_text).size(18.0).color(BG_DARK))
            .fill(ACCENT_PRIMARY)
            .min_size(Vec2::new(ui.available_width(), 45.0));

        if ui.add_enabled(!self.is_downloading && self.backend_connected, button).clicked() {
            self.start_download();
        }
    }

    fn render_status(&self, ui: &mut Ui) {
        ui.add_space(10.0);
        ui.label(RichText::new(&self.status_message).size(14.0).color(TEXT_SECONDARY));
    }

    fn render_downloads(&self, ui: &mut Ui) {
        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        ui.label(RichText::new("Downloads").size(18.0).color(ACCENT_PRIMARY));
        ui.add_space(10.0);

        let downloads = self.downloads.lock().unwrap();
        
        if downloads.is_empty() {
            ui.label(RichText::new("Nenhum download ainda").color(TEXT_DISABLED).size(14.0));
        } else {
            for (id, status) in downloads.iter() {
                self.render_download_item(ui, id, status);
            }
        }
    }

    fn render_download_item(&self, ui: &mut Ui, _id: &str, status: &DownloadStatus) {
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());

            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(status.filename.as_ref().unwrap_or(&"Baixando...".to_string()))
                        .color(TEXT_PRIMARY)
                        .size(14.0),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let status_text = match status.status.as_str() {
                        "completed" => "✅ Completo",
                        "failed" => "❌ Falhou",
                        "downloading" => "⏳ Baixando",
                        _ => "⏸️ Aguardando",
                    };
                    ui.label(RichText::new(status_text).size(12.0).color(TEXT_SECONDARY));
                });
            });

            if status.status == "downloading" {
                ui.add_space(5.0);
                let progress = status.progress / 100.0;
                ui.add(ProgressBar::new(progress).fill(ACCENT_PRIMARY));

                if let Some(speed) = &status.speed {
                    ui.label(RichText::new(format!("Velocidade: {}", speed)).size(12.0).color(TEXT_SECONDARY));
                }
            }
        });

        ui.add_space(5.0);
    }
}

impl eframe::App for DownloaderApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Tenta reconectar se estiver desconectado (a cada 5 segundos aproximadamente)
        if !self.backend_connected {
            let last_check = ctx.data(|d| d.get_temp::<f64>(egui::Id::new("last_conn_check")).unwrap_or(0.0));
            let now = ctx.input(|i| i.time);

            if now - last_check > 5.0 {
                let runtime = self.runtime.clone();
                
                // Salva o tempo do último check
                ctx.data_mut(|d| d.insert_temp(egui::Id::new("last_conn_check"), now));

                // Spawn check async
                let ctx_clone = ctx.clone();
                let api_client_spawn = self.api_client.clone();
                runtime.spawn(async move {
                    if let Ok(true) = api_client_spawn.health_check().await {
                        ctx_clone.request_repaint();
                    }
                });

                // Check síncrono rápido (opcional, mas block_on trava a UI)
                let api_client_sync = self.api_client.clone();
                self.backend_connected = runtime.block_on(async move {
                    api_client_sync.health_check().await.unwrap_or(false)
                });

                if self.backend_connected {
                    self.status_message = "✅ Conectado ao backend".to_string();
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = Vec2::new(8.0, 12.0);

            // Header
            self.render_header(ui);

            // URL Input
            self.render_url_input(ui);

            // Download Button
            self.render_download_button(ui);

            // Status
            self.render_status(ui);

            // Lista de Downloads
            self.render_downloads(ui);

            ui.add_space(20.0);
        });

        // Request repaint para atualizar UI
        ctx.request_repaint();
    }
}
