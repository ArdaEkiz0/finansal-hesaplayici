//! Bağımsız mini hesaplayıcı widget'ı (egui `Window` tabanlı).
//!
//! Tüm hesaplama `fh_core::hesapla` üzerinden yapılır; bu dosyada finansal
//! formül yoktur. Görünüm durumu `AppState::widget` (`WidgetAyarlari`) içinde
//! saklanır ve `state.kaydet()` ile kalıcılaşır.
//!
//! Kompakt görünüm tek satırlık sonuç kartı; geniş görünüm mod seçici, alan
//! girişleri, canlı sonuç, döküm, hızlı eylemler ve son kayıtlardan oluşur.

use std::time::Instant;

use eframe::egui::{
    self, Align, Align2, Color32, Context, CornerRadius, FontId, Layout, Margin, Order, RichText,
    Sense, Stroke, TextWrapMode, Ui, Vec2,
};
use fh_core::HesapSonucu;

use crate::commands::turkce_kucuk;
use crate::model::{
    gecmis_ekle, para_bicimi, satir_bicimi, HistEntry, InputState, Settings,
    WIDGET_GENISLIK_ARALIGI, WIDGET_YUKSEKLIK_ARALIGI,
};
use crate::modes::{self, MODLAR, FieldKind};
use crate::theme::{self, Palet};

use super::{birincil_dugme, hairline, ikincil_dugme, pencere_cercevesi, FhApp, T_BODY, T_SMALL};

// model.rs'teki doğrulama aralıklarıyla aynı sınırlar: dışarıda kalan değerler
// yedekten geri yüklenirken varsayılana düşer, burada da aynı kural uygulanır.
const COMPACT_SIZE: (f32, f32) = (220.0, 160.0);
const WIDE_SIZE: (f32, f32) = (340.0, 430.0);

/// Kaydedilmiş boyut geçerli aralıktaysa onu, değilse görünüme göre ölçüyü kullanır.
fn etkili_boyut(kompakt: bool, genislik: f32, yukseklik: f32) -> (f32, f32) {
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

/// Onaylı paletten ₺ rozeti (döndürülmüş yazı veya yeni grafik yok).
fn logo(ui: &mut Ui, p: Palet) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(22.0, 22.0), Sense::hover());
    ui.painter().circle_filled(r.center(), 10.0, p.birincil.gamma_multiply(0.22));
    ui.painter().circle_stroke(r.center(), 10.0, Stroke::new(1.0, p.aksan));
    ui.painter().text(
        r.center(),
        Align2::CENTER_CENTER,
        "₺",
        FontId::proportional(12.0),
        p.birincil,
    );
}

fn sonuc_metni(mod_id: &str, r: &HesapSonucu, st: &Settings) -> String {
    let ana = para_bicimi(&r.ana, st.precision, &st.currency_symbol, st.show_currency_symbol);
    let mut metin = format!("{}: {}", modes::mod_etiketi(mod_id), ana);
    if !r.alt.is_empty() {
        metin.push_str(&format!(
            "\nDiğer: {}",
            para_bicimi(&r.alt, st.precision, &st.currency_symbol, st.show_currency_symbol)
        ));
    }
    if !r.detay.is_empty() {
        metin.push_str(&format!("\n{}", r.detay));
    }
    for (k, v) in &r.satirlar {
        metin.push_str(&format!(
            "\n{k}: {}",
            satir_bicimi(k, v, st.precision, &st.currency_symbol, st.show_currency_symbol)
        ));
    }
    metin
}

/// Widget penceresini çizer. `pencereleri_goster` yalnızca `aktif` ise çağırır.
pub(crate) fn goster(app: &mut FhApp, ctx: &Context) {
    let koyu = theme::koyu_mu(&app.state.settings.theme);
    let p = theme::palet(koyu);
    let st: Settings = app.state.settings.clone();
    let mod_id = app.state.widget.r#mod.clone();
    let kompakt = app.state.widget.kompakt;
    let ustte = app.state.widget.ustte;
    let (sw, sh) = etkili_boyut(kompakt, app.state.widget.genislik, app.state.widget.yukseklik);
    let kopyalandi_yeni = app.widget_kopyalandi;
    let mut window_open = app.state.widget.aktif;

    // Widget'in kendi modu için girdi hazır olsun; sonuç anlık görüntüsü
    // `fh_core::hesapla` ile üretilir (formül kopyalanmaz).
    {
        let e = app.state.inputs.entry(mod_id.clone()).or_default();
        modes::varsayilanlari_doldur(e, &mod_id);
    }
    let girdi = app.state.inputs.get(&mod_id).cloned().unwrap_or_default();
    let sonuc: Option<HesapSonucu> = fh_core::hesapla(&mod_id, &modes::build_girdi(&mod_id, &girdi));

    let gecmis: Vec<(String, InputState, String, String)> = app
        .state
        .history
        .iter()
        .take(3)
        .map(|e| {
            let tarih = chrono::DateTime::from_timestamp(e.timestamp, 0)
                .map(|d| d.format("%d.%m %H:%M").to_string())
                .unwrap_or_default();
            let deger = para_bicimi(&e.ana, st.precision, &st.currency_symbol, st.show_currency_symbol);
            (
                e.mode.clone(),
                e.inputs.clone(),
                format!("{} · {}", modes::mod_etiketi(&e.mode), deger),
                tarih,
            )
        })
        .collect();

    // Başlıktaki özel kapat düğmesi ayrı bayrak kullanır: `window_open`
    // Window::open tarafından ödünç alınmıştır, closure içinde yazılamaz.
    let mut boyut_sifirlandi = false;
    let yanit = egui::Window::new("Mini Hesaplayıcı · FH")
        .open(&mut window_open)
        .resizable(true)
        .collapsible(false)
        .order(if ustte { Order::Foreground } else { Order::Middle })
        .min_size(if kompakt { Vec2::new(200.0, 160.0) } else { Vec2::new(280.0, 240.0) })
        .default_size(Vec2::new(sw, sh))
        .frame(pencere_cercevesi(p))
        .show(ctx, |ui| {
            let mut yeni_mod: Option<String> = None;
            let mut kopyala = false;
            let mut kaydet = false;
            let mut temizle = false;
            let mut geri_yukle: Option<(String, InputState)> = None;

            // ---- başlık: logo, mod adı, kompakt/geniş, üstte kal ----
            ui.horizontal(|ui| {
                logo(ui, p);
                ui.add_space(4.0);
                ui.label(RichText::new(modes::mod_etiketi(&mod_id)).size(T_BODY).color(p.metin).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Kapatma egui pencere başlığındaki tek × ile yapılır.
                    if ikon_dugme(ui, if ustte { "◉" } else { "◎" }, p, koyu, ustte, false) {
                        app.state.widget.ustte = !ustte;
                        app.state.kaydet();
                    }
                    if ikon_dugme(ui, if kompakt { "▣" } else { "▤" }, p, koyu, kompakt, false) {
                        let yeni = !kompakt;
                        app.state.widget.kompakt = yeni;
                        let (w, h) = if yeni { COMPACT_SIZE } else { WIDE_SIZE };
                        app.state.widget.genislik = w;
                        app.state.widget.yukseklik = h;
                        boyut_sifirlandi = true;
                        app.state.kaydet();
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
                                ui.label(RichText::new("AN TUTAR").size(10.0).color(p.aksan).strong());
                                let v = sonuc
                                    .as_ref()
                                    .map(|r| para_bicimi(&r.ana, st.precision, &st.currency_symbol, st.show_currency_symbol))
                                    .unwrap_or_else(|| "—".to_string());
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(v).size(25.0).color(p.metin).strong().monospace(),
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
                if kopyalandi_yeni.is_some_and(|t| t.elapsed() < std::time::Duration::from_secs(2)) {
                    ui.label(RichText::new("✓ Panoya kopyalandı").size(10.0).color(p.basari).strong());
                }
            } else {
                // ---- geniş: mod seçici (arama + dropdown) ----
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut app.widget_arama)
                            .hint_text("Mod ara")
                            .desired_width(84.0),
                    );
                    let etiket = modes::mod_etiketi(&mod_id).to_string();
                    egui::ComboBox::from_id_salt("widget-mod-secici")
                        .selected_text(etiket)
                        .width(148.0)
                        .show_ui(ui, |ui| {
                            let q = turkce_kucuk(&app.widget_arama);
                            let mut bulundu = 0usize;
                            for m in MODLAR {
                                if !q.is_empty() && !turkce_kucuk(m.etiket).contains(&q) {
                                    continue;
                                }
                                bulundu += 1;
                                if ui.selectable_label(m.id == mod_id, m.etiket).clicked() {
                                    yeni_mod = Some(m.id.to_string());
                                }
                            }
                            if bulundu == 0 {
                                ui.label(RichText::new("Eşleşme yok").size(T_SMALL).color(p.soluk));
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
                    for f in metin_alanlari {
                        ui.label(RichText::new(f.etiket).size(11.0).color(p.ikinci).strong());
                        if let Some(e) = app.state.inputs.get_mut(&mod_id) {
                            let t = egui::TextEdit::singleline(e.get_mut(f.alan))
                                .hint_text(f.ipucu)
                                .font(FontId::monospace(13.0));
                            ui.add_sized(Vec2::new(ui.available_width(), 28.0), t);
                        }
                        ui.add_space(4.0);
                    }
                    for f in def
                        .alanlar
                        .iter()
                        .filter(|f| matches!(f.tur, FieldKind::Secim | FieldKind::PaySecim))
                        .take(1)
                    {
                        let secili = app
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
                            ui.label(RichText::new(f.etiket).size(11.0).color(p.ikinci));
                            egui::ComboBox::from_id_salt(format!("widget-secim-{:?}", f.alan))
                                .selected_text(etiket)
                                .width(118.0)
                                .show_ui(ui, |ui| {
                                    for (label, value) in secenekler {
                                        if ui.selectable_label(secili == *value, *label).clicked() {
                                            if let Some(e) = app.state.inputs.get_mut(&mod_id) {
                                                *e.get_mut(f.alan) = value.to_string();
                                            }
                                        }
                                    }
                                });
                        });
                    }
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
                            let v = para_bicimi(&r.ana, st.precision, &st.currency_symbol, st.show_currency_symbol);
                            ui.add(
                                egui::Label::new(RichText::new(v).size(29.0).color(p.metin).strong().monospace())
                                    .wrap_mode(TextWrapMode::Truncate),
                            );
                            if !r.alt.is_empty() {
                                let alt = para_bicimi(&r.alt, st.precision, &st.currency_symbol, st.show_currency_symbol);
                                ui.label(RichText::new(format!("Diğer: {alt}")).size(T_SMALL).color(p.ikinci).monospace());
                            }
                        });

                    // ---- hesap dökümü: ilk 3 satır ----
                    if !r.satirlar.is_empty() {
                        ui.add_space(5.0);
                        ui.label(RichText::new("HESAP DÖKÜMÜ").size(10.0).color(p.aksan).strong());
                        for (k, v) in r.satirlar.iter().take(3) {
                            let genis = ui.available_width();
                            let sol = genis * 0.52;
                            let sag = (genis - sol - 6.0).max(60.0);
                            ui.horizontal(|ui| {
                                ui.allocate_ui(Vec2::new(sol, 20.0), |ui| {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(k.clone()).size(11.0).color(p.ikinci),
                                        )
                                        .wrap_mode(TextWrapMode::Truncate),
                                    );
                                });
                                ui.add_space(6.0);
                                let d = satir_bicimi(
                                    k, v, st.precision, &st.currency_symbol, st.show_currency_symbol,
                                );
                                ui.allocate_ui(Vec2::new(sag, 20.0), |ui| {
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        ui.add(
                                            egui::Label::new(
                                                RichText::new(d).size(11.0).color(p.metin).monospace(),
                                            )
                                            .wrap_mode(TextWrapMode::Truncate),
                                        );
                                    });
                                });
                            });
                        }
                    }
                } else {
                    ui.label(RichText::new("Hesaplanamadı — alanları kontrol edin").size(T_SMALL).color(p.hata));
                }

                // ---- hızlı eylemler ----
                ui.add_space(6.0);
                hairline(ui, p.kenar);
                ui.horizontal_wrapped(|ui| {
                    if kopyalandi_yeni.is_some_and(|t| t.elapsed() < std::time::Duration::from_secs(2)) {
                        ui.label(RichText::new("✓").size(T_SMALL).color(p.basari).strong());
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
                    ui.label(RichText::new("SON KAYITLAR").size(10.0).color(p.aksan).strong());
                    for (m, inputs, etiket, tarih) in &gecmis {
                        let b = egui::Button::new(
                            RichText::new(format!("{tarih} · {etiket}")).size(11.0).color(p.ikinci),
                        )
                        .fill(Color32::TRANSPARENT)
                        .min_size(Vec2::new(ui.available_width().max(100.0) - 8.0, 22.0));
                        if ui.add(b).clicked() {
                            geri_yukle = Some((m.clone(), inputs.clone()));
                        }
                    }
                }
            }

            // ---- eylemler: doğrudan kalıcı duruma uygulanır ----
            if let Some(m) = yeni_mod {
                app.state.widget.r#mod = m.clone();
                app.widget_arama.clear();
                let e = app.state.inputs.entry(m).or_default();
                modes::varsayilanlari_doldur(e, &app.state.widget.r#mod);
                app.state.kaydet();
            }
            if kopyala {
                if let Some(r) = sonuc.as_ref() {
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(sonuc_metni(&mod_id, r, &st));
                    }
                    app.widget_kopyalandi = Some(Instant::now());
                }
            }
            if kaydet {
                if let Some(r) = sonuc.as_ref() {
                    let now = chrono::Local::now();
                    let e = HistEntry {
                        id: format!("w{}", now.timestamp_nanos_opt().unwrap_or(now.timestamp_millis())),
                        timestamp: now.timestamp(),
                        mode: mod_id.clone(),
                        inputs: girdi.clone(),
                        ana: r.ana.clone(),
                        alt: r.alt.clone(),
                        detay: r.detay.clone(),
                    };
                    gecmis_ekle(&mut app.state.history, e);
                    app.state.kaydet();
                    app.bildir("Widget sonucu geçmişe eklendi");
                }
            }
            if let Some((m, inputs)) = geri_yukle {
                app.state.widget.r#mod = m.clone();
                app.state.inputs.insert(m.clone(), inputs);
                app.mode = m;
                app.state.kaydet();
                app.bildir("Kayıt widget'a geri yüklendi");
            }
            if temizle {
                let e = app.state.inputs.entry(mod_id.clone()).or_default();
                *e = InputState::default();
                modes::varsayilanlari_doldur(e, &mod_id);
                app.state.kaydet();
                app.bildir("Widget girdileri temizlendi");
            }
        });

    app.state.widget.aktif = window_open;

    // Sürükleme sonrası boyutu kaydet (sürükleme sırasında yazma yapılmaz).
    if !boyut_sifirlandi {
        if let Some(ir) = yanit {
            let sz = ir.response.rect.size();
            let degisti = (sz.x - sw).abs() > 2.0 || (sz.y - sh).abs() > 2.0;
            if degisti && !ir.response.dragged() {
                let (gmin, gmax) = WIDGET_GENISLIK_ARALIGI;
                let (ymin, ymax) = WIDGET_YUKSEKLIK_ARALIGI;
                app.state.widget.genislik = sz.x.clamp(gmin, gmax);
                app.state.widget.yukseklik = sz.y.clamp(ymin, ymax);
                app.state.kaydet();
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
