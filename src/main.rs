mod api;
mod ui;

use ui::app::DownloaderApp;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("Social Media Downloader"),
        ..Default::default()
    };

    eframe::run_native(
        "Social Media Downloader",
        options,
        Box::new(|cc| Ok(Box::new(DownloaderApp::new(cc)))),
    )
}
