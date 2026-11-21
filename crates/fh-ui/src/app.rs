//! eframe/egui masaüstü arayüzü. Tüm hesaplama fh-core, tüm veri dönüşümleri
//! fh-data üzerinden yapılır; bu dosyada finansal formül yoktur.
//!
//! Görsel düzen: premium finans konsolu.
//!   üstte kimlik + araç şeridi (kırpılmaz),
//!   solda 20 modun gruplu dikey navigasyonu,
//!   ortada geniş girdi çalışma alanı,
//!   sağda büyük sonuç paneli,
//!   en altta ince tema renginde durum şeridi (sürüm / çevrimdışı / veri / para).
//! Kayıt, dönem ve takvim panelleri üst şeritten açılır; alt kayıt paneli yoktur.
//! Dar ekranda (<1040) sonuç paneli çalışma alanının altına iner.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::Datelike;
use eframe::egui::{
    self, Align, Align2, Color32, Context, CornerRadius, FontId, Frame, Key, Layout, Margin, Rect,
    RichText, Shadow, Stroke, TextWrapMode, Ui, Vec2,
};
use fh_core::HesapSonucu;
use fh_data::{
    DonemRecord, compute_donem_net, csv_metin, efatura_matrahlar, gun_kaldi, takvim_listesi,
    yillara_gir,
};

use crate::commands::{self, Eylem, Komut};
use crate::kararlar::{
    GuncellemeKarari, WidgetEtki, ag_istegi_yapilir, guncelleme_karari, kisayol_kararlari,
    palet_secim_duzelt, widget_etki,
};
use crate::model::{
    AppState, Cari, DonemKayit, HistEntry, InputState, SURUM, WIDGET_GENISLIK_ARALIGI,
    WIDGET_YUKSEKLIK_ARALIGI, para_bicimi, satir_bicimi,
};
use crate::modes::{self, MODLAR, ModeDef};
use crate::theme::{self, Palet};

/// Bağımsız mini hesaplayıcı; görünüm durumu `AppState::widget` içinde kalıcıdır.
#[path = "widget.rs"]
mod widget;

// Tipografi ölçeği — tek yerden yönetilir, yazı ölçeği zoom ile katlanır.
// Hiçbir metin 12px altına inmez; gövde ve tablo satırları ≥14px okunur.
const T_BUILD: f32 = 12.0;
const T_SMALL: f32 = 13.5;
const T_ROW: f32 = 14.5;
const T_BODY: f32 = 15.5;
const T_LEAD: f32 = 18.5;
const T_TITLE: f32 = 23.0;
const T_AMOUNT: f32 = 58.0;

/// Mod navigasyonunda bir satırın yaklaşık yüksekliği (buton 40 + aralık 3 + pay).
/// `ScrollArea::vertical_scroll_offset` hesabında kullanılır; test için `pub(super)`.
pub(super) const SATIR_YUKSEKLIGI: f32 = 46.0;
/// Bir karede görünür kabul edilen mod satırı sayısı (↓ tuşu sınırı ve gösterge).
pub(super) const GORUNUR_SATIR: usize = 8;

/// Portable sürümde otomatik kurulum desteklenmez; genel "ağ veya doğrulama"
/// mesajı kullanıcıyı yanıltıyordu.
const PORTABLE_GUNCELLEME_MESAJI: &str =
    "Portable sürüm otomatik güncellenemez — lütfen kurulum exe'sini indirin.";

/// Komut paleti arama alanının sabit kimliği: odak isteği/bırakma bu kimliğe yapılır.
const PALET_ARAMA_ID: &str = "fh-palet-arama";

/// `ilindeks` numaralı mod satırını listenin tepesine getiren dikey ofset (px).
#[allow(dead_code)] // eski ScrollArea ofseti — saf testlerde korunur
pub(super) fn mod_ofseti(ilindeks: usize, satir_yuksekligi: f32) -> f32 {
    ilindeks as f32 * satir_yuksekligi
}

/// İlk görünür mod indeksini `[0, toplam - gorunur]` aralığına sınırlar.
#[allow(dead_code)] // sayfalama kaldırıldı; saf testlerde korunur
pub(super) fn mod_indeksi_sinirla(idx: usize, toplam: usize, gorunur: usize) -> usize {
    idx.min(toplam.saturating_sub(gorunur))
}

#[derive(Default)]
pub struct Pencereler {
    pub gecmis: bool,
    pub ayarlar: bool,
    pub takvim: bool,
    pub cariler: bool,
    pub donemler: bool,
    pub palet: bool,
    pub onizleme: bool,
}

pub struct FhApp {
    pub state: AppState,
    pub mode: String,
    pub result: Option<HesapSonucu>,
    pub p: Pencereler,
    pub komutlar: Vec<Komut>,
    pub palet_sorgu: String,
    pub palet_sec: usize,
    pub yeni_cari: String,
    pub cari_filtre: String,
    pub onizleme_baslik: String,
    pub onizleme_metin: String,
    pub toast: Option<(String, Instant)>,
    pub kopyalandi: Option<Instant>,
    /// Widget mod seçicisindeki arama metni (kalıcı state'e ait değil).
    pub widget_arama: String,
    /// Widget kopyalama bildiriminin zamanı.
    pub widget_kopyalandi: Option<Instant>,
    /// Sayısal tuş takımının hedefi: 0 = ilk, 1 = ikinci metin alanı.
    pub widget_alan: usize,
    /// Kaydedilen widget konumu bu açılışta bir kez uygulandı mı?
    pub widget_konum_uygulandi: bool,
    pub son_tema: String,
    pub son_olcek: f64,
    /// Ayarlar penceresinde açık sekme: 0 görünüm, 1 widget, 2 veri, 3 hakkında.
    pub ayarlar_sekmesi: usize,
    /// Güncelleme indirmesi sürüyor (yükleniyor durumu).
    pub indiriliyor: bool,
    /// Arka planda kontrol edilen güncelleme (kilitli paylaşımlı durum).
    pub guncelleme: Arc<Mutex<Option<fh_updater::Guncelleme>>>,
    /// `indir_ve_kur` sonucu: Some(true) başarı, Some(false) başarısız.
    pub indirme_durumu: Arc<Mutex<Option<bool>>>,
    pub guncelleme_bildirildi: bool,
    pub indirme_bildirildi: bool,
    /// Palet açılınca arama alanına odak isteği (bir sonraki karede uygulanır).
    pub palet_odak_iste: bool,
    /// Son filtrelemede kullanılan sorgu; değişirse seçim sıfırlanır.
    pub palet_son_sorgu: String,
    /// Geçmiş temizlenince geri getirmek için saklanan kopya (Undo).
    pub gecmis_undo: Option<Vec<HistEntry>>,
    /// Bildirimin görünme süresi (sn); Undo bildirimi daha uzun sürer.
    pub toast_sure: u64,
}

// ---------- küçük çizim yardımcıları ----------

/// Hairline ayırıcı çizer ve imleci 1px ilerletir.
fn hairline(ui: &mut Ui, renk: Color32) {
    let r = ui.available_rect_before_wrap();
    ui.painter().rect_filled(
        Rect::from_min_size(r.min, Vec2::new(r.width(), 1.0)),
        CornerRadius::ZERO,
        renk,
    );
    ui.add_space(1.0);
}

/// Tablo hücresi: sabit genişlik, tek satır, isteğe bağlı sabit genişlikli rakam.
fn hucre(ui: &mut Ui, genislik: f32, metin: String, renk: Color32, mono: bool, sag: bool) {
    ui.allocate_ui(Vec2::new(genislik, 30.0), |ui| {
        let yerlesim = if sag {
            Layout::right_to_left(Align::Center)
        } else {
            Layout::left_to_right(Align::Center)
        };
        ui.with_layout(yerlesim, |ui| {
            let mut z = RichText::new(metin).size(T_ROW).color(renk);
            if mono {
                z = z.monospace();
            }
            ui.add(egui::Label::new(z).wrap_mode(TextWrapMode::Truncate));
        });
    });
}

fn dugme_cercevesi(p: Palet) -> Frame {
    Frame::new()
        .fill(p.yuksek)
        .stroke(Stroke::new(1.0, p.kenar))
        .corner_radius(CornerRadius::same(6))
}

fn pencere_cercevesi(p: Palet) -> Frame {
    Frame::new()
        .fill(p.yuzey)
        .stroke(Stroke::new(1.0, p.kenar))
        .corner_radius(CornerRadius::same(10))
        .shadow(Shadow {
            offset: [0, 14],
            blur: 36,
            spread: 0,
            color: Color32::from_black_alpha(140),
        })
}

fn birincil_dugme<'a>(ad: &'a str, p: Palet) -> egui::Button<'a> {
    egui::Button::new(
        RichText::new(ad)
            .size(T_BODY)
            .color(Color32::WHITE)
            .strong(),
    )
    .fill(p.birincil)
    .stroke(Stroke::new(1.0, p.birincil))
    .corner_radius(CornerRadius::same(6))
    .min_size(Vec2::new(74.0, 32.0))
}

fn ikincil_dugme<'a>(ad: &'a str, p: Palet) -> egui::Button<'a> {
    egui::Button::new(RichText::new(ad).size(T_BODY).color(p.ikinci).strong())
        .fill(p.yuksek)
        .stroke(Stroke::new(1.0, p.kenar))
        .corner_radius(CornerRadius::same(6))
        .min_size(Vec2::new(64.0, 34.0))
}

/// Üst şeridin sağ ucundaki pencere kontrolü: alta alma, büyüt/küçült, kapat.
/// Hover/focus/devre-dışı durumları tema widget stillerinden gelir; kapat
/// glifi hata rengiyle diğer iki kontrolden ayrılır.
fn pencere_kontrol_dugmesi(ui: &mut Ui, glif: &str, ad: &str, p: Palet, kapat: bool) -> bool {
    let b = egui::Button::new(
        RichText::new(glif)
            .size(if kapat { 18.0 } else { 16.0 })
            .color(if kapat { p.hata } else { p.ikinci })
            .strong(),
    )
    .fill(p.yuksek)
    .stroke(Stroke::new(1.0, p.kenar))
    .corner_radius(CornerRadius::same(5))
    // Erişilebilirlik: dokunma alanı en az 44×44 px; glif boyutu aynı kalır.
    .min_size(Vec2::splat(44.0));
    ui.add(b).on_hover_text(ad).clicked()
}

impl FhApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = AppState::yukle();
        let son_tema = state.settings.theme.clone();
        let son_olcek = state.settings.font_scale;
        theme::uygula(&cc.egui_ctx, &son_tema, son_olcek, true);

        // Ağ kontrolü yalnızca kullanıcı izin verdiyse yapılır. Uygulama
        // "çevrimdışı · veriler dışarı çıkmaz" diyor; opt-out kapalıyken hiçbir
        // ağ isteği başlatılmaz (ağ varsa bile).
        let guncelleme: Arc<Mutex<Option<fh_updater::Guncelleme>>> = Arc::new(Mutex::new(None));
        if ag_istegi_yapilir(state.settings.guncelleme_kontrol) {
            let paylasilan = Arc::clone(&guncelleme);
            std::thread::spawn(move || {
                if let Some(gunc) = fh_updater::kontrol(env!("CARGO_PKG_VERSION")) {
                    if let Ok(mut kilit) = paylasilan.lock() {
                        *kilit = Some(gunc);
                    }
                }
            });
        }

        let mut app = Self {
            state,
            mode: "kdv".to_string(),
            result: None,
            p: Pencereler::default(),
            komutlar: commands::komutlar(),
            palet_sorgu: String::new(),
            palet_sec: 0,
            yeni_cari: String::new(),
            cari_filtre: String::new(),
            onizleme_baslik: String::new(),
            onizleme_metin: String::new(),
            toast: None,
            kopyalandi: None,
            widget_arama: String::new(),
            widget_kopyalandi: None,
            widget_alan: 0,
            widget_konum_uygulandi: false,
            son_tema,
            son_olcek,
            ayarlar_sekmesi: 0,
            indiriliyor: false,
            guncelleme,
            indirme_durumu: Arc::new(Mutex::new(None)),
            guncelleme_bildirildi: false,
            indirme_bildirildi: false,
            palet_odak_iste: false,
            palet_son_sorgu: String::new(),
            gecmis_undo: None,
            toast_sure: 3,
        };
        for m in MODLAR {
            let g = app.state.inputs.entry(m.id.to_string()).or_default();
            modes::varsayilanlari_doldur(g, m.id);
        }
        app
    }

    fn bildir(&mut self, metin: impl Into<String>) {
        self.toast_sure = 3;
        self.toast = Some((metin.into(), Instant::now()));
    }

    /// Undo seçeneği sunulan bildirimler daha uzun görünür.
    fn bildir_sureli(&mut self, metin: impl Into<String>, sure: u64) {
        self.toast_sure = sure;
        self.toast = Some((metin.into(), Instant::now()));
    }

    /// Kalıcı kaydı dener; hatayı yutmaz, kullanıcıya bildirir.
    fn kaydet(&mut self) {
        if let Err(e) = self.state.kaydet() {
            self.bildir(format!("Kaydedilemedi: {e}"));
        }
    }

    /// Geçmişi temizler ve geri alma kopyasını saklar. Üç çağrı yolu da
    /// (komut, geçmiş penceresi, ayarlar) aynı korumayı kullanır.
    fn gecmis_temizle(&mut self) {
        if self.state.history.is_empty() {
            self.bildir("Geçmiş zaten boş");
            return;
        }
        self.gecmis_undo = Some(self.state.history.clone());
        self.state.history.clear();
        self.kaydet();
        self.bildir_sureli("Geçmiş temizlendi — 'Geri al' ile getirebilirsiniz", 12);
    }

    fn gecmis_undo_yap(&mut self) {
        if let Some(h) = self.gecmis_undo.take() {
            self.state.history = h;
            self.kaydet();
            self.bildir("Geçmiş geri getirildi");
        }
    }

    /// Arka plan thread'lerinin sonuçlarını arayüz bildirimine çevirir.
    /// Kurulum başarıyla hazırlandıysa uygulamayı kapatır; geri kalan işi
    /// `fh-updater`ın yazdığı köprü dosyası yapar (sessiz kurulum + yeniden
    /// başlatma). Windows'ta çalışan exe'nin üzerine yazılamadığı için
    /// kapanış zorunludur.
    fn guncelleme_durumunu_kontrol(&mut self, ctx: &Context) {
        let yeni_surum = self.guncelleme.lock().map(|k| k.is_some()).unwrap_or(false);
        let indirme = self.indirme_durumu.lock().ok().and_then(|k| *k);
        let karar = guncelleme_karari(
            yeni_surum,
            self.guncelleme_bildirildi,
            indirme,
            self.indirme_bildirildi,
        );
        match karar {
            GuncellemeKarari::YeniSurum => {
                self.guncelleme_bildirildi = true;
                self.bildir("Yeni sürüm mevcut — üst panelden 'İndir ve kur' yapabilirsiniz");
            }
            GuncellemeKarari::IndirmeBasarili => {
                self.indirme_bildirildi = true;
                self.indiriliyor = false;
                self.bildir("Güncelleme kuruluyor; uygulama yeniden başlatılacak…");
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            GuncellemeKarari::IndirmeBasarisiz => {
                self.indirme_bildirildi = true;
                self.indiriliyor = false;
                // fh-updater portable modda indir_ve_kur'dan false döner;
                // genel ağ mesajı yerine kesin nedeni söyle.
                let mesaj = if fh_updater::portable_mi() {
                    PORTABLE_GUNCELLEME_MESAJI
                } else {
                    "Güncelleme indirilemedi (ağ veya doğrulama)"
                };
                self.bildir(mesaj);
            }
            GuncellemeKarari::Yok => {}
        }
    }

    /// "Güncelle" düğmesi: indirmeyi/doğrulamayı arka planda başlatır.
    fn guncelleme_baslat(&mut self) {
        if !ag_istegi_yapilir(self.state.settings.guncelleme_kontrol) {
            self.bildir("Güncelleme kontrolü kapalı — ağ isteği yapılmaz");
            return;
        }
        if fh_updater::portable_mi() {
            self.bildir(PORTABLE_GUNCELLEME_MESAJI);
            return;
        }
        let Some(g) = self.guncelleme.lock().ok().and_then(|k| k.clone()) else {
            return;
        };
        self.indirme_bildirildi = false;
        self.indiriliyor = true;
        if let Ok(mut kilit) = self.indirme_durumu.lock() {
            *kilit = None;
        }
        let durum = Arc::clone(&self.indirme_durumu);
        std::thread::spawn(move || {
            let basari = fh_updater::indir_ve_kur(&g);
            if let Ok(mut kilit) = durum.lock() {
                *kilit = Some(basari);
            }
        });
        self.bildir("Güncelleme indiriliyor…");
    }

    /// Panoya kopyalama ve yazdırma önizlemesinin paylaştığı düz metin özeti.
    fn sonuc_metni(&self) -> String {
        let st = &self.state.settings;
        let Some(r) = &self.result else {
            return "Geçerli bir sonuç yok.".to_string();
        };
        let ana = para_bicimi(
            &r.ana,
            st.precision,
            &st.currency_symbol,
            st.show_currency_symbol,
        );
        let mut metin = format!("{}: {}", modes::mod_etiketi(&self.mode), ana);
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
        metin.push_str(&format!("\n{}", r.detay));
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

    fn yazdir(&mut self) {
        self.onizleme_baslik = "Yazdırma Önizlemesi".to_string();
        self.onizleme_metin = self.sonuc_metni();
        self.p.onizleme = true;
    }

    fn girdi(&mut self) -> &mut InputState {
        let mode = self.mode.clone();
        let e = self.state.inputs.entry(mode.clone()).or_default();
        modes::varsayilanlari_doldur(e, &mode);
        e
    }

    fn hesapla(&mut self) {
        let mode = self.mode.clone();
        let girdi = {
            let e = self.state.inputs.entry(mode.clone()).or_default();
            modes::varsayilanlari_doldur(e, &mode);
            e.hatalar = modes::parse_hatalari(e);
            modes::build_girdi(&mode, e)
        };
        // Parse hatası varsa tamsayı alanlar 0'a düşmüş olabilir; böyle bir
        // sonucu "Canlı hesaplama" olarak göstermek yanlış finansal güven verir.
        let hata_var = self
            .state
            .inputs
            .get(&mode)
            .is_some_and(|e| !e.hatalar.is_empty());
        self.result = if hata_var {
            None
        } else {
            fh_core::hesapla(&mode, &girdi)
        };
    }

    /// Kullanıcı modun en az bir gerçek (seçim olmayan) alanını doldurdu mu?
    fn girdi_dolu(&self) -> bool {
        let Some(def) = modes::mod_def(&self.mode) else {
            return false;
        };
        let Some(g) = self.state.inputs.get(&self.mode) else {
            return false;
        };
        def.alanlar.iter().any(|f| {
            matches!(f.tur, modes::FieldKind::Metin | modes::FieldKind::CokSatir)
                && !g.get(f.alan).trim().is_empty()
        })
    }

    fn tema_guncelle(&mut self, ctx: &Context) {
        let s = &self.state.settings;
        if self.son_tema != s.theme || (self.son_olcek - s.font_scale).abs() > f64::EPSILON {
            theme::uygula(ctx, &s.theme, s.font_scale, false);
            self.son_tema = s.theme.clone();
            self.son_olcek = s.font_scale;
        }
    }

    fn font_degistir(&mut self, delta: f64) {
        let v = (self.state.settings.font_scale + delta).clamp(0.85, 1.25);
        self.state.settings.font_scale = (v * 100.0).round() / 100.0;
        self.kaydet();
    }

    fn filtreli_komutlar(&self) -> Vec<usize> {
        commands::filtrele(&self.komutlar, &self.palet_sorgu)
    }

    fn palet_arama_odagi_birak(&self, ctx: &Context) {
        ctx.memory_mut(|m| m.surrender_focus(egui::Id::unique(PALET_ARAMA_ID)));
    }

    fn kisayollar(&mut self, ctx: &Context) {
        let (ctrl_k, ctrl_shift_w, esc, enter, up, down) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::K) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::W) && i.modifiers.ctrl && i.modifiers.shift,
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Enter),
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
            )
        });
        let karar = kisayol_kararlari(ctrl_k, ctrl_shift_w, esc);
        if karar.palet_degistir {
            self.p.palet = !self.p.palet;
            if self.p.palet {
                self.palet_sorgu.clear();
                self.palet_son_sorgu.clear();
                self.palet_sec = 0;
                self.palet_odak_iste = true;
            } else {
                self.palet_arama_odagi_birak(ctx);
            }
        }
        // Ctrl+Shift+W: mini hesaplayıcıyı aç/kapat (kalıcı widget durumu).
        if karar.widget_degistir {
            self.state.widget.aktif = !self.state.widget.aktif;
            self.kaydet();
        }
        if karar.pencereleri_kapat {
            self.p.gecmis = false;
            self.p.ayarlar = false;
            self.p.takvim = false;
            self.p.cariler = false;
            self.p.donemler = false;
            self.p.onizleme = false;
            self.p.palet = false;
            self.palet_arama_odagi_birak(ctx);
            if self.state.widget.aktif {
                self.state.widget.aktif = false;
                self.kaydet();
            }
        }
        if self.p.palet {
            // Sorgu değişince eski seçim geçersizdir; 0'a döner. Aynı sorguda
            // liste küçüldüyse aralığa sınırlanır (yanlış komut çalışması engellenir).
            let n = self.filtreli_komutlar().len();
            let degisti = self.palet_sorgu != self.palet_son_sorgu;
            self.palet_sec = palet_secim_duzelt(self.palet_sec, n, degisti);
            self.palet_son_sorgu = self.palet_sorgu.clone();
            if up && n > 0 {
                self.palet_sec = (self.palet_sec + n - 1) % n;
            }
            if down && n > 0 {
                self.palet_sec = (self.palet_sec + 1) % n;
            }
            if enter && n > 0 {
                let idxs = self.filtreli_komutlar();
                if let Some(&i) = idxs.get(self.palet_sec) {
                    let eylem = self.komutlar[i].eylem.clone();
                    self.komut_calistir(eylem);
                    self.p.palet = false;
                    self.palet_arama_odagi_birak(ctx);
                }
            }
        }
    }

    fn komut_calistir(&mut self, eylem: Eylem) {
        match eylem {
            Eylem::Mod(id) => {
                self.mode = id.to_string();
                let e = self.girdi();
                modes::varsayilanlari_doldur(e, id);
            }
            Eylem::Tema(t) => {
                self.state.settings.theme = t.to_string();
                self.kaydet();
            }
            Eylem::FontBuyut => self.font_degistir(0.05),
            Eylem::FontKucult => self.font_degistir(-0.05),
            Eylem::YedekAl => self.yedek_al(),
            Eylem::YedekYukle => self.yedek_yukle(),
            Eylem::GecmisCsv => self.gecmis_csv(),
            Eylem::EfaturaIce => self.efatura_ice(),
            Eylem::Takvim => self.p.takvim = true,
            Eylem::Gecmis => self.p.gecmis = true,
            Eylem::GecmisTemizle => self.gecmis_temizle(),
            Eylem::Ayarlar => self.p.ayarlar = true,
            Eylem::Cariler => self.p.cariler = true,
            Eylem::Donemler => self.p.donemler = true,
            e @ (Eylem::WidgetAcKapat | Eylem::WidgetSadeleştir | Eylem::WidgetÜsteSabitle) => {
                match widget_etki(&e).expect("widget komutu") {
                    WidgetEtki::Aktiflik => self.state.widget.aktif = !self.state.widget.aktif,
                    WidgetEtki::Sadeleştir => {
                        self.state.widget.kompakt = !self.state.widget.kompakt
                    }
                    WidgetEtki::Ustte => self.state.widget.ustte = !self.state.widget.ustte,
                }
                self.kaydet();
            }
            Eylem::Yazdir => self.yazdir(),
        }
    }

    fn yedek_al(&mut self) {
        let veri = self.state.yedek_degeri();
        let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_file_name("finansal-hesaplayici-yedek.json")
            .save_file()
        else {
            return;
        };
        let sonuc = serde_json::to_string_pretty(&veri)
            .map_err(|e| e.to_string())
            .and_then(|s| std::fs::write(path, s).map_err(|e| e.to_string()));
        match sonuc {
            Ok(_) => self.bildir("Yedek dosyası oluşturuldu"),
            Err(e) => self.bildir(format!("Yedek alınamadı: {e}")),
        }
    }

    fn yedek_yukle(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file()
        else {
            return;
        };
        let metin = match std::fs::read_to_string(&path) {
            Ok(m) => m,
            Err(_) => {
                self.bildir("Dosya okunamadı");
                return;
            }
        };
        let value = match serde_json::from_str::<serde_json::Value>(&metin) {
            Ok(v) => v,
            Err(_) => {
                self.bildir("Geçersiz JSON");
                return;
            }
        };
        match AppState::from_backup(&value) {
            Some(s) => {
                let mut yeni = s;
                for m in MODLAR {
                    let e = yeni.inputs.entry(m.id.to_string()).or_default();
                    modes::varsayilanlari_doldur(e, m.id);
                }
                self.state = yeni;
                self.kaydet();
                self.bildir("Yedek yüklendi");
            }
            None => self.bildir("Bu dosya geçerli bir yedek değil"),
        }
    }

    fn gecmis_csv(&mut self) {
        if self.state.history.is_empty() {
            self.bildir("Aktarılacak geçmiş kaydı yok");
            return;
        }
        let mut satirlar = vec![vec![
            "Tarih".to_string(),
            "Mod".to_string(),
            "Sonuç".to_string(),
            "Detay".to_string(),
        ]];
        for e in &self.state.history {
            let tarih = chrono::DateTime::from_timestamp(e.timestamp, 0)
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_default();
            satirlar.push(vec![
                tarih,
                modes::mod_etiketi(&e.mode).to_string(),
                e.ana.clone(),
                e.detay.clone(),
            ]);
        }
        self.csv_yaz(
            csv_metin(&satirlar),
            "gecmis.csv",
            "Geçmiş CSV dışa aktarıldı",
        );
    }

    fn csv_yaz(&mut self, metin: String, ad: &str, basari: &str) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .set_file_name(ad)
            .save_file()
        else {
            return;
        };
        match std::fs::write(path, metin) {
            Ok(_) => self.bildir(basari),
            Err(_) => self.bildir("CSV yazılamadı"),
        }
    }

    fn efatura_ice(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv", "txt"])
            .pick_file()
        else {
            return;
        };
        let Ok(icerik) = std::fs::read_to_string(&path) else {
            self.bildir("Dosya okunamadı");
            return;
        };
        let matrahlar = efatura_matrahlar(&icerik);
        if matrahlar.is_empty() {
            self.bildir("Matrah bulunamadı");
            return;
        }
        self.mode = "batch".to_string();
        let e = self.girdi();
        e.ek1 = matrahlar.join("\n");
        e.secim = "kdv".to_string();
        if e.oran.is_empty() {
            e.oran = "20".to_string();
        }
        self.kaydet();
        self.bildir(format!("{} matrah içe aktarıldı", matrahlar.len()));
    }

    fn gecmis_kaydet(&mut self) {
        let Some(r) = self.result.clone() else {
            self.bildir("Kaydedilecek geçerli sonuç yok");
            return;
        };
        let now = chrono::Local::now();
        let e = HistEntry {
            id: format!(
                "h{}",
                now.timestamp_nanos_opt().unwrap_or(now.timestamp_millis())
            ),
            timestamp: now.timestamp(),
            mode: self.mode.clone(),
            inputs: self.girdi().clone(),
            ana: r.ana,
            alt: r.alt,
            detay: r.detay,
        };
        crate::model::gecmis_ekle(&mut self.state.history, e);
        self.kaydet();
        self.bildir("Sonuç geçmişe eklendi");
    }

    // ---------- üst şerit ----------

    fn arac_dugmesi(&self, ui: &mut Ui, ad: &str, aktif: bool, p: Palet) -> egui::Response {
        let b = egui::Button::new(
            RichText::new(ad)
                .size(T_SMALL)
                .color(if aktif { Color32::WHITE } else { p.ikinci })
                .strong(),
        )
        .fill(if aktif { p.birincil } else { p.yuksek })
        .stroke(Stroke::new(1.0, if aktif { p.birincil } else { p.kenar }))
        .corner_radius(CornerRadius::same(6))
        .min_size(Vec2::new(48.0, 32.0));
        ui.add(b)
    }

    fn ust_panel(&mut self, ui: &mut Ui) {
        let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));
        let guncelleme = self.guncelleme.lock().ok().and_then(|k| k.clone());
        let genis = ui.ctx().content_rect().width();
        let dar = genis < 1040.0;
        // Araç etiketleri, sağ uçtaki pencere kontrollerine yer açmak için
        // ana düzenden daha erken kısalır; düğmeler hiçbir genişlikte kırpılmaz.
        let dar_ust = genis < 1180.0;
        let (l_k, l_ap, l_am, l_w, l_ay, l_tk, l_cr, l_dn, l_gc) = if dar_ust {
            ("⌘K", "A+", "A−", "W", "Ayar", "Tak", "Cari", "Dön", "Geç")
        } else {
            (
                "⌘K",
                "A+",
                "A−",
                "Widget",
                "Ayarlar",
                "Takvim",
                "Cari",
                "Dönemler",
                "Geçmiş",
            )
        };

        ui.horizontal(|ui| {
            // Başlık sabit alana alınır: araç düğmeleri taşsa da üstüne binmez.
            let baslik_genis = if dar { 74.0 } else { 330.0 };
            ui.allocate_ui_with_layout(
                Vec2::new(baslik_genis, 42.0),
                egui::Layout::left_to_right(Align::Center),
                |ui| {
                    ui.label(RichText::new("◈").size(30.0).color(p.birincil).strong());
                    ui.add_space(3.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(if dar { "FH" } else { "FİNANSAL HESAPLAYICI" })
                                .size(if dar { T_TITLE - 4.0 } else { T_TITLE - 1.0 })
                                .color(p.metin)
                                .strong(),
                        );
                        ui.label(
                            RichText::new(if dar {
                                format!("v{SURUM}")
                            } else {
                                format!("v{SURUM}  ·  çevrimdışı finans konsolu")
                            })
                            .size(T_BUILD)
                            .color(p.soluk)
                            .monospace(),
                        );
                    });
                },
            );

            ui.add_space(10.0);

            // Başlıktaki boş alan pencereyi sürükler (OS çubuğu kapalı).
            let surukle_genis = ui.available_width().min(140.0).max(12.0);
            let surukle = ui.allocate_rect(
                Rect::from_min_size(ui.cursor().min, Vec2::new(surukle_genis, 36.0)),
                egui::Sense::drag(),
            );
            if surukle.drag_started() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            // Tek satır, sıralı yerleştirme: dar ekranda kısa etiketlere düşer,
            // hiçbir düğme kırpılmaz.
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Pencere kontrolleri ÖNCE eklenir: right_to_left'te ilk eklenen
                // sağ uçta kalır, taşan araç düğmeleri onu iter/çıkarmaz.
                let maksimize = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                if pencere_kontrol_dugmesi(ui, "×", "Pencereyi kapat", p, true) {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                let buyut_glif = if maksimize { "❐" } else { "□" };
                let buyut_ad = if maksimize {
                    "Pencereyi küçült"
                } else {
                    "Pencereyi büyüt"
                };
                if pencere_kontrol_dugmesi(ui, buyut_glif, buyut_ad, p, false) {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Maximized(!maksimize));
                }
                if pencere_kontrol_dugmesi(ui, "─", "Pencereyi simge durumuna küçült", p, false)
                {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                }

                // Araç düğmeleri ters sırayla: görünür sıra ⌘K A+ A− … Geçmiş.
                if self.arac_dugmesi(ui, l_gc, self.p.gecmis, p).clicked() {
                    self.p.gecmis = !self.p.gecmis;
                }
                if self.arac_dugmesi(ui, l_dn, self.p.donemler, p).clicked() {
                    self.p.donemler = !self.p.donemler;
                }
                if self.arac_dugmesi(ui, l_cr, self.p.cariler, p).clicked() {
                    self.p.cariler = !self.p.cariler;
                }
                if self.arac_dugmesi(ui, l_tk, self.p.takvim, p).clicked() {
                    self.p.takvim = !self.p.takvim;
                }
                if self.arac_dugmesi(ui, l_ay, self.p.ayarlar, p).clicked() {
                    self.p.ayarlar = !self.p.ayarlar;
                }
                if self
                    .arac_dugmesi(ui, l_w, self.state.widget.aktif, p)
                    .clicked()
                {
                    self.state.widget.aktif = !self.state.widget.aktif;
                    self.kaydet();
                }
                if self.arac_dugmesi(ui, l_am, false, p).clicked() {
                    self.font_degistir(-0.05);
                }
                if self.arac_dugmesi(ui, l_ap, false, p).clicked() {
                    self.font_degistir(0.05);
                }
                if self.arac_dugmesi(ui, l_k, self.p.palet, p).clicked() {
                    self.p.palet = !self.p.palet;
                    self.palet_sorgu.clear();
                    self.palet_son_sorgu.clear();
                    self.palet_sec = 0;
                    if self.p.palet {
                        self.palet_odak_iste = true;
                    } else {
                        self.palet_arama_odagi_birak(ui.ctx());
                    }
                }
            });
        });

        ui.add_space(6.0);
        hairline(ui, p.kenar);

        if let Some(g) = &guncelleme {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("● Yeni sürüm v{} mevcut", g.version))
                        .size(T_SMALL)
                        .color(p.uyari)
                        .strong(),
                );
                if self.indiriliyor {
                    ui.add_enabled(false, egui::Button::new("İndiriliyor…"));
                } else if ui.add(birincil_dugme("İndir ve kur", p)).clicked() {
                    self.guncelleme_baslat();
                }
            });
        }
    }

    // ---------- sol: dikey mod navigasyonu ----------

    /// Sol raydaki tek mod satırı. Aktif satırda cyan şerit + mavi zemin görünür.
    fn mod_satiri(&mut self, ui: &mut Ui, m: &ModeDef, p: Palet) -> egui::Response {
        let aktif = self.mode == m.id;
        let mut b = egui::Button::new(
            RichText::new(m.etiket)
                .size(T_BODY)
                .color(if aktif { p.metin } else { p.ikinci })
                .strong(),
        )
        .corner_radius(CornerRadius::same(6))
        .min_size(Vec2::new(ui.available_width().max(120.0), 40.0));
        if aktif {
            b = b
                .fill(p.birincil.gamma_multiply(0.16))
                .stroke(Stroke::new(1.0, p.birincil.gamma_multiply(0.55)));
        }
        let resp = ui.add(b);
        if aktif {
            let r = resp.rect;
            ui.painter().rect_filled(
                Rect::from_min_size(r.min, Vec2::new(3.0, r.height())),
                CornerRadius::ZERO,
                p.aksan,
            );
        }
        if resp.clicked() {
            self.mode = m.id.to_string();
            let e = self.girdi();
            modes::varsayilanlari_doldur(e, m.id);
        }
        resp
    }

    fn mod_nav_panel(&mut self, ui: &mut Ui, panel_yukseklik: f32) {
        let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));

        // Klavye: ↑/↓ yalnızca başka bir widget (özellikle metin alanı) odaktayken
        // müdahale etme — ok tuşları o widget'ın olmalı.
        let (klavye_yukari, klavye_asagi) = if ui.memory(|m| m.focused()).is_none() {
            ui.input(|i| (i.key_pressed(Key::ArrowUp), i.key_pressed(Key::ArrowDown)))
        } else {
            (false, false)
        };
        let mut yukari = klavye_yukari;
        let mut asagi = klavye_asagi;

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("MODÜLLER")
                    .size(T_SMALL)
                    .color(p.soluk)
                    .strong(),
            );
            ui.add_space(4.0);
            if ui.button("↑").clicked() {
                yukari = true;
            }
            if ui.button("↓").clicked() {
                asagi = true;
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{} mod", MODLAR.len()))
                        .size(T_BUILD)
                        .color(p.aksan)
                        .monospace(),
                );
            });
        });
        ui.add_space(4.0);
        hairline(ui, p.kenar);
        ui.add_space(6.0);

        // Tek sayfalık liste: 20 modun tamamı tek ScrollArea içinde. Kaydırmayı
        // egui yönetir — fare tekerleği ve kaydırma çubuğu doğal çalışır.
        // ↑/↓ düğmeleri ve klavye ok tuşları yalnızca ScrollArea içinden
        // `scroll_with_delta` ile ofseti değiştirir. `scroll_to_me` her kare
        // çağrıldığında listeyi başa çekiyor, zorlanan `vertical_scroll_offset`
        // ise egui'nin iç state'i tarafından geri alınıyordu; ikisi de yok.
        //
        // egui 0.37'nin kinetic kaydırma fiziği (decay_time / rubber_band)
        // ofseti her kare biraz geri çekiyor, fare tekerleğiyle kaydırma
        // "tutmuyordu". Bu panelde kinetic kaydırmayı kapatıyoruz: ofset
        // anında ve kalıcı uygulanır.
        let kaydirma_y = (panel_yukseklik - 74.0).max(modes::MOD_NAV_MIN_HEIGHT);
        let adim = GORUNUR_SATIR as f32 * SATIR_YUKSEKLIGI * 0.5;

        ui.scope(|ui| {
            ui.style_mut().spacing.scroll.kinetic.decay_time = 0.0;
            ui.style_mut().spacing.scroll.kinetic.rubber_band = false;

            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .max_height(kaydirma_y)
                .animated(false)
                .show(ui, |ui| {
                    // egui'de negatif delta içeriği yukarı taşır = aşağı kaydırır
                    // (bkz. Ui::scroll_with_delta dokümanı: `scroll_delta.y -= 64.0`).
                    if yukari {
                        ui.scroll_with_delta(Vec2::new(0.0, adim));
                    }
                    if asagi {
                        ui.scroll_with_delta(Vec2::new(0.0, -adim));
                    }

                    for grup in modes::gruplar() {
                        let sayi = MODLAR.iter().filter(|m| m.grup == grup).count();
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(grup.to_uppercase())
                                    .size(T_BUILD)
                                    .color(p.aksan)
                                    .strong(),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    RichText::new(sayi.to_string())
                                        .size(T_BUILD)
                                        .color(p.soluk)
                                        .monospace(),
                                );
                            });
                        });
                        ui.add_space(2.0);
                        for m in MODLAR.iter().filter(|m| m.grup == grup) {
                            let _ = self.mod_satiri(ui, m, p);
                            ui.add_space(3.0);
                        }
                    }
                    ui.add_space(10.0);
                    hairline(ui, p.kenar);
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new("Tüm hesaplamalar yerel ve çevrimdışıdır.")
                            .size(T_BUILD)
                            .color(p.soluk),
                    );
                });
        });
    }

    // ---------- merkez: girdi çalışma alanı ----------

    fn girdi_panel(&mut self, ui: &mut Ui) {
        let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));
        let mode = self.mode.clone();
        let def = modes::mod_def(&mode);
        let mut temizle = false;

        ui.horizontal(|ui| {
            ui.label(
                RichText::new("ÇALIŞMA ALANI  /  GİRDİ")
                    .size(T_SMALL)
                    .color(p.soluk)
                    .strong(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(ikincil_dugme("Temizle", p)).clicked() {
                    temizle = true;
                }
            });
        });
        if temizle {
            self.state
                .inputs
                .insert(mode.clone(), InputState::default());
            let e = self.girdi();
            modes::varsayilanlari_doldur(e, &mode);
        }
        ui.add_space(4.0);
        ui.label(
            RichText::new(modes::mod_etiketi(&mode))
                .size(T_TITLE + 2.0)
                .color(p.metin)
                .strong(),
        );
        ui.add_space(5.0);

        // Durum çizgisi: kırmızı girdi hatası / yeşil doğrulandı / soluk hazır.
        // Parse hatası varsa result zaten None yapıldı; yine de hata bayrağı
        // önce kontrol edilir ki yanlış sonuç hiçbir zaman doğrulanmış görünmesin.
        let hatalar = self
            .state
            .inputs
            .get(&mode)
            .map(|e| e.hatalar.clone())
            .unwrap_or_default();
        let (durum_rengi, durum_metni) = if !hatalar.is_empty() {
            (p.hata, "● Girdi hatası")
        } else if self.result.is_some() {
            (p.basari, "● Canlı hesaplama")
        } else if self.girdi_dolu() {
            (p.hata, "● Girdi bekleniyor")
        } else {
            (p.soluk, "● Hazır")
        };
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(durum_metni)
                    .size(T_SMALL)
                    .color(durum_rengi)
                    .strong(),
            );
            if let Some(d) = def {
                ui.label(
                    RichText::new(format!("· {} alan", d.alanlar.len()))
                        .size(T_SMALL)
                        .color(p.soluk),
                );
            }
        });
        if !hatalar.is_empty() {
            ui.add_space(6.0);
            egui::Frame::new()
                .fill(p.hata.gamma_multiply(0.14))
                .stroke(Stroke::new(1.0, p.hata.gamma_multiply(0.6)))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(10, 8))
                .show(ui, |ui| {
                    for (alan, mesaj) in &hatalar {
                        let etiket = def
                            .and_then(|d| d.alanlar.iter().find(|f| f.alan == *alan))
                            .map(|f| f.etiket)
                            .unwrap_or("Alan");
                        ui.label(
                            RichText::new(format!("⚠ {etiket} — {mesaj}"))
                                .size(T_SMALL)
                                .color(p.hata)
                                .strong(),
                        );
                    }
                });
        }

        ui.add_space(10.0);
        hairline(ui, p.kenar);
        ui.add_space(12.0);

        let Some(def) = def else {
            return;
        };
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            for (n, f) in def.alanlar.iter().enumerate() {
                if n > 0 {
                    ui.add_space(14.0);
                }
                ui.label(RichText::new(f.etiket).size(T_SMALL).color(p.ikinci).strong());
                ui.add_space(6.0);
                let inp = self.state.inputs.entry(mode.clone()).or_default();
                match f.tur {
                    crate::modes::FieldKind::Metin => {
                        let t = egui::TextEdit::singleline(inp.get_mut(f.alan))
                            .hint_text(f.ipucu)
                            .font(FontId::monospace(T_BODY));
                        ui.add_sized(Vec2::new(ui.available_width(), 44.0), t);
                    }
                    crate::modes::FieldKind::CokSatir => {
                        let t = egui::TextEdit::multiline(inp.get_mut(f.alan))
                            .hint_text(f.ipucu)
                            .font(FontId::monospace(T_BODY))
                            .desired_width(ui.available_width())
                            .desired_rows(6);
                        ui.add(t);
                    }
                    crate::modes::FieldKind::Secim | crate::modes::FieldKind::PaySecim => {
                        let secili = inp.get(f.alan).to_string();
                        let secenekler = if f.tur == crate::modes::FieldKind::PaySecim {
                            &crate::modes::SEC_PAY[..]
                        } else {
                            f.secenekler
                        };
                        let etiket = secenekler
                            .iter()
                            .find(|(_, v)| *v == secili)
                            .map(|(l, _)| *l)
                            .or_else(|| secenekler.first().map(|(l, _)| *l))
                            .unwrap_or("—");
                        egui::ComboBox::from_id_salt(format!("{mode}-{:?}", f.alan))
                            .selected_text(etiket)
                            .width(ui.available_width().max(180.0))
                            .show_ui(ui, |ui| {
                                for (label, value) in secenekler {
                                    let v = value.to_string();
                                    if ui.selectable_label(secili == v, *label).clicked() {
                                        *inp.get_mut(f.alan) = v;
                                    }
                                }
                            });
                    }
                }
            }

            ui.add_space(16.0);
            hairline(ui, p.kenar);
            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                if ui.add(ikincil_dugme("Yedek al", p)).clicked() {
                    self.yedek_al();
                }
                if ui.add(ikincil_dugme("Yedek yükle", p)).clicked() {
                    self.yedek_yukle();
                }
                if mode == "batch" && ui.add(ikincil_dugme("e-Fatura CSV", p)).clicked() {
                    self.efatura_ice();
                }
                if mode == "donem" && ui.add(ikincil_dugme("Dönemi arşive kaydet", p)).clicked() {
                    self.donemi_kaydet();
                }
            });

            // Boş kalan alanı gerçek içerikle doldur: çalışma bağlamı.
            ui.add_space(18.0);
            dugme_cercevesi(p).show(ui, |ui| {
                ui.label(RichText::new("ÇALIŞMA NOTU").size(T_BUILD).color(p.aksan).strong());
                ui.add_space(5.0);
                ui.label(
                    RichText::new(format!(
                        "{} için {} alan tanımlı. Tutarları 1.234,56 biçiminde girin; sonuç sağ panelde anında güncellenir.",
                        modes::mod_etiketi(&mode),
                        def.alanlar.len()
                    ))
                    .size(T_SMALL)
                    .color(p.ikinci),
                );
                ui.add_space(7.0);
                ui.label(
                    RichText::new("Ctrl+K komut paleti  ·  Esc pencereleri kapatır  ·  A− / A+ yazı ölçeği")
                        .size(T_BUILD)
                        .color(p.soluk),
                );
            });
        });
    }

    // ---------- sağ: büyük sonuç paneli ----------

    fn sonuc_panel(&mut self, ui: &mut Ui) {
        let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));
        let st = self.state.settings.clone();
        let res = self.result.clone();
        let bos = !self.girdi_dolu();

        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("SONUÇ").size(T_SMALL).color(p.soluk).strong());
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    egui::Frame::new()
                        .fill(p.birincil.gamma_multiply(0.16))
                        .corner_radius(CornerRadius::same(5))
                        .inner_margin(Margin::symmetric(9, 4))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(modes::mod_etiketi(&self.mode))
                                    .size(T_SMALL)
                                    .color(p.birincil)
                                    .strong(),
                            );
                        });
                });
            });
            ui.add_space(6.0);
            hairline(ui, p.kenar);
            ui.add_space(16.0);

            if let Some(r) = &res {
                ui.label(RichText::new("ANA TUTAR").size(T_BUILD).color(p.aksan).strong());
                ui.add_space(4.0);
                let ana = para_bicimi(&r.ana, st.precision, &st.currency_symbol, st.show_currency_symbol);
                let genis = ui.available_width();
                let ana_boyut = if genis < 320.0 {
                    34.0
                } else if genis < 460.0 {
                    46.0
                } else {
                    T_AMOUNT
                };
                ui.add(
                    egui::Label::new(
                        RichText::new(ana).size(ana_boyut).color(p.metin).strong().monospace(),
                    )
                    .wrap_mode(TextWrapMode::Truncate),
                );
                ui.add_space(6.0);
                let serit = ui.available_rect_before_wrap();
                ui.painter().rect_filled(
                    Rect::from_min_size(serit.min, Vec2::new(serit.width().min(160.0), 4.0)),
                    CornerRadius::ZERO,
                    p.aksan,
                );
                ui.add_space(10.0);
                if !r.alt.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Diğer sonuç").size(T_SMALL).color(p.soluk));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let alt = para_bicimi(&r.alt, st.precision, &st.currency_symbol, st.show_currency_symbol);
                            ui.label(RichText::new(alt).size(T_LEAD).color(p.ikinci).monospace());
                        });
                    });
                }
            } else if bos {
                ui.label(RichText::new("—").size(T_AMOUNT).color(p.soluk).monospace());
                ui.add_space(8.0);
                ui.label(RichText::new("Sonuç bekleniyor").size(T_LEAD).color(p.ikinci).strong());
                ui.add_space(5.0);
                ui.label(
                    RichText::new("Soldaki çalışma alanına bir tutar yazın; döküm anında burada görünür.")
                        .size(T_BODY)
                        .color(p.soluk),
                );
            } else {
                ui.label(RichText::new("Hesaplanamadı").size(T_LEAD).color(p.hata).strong());
                ui.add_space(5.0);
                ui.label(
                    RichText::new("Tutar ve oran alanlarını kontrol edin; sayısal değerler 1.234,56 biçiminde olmalı.")
                        .size(T_BODY)
                        .color(p.ikinci),
                );
            }

            ui.add_space(22.0);
            hairline(ui, p.kenar);
            ui.add_space(14.0);

            ui.label(RichText::new("HESAP DÖKÜMÜ").size(T_BUILD).color(p.aksan).strong());
            ui.add_space(8.0);
            if let Some(r) = &res {
                if r.satirlar.is_empty() {
                    ui.label(RichText::new("Bu hesabın ek satırı yok.").size(T_BODY).color(p.soluk));
                } else {
                    let genis = ui.available_width();
                    let sol = genis * 0.56;
                    let sag = genis - sol - 10.0;
                    for (etiket, deger) in &r.satirlar {
                        hairline(ui, p.kenar.gamma_multiply(0.6));
                        ui.horizontal(|ui| {
                            hucre(ui, sol, etiket.clone(), p.ikinci, false, false);
                            ui.add_space(10.0);
                            let d = satir_bicimi(
                                etiket, deger, st.precision, &st.currency_symbol, st.show_currency_symbol,
                            );
                            hucre(ui, sag, d, p.metin, true, true);
                        });
                    }
                    hairline(ui, p.kenar.gamma_multiply(0.6));
                }
                ui.add_space(14.0);
                if !r.detay.is_empty() {
                    dugme_cercevesi(p).show(ui, |ui| {
                        ui.label(RichText::new(r.detay.as_str()).size(T_BODY).color(p.ikinci));
                    });
                }
            } else {
                ui.label(
                    RichText::new("Döküm, hesaplama yapıldığında burada listelenir.")
                        .size(T_BODY)
                        .color(p.soluk),
                );
            }

            ui.add_space(20.0);
            hairline(ui, p.kenar);
            ui.add_space(12.0);
            ui.horizontal_wrapped(|ui| {
                if self.kopyalandi.is_some_and(|t| t.elapsed() < Duration::from_secs(2)) {
                    ui.label(RichText::new("✓ Panoya kopyalandı").size(T_SMALL).color(p.basari).strong());
                    ui.add_space(6.0);
                }
                let hazir = res.is_some();
                if ui.add_enabled(hazir, birincil_dugme("Panoya kopyala", p)).clicked() {
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(self.sonuc_metni());
                    }
                    self.kopyalandi = Some(Instant::now());
                }
                if ui.add_enabled(hazir, ikincil_dugme("Yazdır", p)).clicked() {
                    self.yazdir();
                }
                if ui.add_enabled(hazir, ikincil_dugme("Geçmişe kaydet", p)).clicked() {
                    self.gecmis_kaydet();
                }
            });

            // Kalan alanı doğrulama/bağlam şeridiyle doldur.
            ui.add_space(18.0);
            hairline(ui, p.kenar);
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("✓ Yerel hesaplama").size(T_SMALL).color(p.basari).strong());
                ui.add_space(10.0);
                ui.label(RichText::new("✓ Anlık güncelleme").size(T_SMALL).color(p.basari).strong());
            });
            ui.add_space(6.0);
            ui.label(
                RichText::new(format!(
                    "Para birimi: {}  ·  Hassasiyet: {} hane  ·  Tema: {}",
                    if st.show_currency_symbol { st.currency_symbol.as_str() } else { "—" },
                    st.precision,
                    if theme::koyu_mu(&st.theme) { "koyu" } else { "açık" }
                ))
                .size(T_BUILD)
                .color(p.soluk)
                .monospace(),
            );
        });
    }

    // ---------- pencereler ----------

    fn donemi_kaydet(&mut self) {
        let mode = self.mode.clone();
        let e = self.girdi();
        let (satis, alis, oran, devreden) = (
            e.birinci.clone(),
            e.ikinci.clone(),
            e.oran.clone(),
            e.ek1.clone(),
        );
        let Some((hesaplanan, indirilecek, net, odenecek)) =
            compute_donem_net(&satis, &alis, &devreden, &oran)
        else {
            self.bildir("Dönem hesaplanamadı — alanları kontrol edin");
            return;
        };
        let now = chrono::Local::now();
        let id = format!("{:04}-{:02}", now.year(), now.month());
        let kayit = DonemKayit {
            id: id.clone(),
            year: now.year(),
            month: now.month() as i32,
            satis,
            alis,
            devreden_giris: devreden,
            hesaplanan,
            indirilecek,
            net,
            odenecek,
            kdv_rate: oran,
            cari: self.state.active_cari.clone(),
            closed_at: now.timestamp(),
        };
        self.state.donemler.retain(|d| d.id != id);
        self.state.donemler.insert(0, kayit);
        self.kaydet();
        self.bildir(format!("{id} dönemi arşive kaydedildi"));
        let _ = mode;
    }

    fn pencereleri_goster(&mut self, ctx: &Context) {
        if self.p.gecmis {
            self.gecmis_penceresi(ctx);
        }
        if self.p.ayarlar {
            self.ayarlar_penceresi(ctx);
        }
        if self.p.takvim {
            self.takvim_penceresi(ctx);
        }
        if self.p.cariler {
            self.cariler_penceresi(ctx);
        }
        if self.p.donemler {
            self.donemler_penceresi(ctx);
        }
        if self.p.palet {
            self.palet_penceresi(ctx);
        }
        if self.p.onizleme {
            self.onizleme_penceresi(ctx);
        }
        let toast = self.toast.clone();
        if let Some((metin, at)) = toast {
            if at.elapsed() < Duration::from_secs(self.toast_sure) {
                let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));
                let undo_var = self.gecmis_undo.is_some();
                let mut undo_tiklandi = false;
                egui::Window::new("Bildirim")
                    .title_bar(false)
                    .resizable(false)
                    .collapsible(false)
                    .frame(pencere_cercevesi(p))
                    .anchor(Align2::RIGHT_BOTTOM, Vec2::new(-16.0, -16.0))
                    .show(ctx, |ui| {
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(metin).size(T_BODY));
                            if undo_var && ui.add(ikincil_dugme("Geri al", p)).clicked() {
                                undo_tiklandi = true;
                            }
                        });
                        ui.add_space(2.0);
                    });
                if undo_tiklandi {
                    self.gecmis_undo_yap();
                }
            }
        }
    }

    fn gecmis_penceresi(&mut self, ctx: &Context) {
        let st = self.state.settings.clone();
        let p = theme::palet(theme::koyu_mu(&st.theme));
        let secilecek = std::cell::RefCell::new(None);
        let mut open = self.p.gecmis;
        egui::Window::new("Geçmiş · son 100 kayıt")
            .open(&mut open)
            .frame(pencere_cercevesi(p))
            .default_size((540.0, 460.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.add(ikincil_dugme("CSV indir", p)).clicked() {
                        self.gecmis_csv();
                    }
                    if ui.add(ikincil_dugme("Geçmişi temizle", p)).clicked() {
                        self.gecmis_temizle();
                    }
                    ui.label(
                        RichText::new(format!("{} kayıt", self.state.history.len()))
                            .size(T_SMALL)
                            .color(p.soluk),
                    );
                });
                ui.add_space(6.0);
                hairline(ui, p.kenar);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if self.state.history.is_empty() {
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new(
                                "Henüz kayıt yok. Sonuç biletinden 'Geçmişe kaydet' ile ekleyin.",
                            )
                            .size(T_SMALL)
                            .color(p.soluk),
                        );
                    }
                    for e in &self.state.history {
                        let tarih = chrono::DateTime::from_timestamp(e.timestamp, 0)
                            .map(|d| d.format("%d.%m %H:%M").to_string())
                            .unwrap_or_default();
                        let deger = para_bicimi(
                            &e.ana,
                            st.precision,
                            &st.currency_symbol,
                            st.show_currency_symbol,
                        );
                        let b = egui::Button::new(
                            RichText::new(format!(
                                "{tarih}  ·  {}  ·  {deger}",
                                modes::mod_etiketi(&e.mode)
                            ))
                            .size(T_SMALL)
                            .color(p.ikinci),
                        )
                        .fill(Color32::TRANSPARENT)
                        .min_size(Vec2::new(ui.available_width().max(100.0) - 10.0, 24.0));
                        if ui.add(b).clicked() {
                            *secilecek.borrow_mut() = Some((e.mode.clone(), e.inputs.clone()));
                        }
                    }
                });
            });
        self.p.gecmis = open;
        if let Some((m, i)) = secilecek.into_inner() {
            self.mode = m.clone();
            self.state.inputs.insert(m, i);
            self.p.gecmis = false;
            self.bildir("Kayıt geri yüklendi");
        }
    }

    /// Sekmeli ayarlar penceresi: Görünüm / Widget / Veri / Hakkında.
    /// Yeni bir `Settings` alanı eklenmez; sekme seçimi geçici UI durumudur.
    fn ayarlar_penceresi(&mut self, ctx: &Context) {
        let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));
        let mut sekme = self.ayarlar_sekmesi;
        let mut open = self.p.ayarlar;
        let guncelleme_durumu = if !ag_istegi_yapilir(self.state.settings.guncelleme_kontrol) {
            "Güncelleme kontrolü kapalı — ağ isteği yapılmaz".to_string()
        } else {
            match self.guncelleme.lock().ok().and_then(|k| k.clone()) {
                Some(g) => format!("Yeni sürüm v{} mevcut", g.version),
                None => "Yeni sürüm bulunamadı".to_string(),
            }
        };

        egui::Window::new("Ayarlar")
            .open(&mut open)
            .frame(pencere_cercevesi(p))
            .default_size((520.0, 520.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (i, ad) in ["Görünüm", "Widget", "Veri", "Hakkında"].iter().enumerate() {
                        if ui.selectable_label(sekme == i, *ad).clicked() {
                            sekme = i;
                        }
                    }
                });
                ui.add_space(4.0);
                hairline(ui, p.kenar);
                ui.add_space(10.0);

                egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                    match sekme {
                        // ---- Görünüm: tema, ölçek, hassasiyet, simge + canlı örnek ----
                        0 => {
                            ui.label(RichText::new("Tema").size(T_BODY).color(p.metin).strong());
                            ui.label(RichText::new("Uygulamanın renk kimliği. Sistem seçeneği koyu varsayılır.")
                                .size(T_BUILD).color(p.soluk));
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                for (ad, val) in [("Koyu", "dark"), ("Açık", "light"), ("Sistem", "system")] {
                                    if ui.selectable_label(self.state.settings.theme == val, ad).clicked() {
                                        self.state.settings.theme = val.to_string();
                                        self.kaydet();
                                    }
                                }
                            });

                            ui.add_space(14.0);
                            hairline(ui, p.kenar);
                            ui.add_space(10.0);
                            ui.label(RichText::new("Yazı ölçeği").size(T_BODY).color(p.metin).strong());
                            ui.label(RichText::new("Tüm arayüz metinlerinin boyutu (0,85× – 1,25×).")
                                .size(T_BUILD).color(p.soluk));
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                if ui.add(ikincil_dugme("A−", p)).clicked() {
                                    self.font_degistir(-0.05);
                                }
                                ui.label(RichText::new(format!("{:.2}×", self.state.settings.font_scale))
                                    .size(T_SMALL).color(p.birincil).monospace());
                                if ui.add(ikincil_dugme("A+", p)).clicked() {
                                    self.font_degistir(0.05);
                                }
                            });

                            ui.add_space(14.0);
                            hairline(ui, p.kenar);
                            ui.add_space(10.0);
                            ui.label(RichText::new("Hassasiyet").size(T_BODY).color(p.metin).strong());
                            ui.label(RichText::new("Para tutarlarında gösterilecek ondalık hane sayısı.")
                                .size(T_BUILD).color(p.soluk));
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                for h in [0usize, 2, 4] {
                                    if ui.selectable_label(self.state.settings.precision == h, format!("{h} hane")).clicked() {
                                        self.state.settings.precision = h;
                                        self.kaydet();
                                    }
                                }
                            });

                            ui.add_space(14.0);
                            hairline(ui, p.kenar);
                            ui.add_space(10.0);
                            ui.label(RichText::new("Para birimi").size(T_BODY).color(p.metin).strong());
                            ui.label(RichText::new("Simgeyi göster/gizle ve metnini düzenle.")
                                .size(T_BUILD).color(p.soluk));
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.state.settings.show_currency_symbol, "Simgeyi göster");
                                ui.add(egui::TextEdit::singleline(&mut self.state.settings.currency_symbol)
                                    .hint_text("₺").desired_width(60.0));
                            });

                            ui.add_space(16.0);
                            hairline(ui, p.kenar);
                            ui.add_space(10.0);
                            ui.label(RichText::new("CANLI ÖRNEK").size(T_BUILD).color(p.aksan).strong());
                            ui.add_space(5.0);
                            dugme_cercevesi(p).show(ui, |ui| {
                                let ornek = para_bicimi(
                                    "1234.56",
                                    self.state.settings.precision,
                                    &self.state.settings.currency_symbol,
                                    self.state.settings.show_currency_symbol,
                                );
                                ui.label(RichText::new(ornek).size(T_TITLE).color(p.metin).strong().monospace());
                                ui.label(RichText::new("Seçilen tema ve biçim ayarlarıyla örnek tutar.")
                                    .size(T_BUILD).color(p.soluk));
                            });
                        }

                        // ---- Widget: kalıcı mini hesaplayıcı ayarları ----
                        1 => {
                            ui.label(RichText::new("Mini hesaplayıcı").size(T_BODY).color(p.metin).strong());
                            ui.label(RichText::new("Ayrı pencerede çalışır · Ctrl+Shift+W ile açılıp kapanır.")
                                .size(T_BUILD).color(p.soluk));
                            ui.add_space(8.0);
                            let mut kaydet = false;
                            if ui.checkbox(&mut self.state.widget.aktif, "Etkin").changed() { kaydet = true; }
                            if ui.checkbox(&mut self.state.widget.kompakt, "Kompakt görünüm").changed() { kaydet = true; }
                            if ui.checkbox(&mut self.state.widget.ustte, "Her zaman üstte tut").changed() { kaydet = true; }

                            ui.add_space(12.0);
                            hairline(ui, p.kenar);
                            ui.add_space(10.0);
                            let aktif_mod = self.state.widget.r#mod.clone();
                            ui.label(RichText::new("Hesaplama modu").size(T_SMALL).color(p.ikinci).strong());
                            egui::ComboBox::from_id_salt("ayarlar-widget-mod")
                                .selected_text(modes::mod_etiketi(&aktif_mod).to_string())
                                .width(ui.available_width().max(160.0))
                                .show_ui(ui, |ui| {
                                    for m in MODLAR {
                                        if ui.selectable_label(aktif_mod == m.id, m.etiket).clicked() {
                                            self.state.widget.r#mod = m.id.to_string();
                                            kaydet = true;
                                        }
                                    }
                                });

                            ui.add_space(12.0);
                            hairline(ui, p.kenar);
                            ui.add_space(10.0);
                            let (wlo, whi) = WIDGET_GENISLIK_ARALIGI;
                            if ui.add(egui::Slider::new(&mut self.state.widget.genislik, wlo..=whi).text("Genişlik")).changed() {
                                kaydet = true;
                            }
                            let (hlo, hhi) = WIDGET_YUKSEKLIK_ARALIGI;
                            if ui.add(egui::Slider::new(&mut self.state.widget.yukseklik, hlo..=hhi).text("Yükseklik")).changed() {
                                kaydet = true;
                            }
                            ui.label(RichText::new(format!(
                                "Geçerli boyut: {} × {} px",
                                self.state.widget.genislik.round(), self.state.widget.yukseklik.round()
                            )).size(T_BUILD).color(p.soluk).monospace());

                            ui.add_space(12.0);
                            hairline(ui, p.kenar);
                            ui.add_space(10.0);
                            if ui.add(ikincil_dugme("Konumu sıfırla", p)).clicked() {
                                self.state.widget.konum = None;
                                self.widget_konum_uygulandi = false;
                                self.kaydet();
                                self.bildir("Widget konumu sıfırlandı");
                            }
                            ui.label(RichText::new("Pencere bir sonraki açılışta varsayılan konuma yerleşir.")
                                .size(T_BUILD).color(p.soluk));
                            if kaydet {
                                self.kaydet();
                            }
                        }

                        // ---- Veri: yedekleme ve aktarım eylemleri ----
                        2 => {
                            ui.label(RichText::new("Yedekleme ve aktarım").size(T_BODY).color(p.metin).strong());
                            ui.label(RichText::new("Verileriniz yalnızca bu cihazda tutulur; hiçbir yere gönderilmez.")
                                .size(T_BUILD).color(p.soluk));
                            ui.add_space(10.0);
                            ui.horizontal_wrapped(|ui| {
                                if ui.add(ikincil_dugme("Yedek al", p)).clicked() {
                                    self.komut_calistir(Eylem::YedekAl);
                                }
                                if ui.add(ikincil_dugme("Yedek yükle", p)).clicked() {
                                    self.komut_calistir(Eylem::YedekYukle);
                                }
                            });
                            ui.add_space(6.0);
                            ui.horizontal_wrapped(|ui| {
                                if ui.add(ikincil_dugme("Geçmiş CSV", p)).clicked() {
                                    self.komut_calistir(Eylem::GecmisCsv);
                                }
                                if ui.add(ikincil_dugme("e-Fatura içe aktar", p)).clicked() {
                                    self.komut_calistir(Eylem::EfaturaIce);
                                }
                            });
                            ui.add_space(6.0);
                            if ui.add(ikincil_dugme("Geçmişi temizle", p)).clicked() {
                                self.komut_calistir(Eylem::GecmisTemizle);
                            }

                            ui.add_space(16.0);
                            hairline(ui, p.kenar);
                            ui.add_space(10.0);
                            ui.label(RichText::new("Güncelleme kontrolü").size(T_BODY).color(p.metin).strong());
                            ui.label(
                                RichText::new("Kapattığınızda uygulama hiçbir ağ isteği yapmaz.")
                                    .size(T_BUILD).color(p.soluk),
                            );
                            let mut ag_var = self.state.settings.guncelleme_kontrol;
                            if ui
                                .checkbox(&mut ag_var, "Güncellemeleri kontrol et (ağ isteği)")
                                .changed()
                            {
                                self.state.settings.guncelleme_kontrol = ag_var;
                                if !ag_var {
                                    // Bekleyen güncelleme bilgisi ve ağ durumu temizlenir.
                                    if let Ok(mut kilit) = self.guncelleme.lock() {
                                        *kilit = None;
                                    }
                                    self.guncelleme_bildirildi = false;
                                    self.indiriliyor = false;
                                }
                                self.kaydet();
                            }

                            ui.add_space(16.0);
                            hairline(ui, p.kenar);
                            ui.add_space(10.0);
                            ui.label(RichText::new("Veri klasörü: %APPDATA%/FinansalHesaplayici")
                                .size(T_SMALL).color(p.ikinci));
                            ui.label(RichText::new(AppState::dosya_yolu().to_string_lossy())
                                .size(T_BUILD).color(p.soluk).monospace());
                            ui.add_space(4.0);
                            ui.label(RichText::new("Ayarlar otomatik kaydedilir")
                                .size(T_BUILD).color(p.soluk));
                        }

                        // ---- Hakkında: sürüm, kısayollar, güncelleme ----
                        _ => {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("◈").size(34.0).color(p.birincil).strong());
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("FİNANSAL HESAPLAYICI")
                                        .size(T_TITLE).color(p.metin).strong());
                                    ui.label(RichText::new(format!("Sürüm {SURUM}"))
                                        .size(T_SMALL).color(p.soluk).monospace());
                                });
                            });
                            ui.add_space(10.0);
                            ui.label(RichText::new("Çevrimdışı çalışır · hesaplamalar yerel")
                                .size(T_BODY).color(p.basari).strong());
                            ui.label(RichText::new("Verileriniz cihazınızdan dışarı çıkmaz; ağ bağlantısı gerekmez.")
                                .size(T_BUILD).color(p.soluk));

                            ui.add_space(16.0);
                            hairline(ui, p.kenar);
                            ui.add_space(10.0);
                            ui.label(RichText::new("KISAYOLLAR").size(T_BUILD).color(p.aksan).strong());
                            ui.add_space(6.0);
                            for (k, aciklama) in [
                                ("Ctrl+K", "Komut paleti"),
                                ("Ctrl+Shift+W", "Widget aç / kapat"),
                                ("Esc", "Pencereleri kapat"),
                            ] {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(k).size(T_SMALL).color(p.birincil).monospace().strong());
                                    ui.add_space(10.0);
                                    ui.label(RichText::new(aciklama).size(T_SMALL).color(p.ikinci));
                                });
                            }

                            ui.add_space(16.0);
                            hairline(ui, p.kenar);
                            ui.add_space(10.0);
                            ui.label(RichText::new("GÜNCELLEME").size(T_BUILD).color(p.aksan).strong());
                            ui.add_space(5.0);
                            ui.label(RichText::new(guncelleme_durumu.as_str()).size(T_SMALL).color(p.ikinci));
                            ui.add_space(6.0);
                            let ag_var = ag_istegi_yapilir(self.state.settings.guncelleme_kontrol);
                            let ipucu = if ag_var {
                                "GitHub API'den sürüm kontrolü yapılır"
                            } else {
                                "Ağ istekleri kapalı — ayarlardan açın"
                            };
                            if ui
                                .add_enabled(ag_var, ikincil_dugme("Güncellemeyi kontrol et", p))
                                .on_hover_text(ipucu)
                                .clicked()
                            {
                                let paylasilan = Arc::clone(&self.guncelleme);
                                self.guncelleme_bildirildi = false;
                                std::thread::spawn(move || {
                                    let sonuc = fh_updater::kontrol(SURUM);
                                    if let Ok(mut kilit) = paylasilan.lock() {
                                        *kilit = sonuc;
                                    }
                                });
                                self.bildir("Güncelleme kontrol ediliyor…");
                            }
                        }
                    }
                });
            });
        self.ayarlar_sekmesi = sekme;
        self.p.ayarlar = open;
    }

    fn takvim_penceresi(&mut self, ctx: &Context) {
        let bugun = chrono::Local::now().date_naive();
        let kayitlar = takvim_listesi(bugun);
        let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));
        let mut open = self.p.takvim;
        egui::Window::new("Vergi Takvimi")
            .open(&mut open)
            .frame(pencere_cercevesi(p))
            .default_size((540.0, 460.0))
            .show(ctx, |ui| {
                ui.label(
                    RichText::new("Yaklaşan beyan ve ödeme tarihleri (statik VUK takvimi)")
                        .size(T_SMALL)
                        .color(p.soluk),
                );
                ui.add_space(6.0);
                hairline(ui, p.kenar);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for k in kayitlar.iter().take(80) {
                        let gun = gun_kaldi(&k.tarih, bugun);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(k.tarih.as_str())
                                    .size(T_SMALL)
                                    .color(p.birincil)
                                    .monospace(),
                            );
                            ui.label(
                                RichText::new(k.ad.as_str())
                                    .size(T_BODY)
                                    .color(p.metin)
                                    .strong(),
                            );
                            if (0..=7).contains(&gun) {
                                ui.label(
                                    RichText::new(format!("{gun} gün kaldı"))
                                        .size(T_BUILD)
                                        .color(p.uyari)
                                        .strong(),
                                );
                            }
                        });
                        ui.label(
                            RichText::new(format!("{} — {}", k.donemi, k.aciklama))
                                .size(T_SMALL)
                                .color(p.soluk),
                        );
                        hairline(ui, p.kenar.gamma_multiply(0.6));
                    }
                });
            });
        self.p.takvim = open;
    }

    fn cariler_penceresi(&mut self, ctx: &Context) {
        let idler: Vec<String> = self.state.cariler.iter().map(|c| c.id.clone()).collect();
        let aktif = self.state.active_cari.clone();
        let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));
        let mut open = self.p.cariler;
        egui::Window::new("Cari Kartlar")
            .open(&mut open)
            .frame(pencere_cercevesi(p))
            .default_size((430.0, 420.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.yeni_cari)
                            .hint_text("Cari adı")
                            .desired_width(180.0),
                    );
                    if ui.add(birincil_dugme("Ekle", p)).clicked() {
                        let ad = self.yeni_cari.trim().to_string();
                        if !ad.is_empty() {
                            let id = format!(
                                "c{}",
                                chrono::Local::now().timestamp_nanos_opt().unwrap_or(1)
                            );
                            self.state.cariler.push(Cari {
                                id: id.clone(),
                                name: ad,
                            });
                            self.state.active_cari = id;
                            self.yeni_cari.clear();
                            self.kaydet();
                        }
                    }
                });
                ui.add_space(6.0);
                hairline(ui, p.kenar);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for id in &idler {
                        let cari = self.state.cariler.iter().find(|c| &c.id == id).cloned();
                        let Some(c) = cari else {
                            continue;
                        };
                        let mut sil = false;
                        let mut sec = false;
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(aktif == c.id, c.name.as_str())
                                .clicked()
                            {
                                sec = true;
                            }
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui.add(ikincil_dugme("Sil", p)).clicked() {
                                    sil = true;
                                }
                            });
                        });
                        hairline(ui, p.kenar.gamma_multiply(0.6));
                        if sec {
                            self.state.active_cari = c.id.clone();
                            self.kaydet();
                        }
                        if sil {
                            self.state.cariler.retain(|x| x.id != c.id);
                            if self.state.active_cari == c.id {
                                self.state.active_cari.clear();
                            }
                            self.kaydet();
                        }
                    }
                    if idler.is_empty() {
                        ui.label("Henüz cari yok.");
                    }
                });
            });
        self.p.cariler = open;
    }

    fn donemler_penceresi(&mut self, ctx: &Context) {
        let st = self.state.settings.clone();
        let p = theme::palet(theme::koyu_mu(&st.theme));
        let filtre = self.cari_filtre.clone();
        let kayitlar: Vec<DonemRecord> = self
            .state
            .donemler
            .iter()
            .filter(|d| filtre.is_empty() || d.cari == filtre)
            .map(|d| d.kayit())
            .collect();
        let gruplar = yillara_gir(&kayitlar);
        let csv = donem_csv(&kayitlar);
        let ozet = beyan_ozeti(
            &kayitlar,
            st.precision,
            &st.currency_symbol,
            st.show_currency_symbol,
        );
        let cariler = self.state.cariler.clone();
        let mut open = self.p.donemler;
        egui::Window::new("Dönem KDV Arşivi")
            .open(&mut open)
            .frame(pencere_cercevesi(p))
            .default_size((620.0, 500.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Cari filtresi:").size(T_SMALL).color(p.ikinci));
                    egui::ComboBox::from_id_salt("cari-filtre")
                        .selected_text(if filtre.is_empty() { "Tümü".to_string() }
                            else { cariler.iter().find(|c| c.id == filtre).map(|c| c.name.clone()).unwrap_or(filtre.clone()) })
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(filtre.is_empty(), "Tümü").clicked() {
                                self.cari_filtre.clear();
                            }
                            for c in &cariler {
                                if ui.selectable_label(filtre == c.id, c.name.as_str()).clicked() {
                                    self.cari_filtre = c.id.clone();
                                }
                            }
                        });
                    let mut csv_istendi = false;
                    let mut ozet_istendi = false;
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(ikincil_dugme("Beyan özeti", p)).clicked() {
                            ozet_istendi = true;
                        }
                        if ui.add(ikincil_dugme("CSV indir", p)).clicked() {
                            csv_istendi = true;
                        }
                    });
                    if csv_istendi {
                        self.csv_yaz(csv.clone(), "donemler.csv", "Dönem CSV indirildi");
                    }
                    if ozet_istendi {
                        self.onizleme_baslik = "Beyan Özeti".to_string();
                        self.onizleme_metin = ozet.clone();
                        self.p.onizleme = true;
                    }
                });
                ui.add_space(6.0);
                hairline(ui, p.kenar);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if gruplar.is_empty() {
                        ui.add_space(10.0);
                        ui.label("Arşivde dönem yok. 'Dönem KDV' modundan kayıt ekleyin.");
                    }
                    for (yil, kayitlar, ozet) in &gruplar {
                        let baslik = format!(
                            "{yil}  ·  {} dönem  ·  Net {}",
                            ozet.sayi,
                            para_bicimi(&ozet.net, st.precision, &st.currency_symbol, st.show_currency_symbol)
                        );
                        egui::CollapsingHeader::new(RichText::new(baslik).size(T_BODY).color(p.metin).strong())
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.label(RichText::new(format!(
                                    "Hesaplanan {}  ·  İndirilecek {}  ·  Ödenecek {}  ·  Devreden {}",
                                    para_bicimi(&ozet.hesaplanan, st.precision, &st.currency_symbol, st.show_currency_symbol),
                                    para_bicimi(&ozet.indirilecek, st.precision, &st.currency_symbol, st.show_currency_symbol),
                                    para_bicimi(&ozet.odenecek_net, st.precision, &st.currency_symbol, st.show_currency_symbol),
                                    para_bicimi(&ozet.devreden_net, st.precision, &st.currency_symbol, st.show_currency_symbol),
                                )).size(T_SMALL).color(p.ikinci));
                                hairline(ui, p.kenar);
                                for k in kayitlar {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&k.id).size(T_SMALL).color(p.birincil).monospace());
                                        let durum = if k.odenecek { "Ödenecek" } else { "Devreden" };
                                        ui.label(RichText::new(durum).size(T_BUILD).color(if k.odenecek { p.uyari } else { p.basari }));
                                        ui.add_space(ui.available_width().max(0.0));
                                        let v = para_bicimi(&k.net, st.precision, &st.currency_symbol, st.show_currency_symbol);
                                        ui.label(RichText::new(v).size(T_SMALL).color(p.metin).monospace());
                                    });
                                }
                            });
                        ui.add_space(5.0);
                    }
                });
            });
        self.p.donemler = open;
    }

    fn palet_penceresi(&mut self, ctx: &Context) {
        let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));
        let mut secilen = None;
        let mut open = self.p.palet;
        let arama_id = egui::Id::unique(PALET_ARAMA_ID);
        if self.palet_odak_iste {
            ctx.memory_mut(|m| m.request_focus(arama_id));
            self.palet_odak_iste = false;
        }
        let n = self.filtreli_komutlar().len();
        let degisti = self.palet_sorgu != self.palet_son_sorgu;
        self.palet_sec = palet_secim_duzelt(self.palet_sec, n, degisti);
        self.palet_son_sorgu = self.palet_sorgu.clone();
        egui::Window::new("Komut Paleti · Ctrl+K")
            .open(&mut open)
            .frame(pencere_cercevesi(p))
            .default_size((500.0, 440.0))
            .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 70.0))
            .show(ctx, |ui| {
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut self.palet_sorgu)
                        .id(arama_id)
                        .hint_text("Komut ara… (↑↓ gezin, Enter seç, Esc kapat)")
                        .desired_width(f32::INFINITY),
                );
                if resp.changed() {
                    self.palet_sec = 0;
                    self.palet_son_sorgu = self.palet_sorgu.clone();
                }
                ui.add_space(6.0);
                hairline(ui, p.kenar);
                let idxs = self.filtreli_komutlar();
                egui::ScrollArea::vertical()
                    .max_height(340.0)
                    .show(ui, |ui| {
                        if idxs.is_empty() {
                            ui.label(
                                RichText::new("Eşleşen komut yok.")
                                    .size(T_SMALL)
                                    .color(p.soluk),
                            );
                        }
                        for (n, &idx) in idxs.iter().enumerate() {
                            let c = &self.komutlar[idx];
                            let aktif = n == self.palet_sec;
                            let b = egui::Button::new(
                                RichText::new(format!("{}   · {}", c.ad, c.kategori))
                                    .size(T_SMALL)
                                    .color(if aktif { Color32::WHITE } else { p.ikinci }),
                            )
                            .fill(if aktif {
                                p.birincil
                            } else {
                                Color32::TRANSPARENT
                            })
                            .min_size(Vec2::new(ui.available_width().max(100.0) - 8.0, 24.0));
                            if ui.add(b).clicked() {
                                secilen = Some(idx);
                            }
                        }
                    });
            });
        self.p.palet = open;
        if !self.p.palet {
            ctx.memory_mut(|m| m.surrender_focus(arama_id));
        }
        if let Some(i) = secilen {
            let e = self.komutlar[i].eylem.clone();
            self.komut_calistir(e);
            self.p.palet = false;
        }
    }

    fn onizleme_penceresi(&mut self, ctx: &Context) {
        let baslik = self.onizleme_baslik.clone();
        let metin = self.onizleme_metin.clone();
        let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));
        let mut open = self.p.onizleme;
        egui::Window::new(baslik.as_str())
            .open(&mut open)
            .frame(pencere_cercevesi(p))
            .default_size((560.0, 400.0))
            .show(ctx, |ui| {
                if ui.add(ikincil_dugme("Panoya kopyala", p)).clicked() {
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(metin.clone());
                    }
                    self.bildir("Özet panoya kopyalandı");
                }
                ui.add_space(6.0);
                hairline(ui, p.kenar);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Salt okunur önizleme: eski kod geçici bir klonu düzenliyordu,
                    // yazılanlar kayboluyordu. Gerçek state zaten `self.onizleme_metin`.
                    ui.add(egui::Label::new(metin.as_str()).wrap_mode(TextWrapMode::Wrap));
                });
            });
        self.p.onizleme = open;
    }
}

impl eframe::App for FhApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.kisayollar(&ctx);
        self.tema_guncelle(&ctx);
        self.hesapla();
        self.guncelleme_durumunu_kontrol(&ctx);
        let koyu = theme::koyu_mu(&self.state.settings.theme);
        let p = theme::palet(koyu);
        let genis = ctx.content_rect().width();
        let dar = genis < 1040.0;
        let kenar = Stroke::new(1.0, p.kenar);

        // Paneller kök Ui'de ve CentralPanel'den ÖNCE gösterilir; aksi halde
        // üst panel merkez panelin içine gömülür.
        egui::Panel::top("kimlik_seridi")
            .resizable(false)
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(p.yuzey)
                    .stroke(kenar)
                    .inner_margin(Margin::symmetric(18, 12)),
            )
            .show(ui, |ui| self.ust_panel(ui));

        // egui 0.37 Panel::right/bottom istenen exact_size'ı uygulamıyor
        // (sonuç paneli 250px'e sıkışıyordu). Açık yerleşim: alt kayıt paneli
        // kaldırıldı; üç kolon dikey alanı tamamen doldurur, en altta yalnızca
        // ince durum şeridi kalır.
        let durum_y = 24.0;
        let ana_y = (ui.available_height() - durum_y).max(340.0);
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), ana_y),
            egui::Layout::top_down(Align::Min),
            |ui| {
                if dar {
                    ui.columns(2, |cols| {
                        Frame::new()
                            .fill(p.yuzey)
                            .stroke(kenar)
                            .inner_margin(Margin::symmetric(14, 14))
                            .show(&mut cols[0], |ui| self.mod_nav_panel(ui, ana_y));
                        Frame::new()
                            .fill(p.arka)
                            .stroke(kenar)
                            .inner_margin(Margin::symmetric(16, 14))
                            .show(&mut cols[1], |ui| {
                                theme::doku(ui, koyu);
                                let ust_y = (ui.available_height() * 0.5).clamp(200.0, 330.0);
                                ui.allocate_ui_with_layout(
                                    Vec2::new(ui.available_width(), ust_y),
                                    egui::Layout::top_down(Align::Min),
                                    |ui| self.girdi_panel(ui),
                                );
                                ui.add_space(8.0);
                                egui::ScrollArea::vertical().show(ui, |ui| self.sonuc_panel(ui));
                            });
                    });
                } else {
                    // Sabit oranlar: toplamları tam olarak ui.available_width().
                    let wt = ui.available_width();
                    let wn = (wt * 0.19).clamp(210.0, 320.0);
                    let wr = (wt * 0.36).clamp(360.0, 700.0);
                    let wm = (wt - wn - wr).max(240.0);
                    ui.horizontal(|ui| {
                        ui.allocate_ui_with_layout(
                            Vec2::new(wn, ana_y),
                            egui::Layout::top_down(Align::Min),
                            |ui| {
                                Frame::new()
                                    .fill(p.yuzey)
                                    .stroke(kenar)
                                    .inner_margin(Margin::symmetric(14, 14))
                                    .show(ui, |ui| self.mod_nav_panel(ui, ana_y));
                            },
                        );
                        ui.allocate_ui_with_layout(
                            Vec2::new(wm, ana_y),
                            egui::Layout::top_down(Align::Min),
                            |ui| {
                                Frame::new()
                                    .fill(p.arka)
                                    .stroke(kenar)
                                    .inner_margin(Margin::symmetric(24, 20))
                                    .show(ui, |ui| {
                                        theme::doku(ui, koyu);
                                        egui::ScrollArea::vertical()
                                            .show(ui, |ui| self.girdi_panel(ui));
                                    });
                            },
                        );
                        ui.allocate_ui_with_layout(
                            Vec2::new(wr, ana_y),
                            egui::Layout::top_down(Align::Min),
                            |ui| {
                                Frame::new()
                                    .fill(p.arka)
                                    .stroke(kenar)
                                    .inner_margin(Margin::symmetric(22, 18))
                                    .show(ui, |ui| {
                                        theme::doku(ui, koyu);
                                        egui::ScrollArea::vertical()
                                            .show(ui, |ui| self.sonuc_panel(ui));
                                    });
                            },
                        );
                    });
                }
            },
        );

        // En altta yalnızca ince tema renginde durum şeridi: sürüm, çevrimdışı
        // çalışma, verinin cihazda olduğu ve para/hassasiyet bilgisi. Ne sekme
        // ne kayıt paneli var; geçmiş/dönem/takvim üst şeritten açılır.
        ui.allocate_ui_with_layout(
            Vec2::new(ui.available_width(), durum_y),
            egui::Layout::left_to_right(Align::Center),
            |ui| {
                Frame::new()
                    .fill(p.yuzey)
                    .stroke(kenar)
                    .inner_margin(Margin::symmetric(14, 2))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(format!("v{SURUM} · çevrimdışı · veri cihazda"))
                                .size(T_BUILD)
                                .color(p.soluk),
                        );
                        ui.add_space(ui.available_width().max(0.0));
                        let st = &self.state.settings;
                        let birim = if st.show_currency_symbol && !st.currency_symbol.is_empty() {
                            st.currency_symbol.as_str()
                        } else {
                            "—"
                        };
                        ui.label(
                            RichText::new(format!("{birim} · {} hane", st.precision))
                                .size(T_BUILD)
                                .color(p.soluk)
                                .monospace(),
                        );
                    });
            },
        );

        // OS çubuğu olmadığından sağ alt köşeden yeniden boyutlandırma.
        let grip = ui.allocate_rect(
            Rect::from_min_size(
                ui.cursor().min + Vec2::new(ui.available_width() - 18.0, 0.0),
                Vec2::new(18.0, 18.0),
            ),
            egui::Sense::drag(),
        );
        if grip.drag_started() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::BeginResize(
                    egui::viewport::ResizeDirection::SouthEast,
                ));
        }
        ui.painter().line_segment(
            [
                grip.rect.right_bottom() - Vec2::new(2.0, 14.0),
                grip.rect.right_bottom() - Vec2::new(14.0, 2.0),
            ],
            Stroke::new(2.0, p.soluk),
        );

        self.pencereleri_goster(&ctx);

        // Widget gerçek bir OS penceresi (immediate viewport). Ana pencere
        // minimize edilse de bu viewport açık kalır; yalnızca kalıcı `aktif`
        // durumu onu gösterir. `aktif=false` ise pencere bir sonraki karede
        // kapanır (CancelClose gönderilmez).
        // Kapalıyken konum/alan bayraklarını sıfırla; bir sonraki açılışta
        // kaydedilmiş konum yeniden uygulanır ve tuş takımı ilk alana döner.
        if !self.state.widget.aktif {
            self.widget_konum_uygulandi = false;
            self.widget_alan = 0;
        }
        if self.state.widget.aktif {
            let kompakt = self.state.widget.kompakt;
            let (genislik, yukseklik) = widget::etkili_boyut(
                kompakt,
                self.state.widget.genislik,
                self.state.widget.yukseklik,
            );
            let ustte = self.state.widget.ustte;
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(crate::model::WIDGET_VIEWPORT_ID),
                egui::ViewportBuilder::default()
                    .with_title("Finansal Hesaplayıcı · Widget")
                    .with_inner_size([genislik, yukseklik])
                    .with_min_inner_size(crate::model::WIDGET_MIN_SIZE)
                    .with_decorations(false)
                    .with_resizable(true)
                    .with_window_level(if ustte {
                        egui::WindowLevel::AlwaysOnTop
                    } else {
                        egui::WindowLevel::Normal
                    }),
                |ui, _class| self.widget_viewport_ui(ui, ctx.clone()),
            );
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Pencere boşlukları siyah kalmasın: temanın zemin rengiyle boyanır.
        let c = theme::palet(theme::koyu_mu(&self.state.settings.theme)).arka;
        [
            c.r() as f32 / 255.0,
            c.g() as f32 / 255.0,
            c.b() as f32 / 255.0,
            1.0,
        ]
    }
}

/// Saf yardımcı: dönem kayıtlarından CSV metni (`fh_data::csv_metin` kullanılır).
pub fn donem_csv(kayitlar: &[DonemRecord]) -> String {
    let mut satirlar = vec![vec![
        "Dönem".to_string(),
        "Yıl".to_string(),
        "Cari".to_string(),
        "Satış".to_string(),
        "Alış".to_string(),
        "Hesaplanan".to_string(),
        "İndirilecek".to_string(),
        "Net".to_string(),
        "Durum".to_string(),
    ]];
    for k in kayitlar {
        satirlar.push(vec![
            k.id.clone(),
            k.year.to_string(),
            k.cari.clone(),
            k.satis.clone(),
            k.alis.clone(),
            k.hesaplanan.clone(),
            k.indirilecek.clone(),
            k.net.clone(),
            if k.odenecek { "Ödenecek" } else { "Devreden" }.to_string(),
        ]);
    }
    csv_metin(&satirlar)
}

/// Saf yardımcı: yıllık beyan özeti metni.
pub fn beyan_ozeti(
    kayitlar: &[DonemRecord],
    precision: usize,
    sembol: &str,
    goster: bool,
) -> String {
    let gruplar = yillara_gir(kayitlar);
    if gruplar.is_empty() {
        return "Arşivde dönem kaydı yok.".to_string();
    }
    let p = |v: &str| para_bicimi(v, precision, sembol, goster);
    let mut out = String::from("DÖNEM KDV BEYAN ÖZETİ\n");
    for (yil, _, o) in &gruplar {
        out.push_str(&format!(
            "\n{yil} — {} dönem\nHesaplanan KDV: {}\nİndirilecek KDV: {}\nÖdenecek net: {}\nDevreden net: {}\nGenel net: {}\n",
            o.sayi, p(&o.hesaplanan), p(&o.indirilecek), p(&o.odenecek_net), p(&o.devreden_net), p(&o.net)
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(id: &str, year: i32, net: &str, odenecek: bool) -> DonemRecord {
        DonemRecord {
            id: id.to_string(),
            year,
            month: 1,
            satis: "1000".into(),
            alis: "400".into(),
            devreden_giris: "0".into(),
            hesaplanan: "200".into(),
            indirilecek: "80".into(),
            net: net.to_string(),
            odenecek,
            kdv_rate: "20".into(),
            cari: String::new(),
            closed_at: 0,
        }
    }

    #[test]
    fn beyan_ozeti_yillik_toplam() {
        let list = vec![
            rec("2026-02", 2026, "120", true),
            rec("2025-12", 2025, "-50", false),
        ];
        let s = beyan_ozeti(&list, 2, "₺", true);
        assert!(s.contains("2026 — 1 dönem"));
        assert!(s.contains("2025 — 1 dönem"));
        assert!(s.contains("120,00 ₺"));
        assert!(s.contains("50,00 ₺"));
    }

    #[test]
    fn beyan_ozeti_bos() {
        assert_eq!(beyan_ozeti(&[], 2, "₺", true), "Arşivde dönem kaydı yok.");
    }

    #[test]
    fn donem_csv_bom_ve_baslik() {
        let s = donem_csv(&[rec("2026-02", 2026, "120", true)]);
        assert!(s.starts_with('\u{feff}'));
        assert!(s.contains("\"Dönem\""));
        assert!(s.contains("\"Ödenecek\""));
    }

    // ---- mod listesi kaydırma yardımcıları ----

    #[test]
    fn mod_ofseti_ilk_satir_sifir() {
        assert_eq!(mod_ofseti(0, SATIR_YUKSEKLIGI), 0.0);
    }

    #[test]
    fn mod_ofseti_carpan_uygula() {
        assert_eq!(mod_ofseti(4, SATIR_YUKSEKLIGI), 4.0 * 46.0);
        assert_eq!(mod_ofseti(12, 46.0), 552.0);
    }

    #[test]
    fn mod_ofseti_ozel_yukseklik() {
        assert_eq!(mod_ofseti(3, 40.0), 120.0);
    }

    #[test]
    fn mod_indeksi_sinirla_alt_sinir() {
        // Toplam görünür satırdan azsa kaydırma sıfırdır.
        assert_eq!(mod_indeksi_sinirla(0, 5, 8), 0);
        assert_eq!(mod_indeksi_sinirla(3, 2, 8), 0);
    }

    #[test]
    fn mod_indeksi_sinirla_ust_sinir() {
        assert_eq!(
            mod_indeksi_sinirla(100, MODLAR.len(), GORUNUR_SATIR),
            MODLAR.len() - GORUNUR_SATIR
        );
    }

    #[test]
    fn mod_indeksi_sinirla_aralik_ici() {
        assert_eq!(mod_indeksi_sinirla(6, MODLAR.len(), GORUNUR_SATIR), 6);
    }

    #[test]
    fn mod_indeksi_sinirla_tam_sinir() {
        let sinir = MODLAR.len() - GORUNUR_SATIR;
        assert_eq!(
            mod_indeksi_sinirla(sinir, MODLAR.len(), GORUNUR_SATIR),
            sinir
        );
    }

    #[test]
    fn mod_sabitleri_makul() {
        assert!(SATIR_YUKSEKLIGI > 40.0);
        assert!(GORUNUR_SATIR > 0 && GORUNUR_SATIR < MODLAR.len());
    }
}
