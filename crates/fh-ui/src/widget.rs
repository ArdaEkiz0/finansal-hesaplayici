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
    HistEntry, InputState, Settings, WIDGET_GENISLIK_ARALIGI, WIDGET_KONUM_ARALIGI,
    WIDGET_YUKSEKLIK_ARALIGI, WidgetAyarlari, gecmis_ekle, para_bicimi, satir_bicimi,
};
use crate::modes::{self, FieldKind, MODLAR};
use crate::theme::{self, Palet};

use super::{FhApp, T_BODY, T_SMALL, birincil_dugme, hairline, ikincil_dugme};

// model.rs'teki doğrulama aralıklarıyla aynı sınırlar: dışarıda kalan değerler
// yedekten geri yüklenirken varsayılana düşer, burada da aynı kural uygulanır.
const COMPACT_SIZE: (f32, f32) = (220.0, 160.0);
const WIDE_SIZE: (f32, f32) = (340.0, 430.0);

/// Manuel sürükleme için `egui::Context::data` içinde tutulan geçici durum.
/// `FhApp`'e alan eklemeden (app.rs'e dokunmadan) kareler arası taşınır.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct SurukleDurum {
    /// Sürükleme başladığı andaki `outer_rect.min`; sürükleme boyunca SABİT.
    /// Her kare yeniden okunmaz: gecikmeli viewport güncellemeleri referansı
    /// kaydırıp sürüklemeyi titretirdi.
    baslangic: [f32; 2],
    /// Başlangıçtan bu yana biriken imleç deltası (piksel).
    toplam_delta: [f32; 2],
}

/// Bir karenin imleç deltasını birikmiş deltaya ekler (saf).
fn delta_biriktir(toplam: [f32; 2], delta: Vec2) -> [f32; 2] {
    [toplam[0] + delta.x, toplam[1] + delta.y]
}

/// Sürüklemenin mutlak dış pencere konumunu üretir (saf yardımcı).
///
/// - `baslangic`: yalnızca `drag_started` anında alınan `outer_rect.min`.
/// - `delta`: bu karenin imleç deltası (`pointer.delta()`).
/// - `onceki_toplam`: o ana kadar biriken delta; `None` = ilk kare.
///
/// `ViewportCommand::StartDrag` yerine `OuterPosition` gönderildiği için konum
/// her kare `baslangic + toplam_delta` olarak yeniden hesaplanır; winit'in "sol
/// tuş hemen önce basılmış olmalı" garantisine gerek kalmaz. Sonuç geçerli
/// konum aralığına kısıtlanır: bozuk/büyük bir delta pencereyi ekran dışına
/// sürmez ve kaydedilen konum her zaman `konum_gecerli` ile uyumlu kalır.
fn konum_uygula(baslangic: [f32; 2], delta: Vec2, onceki_toplam: Option<[f32; 2]>) -> [f32; 2] {
    let (lo, hi) = WIDGET_KONUM_ARALIGI;
    let toplam = onceki_toplam.unwrap_or([0.0, 0.0]);
    let eksen = |bas: f32, top: f32, d: f32| {
        let v = bas + top + d;
        if v.is_finite() {
            v.clamp(lo, hi)
        } else {
            bas.clamp(lo, hi)
        }
    };
    [
        eksen(baslangic[0], toplam[0], delta.x),
        eksen(baslangic[1], toplam[1], delta.y),
    ]
}

/// Kompakt görünüm ALWAYS küçük kalır: kaydedilmiş büyük boyut "yoluma giriyor".
/// Geniş görünümde kullanıcı boyutu serbestçe ayarlayabilir.
pub(super) fn etkili_boyut(kompakt: bool, genislik: f32, yukseklik: f32) -> (f32, f32) {
    if kompakt {
        return COMPACT_SIZE;
    }
    let (gmin, gmax) = WIDGET_GENISLIK_ARALIGI;
    let (ymin, ymax) = WIDGET_YUKSEKLIK_ARALIGI;
    if genislik.is_finite()
        && yukseklik.is_finite()
        && (gmin..=gmax).contains(&genislik)
        && (ymin..=ymax).contains(&yukseklik)
    {
        (genislik, yukseklik)
    } else {
        WIDE_SIZE
    }
}

/// Küçük kare ikon düğmesi (kompakt/geniş, üstte kal, kapat, kopyala).
/// `ad` hem erişilebilir isim hem hover açıklamasıdır; dokunma alanı 44×44 px'e
/// çıkarılmıştır (eski 24×22 alan fareyle kolay kaçırılıyordu).
fn ikon_dugme(
    ui: &mut Ui,
    glif: &str,
    ad: &str,
    p: Palet,
    koyu: bool,
    aktif: bool,
    kapat: bool,
) -> bool {
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
        .min_size(Vec2::splat(44.0));
    ui.add(b).on_hover_text(ad).clicked()
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
/// Sürükleme alanının içinde, etkileşim yakalamadan çizilir: merkez doğrudan
/// verilir, `allocate_exact_size` ile cursor ilerletilmez.
fn logo(ui: &mut Ui, merkez: Pos2, p: Palet) {
    ui.painter()
        .circle_filled(merkez, 10.0, p.birincil.gamma_multiply(0.22));
    ui.painter()
        .circle_stroke(merkez, 10.0, Stroke::new(1.0, p.aksan));
    ui.painter().text(
        merkez,
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
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::OuterPosition(Pos2::new(
                            konum[0], konum[1],
                        )));
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
            self.kaydet();
        }

        // Widget'in kendi modu için girdi hazır olsun; sonuç anlık görüntüsü
        // `fh_core::hesapla` ile üretilir (formül kopyalanmaz).
        {
            let e = self.state.inputs.entry(mod_id.clone()).or_default();
            modes::varsayilanlari_doldur(e, &mod_id);
        }
        let girdi = self.state.inputs.get(&mod_id).cloned().unwrap_or_default();
        let hatalar = modes::parse_hatalari(&girdi);
        // Parse hatası varsa yanlış sonuç "canlı" gibi gösterilmesin.
        let sonuc: Option<HesapSonucu> = if hatalar.is_empty() {
            fh_core::hesapla(&mod_id, &modes::build_girdi(&mod_id, &girdi))
        } else {
            None
        };

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
        // Manuel sürükleme bu kare aktifse (veya bittiği kareyse) alttaki
        // outer_rect konum kaydı atlanır; konum zaten sürükleme kodu yazar.
        // Kimlik closure dışında da tutulur: imleç sabit kalan karelerde
        // oturumun sürdüğünü kare sonunda tekrar sorgulamak için.
        let surukle_id = egui::Id::NULL.with("fh-widget-manuel-surukle");
        let mut surukle_aktif = false;

        // Onaylı paletle çerçevesiz zemin dolgusu (ayrı viewport kimliği).
        theme::widget_viewport_frame(p).show(ui, |ui| {
            // ---- özel başlık: logo, mod adı, sürükleme alanı, üstte/kompakt/kapat ----
            ui.horizontal(|ui| {
                let baslangic_nokta = ui.cursor().min;
                let toplam_genislik = ui.available_width();

                // Sağdaki düğmeler için yer ayır; geri kalan tüm şerit
                // (logo + mod adı + boşluk) sürükleme alanıdır. Dokunma alanları
                // 44×44'e çıktığı için kompakt başlıkta 4 düğme ~182px yer kaplar;
                // 220px viewport'ta kalan ~26px logo + sürükleme içindir.
                // Pencere daraltıldığında sürükleme alanı 0'a kısılır; düğmelerin
                // üstüne binmez.
                let dugme_yeri = if kompakt { 182.0 } else { 214.0 };
                let bosluk = toplam_genislik - dugme_yeri;
                let surukle_genislik = bosluk.max(0.0);
                let surukle_rect =
                    Rect::from_min_size(baslangic_nokta, Vec2::new(surukle_genislik, 26.0));
                let surukle = ui.allocate_rect(surukle_rect, Sense::drag());

                // Logo ve mod adı sürükleme alanının üstüne çizilir; kendi
                // widget'ları olmadığı için tıklama sürüklemeye geçer.
                logo(
                    ui,
                    Pos2::new(surukle_rect.min.x + 11.0, surukle_rect.center().y),
                    p,
                );
                if !kompakt {
                    ui.painter().text(
                        Pos2::new(surukle_rect.min.x + 30.0, surukle_rect.center().y),
                        Align2::LEFT_CENTER,
                        modes::mod_etiketi(&mod_id),
                        FontId::proportional(T_BODY),
                        p.metin,
                    );
                }

                // Manuel sürükleme: StartDrag kaldırıldı. Başlangıç konumu bir
                // kez alınır; her kare `baslangic + birikmiş delta` mutlak
                // OuterPosition olarak gönderilir. Sürükleme olmayan karelerde
                // OuterPosition GÖNDERİLMEZ — aksi halde pencere aynı konuma
                // yapışır / titreme olur.
                let ctx = ui.ctx();

                if surukle.drag_started() {
                    let baslangic = ctx
                        .input(|i| i.viewport().outer_rect.map(|r| [r.min.x, r.min.y]))
                        .filter(|k| WidgetAyarlari::konum_gecerli(*k))
                        .or_else(|| {
                            self.state
                                .widget
                                .konum
                                .filter(|k| WidgetAyarlari::konum_gecerli(*k))
                        })
                        .unwrap_or([0.0, 0.0]);
                    ctx.data_mut(|d| {
                        d.insert_temp(
                            surukle_id,
                            SurukleDurum {
                                baslangic,
                                toplam_delta: [0.0, 0.0],
                            },
                        );
                    });
                    surukle_aktif = true;
                }
                if surukle.dragged() {
                    let durum = ctx
                        .data(|d| d.get_temp::<SurukleDurum>(surukle_id))
                        .unwrap_or_else(|| {
                            let baslangic = self
                                .state
                                .widget
                                .konum
                                .filter(|k| WidgetAyarlari::konum_gecerli(*k))
                                .unwrap_or([0.0, 0.0]);
                            SurukleDurum {
                                baslangic,
                                toplam_delta: [0.0, 0.0],
                            }
                        });
                    let delta = ctx.input(|i| i.pointer.delta());
                    let toplam_delta = delta_biriktir(durum.toplam_delta, delta);
                    let konum = konum_uygula(durum.baslangic, delta, Some(durum.toplam_delta));
                    ctx.data_mut(|d| {
                        d.insert_temp(
                            surukle_id,
                            SurukleDurum {
                                baslangic: durum.baslangic,
                                toplam_delta,
                            },
                        );
                    });
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(Pos2::new(
                        konum[0], konum[1],
                    )));
                    // Bellekte güncel tutulur; disk yazımı yalnızca drag bitiminde.
                    self.state.widget.konum = Some(konum);
                    surukle_aktif = true;
                }
                if surukle.drag_stopped() {
                    if let Some(durum) = ctx.data(|d| d.get_temp::<SurukleDurum>(surukle_id)) {
                        // Son konum bir kez hesaplanıp kalıcı duruma yazılır;
                        // sürükleme boyunca hiç disk yazımı yapılmadı.
                        let konum =
                            konum_uygula(durum.baslangic, Vec2::ZERO, Some(durum.toplam_delta));
                        self.state.widget.konum = Some(konum);
                        self.kaydet();
                    }
                    ctx.data_mut(|d| {
                        d.remove_temp::<SurukleDurum>(surukle_id);
                    });
                    surukle_aktif = true;
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = if kompakt { 2.0 } else { 4.0 };
                    // Kapat: tek düğme, kalıcı duruma yazar.
                    if ikon_dugme(ui, "×", "Widget'ı kapat", p, koyu, false, true) {
                        self.state.widget.aktif = false;
                        self.kaydet();
                    }
                    if ikon_dugme(
                        ui,
                        if ustte { "◉" } else { "◎" },
                        "Pencereyi her zaman üstte tut",
                        p,
                        koyu,
                        ustte,
                        false,
                    ) {
                        self.state.widget.ustte = !ustte;
                        self.kaydet();
                    }
                    if ikon_dugme(
                        ui,
                        if kompakt { "▣" } else { "▤" },
                        "Sadeleştir / genişlet",
                        p,
                        koyu,
                        kompakt,
                        false,
                    ) {
                        let yeni = !kompakt;
                        self.state.widget.kompakt = yeni;
                        let (w, h) = if yeni { COMPACT_SIZE } else { WIDE_SIZE };
                        self.state.widget.genislik = w;
                        self.state.widget.yukseklik = h;
                        boyut_sifirlandi = true;
                        self.kaydet();
                    }
                    // Ana pencereyi küçült; widget viewport'u görünür kalır.
                    // Komut ana ctx'e gönderilir (widget ctx'i değil).
                    // Kompakt başlıkta geniş metin düğmesi yerine küçük "W" ikonu:
                    // mod adıyla çakışmaz, pencereyi büyütmez.
                    if kompakt {
                        if ikon_dugme(ui, "W", "Ana pencereyi küçült", p, koyu, false, false) {
                            main_ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    } else {
                        let b = egui::Button::new(
                            RichText::new("Sadece widget")
                                .size(10.0)
                                .color(p.ikinci)
                                .strong(),
                        )
                        .fill(p.yuksek)
                        .stroke(theme::widget_kenar(koyu))
                        .corner_radius(CornerRadius::same(5))
                        .min_size(Vec2::new(70.0, 22.0));
                        if ui.add(b).clicked() {
                            main_ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
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
                        if !hatalar.is_empty() {
                            let metin = hatalar
                                .iter()
                                .map(|(_, m)| m.as_str())
                                .collect::<Vec<_>>()
                                .join(" · ");
                            ui.label(
                                RichText::new(format!("⚠ Girdi hatası: {metin}"))
                                    .size(T_SMALL)
                                    .color(p.hata)
                                    .strong(),
                            );
                            ui.add_space(4.0);
                        }
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
                                    if ikon_dugme(
                                        ui,
                                        "⧉",
                                        "Sonucu panoya kopyala",
                                        p,
                                        koyu,
                                        false,
                                        false,
                                    ) {
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
                                        let resp =
                                            ui.add_sized(Vec2::new(ui.available_width(), 28.0), t);
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
                                if alan_odakta
                                    && ui.ctx().input(|i| i.key_pressed(egui::Key::Enter))
                                {
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
                                RichText::new("TUŞ TAKIMI")
                                    .size(10.0)
                                    .color(p.aksan)
                                    .strong(),
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
                                                c => TusEylemi::Karakter(
                                                    c.chars().next().unwrap_or('0'),
                                                ),
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
            self.kaydet();
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
                self.kaydet();
                self.bildir("Widget sonucu geçmişe eklendi");
            }
        }
        if let Some((m, inputs)) = geri_yukle {
            self.state.widget.r#mod = m.clone();
            self.state.inputs.insert(m.clone(), inputs);
            self.mode = m;
            self.kaydet();
            self.bildir("Kayıt widget'a geri yüklendi");
        }
        if temizle {
            let e = self.state.inputs.entry(mod_id.clone()).or_default();
            *e = InputState::default();
            modes::varsayilanlari_doldur(e, &mod_id);
            self.kaydet();
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
                    self.kaydet();
                }
            }
        }

        // İmleç sabit kaldığı için `dragged()`'ın ateşlenmediği karelerde de
        // oturum açıksa konum kaydını atla: bir kare geride kalan outer_rect
        // sürükleme sonucunun üzerine yazmasın.
        if ui
            .ctx()
            .data(|d| d.get_temp::<SurukleDurum>(surukle_id).is_some())
        {
            surukle_aktif = true;
        }

        // Konum kalıcılığı: dış dikdörtgenin min köşesini kaydet. Komut
        // gönderilmediği ve repaint istenmediği için döngü oluşmaz. Manuel
        // sürükleme sırasında konum yukarıda yazılır; outer_rect bir kare
        // geride kalabileceğinden burada üzerine yazılmasın.
        if !konum_uygulandi_bu_kare && !surukle_aktif {
            let yeni_konum = ui
                .ctx()
                .input(|i| i.viewport().outer_rect.map(|r| [r.min.x, r.min.y]));
            if let Some(konum) = yeni_konum {
                if WidgetAyarlari::konum_gecerli(konum) {
                    let degisti = self.state.widget.konum.map_or(true, |onceki| {
                        (onceki[0] - konum[0]).abs() > 1.0 || (onceki[1] - konum[1]).abs() > 1.0
                    });
                    if degisti {
                        self.state.widget.konum = Some(konum);
                        self.kaydet();
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
        // Kompakt mod her zaman küçük kalır: eski kaydedilmiş büyük boyut
        // widget'in ekranı işgal etmesine yol açıyordu.
        assert_eq!(etkili_boyut(true, 300.0, 200.0), COMPACT_SIZE);
        assert_eq!(etkili_boyut(false, 119.0, 200.0), WIDE_SIZE);
        assert_eq!(etkili_boyut(false, 300.0, 200.0), (300.0, 200.0));
    }

    // ---- konum_uygula / delta_biriktir saf testleri (12) ----

    #[test]
    fn konum_ilk_kare_baslangici_kullanir() {
        assert_eq!(
            konum_uygula([100.0, 200.0], Vec2::new(10.0, -5.0), None),
            [110.0, 195.0]
        );
    }

    #[test]
    fn konum_pozitif_delta_saga_asagi_tasir() {
        assert_eq!(
            konum_uygula([100.0, 200.0], Vec2::new(15.0, 25.0), None),
            [115.0, 225.0]
        );
    }

    #[test]
    fn konum_negatif_delta_sola_yukari_tasir() {
        assert_eq!(
            konum_uygula([300.0, 300.0], Vec2::new(-25.0, -40.0), None),
            [275.0, 260.0]
        );
    }

    #[test]
    fn konum_birikmis_delta_baslangica_eklenir() {
        // Üçüncü argüman birikmiş deltadır; başlangıç asla yok sayılmaz.
        assert_eq!(
            konum_uygula([0.0, 0.0], Vec2::new(5.0, 5.0), Some([100.0, 100.0])),
            [105.0, 105.0]
        );
        assert_eq!(
            konum_uygula([50.0, 70.0], Vec2::new(5.0, -5.0), Some([100.0, 100.0])),
            [155.0, 165.0]
        );
    }

    #[test]
    fn konum_sifir_delta_yeni_toplami_degistirmez() {
        assert_eq!(konum_uygula([50.0, 60.0], Vec2::ZERO, None), [50.0, 60.0]);
        assert_eq!(
            konum_uygula([50.0, 60.0], Vec2::ZERO, Some([80.0, 90.0])),
            [130.0, 150.0]
        );
    }

    #[test]
    fn konum_cok_kare_birikimi_dogru() {
        let b = [100.0, 200.0];
        let mut toplam = [0.0, 0.0];
        let adimlar = [
            Vec2::new(10.0, 5.0),
            Vec2::new(4.0, -3.0),
            Vec2::new(-2.0, 8.0),
        ];
        let beklenen = [[110.0, 205.0], [114.0, 202.0], [112.0, 210.0]];
        for (delta, bek) in adimlar.iter().zip(beklenen.iter()) {
            let konum = konum_uygula(b, *delta, Some(toplam));
            assert_eq!(konum, *bek);
            toplam = delta_biriktir(toplam, *delta);
        }
        assert_eq!(toplam, [12.0, 10.0]);
    }

    #[test]
    fn konum_adimli_ve_tek_kare_esitligi() {
        let b = [10.0, 20.0];
        let t1 = delta_biriktir([0.0, 0.0], Vec2::new(12.0, -7.0));
        let adimli = konum_uygula(b, Vec2::new(-2.0, 3.0), Some(t1));
        let tek = konum_uygula(b, Vec2::new(10.0, -4.0), None);
        assert_eq!(adimli, tek);
        assert_eq!(adimli, [20.0, 16.0]);
    }

    #[test]
    fn konum_kaydedilmis_baslangictan_devam_eder() {
        // Yeni oturum önceki konumu başlangıç olarak alır; sıfır delta o konumu korur.
        let kayitli = [420.0, -120.0];
        assert_eq!(konum_uygula(kayitli, Vec2::ZERO, None), kayitli);
        assert_eq!(
            konum_uygula(kayitli, Vec2::new(3.0, -9.0), None),
            [423.0, -129.0]
        );
    }

    #[test]
    fn konum_ust_sinira_kisitlanir() {
        let (_, hi) = WIDGET_KONUM_ARALIGI;
        let b = [hi - 5.0, hi - 5.0];
        assert_eq!(konum_uygula(b, Vec2::new(50.0, 50.0), None), [hi, hi]);
        // Birikmiş delta ile birlikte de taşma kısıtlanır.
        assert_eq!(
            konum_uygula([0.0, 0.0], Vec2::new(5.0, 5.0), Some([hi, hi])),
            [hi, hi]
        );
    }

    #[test]
    fn konum_alt_sinira_kisitlanir() {
        let (lo, _) = WIDGET_KONUM_ARALIGI;
        let b = [lo + 5.0, lo + 5.0];
        assert_eq!(konum_uygula(b, Vec2::new(-50.0, -50.0), None), [lo, lo]);
        assert_eq!(
            konum_uygula([0.0, 0.0], Vec2::new(-5.0, -5.0), Some([lo, lo])),
            [lo, lo]
        );
    }

    #[test]
    fn konum_eksenler_bagimsiz_kisitlanir() {
        let (lo, hi) = WIDGET_KONUM_ARALIGI;
        assert_eq!(
            konum_uygula([hi - 1.0, 200.0], Vec2::new(10.0, -5.0), None),
            [hi, 195.0]
        );
        assert_eq!(
            konum_uygula([200.0, lo + 1.0], Vec2::new(-5.0, -10.0), None),
            [195.0, lo]
        );
    }

    #[test]
    fn delta_biriktir_saf_ve_negatif() {
        assert_eq!(
            delta_biriktir([0.0, 0.0], Vec2::new(3.0, -4.0)),
            [3.0, -4.0]
        );
        assert_eq!(
            delta_biriktir([10.0, 20.0], Vec2::new(-2.5, 0.5)),
            [7.5, 20.5]
        );
        assert_eq!(delta_biriktir([10.0, 20.0], Vec2::ZERO), [10.0, 20.0]);
    }
}
