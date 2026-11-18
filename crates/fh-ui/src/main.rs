//! Finansal Hesaplayici — eframe/egui masaüstü uygulaması (çevrimdışı).

mod app;
mod commands;
mod model;
mod modes;
mod theme;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Finansal Hesaplayıcı")
            .with_inner_size([1280.0, 740.0])
            .with_min_inner_size([820.0, 560.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Finansal Hesaplayici",
        options,
        Box::new(|cc| Ok(Box::new(app::FhApp::new(cc)))),
    )
}
