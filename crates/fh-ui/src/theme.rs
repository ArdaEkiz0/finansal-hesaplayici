//! Aurora Ledger paleti (index.css 48–101 satırlarının egui karşılığı).

use eframe::egui::{self, Color32, FontDefinitions, FontFamily, FontData, Visuals};

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
    Color32::from_rgb(((hex >> 16) & 0xff) as u8, ((hex >> 8) & 0xff) as u8, (hex & 0xff) as u8)
}

pub fn palet(koyu: bool) -> Palet {
    if koyu {
        Palet {
            arka: rgb(0x06090f),
            yuzey: rgb(0x101827),
            yuksek: rgb(0x17203a),
            metin: rgb(0xeef2f8),
            ikinci: rgb(0xc6cfdd),
            soluk: rgb(0x8792a4),
            birincil: rgb(0x2f7bff),
            vurgu: rgb(0xf5a524),
            basari: rgb(0x10b981),
            uyari: rgb(0xf0a62e),
            hata: rgb(0xf4536e),
            kenar: rgb(0x27324a),
        }
    } else {
        Palet {
            arka: rgb(0xf6f8fc),
            yuzey: rgb(0xffffff),
            yuksek: rgb(0xedf1f8),
            metin: rgb(0x101729),
            ikinci: rgb(0x3d4a61),
            soluk: rgb(0x62708a),
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

fn sistem_fontlari(ctx: &egui::Context) {
    let adaylar = [
        "C:/Windows/Fonts/segoeui.ttf",
        "C:/Windows/Fonts/arial.ttf",
        "C:/Windows/Fonts/calibri.ttf",
    ];
    let mut fontlar = FontDefinitions::default();
    for aday in adaylar {
        if let Ok(bayt) = std::fs::read(aday) {
            fontlar.font_data.insert(
                "SistemFont".to_string(),
                std::sync::Arc::new(FontData::from_owned(bayt)),
            );
            fontlar
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "SistemFont".to_string());
            fontlar
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .insert(0, "SistemFont".to_string());
            break;
        }
    }
    ctx.set_fonts(fontlar);
}

pub fn uygula(ctx: &egui::Context, tema: &str, olcek: f64, ilk: bool) {
    let koyu = koyu_mu(tema);
    let p = palet(koyu);
    let mut visuals = if koyu { Visuals::dark() } else { Visuals::light() };
    visuals.panel_fill = p.arka;
    visuals.window_fill = p.yuzey;
    visuals.extreme_bg_color = p.yuksek;
    visuals.faint_bg_color = p.yuksek;
    visuals.selection.bg_fill = p.birincil.gamma_multiply(0.45);
    visuals.hyperlink_color = p.birincil;
    visuals.error_fg_color = p.hata;
    visuals.warn_fg_color = p.uyari;
    visuals.widgets.noninteractive.bg_fill = p.yuksek;
    visuals.widgets.noninteractive.bg_stroke.color = p.kenar;
    visuals.widgets.noninteractive.fg_stroke.color = p.ikinci;
    visuals.widgets.inactive.fg_stroke.color = p.metin;
    visuals.widgets.hovered.fg_stroke.color = p.metin;
    visuals.widgets.active.fg_stroke.color = p.metin;
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
}
