//! eframe/egui masaüstü arayüzü. Tüm hesaplama fh-core, tüm veri dönüşümleri
//! fh-data üzerinden yapılır; bu dosyada finansal formül yoktur.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use chrono::Datelike;
use eframe::egui::{self, Align, Align2, Color32, Context, RichText, Ui, Vec2};
use fh_core::HesapSonucu;
use fh_data::{
    DonemRecord, compute_donem_net, csv_metin, efatura_matrahlar, gun_kaldi, takvim_listesi,
    yillara_gir,
};

use crate::commands::{self, Eylem, Komut};
use crate::model::{
    AppState, Cari, DonemKayit, HistEntry, InputState, SURUM, para_bicimi, satir_bicimi,
};
use crate::modes::{self, MODLAR, ModeDef};
use crate::theme;

#[derive(Default)]
pub struct Pencereler {
    pub gecmis: bool,
    pub ayarlar: bool,
    pub takvim: bool,
    pub cariler: bool,
    pub donemler: bool,
    pub palet: bool,
    pub widget: bool,
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
    pub son_tema: String,
    pub son_olcek: f64,
    /// Arka planda kontrol edilen güncelleme (kilitli paylaşımlı durum).
    pub guncelleme: Arc<Mutex<Option<fh_updater::Guncelleme>>>,
    /// `indir_ve_kur` sonucu: Some(true) başarı, Some(false) başarısız.
    pub indirme_durumu: Arc<Mutex<Option<bool>>>,
    pub guncelleme_bildirildi: bool,
    pub indirme_bildirildi: bool,
}

impl FhApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = AppState::yukle();
        let son_tema = state.settings.theme.clone();
        let son_olcek = state.settings.font_scale;
        theme::uygula(&cc.egui_ctx, &son_tema, son_olcek, true);

        // Ağ kontrolü arka planda yapılır: ağ yoksa sessizce None kalır, UI
        // bloke olmaz ve hiçbir panic kullanıcıya yansımaz.
        let guncelleme: Arc<Mutex<Option<fh_updater::Guncelleme>>> = Arc::new(Mutex::new(None));
        {
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
            son_tema,
            son_olcek,
            guncelleme,
            indirme_durumu: Arc::new(Mutex::new(None)),
            guncelleme_bildirildi: false,
            indirme_bildirildi: false,
        };
        for m in MODLAR {
            let g = app.state.inputs.entry(m.id.to_string()).or_default();
            modes::varsayilanlari_doldur(g, m.id);
        }
        app
    }

    fn bildir(&mut self, metin: impl Into<String>) {
        self.toast = Some((metin.into(), Instant::now()));
    }

    /// Arka plan thread'lerinin sonuçlarını arayüz bildirimine çevirir.
    fn guncelleme_durumunu_kontrol(&mut self) {
        let yeni_surum = if self.guncelleme_bildirildi {
            false
        } else {
            self.guncelleme.lock().map(|k| k.is_some()).unwrap_or(false)
        };
        if yeni_surum {
            self.guncelleme_bildirildi = true;
            self.bildir("Yeni sürüm mevcut — üst panelden güncelleyebilirsiniz");
        }

        let indirme = if self.indirme_bildirildi {
            None
        } else {
            self.indirme_durumu.lock().ok().and_then(|k| *k)
        };
        if let Some(basari) = indirme {
            self.indirme_bildirildi = true;
            if basari {
                self.bildir("Güncelleme indirildi; kurulum için hazır");
            } else {
                self.bildir("Güncelleme indirilemedi (ağ veya doğrulama)");
            }
        }
    }

    /// "Güncelle" düğmesi: indirmeyi/doğrulamayı arka planda başlatır.
    fn guncelleme_baslat(&mut self) {
        let Some(g) = self.guncelleme.lock().ok().and_then(|k| k.clone()) else {
            return;
        };
        self.indirme_bildirildi = false;
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
        let ana = para_bicimi(&r.ana, st.precision, &st.currency_symbol, st.show_currency_symbol);
        let mut metin = format!("{}: {}", modes::mod_etiketi(&self.mode), ana);
        if !r.alt.is_empty() {
            metin.push_str(&format!(
                "\nDiğer: {}",
                para_bicimi(&r.alt, st.precision, &st.currency_symbol, st.show_currency_symbol)
            ));
        }
        metin.push_str(&format!("\n{}", r.detay));
        for (k, v) in &r.satirlar {
            metin.push_str(&format!(
                "\n{k}: {}",
                satir_bicimi(k, v, st.precision, &st.currency_symbol, st.show_currency_symbol)
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
            modes::build_girdi(&mode, e)
        };
        self.result = fh_core::hesapla(&mode, &girdi);
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
        self.state.kaydet();
    }

    fn filtreli_komutlar(&self) -> Vec<usize> {
        commands::filtrele(&self.komutlar, &self.palet_sorgu)
    }

    fn kisayollar(&mut self, ctx: &Context) {
        let (ctrl_k, esc, enter, up, down) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::K) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Enter),
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
            )
        });
        if ctrl_k {
            self.p.palet = !self.p.palet;
            if self.p.palet {
                self.palet_sorgu.clear();
                self.palet_sec = 0;
            }
        }
        if esc {
            self.p.gecmis = false;
            self.p.ayarlar = false;
            self.p.takvim = false;
            self.p.cariler = false;
            self.p.donemler = false;
            self.p.onizleme = false;
            self.p.palet = false;
            self.p.widget = false;
        }
        if self.p.palet {
            let n = self.filtreli_komutlar().len();
            if up && n > 0 {
                self.palet_sec = (self.palet_sec + n - 1) % n;
            }
            if down && n > 0 {
                self.palet_sec = (self.palet_sec + 1) % n;
            }
            if enter && n > 0 {
                let idxs = self.filtreli_komutlar();
                if let Some(&i) = idxs.get(self.palet_sec.min(n - 1)) {
                    let eylem = self.komutlar[i].eylem.clone();
                    self.komut_calistir(eylem);
                    self.p.palet = false;
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
                self.state.kaydet();
            }
            Eylem::FontBuyut => self.font_degistir(0.05),
            Eylem::FontKucult => self.font_degistir(-0.05),
            Eylem::YedekAl => self.yedek_al(),
            Eylem::YedekYukle => self.yedek_yukle(),
            Eylem::GecmisCsv => self.gecmis_csv(),
            Eylem::EfaturaIce => self.efatura_ice(),
            Eylem::Takvim => self.p.takvim = true,
            Eylem::Gecmis => self.p.gecmis = true,
            Eylem::GecmisTemizle => {
                self.state.history.clear();
                self.state.kaydet();
                self.bildir("Geçmiş temizlendi");
            }
            Eylem::Ayarlar => self.p.ayarlar = true,
            Eylem::Cariler => self.p.cariler = true,
            Eylem::Donemler => self.p.donemler = true,
            Eylem::Widget => self.p.widget = !self.p.widget,
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
        let Some(path) = rfd::FileDialog::new().add_filter("JSON", &["json"]).pick_file() else {
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
                self.state.kaydet();
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
            "Tarih".to_string(), "Mod".to_string(), "Sonuç".to_string(), "Detay".to_string(),
        ]];
        for e in &self.state.history {
            let tarih = chrono::DateTime::from_timestamp(e.timestamp, 0)
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_default();
            satirlar.push(vec![tarih, modes::mod_etiketi(&e.mode).to_string(), e.ana.clone(), e.detay.clone()]);
        }
        self.csv_yaz(csv_metin(&satirlar), "gecmis.csv", "Geçmiş CSV dışa aktarıldı");
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
        self.state.kaydet();
        self.bildir(format!("{} matrah içe aktarıldı", matrahlar.len()));
    }

    fn gecmis_kaydet(&mut self) {
        let Some(r) = self.result.clone() else {
            self.bildir("Kaydedilecek geçerli sonuç yok");
            return;
        };
        let now = chrono::Local::now();
        let e = HistEntry {
            id: format!("h{}", now.timestamp_nanos_opt().unwrap_or(now.timestamp_millis())),
            timestamp: now.timestamp(),
            mode: self.mode.clone(),
            inputs: self.girdi().clone(),
            ana: r.ana,
            alt: r.alt,
            detay: r.detay,
        };
        crate::model::gecmis_ekle(&mut self.state.history, e);
        self.state.kaydet();
        self.bildir("Sonuç geçmişe eklendi");
    }

    fn panelleri_goster(&mut self, ui: &mut Ui) {
        // Üst panel kök Ui'de, CentralPanel'den önce gösterilir (bkz. App::ui).
        // Panel state'i ve allocate_ui genişlik dağıtımı güvenilmez; eşit
        // üç kolon sonuç panelinin kırpılmasını kesin çözer.
        ui.columns(3, |cols| {
            egui::ScrollArea::vertical().show(&mut cols[0], |ui| self.mod_panel(ui));
            egui::ScrollArea::vertical().show(&mut cols[1], |ui| self.girdi_panel(ui));
            egui::ScrollArea::vertical().show(&mut cols[2], |ui| self.sonuc_panel(ui));
        });
    }

    fn ust_panel(&mut self, ui: &mut Ui) {
        let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));
        let guncelleme = self.guncelleme.lock().ok().and_then(|k| k.clone());
        ui.horizontal(|ui| {
            ui.label(RichText::new("◈ Finansal Hesaplayıcı").size(16.0).color(p.birincil).strong());
            ui.label(RichText::new(format!("v{SURUM} · çevrimdışı")).size(11.0).color(p.soluk));
        });
        ui.horizontal_wrapped(|ui| {
            for (ad, tik) in [
                ("Geçmiş", 0), ("Dönemler", 1), ("Cari", 2), ("Takvim", 3),
                ("Ayarlar", 4), ("Mini Widget", 5), ("⌘K", 6),
            ] {
                let acik = match tik {
                    0 => self.p.gecmis, 1 => self.p.donemler, 2 => self.p.cariler,
                    3 => self.p.takvim, 4 => self.p.ayarlar, 5 => self.p.widget,
                    _ => self.p.palet,
                };
                let b = egui::Button::new(RichText::new(ad).size(12.0).color(if acik { p.birincil } else { p.ikinci }))
                    .fill(Color32::TRANSPARENT);
                if ui.add(b).clicked() {
                    match tik {
                        0 => self.p.gecmis = !self.p.gecmis,
                        1 => self.p.donemler = !self.p.donemler,
                        2 => self.p.cariler = !self.p.cariler,
                        3 => self.p.takvim = !self.p.takvim,
                        4 => self.p.ayarlar = !self.p.ayarlar,
                        5 => self.p.widget = !self.p.widget,
                        _ => {
                            self.p.palet = !self.p.palet;
                            self.palet_sorgu.clear();
                            self.palet_sec = 0;
                        }
                    }
                }
            }
        });
        if let Some(g) = guncelleme {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("● Yeni sürüm v{} mevcut", g.version))
                        .size(11.0)
                        .color(p.uyari)
                        .strong(),
                );
                if ui.button("Güncelle").clicked() {
                    self.guncelleme_baslat();
                }
            });
        }
        ui.separator();
    }

    fn mod_panel(&mut self, ui: &mut Ui) {
        let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));
        ui.label(RichText::new("HESAPLAMA MODLARI").size(11.0).color(p.soluk).strong());
        ui.add_space(4.0);
        egui::ScrollArea::vertical().show(ui, |ui| {
            for grup in modes::gruplar() {
                ui.label(RichText::new(grup).size(10.0).color(p.vurgu).strong());
                for m in MODLAR.iter().filter(|m| m.grup == grup) {
                    self.mod_dugmesi(ui, m, p.birincil, p.metin);
                }
                ui.add_space(6.0);
            }
        });
    }

    fn mod_dugmesi(&mut self, ui: &mut Ui, m: &ModeDef, birincil: Color32, metin: Color32) {
        let aktif = self.mode == m.id;
        let b = egui::Button::new(RichText::new(m.etiket).size(12.0).color(if aktif { Color32::WHITE } else { metin }))
            .fill(if aktif { birincil } else { Color32::TRANSPARENT })
            .min_size(Vec2::new(ui.available_width().max(80.0) - 8.0, 24.0));
        if ui.add(b).clicked() {
            self.mode = m.id.to_string();
            let e = self.girdi();
            modes::varsayilanlari_doldur(e, m.id);
        }
    }

    fn sonuc_panel(&mut self, ui: &mut Ui) {
        let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));
        let st = self.state.settings.clone();
        let res = self.result.clone();
        ui.label(RichText::new("SONUÇ").size(11.0).color(p.soluk).strong());
        ui.separator();
        let Some(r) = res else {
            ui.label(RichText::new("Geçerli bir sonuç yok.\nGirdi değerlerini kontrol edin.")
                .size(12.0).color(p.soluk));
            return;
        };
        ui.label(RichText::new(modes::mod_etiketi(&self.mode)).size(12.0).color(p.ikinci));
        ui.add_space(2.0);
        let ana = para_bicimi(&r.ana, st.precision, &st.currency_symbol, st.show_currency_symbol);
        ui.label(RichText::new(ana.clone()).size(28.0).color(p.birincil).strong());
        if !r.alt.is_empty() {
            let alt = para_bicimi(&r.alt, st.precision, &st.currency_symbol, st.show_currency_symbol);
            ui.label(RichText::new(format!("Diğer sonuç: {alt}")).size(14.0).color(p.vurgu));
        }
        ui.add_space(6.0);
        ui.label(RichText::new(r.detay.as_str()).size(12.0).color(p.ikinci));

        if self.kopyalandi.is_some_and(|t| t.elapsed() < Duration::from_secs(2)) {
            ui.colored_label(p.basari, "✓ Panoya kopyalandı");
        }
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("sonuc_satirlari").striped(true).show(ui, |ui| {
                for (etiket, deger) in &r.satirlar {
                    ui.label(RichText::new(etiket).size(11.0).color(p.ikinci));
                    let d = satir_bicimi(etiket, deger, st.precision, &st.currency_symbol, st.show_currency_symbol);
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(d).size(11.0).color(p.metin));
                    });
                    ui.end_row();
                }
            });
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui.button("Panoya kopyala").clicked() {
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    let _ = cb.set_text(self.sonuc_metni());
                }
                self.kopyalandi = Some(Instant::now());
            }
            if ui.button("Yazdır / önizle").clicked() {
                self.yazdir();
            }
            if ui.button("Geçmişe kaydet").clicked() {
                self.gecmis_kaydet();
            }
        });
    }

    fn girdi_panel(&mut self, ui: &mut Ui) {
        let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));
        let mode = self.mode.clone();
        let def = modes::mod_def(&mode);
        let mut temizle = false;
        ui.horizontal(|ui| {
            ui.label(RichText::new(modes::mod_etiketi(&mode)).size(18.0).color(p.metin).strong());
            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                if ui.button("Alanları temizle").clicked() {
                    temizle = true;
                }
            });
        });
        if temizle {
            self.state.inputs.insert(mode.clone(), InputState::default());
            let e = self.girdi();
            modes::varsayilanlari_doldur(e, &mode);
        }
        ui.separator();
        let Some(def) = def else { return; };
        egui::ScrollArea::vertical().show(ui, |ui| {
            for f in def.alanlar {
                ui.label(RichText::new(f.etiket).size(12.0).color(p.ikinci).strong());
                let inp = self.state.inputs.entry(mode.clone()).or_default();
                match f.tur {
                    crate::modes::FieldKind::Metin => {
                        let t = egui::TextEdit::singleline(inp.get_mut(f.alan))
                            .hint_text(f.ipucu)
                            .desired_width(f32::INFINITY);
                        ui.add(t);
                    }
                    crate::modes::FieldKind::CokSatir => {
                        let t = egui::TextEdit::multiline(inp.get_mut(f.alan))
                            .hint_text(f.ipucu)
                            .desired_width(f32::INFINITY)
                            .desired_rows(8);
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
                ui.add_space(7.0);
            }
        });
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            if ui.button("Yedek al").clicked() {
                self.yedek_al();
            }
            if ui.button("Yedek yükle").clicked() {
                self.yedek_yukle();
            }
            if mode == "batch" && ui.button("e-Fatura CSV içe aktar").clicked() {
                self.efatura_ice();
            }
            if mode == "donem" && ui.button("Dönemi arşive kaydet").clicked() {
                self.donemi_kaydet();
            }
        });
    }

    fn donemi_kaydet(&mut self) {
        let mode = self.mode.clone();
        let e = self.girdi();
        let (satis, alis, oran, devreden) = (e.birinci.clone(), e.ikinci.clone(), e.oran.clone(), e.ek1.clone());
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
        self.state.kaydet();
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
        if self.p.widget {
            self.widget_penceresi(ctx);
        }
        if self.p.onizleme {
            self.onizleme_penceresi(ctx);
        }
        if let Some((metin, at)) = &self.toast {
            if at.elapsed() < Duration::from_secs(3) {
                let metin = metin.clone();
                egui::Window::new("Bildirim")
                    .title_bar(false)
                    .resizable(false)
                    .collapsible(false)
                    .anchor(Align2::RIGHT_BOTTOM, Vec2::new(-16.0, -16.0))
                    .show(ctx, |ui| {
                        ui.label(RichText::new(metin).size(12.0));
                    });
            }
        }
    }

    fn gecmis_penceresi(&mut self, ctx: &Context) {
        let st = self.state.settings.clone();
        let secilecek = std::cell::RefCell::new(None);
        let mut open = self.p.gecmis;
        egui::Window::new("Geçmiş · son 100 kayıt")
            .open(&mut open)
            .default_size((540.0, 460.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("CSV indir").clicked() {
                        self.gecmis_csv();
                    }
                    if ui.button("Geçmişi temizle").clicked() {
                        self.state.history.clear();
                        self.state.kaydet();
                        self.bildir("Geçmiş temizlendi");
                    }
                    ui.label(RichText::new(format!("{} kayıt", self.state.history.len())).color(theme::palet(theme::koyu_mu(&st.theme)).soluk));
                });
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if self.state.history.is_empty() {
                        ui.label("Henüz kayıt yok. Sonuç panelinden 'Geçmişe kaydet' ile ekleyin.");
                    }
                    for e in &self.state.history {
                        let tarih = chrono::DateTime::from_timestamp(e.timestamp, 0)
                            .map(|d| d.format("%d.%m %H:%M").to_string())
                            .unwrap_or_default();
                        let deger = para_bicimi(&e.ana, st.precision, &st.currency_symbol, st.show_currency_symbol);
                        let b = egui::Button::new(format!("{tarih}  ·  {}  ·  {deger}", modes::mod_etiketi(&e.mode)))
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

    fn ayarlar_penceresi(&mut self, ctx: &Context) {
        let tema = self.state.settings.theme.clone();
        let sembol = self.state.settings.currency_symbol.clone();
        let goster = self.state.settings.show_currency_symbol;
        let hass = self.state.settings.precision;
        let olcek = self.state.settings.font_scale;
        let p = theme::palet(theme::koyu_mu(&tema));
        let mut open = self.p.ayarlar;
        egui::Window::new("Ayarlar")
            .open(&mut open)
            .default_size((420.0, 380.0))
            .show(ctx, |ui| {
                ui.label(RichText::new("Tema").size(12.0).color(p.ikinci).strong());
                ui.horizontal(|ui| {
                    for (ad, val) in [("Koyu", "dark"), ("Açık", "light"), ("Sistem", "system")] {
                        if ui.selectable_label(tema == val, ad).clicked() {
                            self.state.settings.theme = val.to_string();
                            self.state.kaydet();
                        }
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Yazı ölçeği").size(12.0).color(p.ikinci).strong());
                    if ui.button("A−").clicked() {
                        self.font_degistir(-0.05);
                    }
                    ui.label(RichText::new(format!("{:.2}×", olcek)).size(12.0).color(p.birincil));
                    if ui.button("A+").clicked() {
                        self.font_degistir(0.05);
                    }
                });
                ui.separator();
                ui.label(RichText::new("Hassasiyet").size(12.0).color(p.ikinci).strong());
                ui.horizontal(|ui| {
                    for h in [0usize, 2, 4] {
                        if ui.selectable_label(hass == h, format!("{h} hane")).clicked() {
                            self.state.settings.precision = h;
                            self.state.kaydet();
                        }
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.state.settings.show_currency_symbol, "Para simgesini göster");
                    ui.add(egui::TextEdit::singleline(&mut self.state.settings.currency_symbol)
                        .hint_text("₺").desired_width(50.0));
                });
                let _ = (sembol, goster);
                ui.separator();
                ui.label(RichText::new("Ayarlar otomatik kaydedilir · %APPDATA%/FinansalHesaplayici").size(10.0).color(p.soluk));
            });
        self.p.ayarlar = open;
    }

    fn takvim_penceresi(&mut self, ctx: &Context) {
        let bugun = chrono::Local::now().date_naive();
        let kayitlar = takvim_listesi(bugun);
        let p = theme::palet(theme::koyu_mu(&self.state.settings.theme));
        let mut open = self.p.takvim;
        egui::Window::new("Vergi Takvimi")
            .open(&mut open)
            .default_size((520.0, 460.0))
            .show(ctx, |ui| {
                ui.label(RichText::new("Yaklaşan beyan ve ödeme tarihleri (statik VUK takvimi)").size(11.0).color(p.soluk));
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for k in kayitlar.iter().take(80) {
                        let gun = gun_kaldi(&k.tarih, bugun);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(k.tarih.as_str()).size(12.0).color(p.birincil).monospace());
                            ui.label(RichText::new(k.ad.as_str()).size(12.0).color(p.metin).strong());
                            if (0..=7).contains(&gun) {
                                ui.label(RichText::new(format!("{gun} gün kaldı")).size(10.0).color(p.uyari).strong());
                            }
                        });
                        ui.label(RichText::new(format!("{} — {}", k.donemi, k.aciklama)).size(10.0).color(p.soluk));
                        ui.separator();
                    }
                });
            });
        self.p.takvim = open;
    }

    fn cariler_penceresi(&mut self, ctx: &Context) {
        let idler: Vec<String> = self.state.cariler.iter().map(|c| c.id.clone()).collect();
        let aktif = self.state.active_cari.clone();
        let mut open = self.p.cariler;
        egui::Window::new("Cari Kartlar")
            .open(&mut open)
            .default_size((420.0, 420.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.yeni_cari).hint_text("Cari adı").desired_width(180.0));
                    if ui.button("Ekle").clicked() {
                        let ad = self.yeni_cari.trim().to_string();
                        if !ad.is_empty() {
                            let id = format!("c{}", chrono::Local::now().timestamp_nanos_opt().unwrap_or(1));
                            self.state.cariler.push(Cari { id: id.clone(), name: ad });
                            self.state.active_cari = id;
                            self.yeni_cari.clear();
                            self.state.kaydet();
                        }
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for id in &idler {
                        let cari = self.state.cariler.iter().find(|c| &c.id == id).cloned();
                        let Some(c) = cari else { continue; };
                        let mut sil = false;
                        let mut sec = false;
                        ui.horizontal(|ui| {
                            if ui.selectable_label(aktif == c.id, c.name.as_str()).clicked() {
                                sec = true;
                            }
                            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                if ui.button("Sil").clicked() {
                                    sil = true;
                                }
                            });
                        });
                        if sec {
                            self.state.active_cari = c.id.clone();
                            self.state.kaydet();
                        }
                        if sil {
                            self.state.cariler.retain(|x| x.id != c.id);
                            if self.state.active_cari == c.id {
                                self.state.active_cari.clear();
                            }
                            self.state.kaydet();
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
        let ozet = beyan_ozeti(&kayitlar, st.precision, &st.currency_symbol, st.show_currency_symbol);
        let cariler = self.state.cariler.clone();
        let mut open = self.p.donemler;
        egui::Window::new("Dönem KDV Arşivi")
            .open(&mut open)
            .default_size((600.0, 500.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Cari filtresi:");
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
                    ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("Beyan özeti").clicked() {
                            ozet_istendi = true;
                        }
                        if ui.button("CSV indir").clicked() {
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
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if gruplar.is_empty() {
                        ui.label("Arşivde dönem yok. 'Dönem KDV' modundan kayıt ekleyin.");
                    }
                    for (yil, kayitlar, ozet) in &gruplar {
                        let baslik = format!(
                            "{yil}  ·  {} dönem  ·  Net {}",
                            ozet.sayi,
                            para_bicimi(&ozet.net, st.precision, &st.currency_symbol, st.show_currency_symbol)
                        );
                        egui::CollapsingHeader::new(RichText::new(baslik).size(13.0).color(p.metin).strong())
                            .default_open(true)
                            .show(ui, |ui| {
                                ui.label(RichText::new(format!(
                                    "Hesaplanan {}  ·  İndirilecek {}  ·  Ödenecek {}  ·  Devreden {}",
                                    para_bicimi(&ozet.hesaplanan, st.precision, &st.currency_symbol, st.show_currency_symbol),
                                    para_bicimi(&ozet.indirilecek, st.precision, &st.currency_symbol, st.show_currency_symbol),
                                    para_bicimi(&ozet.odenecek_net, st.precision, &st.currency_symbol, st.show_currency_symbol),
                                    para_bicimi(&ozet.devreden_net, st.precision, &st.currency_symbol, st.show_currency_symbol),
                                )).size(11.0).color(p.ikinci));
                                ui.separator();
                                for k in kayitlar {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&k.id).size(11.0).color(p.birincil).monospace());
                                        let durum = if k.odenecek { "Ödenecek" } else { "Devreden" };
                                        ui.label(RichText::new(durum).size(10.0).color(if k.odenecek { p.hata } else { p.basari }));
                                        ui.add_space(ui.available_width().max(0.0));
                                        let v = para_bicimi(&k.net, st.precision, &st.currency_symbol, st.show_currency_symbol);
                                        ui.label(RichText::new(v).size(11.0).color(p.metin));
                                    });
                                }
                            });
                        ui.add_space(4.0);
                    }
                });
            });
        self.p.donemler = open;
    }

    fn palet_penceresi(&mut self, ctx: &Context) {
        let idxs = self.filtreli_komutlar();
        let mut secilen = None;
        let mut open = self.p.palet;
        egui::Window::new("Komut Paleti · Ctrl+K")
            .open(&mut open)
            .default_size((500.0, 440.0))
            .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 70.0))
            .show(ctx, |ui| {
                ui.add(egui::TextEdit::singleline(&mut self.palet_sorgu)
                    .hint_text("Komut ara… (↑↓ gezin, Enter seç, Esc kapat)")
                    .desired_width(f32::INFINITY));
                ui.separator();
                egui::ScrollArea::vertical().max_height(340.0).show(ui, |ui| {
                    if idxs.is_empty() {
                        ui.label("Eşleşen komut yok.");
                    }
                    for (n, &idx) in idxs.iter().enumerate() {
                        let c = &self.komutlar[idx];
                        let aktif = n == self.palet_sec;
                        let b = egui::Button::new(format!("{}   · {}", c.ad, c.kategori))
                            .fill(if aktif { theme::palet(theme::koyu_mu(&self.state.settings.theme)).birincil } else { Color32::TRANSPARENT })
                            .min_size(Vec2::new(ui.available_width().max(100.0) - 8.0, 24.0));
                        if ui.add(b).clicked() {
                            secilen = Some(idx);
                        }
                    }
                });
            });
        self.p.palet = open;
        if let Some(i) = secilen {
            let e = self.komutlar[i].eylem.clone();
            self.komut_calistir(e);
            self.p.palet = false;
        }
    }

    fn widget_penceresi(&mut self, ctx: &Context) {
        let res = self.result.clone();
        let st = self.state.settings.clone();
        let p = theme::palet(theme::koyu_mu(&st.theme));
        let mut open = self.p.widget;
        egui::Window::new("Mini Hesaplayıcı")
            .open(&mut open)
            .resizable(true)
            .default_size((250.0, 130.0))
            .show(ctx, |ui| {
                ui.label(RichText::new(modes::mod_etiketi(&self.mode)).size(11.0).color(p.soluk));
                if let Some(r) = res {
                    let v = para_bicimi(&r.ana, st.precision, &st.currency_symbol, st.show_currency_symbol);
                    ui.label(RichText::new(v).size(22.0).color(p.birincil).strong());
                } else {
                    ui.label("—");
                }
            });
        self.p.widget = open;
    }

    fn onizleme_penceresi(&mut self, ctx: &Context) {
        let baslik = self.onizleme_baslik.clone();
        let metin = self.onizleme_metin.clone();
        let mut open = self.p.onizleme;
        egui::Window::new(baslik.as_str())
            .open(&mut open)
            .default_size((560.0, 400.0))
            .show(ctx, |ui| {
                if ui.button("Panoya kopyala").clicked() {
                    if let Ok(mut cb) = arboard::Clipboard::new() {
                        let _ = cb.set_text(metin.clone());
                    }
                    self.bildir("Özet panoya kopyalandı");
                }
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut metin.clone()).desired_width(f32::INFINITY));
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
        self.guncelleme_durumunu_kontrol();

        // egui panelleri kök Ui'de ve CentralPanel'den ÖNCE gösterilmelidir;
        // aksi halde üst panel merkez panelin içine gömülür.
        egui::Panel::top("ust_panel_v3").show(ui, |ui| self.ust_panel(ui));
        egui::CentralPanel::default().show(ui, |ui| self.panelleri_goster(ui));
        self.pencereleri_goster(&ctx);
    }
}

/// Saf yardımcı: dönem kayıtlarından CSV metni (`fh_data::csv_metin` kullanılır).
pub fn donem_csv(kayitlar: &[DonemRecord]) -> String {
    let mut satirlar = vec![vec![
        "Dönem".to_string(), "Yıl".to_string(), "Cari".to_string(), "Satış".to_string(),
        "Alış".to_string(), "Hesaplanan".to_string(), "İndirilecek".to_string(),
        "Net".to_string(), "Durum".to_string(),
    ]];
    for k in kayitlar {
        satirlar.push(vec![
            k.id.clone(), k.year.to_string(), k.cari.clone(), k.satis.clone(), k.alis.clone(),
            k.hesaplanan.clone(), k.indirilecek.clone(), k.net.clone(),
            if k.odenecek { "Ödenecek" } else { "Devreden" }.to_string(),
        ]);
    }
    csv_metin(&satirlar)
}

/// Saf yardımcı: yıllık beyan özeti metni.
pub fn beyan_ozeti(kayitlar: &[DonemRecord], precision: usize, sembol: &str, goster: bool) -> String {
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
            id: id.to_string(), year, month: 1, satis: "1000".into(), alis: "400".into(),
            devreden_giris: "0".into(), hesaplanan: "200".into(), indirilecek: "80".into(),
            net: net.to_string(), odenecek, kdv_rate: "20".into(), cari: String::new(),
            closed_at: 0,
        }
    }

    #[test]
    fn beyan_ozeti_yillik_toplam() {
        let list = vec![rec("2026-02", 2026, "120", true), rec("2025-12", 2025, "-50", false)];
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
}
