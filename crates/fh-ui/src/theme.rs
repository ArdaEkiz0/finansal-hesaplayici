//! Görsel sistem: iki ayrı kimlik.
//!
//! Koyu  → derin lacivert zemin, kontrollü mavi/cyan vurgu, ince ızgara dokusu.
//! Açık  → sıcak kâğıt zemin, koyu mürekkep, aynı mavi vurgu.
//!
//! Renk yalnızca durum, seçim ve eylem içindir:
//!   mavi  → seçim / eylem      cyan  → odak / aktif gösterge
//!   amber → uyarı / son gün    yeşil → doğrulama / başarı
//!   kırmızı → hata
//! Ayırıcılar 1px hairline, rakamlar sabit genişliklidir.

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Margin, Pos2, Rect, Stroke,
    Ui, Vec2, Visuals,
};

#[derive(Debug, Clone, Copy)]
pub struct Palet {
    pub arka: Color32,
    pub yuzey: Color32,
    pub yuksek: Color32,
    pub metin: Color32,
    pub ikinci: Color32,
    pub soluk: Color32,
    pub birincil: Color32,
    /// Cyan odak/aktif göstergesi — yalnızca mavi ile birlikte kullanılır.
    pub aksan: Color32,
    pub vurgu: Color32,
    pub basari: Color32,
    pub uyari: Color32,
    pub hata: Color32,
    pub kenar: Color32,
}

fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

pub fn palet(koyu: bool) -> Palet {
    if koyu {
        Palet {
            arka: rgb(0x080e1b),
            yuzey: rgb(0x111b2d),
            yuksek: rgb(0x1a263c),
            metin: rgb(0xf2f6fc),
            ikinci: rgb(0xc6d2e2),
            soluk: rgb(0x8a99b0),
            birincil: rgb(0x3b8dff),
            aksan: rgb(0x39c9d6),
            vurgu: rgb(0xf5a524),
            basari: rgb(0x34d399),
            uyari: rgb(0xf5a524),
            hata: rgb(0xf4637a),
            kenar: rgb(0x2a3852),
        }
    } else {
        Palet {
            arka: rgb(0xf3efe6),
            yuzey: rgb(0xfffdf8),
            yuksek: rgb(0xf1ece1),
            metin: rgb(0x172033),
            ikinci: rgb(0x3c485c),
            soluk: rgb(0x6c7789),
            birincil: rgb(0x2563eb),
            aksan: rgb(0x0e9db0),
            vurgu: rgb(0xb45309),
            basari: rgb(0x047857),
            uyari: rgb(0xb45309),
            hata: rgb(0xbe123c),
            kenar: rgb(0xd9d2c4),
        }
    }
}

/// "system" için koyu varsayılan; eframe'in sistem tema algılama API'si yok.
pub fn koyu_mu(tema: &str) -> bool {
    tema != "light"
}

fn karisim(a: Color32, b: Color32, t: f32) -> Color32 {
    let ar = a.r() as f32;
    let ag = a.g() as f32;
    let ab = a.b() as f32;
    Color32::from_rgb(
        (ar + (b.r() as f32 - ar) * t) as u8,
        (ag + (b.g() as f32 - ag) * t) as u8,
        (ab + (b.b() as f32 - ab) * t) as u8,
    )
}

/// Kompakt widget kartının dolgusu: panel yüzeyinden bir tık yükseltilmiş.
/// Mevcut paletin `yuzey`/`yuksek` renklerini karıştırır; yeni kimlik katmaz.
pub fn widget_kart(koyu: bool) -> Color32 {
    let p = palet(koyu);
    karisim(p.yuzey, p.yuksek, if koyu { 0.55 } else { 0.35 })
}

/// Widget başlığındaki ince cyan vurgu çizgisi (odak/aktif göstergesi).
pub fn widget_vurgu_cizgi(koyu: bool) -> Stroke {
    Stroke::new(2.0, palet(koyu).aksan)
}

/// Widget kenarı: tema genelindeki 1px hairline kuralıyla aynı.
pub fn widget_kenar(koyu: bool) -> Stroke {
    Stroke::new(1.0, palet(koyu).kenar)
}

/// Widget hover durumu: (dolgu, kenar). Temanın mavi hover kimliğini kullanır.
pub fn widget_hover(koyu: bool) -> (Color32, Color32) {
    let p = palet(koyu);
    (p.birincil.gamma_multiply(0.18), p.birincil)
}

/// Ayrı widget viewport'u için çerçevesiz dolgu/stil yardımcısı.
///
/// Ana pencere çerçevesinin aksine dış kenar, gölge veya ayrı bir renk kimliği
/// yoktur: yalnızca mevcut paletin `yuzey` rengiyle dolgu, sabit iç boşluk ve
/// köşe yuvarlaklığı vardır. Böylece koyu/açık tema kimliği bozulmaz.
pub fn widget_viewport_frame(p: Palet) -> egui::Frame {
    egui::Frame::NONE
        .inner_margin(Margin::same(6))
        .corner_radius(CornerRadius::same(6))
        .fill(p.yuzey)
        .stroke(Stroke::NONE)
}

/// Panel içine çok hafif dikey gradyan + ızgara dokusu çizer.
/// İçerik eklenmeden ÖNCE çağrılır; boyamalar içeriğin altında kalır.
pub fn doku(ui: &mut Ui, koyu: bool) {
    let r = ui.max_rect();
    let (ust, alt, izgara) = if koyu {
        (
            rgb(0x0d1830),
            rgb(0x080e1b),
            Color32::from_rgba_unmultiplied(0x5a, 0x8c, 0xdc, 11),
        )
    } else {
        (
            rgb(0xfffdf8),
            rgb(0xf2ede1),
            Color32::from_rgba_unmultiplied(0x78, 0x6e, 0x5a, 9),
        )
    };

    let painter = ui.painter();
    let n = 32;
    for i in 0..n {
        let t = (i as f32 + 0.5) / n as f32;
        let y0 = r.min.y + r.height() * (i as f32 / n as f32);
        let y1 = r.min.y + r.height() * ((i + 1) as f32 / n as f32);
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(r.min.x, y0), Pos2::new(r.max.x, y1)),
            CornerRadius::ZERO,
            karisim(ust, alt, t),
        );
    }

    let adim = 34.0;
    let mut x = r.min.x + adim;
    while x < r.max.x {
        painter.line_segment(
            [Pos2::new(x, r.min.y), Pos2::new(x, r.max.y)],
            Stroke::new(1.0, izgara),
        );
        x += adim;
    }
    let mut y = r.min.y + adim;
    while y < r.max.y {
        painter.line_segment(
            [Pos2::new(r.min.x, y), Pos2::new(r.max.x, y)],
            Stroke::new(1.0, izgara),
        );
        y += adim;
    }
}

/// Gövde ve sayı fontlarını ayrı yükler: tablo rakamları gerçek sabit genişlikli
/// bir yazı tipiyle çizilir (Consolas → Lucida Console → egui varsayılanı).
fn sistem_fontlari(ctx: &egui::Context) {
    let mut fontlar = FontDefinitions::default();

    if let Ok(bayt) = std::fs::read("C:/Windows/Fonts/segoeui.ttf") {
        fontlar
            .font_data
            .insert("Govde".to_string(), std::sync::Arc::new(FontData::from_owned(bayt)));
        fontlar
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "Govde".to_string());
    }
    if let Ok(bayt) = std::fs::read("C:/Windows/Fonts/seguisb.ttf") {
        fontlar
            .font_data
            .insert("Baslik".to_string(), std::sync::Arc::new(FontData::from_owned(bayt)));
        fontlar
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "Baslik".to_string());
    }

    for aday in ["C:/Windows/Fonts/consola.ttf", "C:/Windows/Fonts/lucon.ttf"] {
        if let Ok(bayt) = std::fs::read(aday) {
            fontlar
                .font_data
                .insert("Sayi".to_string(), std::sync::Arc::new(FontData::from_owned(bayt)));
            fontlar
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .insert(0, "Sayi".to_string());
            break;
        }
    }

    // Pencere kontrol glifleri (─ □ ❐ ×) için sembol yüzü. Ailenin SONUNA
    // eklenir: Latin metinlerin mevcut yüzleri değişmez, yalnızca eksik
    // glifler bu yüzden karşılanır.
    for aday in ["C:/Windows/Fonts/seguisym.ttf", "C:/Windows/Fonts/segsym.ttf"] {
        if let Ok(bayt) = std::fs::read(aday) {
            fontlar
                .font_data
                .insert("Sembol".to_string(), std::sync::Arc::new(FontData::from_owned(bayt)));
            fontlar
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .push("Sembol".to_string());
            fontlar
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .push("Sembol".to_string());
            break;
        }
    }

    ctx.set_fonts(fontlar);
}

pub fn uygula(ctx: &egui::Context, tema: &str, olcek: f64, ilk: bool) {
    let koyu = koyu_mu(tema);
    let p = palet(koyu);
    let mut visuals = if koyu {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    visuals.dark_mode = koyu;
    visuals.override_text_color = None;
    visuals.weak_text_color = Some(p.soluk);

    // Yüzeyler: zemin → panel → yükseltilmiş kutu.
    visuals.panel_fill = p.arka;
    visuals.window_fill = p.yuzey;
    visuals.window_stroke = Stroke::new(1.0, p.kenar);
    visuals.window_corner_radius = CornerRadius::same(10);
    visuals.menu_corner_radius = CornerRadius::same(8);
    visuals.extreme_bg_color = p.yuksek;
    visuals.text_edit_bg_color = Some(p.yuksek);
    visuals.code_bg_color = p.yuksek;
    visuals.faint_bg_color = p.yuksek.gamma_multiply(if koyu { 1.0 } else { 0.8 });

    visuals.selection.bg_fill = p.birincil.gamma_multiply(0.35);
    visuals.selection.stroke.color = p.birincil;
    visuals.hyperlink_color = p.birincil;
    visuals.error_fg_color = p.hata;
    visuals.warn_fg_color = p.vurgu;

    // Tutarlı köşeler, 1px hairline kenarlar, belirgin hover/focus/seçim durumları.
    let kose = CornerRadius::same(6);
    let kenar = Stroke::new(1.0, p.kenar);
    let mavi_kenar = Stroke::new(1.0, p.birincil);

    visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;
    visuals.widgets.noninteractive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.noninteractive.bg_stroke = Stroke::NONE;
    visuals.widgets.noninteractive.fg_stroke.color = p.ikinci;
    visuals.widgets.noninteractive.corner_radius = kose;

    visuals.widgets.inactive.bg_fill = p.yuksek;
    visuals.widgets.inactive.weak_bg_fill = p.yuksek;
    visuals.widgets.inactive.bg_stroke = kenar;
    visuals.widgets.inactive.fg_stroke.color = p.metin;
    visuals.widgets.inactive.corner_radius = kose;

    visuals.widgets.hovered.bg_fill = p.birincil.gamma_multiply(0.18);
    visuals.widgets.hovered.weak_bg_fill = p.birincil.gamma_multiply(0.18);
    visuals.widgets.hovered.bg_stroke = mavi_kenar;
    visuals.widgets.hovered.fg_stroke.color = p.metin;
    visuals.widgets.hovered.corner_radius = kose;

    // egui, has_focus() durumunu da active ile çizer → klavye odağı görünür.
    visuals.widgets.active.bg_fill = p.birincil.gamma_multiply(0.32);
    visuals.widgets.active.weak_bg_fill = p.birincil.gamma_multiply(0.32);
    visuals.widgets.active.bg_stroke = mavi_kenar;
    visuals.widgets.active.fg_stroke.color = p.metin;
    visuals.widgets.active.corner_radius = kose;

    visuals.widgets.open.bg_fill = p.birincil.gamma_multiply(0.24);
    visuals.widgets.open.weak_bg_fill = p.birincil.gamma_multiply(0.24);
    visuals.widgets.open.bg_stroke = mavi_kenar;
    visuals.widgets.open.fg_stroke.color = p.metin;
    visuals.widgets.open.corner_radius = kose;

    ctx.set_visuals(visuals);

    // Nefes alan düzen: iki tema aynı ölçüyü kullanır, yalnızca renk kimliği
    // ayrılır. A−/A+ zoom ile bu ölçüler birlikte büyür/küçülür.
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.button_padding = Vec2::new(10.0, 4.0);
        style.spacing.interact_size = Vec2::new(44.0, 26.0);
        style.spacing.window_margin = Margin::same(10);
    });

    ctx.set_zoom_factor(olcek as f32);
    if ilk {
        sistem_fontlari(ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tema_koyu_varsayilan() {
        assert!(koyu_mu("dark"));
        assert!(koyu_mu("system"));
        assert!(!koyu_mu("light"));
    }

    #[test]
    fn paletler_farkli() {
        assert_ne!(palet(true).arka, palet(false).arka);
        assert_ne!(palet(true).yuzey, palet(false).yuzey);
        assert_eq!(palet(true).birincil, rgb(0x3b8dff));
        assert_eq!(palet(false).birincil, rgb(0x2563eb));
    }

    #[test]
    fn koyu_zemin_ve_durum_renkleri() {
        let p = palet(true);
        assert_eq!(p.arka, rgb(0x080e1b));
        assert_eq!(p.yuzey, rgb(0x111b2d));
        assert_eq!(p.aksan, rgb(0x39c9d6));
        assert_eq!(p.vurgu, rgb(0xf5a524));
        assert_eq!(p.basari, rgb(0x34d399));
    }

    #[test]
    fn acik_tema_kagit_ve_murekkep() {
        let p = palet(false);
        assert_eq!(p.arka, rgb(0xf3efe6));
        assert_eq!(p.metin, rgb(0x172033));
        assert_ne!(p.birincil, p.arka);
        assert_ne!(p.kenar, palet(true).kenar);
    }
}
