//! Finansal Hesaplayici — eframe/egui masaüstü uygulaması (çevrimdışı).

mod app;
mod commands;
mod kararlar;
mod model;
mod modes;
mod theme;

use eframe::egui;

fn main() -> eframe::Result<()> {
    // Finansal Hesaplayıcı HEDİYE ÜRÜNDÜR: lisans/deneme kapısı YOKTUR
    // (21 Kas 2026 kararı — satılmaz, süre işlemez; v5.7.6'daki kapı yanlıştı).
    let p = model::PENCERE_AYARLARI;
    // Maksimum açılış: pencere ekrana sığar, küçük görünmez. Ölçü yalnızca
    // kullanıcı pencereyi geri restore ettiğinde (maksimum kapalıyken) kullanılır.
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Finansal Hesaplayıcı")
        // OS başlık çubuğu kaldırılır: üst şeritteki ─ □ × tek kontrol seti olur.
        .with_decorations(false)
        .with_min_inner_size(p.min_inner_size);
    viewport = if p.maksimum_acilis {
        viewport.with_maximized(true)
    } else {
        viewport.with_inner_size(p.inner_size)
    };
    let options = eframe::NativeOptions {
        viewport,
        // Bozuk/14x14 kaydedilmiş pencere boyutu sonraki açılışları gizliyordu.
        persist_window: p.persist_window,
        ..Default::default()
    };
    eframe::run_native(
        "Finansal Hesaplayici",
        options,
        Box::new(|cc| Ok(Box::new(app::FhApp::new(cc)))),
    )
}
