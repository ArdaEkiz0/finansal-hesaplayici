//! Görsel sistem: mat lacivert zemin, yüzey katmanları ve anlam taşıyan renk.
//!
//! Renk yalnızca durum, seçim ve eylem içindir:
//!   mavi  (#2f7bff) → seçim / eylem
//!   amber (#f5a524) → uyarı / son gün
//!   yeşil (#10b981) → doğrulama / başarı
//!   kırmızı(#f4536e) → hata
//! Dekoratif vurgu yoktur; ayırıcılar hairline, rakamlar tablo tipindedir.

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Stroke, Visuals,
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
            arka: rgb(0x0a0f1c),
            yuzey: rgb(0x101827),
            yuksek: rgb(0x161f31),
            metin: rgb(0xeef2f8),
            ikinci: rgb(0xc6cfdd),
            soluk: rgb(0x8792a4),
            birincil: rgb(0x2f7bff),
            vurgu: rgb(0xf5a524),
            basari: rgb(0x10b981),
            uyari: rgb(0xf5a524),
            hata: rgb(0xf4536e),
            kenar: rgb(0x27324a),
        }
    } else {
        Palet {
            arka: rgb(0xf5f7fb),
            yuzey: rgb(0xffffff),
            yuksek: rgb(0xeef2f8),
            metin: rgb(0x101729),
            ikinci: rgb(0x3d4a61),
            soluk: rgb(0x5c6b83),
            birincil: rgb(0x2563eb),
            vurgu: rgb(0xb45309),
            basari: rgb(0x047857),
            uyari: rgb(0xb45309),
            hata: rgb(0xbe123c),
            kenar: rgb(0xd7deeb),
        }
    }
}

/// "system" için koyu varsayılan; eframe'in sistem tema algılama API'si yok.
pub fn koyu_mu(tema: &str) -> bool {
    tema != "light"
}

/// Gövde ve sayı fontlarını ayrı yükler: tablo rakamları gerçek sabit genişlikli
///bir yazı tipiyle çizilir (Consolas → Lucida Console → egui varsayılanı).
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

    ctx.set_fonts(fontlar);
}

pub fn uygula(ctx: &egui::Context, tema: &str, olcek: f64, ilk: bool) {
    let p = palet(koyu_mu(tema));
    let mut visuals = if koyu_mu(tema) {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    // Yüzeyler: zemin → panel → yükseltilmiş kutu.
    visuals.panel_fill = p.arka;
    visuals.window_fill = p.yuzey;
    visuals.extreme_bg_color = p.yuksek;
    visuals.faint_bg_color = p.yuksek;
    visuals.selection.bg_fill = p.birincil.gamma_multiply(0.35);
    visuals.selection.stroke.color = p.birincil;
    visuals.hyperlink_color = p.birincil;
    visuals.error_fg_color = p.hata;
    visuals.warn_fg_color = p.vurgu;

    // Araç gibi kısa köşeler, hairline kenarlar, tek seçim rengi.
    let kose = CornerRadius::same(4);
    let kenar = Stroke::new(1.0, p.kenar);
    let mavi_kenar = Stroke::new(1.0, p.birincil);

    visuals.widgets.noninteractive.bg_fill = p.yuzey;
    visuals.widgets.noninteractive.weak_bg_fill = p.yuzey;
    visuals.widgets.noninteractive.bg_stroke = kenar;
    visuals.widgets.noninteractive.fg_stroke.color = p.ikinci;
    visuals.widgets.noninteractive.corner_radius = kose;

    visuals.widgets.inactive.bg_fill = p.yuksek;
    visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    visuals.widgets.inactive.bg_stroke = kenar;
    visuals.widgets.inactive.fg_stroke.color = p.metin;
    visuals.widgets.inactive.corner_radius = kose;

    visuals.widgets.hovered.bg_fill = p.birincil.gamma_multiply(0.14);
    visuals.widgets.hovered.weak_bg_fill = p.birincil.gamma_multiply(0.14);
    visuals.widgets.hovered.bg_stroke = mavi_kenar;
    visuals.widgets.hovered.fg_stroke.color = p.metin;
    visuals.widgets.hovered.corner_radius = kose;

    visuals.widgets.active.bg_fill = p.birincil.gamma_multiply(0.28);
    visuals.widgets.active.weak_bg_fill = p.birincil.gamma_multiply(0.28);
    visuals.widgets.active.bg_stroke = mavi_kenar;
    visuals.widgets.active.fg_stroke.color = p.metin;
    visuals.widgets.active.corner_radius = kose;

    ctx.set_visuals(visuals);
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
        assert_eq!(palet(true).birincil, rgb(0x2f7bff));
    }

    #[test]
    fn koyu_zemin_ve_durum_renkleri() {
        let p = palet(true);
        assert_eq!(p.arka, rgb(0x0a0f1c));
        assert_eq!(p.yuzey, rgb(0x101827));
        assert_eq!(p.vurgu, rgb(0xf5a524));
        assert_eq!(p.basari, rgb(0x10b981));
    }
}
