//! Finansal Hesaplayici — eframe/egui masaüstü uygulaması (çevrimdışı).

mod app;
mod commands;
mod kararlar;
mod lisans;
mod model;
mod modes;
mod theme;

use eframe::egui;

fn main() -> eframe::Result<()> {
    // Lisans kapısı (LISANS_SOZLESMESI §10): run_native ÖNCESİ sert kapı —
    // kilitliyken ana pencere hiç kurulmaz. Ağ yok: imzalı anahtar →
    // saat-geri koruması → 15 günlük yerel deneme.
    let kapi = lisans::kapi_uygula();
    let deneme_kalan = match &kapi {
        lisans::Kapi::Deneme { kalan_gun } => Some(*kalan_gun),
        _ => None,
    };
    if let lisans::Kapi::Kilitli { baslik, gerekce } = kapi {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Finansal Hesaplayıcı — Kilitli")
                .with_inner_size([500.0, 300.0])
                .with_resizable(false),
            ..Default::default()
        };
        return eframe::run_native(
            "Finansal Hesaplayici - Kilitli",
            options,
            Box::new(move |cc| {
                // Kilit ekranı da ürünün kendi temasında (veri.json ayarları).
                let (tema, olcek) = lisans::kilit_tema_ayarlari();
                theme::uygula(&cc.egui_ctx, &tema, olcek, true);
                Ok(Box::new(lisans::KilitEkrani { baslik, gerekce }))
            }),
        );
    }
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
        Box::new(|cc| {
            Ok(Box::new(lisans::Kirpik {
                ic: app::FhApp::new(cc),
                deneme_kalan,
            }))
        }),
    )
}
