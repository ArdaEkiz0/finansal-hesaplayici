//! Bağımsız mini hesaplayıcı widget'ı (gerçek ayrı OS penceresi / egui viewport).
//!
//! Tüm hesaplama `fh_core::hesapla` üzerinden yapılır; bu dosyada finansal
//! formül yoktur. Görünüm durumu `AppState::widget` (`WidgetAyarlari`) içinde
//! saklanır ve `state.kaydet()` ile kalıcılaşır.
//!
//! Kompakt görünüm tek satırlık sonuç kartı; geniş görünüm mod seçici, alan
//! girişleri, canlı sonuç, döküm, hızlı eylemler ve son kayıtlardan oluşur.
//! `egui::Window` sarması kaldırıldı; içerik doğrudan viewport Ui'sine çizilir.

use std::time::Instant;

use eframe::egui::{
    self, Align, Align2, Color32, Context, CornerRadius, FontId, Layout, Margin, Pos2, Rect,
    RichText, Sense, Stroke, TextWrapMode, Ui, Vec2,
};
use fh_core::HesapSonucu;

use crate::commands::turkce_kucuk;
use crate::model::{
    HistEntry, InputState, Settings, WidgetAyarlari, WIDGET_GENISLIK_ARALIGI,
    WIDGET_YUKSEKLIK_ARALIGI, gecmis_ekle, para_bicimi, satir_bicimi,
};
use crate::modes::{self, FieldKind, MODLAR};
use crate::theme::{self, Palet};

use super::{FhApp, T_BODY, T_SMALL, birincil_dugme, hairline, ikincil_dugme};

// model.rs'teki doğrulama aralıklarıyla aynı sınırlar: dışarıda kalan değerler
// yedekten geri yüklenirken varsayılana düşer, burada da aynı kural uygulanır.
const COMPACT_SIZE: (f32, f32) = (220.0, 160.0);
const WIDE_SIZE: (f32, f32) = (340.0, 430.0);

/// Kaydedilmiş boyut geçerli aralıktaysa onu, değilse görünüme göre ölçüyü kullanır.
pub(super) fn etkili_boyut(kompakt: bool, genislik: f32, yukseklik: f32) -> (f32, f32) {
    let (gmin, gmax) = WIDGET_GENISLIK_ARALIGI;
    let (ymin, ymax) = WIDGET_YUKSEKLIK_ARALIGI;
    if genislik.is_finite()
        && yukseklik.is_finite()
        && (gmin..=gmax).contains(&genislik)
        && (ymin..=ymax).contains(&yukseklik)
    {
        (genislik, yukseklik)
    } else if kompakt {
        COMPACT_SIZE
    } else {
        WIDE_SIZE
    }
}

/// Küçük kare ikon düğmesi (kompakt/geniş, üstte kal, kapat, kopyala).
fn ikon_dugme(ui: &mut Ui, glif: &str, p: Palet, koyu: bool, aktif: bool, kapat: bool) -> bool {
    let renk = if kapat {
        p.hata
    } else if aktif {
        p.aksan
    } else {
        p.ikinci
    };
    let (hover_dolgu, hover_kenar_rengi) = theme::widget_hover(koyu);
    let b = egui::Button::new(RichText::new(glif).size(14.0).color(renk).strong())
        .fill(if aktif { hover_dolgu } else { p.yuksek })
        .stroke(if aktif {
            Stroke::new(1.0, hover_kenar_rengi)
        } else {
            theme::widget_kenar(koyu)
        })
        .corner_radius(CornerRadius::same(5))
        .min_size(Vec2::new(24.0, 22.0));
    ui.add(b).clicked()
}

/// Sayısal tuş takımının tek bir basışı. Karakter aktif alana eklenir;
/// `Sil` son karakteri düşürür, `Temizle` alanı boşaltır.
#[derive(Clone, Copy)]
enum TusEylemi {
    Karakter(char),
    Sil,
    Temizle,
}

/// Tuş takımı düğmesi: mevcut palet, 30px yükseklik, 6px köşe.
/// Hover/focus'ta aksan çerçevesi çizilir (yeni görsel kimlik katmaz).
fn tus_dugme(ui: &mut Ui, metin: &str, p: Palet, koyu: bool, genis: f32) -> bool {
    let b = egui::Button::new(RichText::new(metin).size(15.0).color(p.metin).strong())
        .fill(p.yuksek)
        .stroke(theme::widget_kenar(koyu))
        .corner_radius(CornerRadius::same(6))
        .min_size(Vec2::new(genis, 30.0));
    let resp = ui.add(b);
    if resp.hovered() || resp.has_focus() {
        let (_, aktif) = theme::tus_takimi_stili(koyu);
        ui.painter().rect_stroke(
            resp.rect,
            CornerRadius::same(6),
            Stroke::new(1.5, aktif),
            egui::StrokeKind::Inside,
        );
    }
    resp.clicked()
}

/// Onaylı paletten ₺ rozeti (döndürülmüş yazı veya yeni grafik yok).
fn logo(ui: &mut Ui, p: Palet) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(22.0, 22.0), Sense::hover());
    ui.painter()
        .circle_filled(r.center(), 10.0, p.birincil.gamma_multiply(0.22));
    ui.painter()
        .circle_stroke(r.center(), 10.0, Stroke::new(1.0, p.aksan));
    ui.painter().text(
        r.center(),
        Align2::CENTER_CENTER,
        "₺",
        FontId::proportional(12.0),
        p.birincil,
    );
}

fn sonuc_metni(mod_id: &str, r: &HesapSonucu, st: &Settings) -> String {
    let ana = para_bicimi(
        &r.ana,
        st.precision,
        &st.currency_symbol,
        st.show_currency_symbol,
    );
    let mut metin = format!("{}: {}", modes::mod_etiketi(mod_id), ana);
    if !r.alt.is_empty() {
        metin.push_str(&format!(
            "\nDiğer: {}",
            para_bicimi(
                &r.alt,
                st.precision,
                &st.currency_symbol,
                st.show_currency_symbol
            )
        ));
    }
    if !r.detay.is_empty() {
        metin.push_str(&format!("\n{}", r.detay));
    }
    for (k, v) in &r.satirlar {
        metin.push_str(&format!(
            "\n{k}: {}",
            satir_bicimi(
                k,
                v,
                st.precision,
                &st.currency_symbol,
                st.show_currency_symbol
            )
        ));
    }
    metin
}

impl FhApp {
    /// Widget içeriğini doğrudan (ayrı OS penceresi olan) viewport Ui'sine çizer.
    /// `app.rs::ui` yalnızca `state.widget.aktif` ise çağırır.
    pub(crate) fn widget_viewport_ui(&mut self, ui: &mut Ui, main_ctx: Context) {
        let koyu = theme::koyu_mu(&self.state.settings.theme);
        let p = theme::palet(koyu);

        // Kaydedilmiş konumu ilk açılışta bir kez uygula. Bu karede
        // `outer_rect` henüz eski konumu göstereceğinden konum kaydını atla.
        let mut konum_uygulandi_bu_kare = false;
        if !self.widget_konum_uygulandi {
            self.widget_konum_uygulandi = true;
            if let Some(konum) = self.state.widget.konum {
                if WidgetAyarlari::konum_gecerli(konum) {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                        Pos2::new(konum[0], konum[1]),
                    ));
                    konum_uygulandi_bu_kare = true;
                }
            }
        }
        let st: Settings = self.state.settings.clone();
        let mod_id = self.state.widget.r#mod.clone();
        let kompakt = self.state.widget.kompakt;
        let ustte = self.state.widget.ustte;
        let (sw, sh) = etkili_boyut(
            kompakt,
            self.state.widget.genislik,
            self.state.widget.yukseklik,
        );
        let kopyalandi_yeni = self.widget_kopyalandi;

        // OS kapatma isteği (Alt+F4 / görev çubuğu): kalıcı kapalı duruma geç.
        // CancelClose gönderilmez; bir sonraki karede viewport açılmayacağı için
        // pencere kendiliğinden kapanır.
        if ui.ctx().input(|i| i.viewport().close_requested()) {
            self.state.widget.aktif = false;
            self.state.kaydet();
        }

        // Widget'in kendi modu için girdi hazır olsun; sonuç anlık görüntüsü
        // `fh_core::hesapla` ile üretilir (formül kopyalanmaz).
        {
            let e = self.state.inputs.entry(mod_id.clone()).or_default();
            modes::varsayilanlari_doldur(e, &mod_id);
        }
        let girdi = self.state.inputs.get(&mod_id).cloned().unwrap_or_default();
        let sonuc: Option<HesapSonucu> =
            fh_core::hesapla(&mod_id, &modes::build_girdi(&mod_id, &girdi));

        let gecmis: Vec<(String, InputState, String, String)> = self
            .state
            .history
            .iter()
            .take(3)
            .map(|e| {
                let tarih = chrono::DateTime::from_timestamp(e.timestamp, 0)
                    .map(|d| d.format("%d.%m %H:%M").to_string())
                    .unwrap_or_default();
                let deger = para_bicimi(
                    &e.ana,
                    st.precision,
                    &st.currency_symbol,
                    st.show_currency_symbol,
                );
                (
                    e.mode.clone(),
                    e.inputs.clone(),
                    format!("{} · {}", modes::mod_etiketi(&e.mode), deger),
                    tarih,
                )
            })
            .collect();

        let mut boyut_sifirlandi = false;
        let mut yeni_mod: Option<String> = None;
        let mut kopyala = false;
        let mut kaydet = false;
        let mut temizle = false;
        let mut geri_yukle: Option<(String, InputState)> = None;
        let mut alan_odakta = false;
        let mut tus_eylemleri: Vec<TusEylemi> = Vec::new();

        // Onaylı paletle çerçevesiz zemin dolgusu (ayrı viewport kimliği).
        theme::widget_viewport_frame(p).show(ui, |ui| {
            // ---- özel başlık: logo, mod adı, sürükleme alanı, üstte/kompakt/kapat ----
            ui.horizontal(|ui| {
                logo(ui, p);
                ui.add_space(4.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width().min(120.0), 24.0),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.add(
                            egui::Label::new(
                                RichText::new(modes::mod_etiketi(&mod_id))
                                    .size(T_BODY)
                                    .color(p.metin)
                                    .strong(),
                            )
                            .wrap_mode(TextWrapMode::Truncate),
                        );
                    },
                );

                // Başlıktaki boş alan pencereyi sürükler (OS çubuğu kapalı).
                let bosluk = (ui.available_width() - 172.0).max(8.0);
                let surukle = ui.allocate_rect(
                    Rect::from_min_size(ui.cursor().min, Vec2::new(bosluk, 26.0)),
                    Sense::drag(),
                );
                if surukle.drag_started() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Kapat: tek düğme, kalıcı duruma yazar.
                    if ikon_dugme(ui, "×", p, koyu, false, true) {
                        self.state.widget.aktif = false;
                        self.state.kaydet();
                    }
                    if ikon_dugme(ui, if ustte { "◉" } else { "◎" }, p, koyu, ustte, false) {
                        self.state.widget.ustte = !ustte;
                        self.state.kaydet();
                    }
                    if ikon_dugme(ui, if kompakt { "▣" } else { "▤" }, p, koyu, kompakt, false)
                    {
                        let yeni = !kompakt;
                        self.state.widget.kompakt = yeni;
                        let (w, h) = if yeni { COMPACT_SIZE } else { WIDE_SIZE };
                        self.state.widget.genislik = w;
                        self.state.widget.yukseklik = h;
                        boyut_sifirlandi = true;
                        self.state.kaydet();
                    }
                    // Ana pencereyi küçült; widget viewport'u görünür kalır.
                    // Komut ana ctx'e gönderilir (widget ctx'i değil).
                    let b = egui::Button::new(
                        RichText::new("Sadece widget").size(10.0).color(p.ikinci).strong(),
                    )
                    .fill(p.yuksek)
                    .stroke(theme::widget_kenar(koyu))
                    .corner_radius(CornerRadius::same(5))
                    .min_size(Vec2::new(70.0, 22.0));
                    if ui.add(b).clicked() {
                        main_ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                });
            });
            let ust_cizgi = ui.available_rect_before_wrap();
            ui.painter().hline(
                ust_cizgi.x_range(),
                ust_cizgi.min.y,
                theme::widget_vurgu_cizgi(koyu),
            );
            ui.add_space(4.0);

            // ---- gövde: sabit viewport içinde kaydırılabilir ----
            let govde_y = (ui.available_height() - 18.0).max(40.0);
            ui.allocate_ui(Vec2::new(ui.available_width(), govde_y), |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        if kompakt {
                            // ---- kompakt: mod adı + büyük sonuç + kopyala ikonu ----
                            ui.horizontal(|ui| {
                                egui::Frame::new()
                                    .fill(theme::widget_kart(koyu))
                                    .stroke(theme::widget_kenar(koyu))
                                    .corner_radius(CornerRadius::same(8))
                                    .inner_margin(Margin::symmetric(10, 6))
                                    .show(ui, |ui| {
                                        ui.vertical(|ui| {
                                            ui.label(
                                                RichText::new("AN TUTAR")
                                                    .size(10.0)
                                                    .color(p.aksan)
                                                    .strong(),
                                            );
                                            let v = sonuc
                                                .as_ref()
                                                .map(|r| {
                                                    para_bicimi(
                                                        &r.ana,
                                                        st.precision,
                                                        &st.currency_symbol,
                                                        st.show_currency_symbol,
                                                    )
                                                })
                                                .unwrap_or_else(|| "—".to_string());
                                            ui.add(
                                                egui::Label::new(
                                                    RichText::new(v)
                                                        .size(25.0)
                                                        .color(p.metin)
                                                        .strong()
                                                        .monospace(),
                                                )
                                                .wrap_mode(TextWrapMode::Truncate),
                                            );
                                        });
                                    });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ikon_dugme(ui, "⧉", p, koyu, false, false) {
                                        kopyala = true;
                                    }
                                });
                            });
                            if kopyalandi_yeni
                                .is_some_and(|t| t.elapsed() < std::time::Duration::from_secs(2))
                            {
                                ui.label(
                                    RichText::new("✓ Panoya kopyalandı")
                                        .size(10.0)
                                        .color(p.basari)
                                        .strong(),
                                );
                            }
                        } else {
                            // ---- geniş: mod seçici (arama + dropdown) ----
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.widget_arama)
                                        .hint_text("Mod ara")
                                        .desired_width(84.0),
                                );
                                let etiket = modes::mod_etiketi(&mod_id).to_string();
                                egui::ComboBox::from_id_salt("widget-mod-secici")
                                    .selected_text(etiket)
                                    .width(148.0)
                                    .show_ui(ui, |ui| {
                                        let q = turkce_kucuk(&self.widget_arama);
                                        let mut bulundu = 0usize;
                                        for m in MODLAR {
                                            if !q.is_empty() && !turkce_kucuk(m.etiket).contains(&q)
                                            {
                                                continue;
                                            }
                                            bulundu += 1;
                                            if ui
                                                .selectable_label(m.id == mod_id, m.etiket)
                                                .clicked()
                                            {
                                                yeni_mod = Some(m.id.to_string());
                                            }
                                        }
                                        if bulundu == 0 {
                                            ui.label(
                                                RichText::new("Eşleşme yok")
                                                    .size(T_SMALL)
                                                    .color(p.soluk),
                                            );
                                        }
                                    });
                            });
                            ui.add_space(6.0);

                            // ---- modun alan tanımlarından 1-2 giriş + gerekiyorsa seçim ----
                            if let Some(def) = modes::mod_def(&mod_id) {
                                let metin_alanlari: Vec<_> = def
                                    .alanlar
                                    .iter()
                                    .filter(|f| f.tur == FieldKind::Metin)
                                    .take(2)
                                    .collect();
                                for (idx, f) in metin_alanlari.iter().enumerate() {
                                    ui.label(
                                        RichText::new(f.etiket).size(11.0).color(p.ikinci).strong(),
                                    );
                                    let mut tiklandi = false;
                                    let mut odakta = false;
                                    let mut alan_rect = None;
                                    if let Some(e) = self.state.inputs.get_mut(&mod_id) {
                                        let t = egui::TextEdit::singleline(e.get_mut(f.alan))
                                            .hint_text(f.ipucu)
                                            .font(FontId::monospace(13.0));
                                        let resp = ui.add_sized(Vec2::new(ui.available_width(), 28.0), t);
                                        tiklandi = resp.clicked() || resp.gained_focus();
                                        odakta = resp.has_focus();
                                        alan_rect = Some(resp.rect);
                                    }
                                    if tiklandi {
                                        self.widget_alan = idx;
                                    }
                                    if odakta {
                                        alan_odakta = true;
                                    }
                                    if idx == self.widget_alan {
                                        if let Some(r) = alan_rect {
                                            ui.painter().rect_stroke(
                                                r.expand(1.0),
                                                CornerRadius::same(5),
                                                Stroke::new(1.0, p.aksan),
                                                egui::StrokeKind::Inside,
                                            );
                                        }
                                    }
                                    ui.add_space(4.0);
                                }
                                // Enter: widget alanlarından biri odaktayken sonucu panoya kopyala.
                                if alan_odakta && ui.ctx().input(|i| i.key_pressed(egui::Key::Enter)) {
                                    kopyala = true;
                                }
                                for f in def
                                    .alanlar
                                    .iter()
                                    .filter(|f| {
                                        matches!(f.tur, FieldKind::Secim | FieldKind::PaySecim)
                                    })
                                    .take(1)
                                {
                                    let secili = self
                                        .state
                                        .inputs
                                        .get(&mod_id)
                                        .map(|e| e.get(f.alan).to_string())
                                        .unwrap_or_default();
                                    let secenekler = if f.tur == FieldKind::PaySecim {
                                        &modes::SEC_PAY[..]
                                    } else {
                                        f.secenekler
                                    };
                                    let etiket = secenekler
                                        .iter()
                                        .find(|(_, v)| *v == secili)
                                        .map(|(l, _)| *l)
                                        .or_else(|| secenekler.first().map(|(l, _)| *l))
                                        .unwrap_or("—");
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(f.etiket).size(11.0).color(p.ikinci),
                                        );
                                        egui::ComboBox::from_id_salt(format!(
                                            "widget-secim-{:?}",
                                            f.alan
                                        ))
                                        .selected_text(etiket)
                                        .width(118.0)
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                for (label, value) in secenekler {
                                                    if ui
                                                        .selectable_label(secili == *value, *label)
                                                        .clicked()
                                                    {
                                                        if let Some(e) =
                                                            self.state.inputs.get_mut(&mod_id)
                                                        {
                                                            *e.get_mut(f.alan) = value.to_string();
                                                        }
                                                    }
                                                }
                                            },
                                        );
                                    });
                                }
                            }

                            // ---- sayısal tuş takımı: aktif metin alanına yazar ----
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new("TUŞ TAKIMI").size(10.0).color(p.aksan).strong(),
                            );
                            let tus_toplam = ui.available_width();
                            let tus_g3 = (tus_toplam - 12.0) / 3.0;
                            let tus_g4 = (tus_toplam - 18.0) / 4.0;
                            let satirlar: [&[&str]; 4] = [
                                &["7", "8", "9"],
                                &["4", "5", "6"],
                                &["1", "2", "3"],
                                &["0", ",", "⌫", "C"],
                            ];
                            for satir in satirlar {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 6.0;
                                    let genis = if satir.len() == 4 { tus_g4 } else { tus_g3 };
                                    for tus in satir {
                                        if tus_dugme(ui, tus, p, koyu, genis) {
                                            tus_eylemleri.push(match *tus {
                                                "⌫" => TusEylemi::Sil,
                                                "C" => TusEylemi::Temizle,
                                                c => TusEylemi::Karakter(c.chars().next().unwrap_or('0')),
                                            });
                                        }
                                    }
                                });
                            }
                            ui.add_space(5.0);
                            hairline(ui, p.kenar);

                            // ---- canlı sonuç ----
                            ui.label(RichText::new("SONUÇ").size(10.0).color(p.aksan).strong());
                            if let Some(r) = &sonuc {
                                egui::Frame::new()
                                    .fill(theme::widget_kart(koyu))
                                    .stroke(theme::widget_kenar(koyu))
                                    .corner_radius(CornerRadius::same(8))
                                    .inner_margin(Margin::symmetric(10, 6))
                                    .show(ui, |ui| {
                                        let v = para_bicimi(
                                            &r.ana,
                                            st.precision,
                                            &st.currency_symbol,
                                            st.show_currency_symbol,
                                        );
                                        ui.add(
                                            egui::Label::new(
                                                RichText::new(v)
                                                    .size(29.0)
                                                    .color(p.metin)
                                                    .strong()
                                                    .monospace(),
                                            )
                                            .wrap_mode(TextWrapMode::Truncate),
                                        );
                                        if !r.alt.is_empty() {
                                            let alt = para_bicimi(
                                                &r.alt,
                                                st.precision,
                                                &st.currency_symbol,
                                                st.show_currency_symbol,
                                            );
                                            ui.label(
                                                RichText::new(format!("Diğer: {alt}"))
                                                    .size(T_SMALL)
                                                    .color(p.ikinci)
                                                    .monospace(),
                                            );
                                        }
                                    });

                                // ---- hesap dökümü: ilk 3 satır ----
                                if !r.satirlar.is_empty() {
                                    ui.add_space(5.0);
                                    ui.label(
                                        RichText::new("HESAP DÖKÜMÜ")
                                            .size(10.0)
                                            .color(p.aksan)
                                            .strong(),
                                    );
                                    for (k, v) in r.satirlar.iter().take(3) {
                                        let genis = ui.available_width();
                                        let sol = genis * 0.52;
                                        let sag = (genis - sol - 6.0).max(60.0);
                                        ui.horizontal(|ui| {
                                            ui.allocate_ui(Vec2::new(sol, 20.0), |ui| {
                                                ui.add(
                                                    egui::Label::new(
                                                        RichText::new(k.clone())
                                                            .size(11.0)
                                                            .color(p.ikinci),
                                                    )
                                                    .wrap_mode(TextWrapMode::Truncate),
                                                );
                                            });
                                            ui.add_space(6.0);
                                            let d = satir_bicimi(
                                                k,
                                                v,
                                                st.precision,
                                                &st.currency_symbol,
                                                st.show_currency_symbol,
                                            );
                                            ui.allocate_ui(Vec2::new(sag, 20.0), |ui| {
                                                ui.with_layout(
                                                    Layout::right_to_left(Align::Center),
                                                    |ui| {
                                                        ui.add(
                                                            egui::Label::new(
                                                                RichText::new(d)
                                                                    .size(11.0)
                                                                    .color(p.metin)
                                                                    .monospace(),
                                                            )
                                                            .wrap_mode(TextWrapMode::Truncate),
                                                        );
                                                    },
                                                );
                                            });
                                        });
                                    }
                                }
                            } else {
                                ui.label(
                                    RichText::new("Hesaplanamadı — alanları kontrol edin")
                                        .size(T_SMALL)
                                        .color(p.hata),
                                );
                            }

                            // ---- hızlı eylemler ----
                            ui.add_space(6.0);
                            hairline(ui, p.kenar);
                            ui.horizontal_wrapped(|ui| {
                                if kopyalandi_yeni.is_some_and(|t| {
                                    t.elapsed() < std::time::Duration::from_secs(2)
                                }) {
                                    ui.label(
                                        RichText::new("✓").size(T_SMALL).color(p.basari).strong(),
                                    );
                                }
                                if ui.add(birincil_dugme("Kopyala", p)).clicked() {
                                    kopyala = true;
                                }
                                if ui.add(ikincil_dugme("Geçmişe kaydet", p)).clicked() {
                                    kaydet = true;
                                }
                                if ui.add(ikincil_dugme("Temizle", p)).clicked() {
                                    temizle = true;
                                }
                            });

                            // ---- son 3 geçmiş kaydı ----
                            if !gecmis.is_empty() {
                                ui.add_space(6.0);
                                ui.label(
                                    RichText::new("SON KAYITLAR")
                                        .size(10.0)
                                        .color(p.aksan)
                                        .strong(),
                                );
                                for (m, inputs, etiket, tarih) in &gecmis {
                                    let b = egui::Button::new(
                                        RichText::new(format!("{tarih} · {etiket}"))
                                            .size(11.0)
                                            .color(p.ikinci),
                                    )
                                    .fill(Color32::TRANSPARENT)
                                    .min_size(Vec2::new(
                                        ui.available_width().max(100.0) - 8.0,
                                        22.0,
                                    ));
                                    if ui.add(b).clicked() {
                                        geri_yukle = Some((m.clone(), inputs.clone()));
                                    }
                                }
                            }
                        }
                    });
            });

            // ---- sağ alt köşeden yeniden boyutlandırma tutamacı ----
            let grip = ui.allocate_rect(
                Rect::from_min_size(
                    ui.cursor().min + Vec2::new(ui.available_width() - 16.0, 0.0),
                    Vec2::new(16.0, 16.0),
                ),
                Sense::drag(),
            );
            if grip.drag_started() {
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::BeginResize(
                        egui::ResizeDirection::SouthEast,
                    ));
            }
            ui.painter().line_segment(
                [
                    grip.rect.right_bottom() - Vec2::new(2.0, 12.0),
                    grip.rect.right_bottom() - Vec2::new(12.0, 2.0),
                ],
                Stroke::new(2.0, p.soluk),
            );
        });

        // Tuş takımı basışlarını aktif metin alanına uygula. Alan sırası
        // çizimdeki `metin_alanlari` ile aynı; mod değişirse indeks sınırlanır.
        if !tus_eylemleri.is_empty() {
            if let Some(def) = modes::mod_def(&mod_id) {
                let metin_alanlari: Vec<_> = def
                    .alanlar
                    .iter()
                    .filter(|f| f.tur == FieldKind::Metin)
                    .take(2)
                    .collect();
                if !metin_alanlari.is_empty() {
                    let idx = self.widget_alan.min(metin_alanlari.len() - 1);
                    let alan = metin_alanlari[idx].alan;
                    if let Some(e) = self.state.inputs.get_mut(&mod_id) {
                        let s = e.get_mut(alan);
                        for tus in tus_eylemleri {
                            match tus {
                                TusEylemi::Karakter(c) => s.push(c),
                                TusEylemi::Sil => {
                                    s.pop();
                                }
                                TusEylemi::Temizle => s.clear(),
                            }
                        }
                    }
                }
            }
        }

        // ---- eylemler: doğrudan kalıcı duruma uygulanır ----
        if let Some(m) = yeni_mod {
            self.state.widget.r#mod = m.clone();
            self.widget_arama.clear();
            let e = self.state.inputs.entry(m).or_default();
            modes::varsayilanlari_doldur(e, &self.state.widget.r#mod);
            self.state.kaydet();
        }
        if kopyala {
            if let Some(r) = sonuc.as_ref() {
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    let _ = cb.set_text(sonuc_metni(&mod_id, r, &st));
                }
                self.widget_kopyalandi = Some(Instant::now());
            }
        }
        if kaydet {
            if let Some(r) = sonuc.as_ref() {
                let now = chrono::Local::now();
                let e = HistEntry {
                    id: format!(
                        "w{}",
                        now.timestamp_nanos_opt().unwrap_or(now.timestamp_millis())
                    ),
                    timestamp: now.timestamp(),
                    mode: mod_id.clone(),
                    inputs: girdi.clone(),
                    ana: r.ana.clone(),
                    alt: r.alt.clone(),
                    detay: r.detay.clone(),
                };
                gecmis_ekle(&mut self.state.history, e);
                self.state.kaydet();
                self.bildir("Widget sonucu geçmişe eklendi");
            }
        }
        if let Some((m, inputs)) = geri_yukle {
            self.state.widget.r#mod = m.clone();
            self.state.inputs.insert(m.clone(), inputs);
            self.mode = m;
            self.state.kaydet();
            self.bildir("Kayıt widget'a geri yüklendi");
        }
        if temizle {
            let e = self.state.inputs.entry(mod_id.clone()).or_default();
            *e = InputState::default();
            modes::varsayilanlari_doldur(e, &mod_id);
            self.state.kaydet();
            self.bildir("Widget girdileri temizlendi");
        }

        // Sürükleme sonrası boyutu kaydet (sürükleme sırasında yazma yapılmaz).
        if !boyut_sifirlandi {
            let sz = ui
                .ctx()
                .input(|i| i.viewport().inner_rect.map(|r| r.size()));
            if let Some(sz) = sz {
                let degisti = (sz.x - sw).abs() > 2.0 || (sz.y - sh).abs() > 2.0;
                if degisti {
                    let (gmin, gmax) = WIDGET_GENISLIK_ARALIGI;
                    let (ymin, ymax) = WIDGET_YUKSEKLIK_ARALIGI;
                    self.state.widget.genislik = sz.x.clamp(gmin, gmax);
                    self.state.widget.yukseklik = sz.y.clamp(ymin, ymax);
                    self.state.kaydet();
                }
            }
        }

        // Konum kalıcılığı: dış dikdörtgenin min köşesini kaydet. Komut
        // gönderilmediği ve repaint istenmediği için döngü oluşmaz.
        if !konum_uygulandi_bu_kare {
            let yeni_konum = ui
                .ctx()
                .input(|i| i.viewport().outer_rect.map(|r| [r.min.x, r.min.y]));
            if let Some(konum) = yeni_konum {
                if WidgetAyarlari::konum_gecerli(konum) {
                    let degisti = self.state.widget.konum.map_or(true, |onceki| {
                        (onceki[0] - konum[0]).abs() > 1.0
                            || (onceki[1] - konum[1]).abs() > 1.0
                    });
                    if degisti {
                        self.state.widget.konum = Some(konum);
                        self.state.kaydet();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boyut_varsayilanlari() {
        assert_eq!(etkili_boyut(true, 0.0, 0.0), COMPACT_SIZE);
        assert_eq!(etkili_boyut(false, 0.0, 0.0), WIDE_SIZE);
    }

    #[test]
    fn kaydedilmis_boyut_oncelikli() {
        assert_eq!(etkili_boyut(true, 300.0, 200.0), (300.0, 200.0));
        assert_eq!(etkili_boyut(false, 119.0, 200.0), WIDE_SIZE);
    }
}
