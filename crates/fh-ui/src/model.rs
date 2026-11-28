//! Kalıcı uygulama durumu. Tüm veri %APPDATA% altında tek bir JSON dosyasında
//! tutulur; okuma `fh_data::temizle_yedek` ile sanitize edilir.

use std::path::{Path, PathBuf};

use fh_data::{DonemRecord, TemizYedek, YedekAyarlari, YedekParca, temizle_yedek, yedek_olustur};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Tek sürüm kaynağı: Cargo workspace sürümü (Kurulum/CI ile aynı değer).
pub const SURUM: &str = env!("CARGO_PKG_VERSION");
pub const GECMIS_SINIRI: usize = 100;

/// Açılış penceresinin saf ayarları. `main.rs` bu değerleri `ViewportBuilder`'a
/// uygular; testler aynı varsayılanları GUI gerektirmeden doğrulayabilir.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PencereAyarlari {
    /// İlk açılışta pencere ekranı doldursun (küçük görünen pencere şikâyeti).
    pub maksimum_acilis: bool,
    /// Bozuk 14x14 kaydedilmiş boyutun sonraki açılışları gizlemesi engellenir.
    pub persist_window: bool,
    /// Maksimum açılış kapalıyken kullanılacak ölçü.
    pub inner_size: [f32; 2],
    /// Küçük ekranlarda taşmayı önleyen makul alt sınır.
    pub min_inner_size: [f32; 2],
}

pub const PENCERE_AYARLARI: PencereAyarlari = PencereAyarlari {
    maksimum_acilis: true,
    persist_window: false,
    inner_size: [1280.0, 740.0],
    min_inner_size: [640.0, 480.0],
};

/// Mini widget'ın kalıcı ayarları. Yedek JSON'unda üst düzey `widget`
/// nesnesi olarak tutulur; eksik/bozuk alanlar güvenli varsayılana düşer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WidgetAyarlari {
    pub aktif: bool,
    pub kompakt: bool,
    /// `mod` Rust anahtar sözcüğüdür; raw identifier gerekir. Serde anahtarı
    /// `#[serde(rename)]` ile sabitlenir (JSON'da her zaman "mod").
    #[serde(rename = "mod")]
    pub r#mod: String,
    pub genislik: f32,
    pub yukseklik: f32,
    pub ustte: bool,
    /// Ayrı widget viewport'unun saklanan mutlak konumu `[x, y]` (piksel).
    /// `None` = henüz konumlanmadı; egui ilk açılışta kendi varsayımını kullanır.
    #[serde(default)]
    pub konum: Option<[f32; 2]>,
}

impl Default for WidgetAyarlari {
    fn default() -> Self {
        Self {
            aktif: false,
            kompakt: false,
            r#mod: "kdv".to_string(),
            genislik: 340.0,
            yukseklik: 300.0,
            ustte: true,
            konum: None,
        }
    }
}

/// Widget boyutları için makul aralık; dışındaki değerler varsayılana döner.
pub const WIDGET_GENISLIK_ARALIGI: (f32, f32) = (200.0, 1200.0);
pub const WIDGET_YUKSEKLIK_ARALIGI: (f32, f32) = (160.0, 900.0);

/// Ayrı widget viewport'unun egui layer/pencere kimliği (sabit, yerelleştirilmez).
pub const WIDGET_VIEWPORT_ID: &str = "fh-widget";
/// Widget viewport'unun mutlak alt sınırı [genişlik, yükseklik] (piksel).
pub const WIDGET_MIN_SIZE: [f32; 2] = [220.0, 120.0];
/// Saklanabilir widget konumu için mutlak piksel aralığı; dışındaki değerler `None` olur.
pub const WIDGET_KONUM_ARALIGI: (f32, f32) = (-10000.0, 10000.0);

impl WidgetAyarlari {
    /// Konum çiftinin saklanmaya uygunluğu: iki değer de finite ve
    /// `WIDGET_KONUM_ARALIGI` içinde olmalı; aksi halde konum `None` kalır.
    pub fn konum_gecerli([x, y]: [f32; 2]) -> bool {
        let (lo, hi) = WIDGET_KONUM_ARALIGI;
        x.is_finite() && y.is_finite() && x >= lo && x <= hi && y >= lo && y <= hi
    }

    /// Yedek değerinden güvenli okuma. Nesne değilse tam varsayılan döner;
    /// alan alan tip/sınır kontrolü yapılır, bozuk alan varsayılana düşer.
    pub fn from_value(v: &Value) -> Self {
        let vars = Self::default();
        let Some(obj) = v.as_object() else {
            return vars;
        };
        let bool_alan = |ad: &str, varsayilan: bool| -> bool {
            obj.get(ad).and_then(Value::as_bool).unwrap_or(varsayilan)
        };
        let mod_ = obj
            .get("mod")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty() && s.chars().count() <= 32)
            .unwrap_or(vars.r#mod.as_str())
            .to_string();
        let sayi = |ad: &str, varsayilan: f32, aralik: (f32, f32)| -> f32 {
            obj.get(ad)
                .and_then(Value::as_f64)
                .map(|n| n as f32)
                .filter(|n| n.is_finite() && *n >= aralik.0 && *n <= aralik.1)
                .unwrap_or(varsayilan)
        };
        let konum = obj
            .get("konum")
            .and_then(Value::as_array)
            .filter(|a| a.len() == 2)
            .and_then(|a| {
                let x = a[0].as_f64()? as f32;
                let y = a[1].as_f64()? as f32;
                Self::konum_gecerli([x, y]).then_some([x, y])
            });
        Self {
            aktif: bool_alan("aktif", vars.aktif),
            kompakt: bool_alan("kompakt", vars.kompakt),
            r#mod: mod_,
            genislik: sayi("genislik", vars.genislik, WIDGET_GENISLIK_ARALIGI),
            yukseklik: sayi("yukseklik", vars.yukseklik, WIDGET_YUKSEKLIK_ARALIGI),
            ustte: bool_alan("ustte", vars.ustte),
            konum,
        }
    }
}

fn varsayilan_guncelleme_kontrol() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// "dark" | "light" | "system"
    pub theme: String,
    pub font_scale: f64,
    pub show_currency_symbol: bool,
    /// 0 | 2 | 4
    pub precision: usize,
    pub currency_symbol: String,
    /// Güncelleme kontrolü için ağ isteğine izin. Varsayılan açık; kullanıcı
    /// kapatırsa açılışta ve el ile yapılan kontrollerde hiç ağ isteği çıkmaz.
    #[serde(default = "varsayilan_guncelleme_kontrol")]
    pub guncelleme_kontrol: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_scale: 1.0,
            show_currency_symbol: true,
            precision: 2,
            currency_symbol: "₺".to_string(),
            guncelleme_kontrol: true,
        }
    }
}

/// Her modun girdileri; sayısal alanlar da metin olarak tutulur ki
/// `fh_core::parse_tr` (1.234,56) doğrudan kullanılabilsin.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct InputState {
    pub birinci: String,
    pub ikinci: String,
    pub oran: String,
    pub ek1: String,
    pub ek2: String,
    pub ek3: String,
    pub ek4: String,
    pub tevkifat_pay: String,
    pub tevkifat_kod: String,
    pub yil: String,
    pub ay: String,
    pub gun: String,
    pub secim: String,
    /// Alan bazlı parse hataları. Yedeğe yazılmaz; her kare `modes::parse_hatalari`
    /// ile yeniden üretilir. Boşsa girdi doğrulanmış sayılır.
    #[serde(skip, default)]
    pub hatalar: Vec<(Alan, String)>,
}

#[allow(dead_code)] // Bazı alanlar gelecek modlar için ayrılmıştır.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alan {
    Birinci,
    Ikinci,
    Oran,
    Ek1,
    Ek2,
    Ek3,
    Ek4,
    TevkifatPay,
    TevkifatKod,
    Yil,
    Ay,
    Gun,
    Secim,
}

impl InputState {
    pub fn get(&self, alan: Alan) -> &str {
        match alan {
            Alan::Birinci => &self.birinci,
            Alan::Ikinci => &self.ikinci,
            Alan::Oran => &self.oran,
            Alan::Ek1 => &self.ek1,
            Alan::Ek2 => &self.ek2,
            Alan::Ek3 => &self.ek3,
            Alan::Ek4 => &self.ek4,
            Alan::TevkifatPay => &self.tevkifat_pay,
            Alan::TevkifatKod => &self.tevkifat_kod,
            Alan::Yil => &self.yil,
            Alan::Ay => &self.ay,
            Alan::Gun => &self.gun,
            Alan::Secim => &self.secim,
        }
    }

    pub fn get_mut(&mut self, alan: Alan) -> &mut String {
        match alan {
            Alan::Birinci => &mut self.birinci,
            Alan::Ikinci => &mut self.ikinci,
            Alan::Oran => &mut self.oran,
            Alan::Ek1 => &mut self.ek1,
            Alan::Ek2 => &mut self.ek2,
            Alan::Ek3 => &mut self.ek3,
            Alan::Ek4 => &mut self.ek4,
            Alan::TevkifatPay => &mut self.tevkifat_pay,
            Alan::TevkifatKod => &mut self.tevkifat_kod,
            Alan::Yil => &mut self.yil,
            Alan::Ay => &mut self.ay,
            Alan::Gun => &mut self.gun,
            Alan::Secim => &mut self.secim,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistEntry {
    pub id: String,
    pub timestamp: i64,
    pub mode: String,
    pub inputs: InputState,
    pub ana: String,
    pub alt: String,
    pub detay: String,
}

/// En yeni kayıt en önde; en fazla 100 kayıt tutulur.
pub fn gecmis_ekle(history: &mut Vec<HistEntry>, entry: HistEntry) {
    history.insert(0, entry);
    if history.len() > GECMIS_SINIRI {
        history.truncate(GECMIS_SINIRI);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cari {
    pub id: String,
    pub name: String,
}

/// fh_data::DonemRecord serde türetmediği için UI'da serileştirilebilir karşılığı.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DonemKayit {
    pub id: String,
    pub year: i32,
    pub month: i32,
    pub satis: String,
    pub alis: String,
    pub devreden_giris: String,
    pub hesaplanan: String,
    pub indirilecek: String,
    pub net: String,
    pub odenecek: bool,
    pub kdv_rate: String,
    pub cari: String,
    pub closed_at: i64,
}

impl DonemKayit {
    pub fn kayit(&self) -> DonemRecord {
        DonemRecord {
            id: self.id.clone(),
            year: self.year,
            month: self.month,
            satis: self.satis.clone(),
            alis: self.alis.clone(),
            devreden_giris: self.devreden_giris.clone(),
            hesaplanan: self.hesaplanan.clone(),
            indirilecek: self.indirilecek.clone(),
            net: self.net.clone(),
            odenecek: self.odenecek,
            kdv_rate: self.kdv_rate.clone(),
            cari: self.cari.clone(),
            closed_at: self.closed_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub settings: Settings,
    pub history: Vec<HistEntry>,
    pub donemler: Vec<DonemKayit>,
    pub cariler: Vec<Cari>,
    pub active_cari: String,
    pub inputs: std::collections::HashMap<String, InputState>,
    /// Mini widget ayarları. Eski `FhApp::p.widget: bool` alanının yerini alır;
    /// yedekte üst düzey `widget` nesnesi olarak saklanır.
    pub widget: WidgetAyarlari,
}

fn value_or<T: serde::de::DeserializeOwned>(v: &Value) -> Option<T> {
    serde_json::from_value(v.clone()).ok()
}

/// Yeni UI biçimi (`ana`/`detay`) ve eski Electron biçimi (`result`/`detail`,
/// metin `inputs`) birlikte desteklenir. fh-data kaydı doğrulayıp geçirdikten
/// sonra bu katmanda alan adı farkı yüzünden veri kaybı olmamalıdır (P0).
fn value_or_hist(v: &Value) -> Option<HistEntry> {
    if let Some(e) = value_or::<HistEntry>(v) {
        return Some(e);
    }
    let ana = v
        .get("ana")
        .or_else(|| v.get("result"))
        .and_then(Value::as_str)?;
    Some(HistEntry {
        id: v.get("id")?.as_str()?.to_string(),
        timestamp: v.get("timestamp")?.as_i64()?,
        mode: v.get("mode")?.as_str()?.to_string(),
        inputs: v
            .get("inputs")
            .and_then(value_or::<InputState>)
            .unwrap_or_default(),
        ana: ana.to_string(),
        alt: v
            .get("alt")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        detay: v
            .get("detay")
            .or_else(|| v.get("detail"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
    })
}

/// Eski Electron dönem kayıtları camelCase alan adı kullanır (`devredenGiris`,
/// `kdvRate`, `closedAt`); yeni biçim snake_case'tir. İkisi de okunur.
fn value_or_donem(v: &Value) -> Option<DonemKayit> {
    if let Some(k) = value_or::<DonemKayit>(v) {
        return Some(k);
    }
    Some(DonemKayit {
        id: v.get("id")?.as_str()?.to_string(),
        year: v.get("year")?.as_i64()? as i32,
        month: v.get("month")?.as_i64()? as i32,
        satis: v
            .get("satis")
            .and_then(Value::as_str)
            .unwrap_or("0")
            .to_string(),
        alis: v
            .get("alis")
            .and_then(Value::as_str)
            .unwrap_or("0")
            .to_string(),
        devreden_giris: v
            .get("devreden_giris")
            .or_else(|| v.get("devredenGiris"))
            .and_then(Value::as_str)
            .unwrap_or("0")
            .to_string(),
        hesaplanan: v
            .get("hesaplanan")
            .and_then(Value::as_str)
            .unwrap_or("0")
            .to_string(),
        indirilecek: v
            .get("indirilecek")
            .and_then(Value::as_str)
            .unwrap_or("0")
            .to_string(),
        net: v
            .get("net")
            .and_then(Value::as_str)
            .unwrap_or("0")
            .to_string(),
        odenecek: v.get("odenecek").and_then(Value::as_bool).unwrap_or(true),
        kdv_rate: v
            .get("kdv_rate")
            .or_else(|| v.get("kdvRate"))
            .and_then(Value::as_str)
            .unwrap_or("20")
            .to_string(),
        cari: v
            .get("cari")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        closed_at: v
            .get("closed_at")
            .or_else(|| v.get("closedAt"))
            .and_then(Value::as_i64)
            .unwrap_or(0),
    })
}

fn temiz_settings(t: &TemizYedek, sembol: String, v: &Value) -> Settings {
    // Eski yedeklerde alan yoksa güncelleme kontrolü açık kalır (geriye uyumluluk).
    let guncelleme_kontrol = v
        .get("ekstra")
        .and_then(|e| e.get("guncelleme_kontrol"))
        .and_then(Value::as_bool)
        .unwrap_or(true);
    Settings {
        theme: t.theme.clone(),
        font_scale: t.font_scale,
        show_currency_symbol: t.show_currency_symbol,
        precision: t.precision,
        currency_symbol: sembol,
        guncelleme_kontrol,
    }
}

impl AppState {
    pub fn from_backup(v: &Value) -> Option<Self> {
        let t = temizle_yedek(v)?;
        let sembol = v
            .get("ekstra")
            .and_then(|e| e.get("currency_symbol"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty() && s.chars().count() <= 4)
            .unwrap_or("₺")
            .to_string();
        let mut inputs = std::collections::HashMap::new();
        if let Some(map) = v.get("inputs").and_then(Value::as_object) {
            for (k, val) in map {
                if let Some(i) = value_or::<InputState>(val) {
                    inputs.insert(k.clone(), i);
                }
            }
        }
        Some(Self {
            settings: temiz_settings(&t, sembol, v),
            history: t.history.iter().filter_map(value_or_hist).collect(),
            donemler: t.donemler.iter().filter_map(value_or_donem).collect(),
            cariler: t.cariler.iter().filter_map(value_or).collect(),
            active_cari: t.active_cari,
            inputs,
            widget: v
                .get("widget")
                .map(WidgetAyarlari::from_value)
                .unwrap_or_default(),
        })
    }

    /// `%APPDATA%/FinansalHesaplayici/veri.json`
    pub fn dosya_yolu() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("FinansalHesaplayici").join("veri.json")
    }

    /// Güncelleme başlamadan önce `veri.json`'un yanına `veri-guncelleme-oncesi.json`
    /// kopyası bırakır (her seferinde üzerine yazılır). Kaynak yoksa sessizce
    /// `Ok(None)` döner: yedeklenecek bir şey yok.
    pub fn guncelleme_oncesi_yedek(kaynak: &Path) -> std::io::Result<Option<PathBuf>> {
        if !kaynak.is_file() {
            return Ok(None);
        }
        let hedef = kaynak.with_file_name("veri-guncelleme-oncesi.json");
        std::fs::copy(kaynak, &hedef)?;
        Ok(Some(hedef))
    }

    /// BOM'lu (`U+FEFF`) veya BOM'suz JSON metninden state üretir.
    /// Saf fonksiyondur: dosya erişimi yoktur; `yukle()` bunu kullanır.
    /// Bazı editörler/Windows araçları JSON'a BOM ekler; serde_json BOM'u
    /// geçersiz bayt sayıp tüm dosyayı reddederdi (kullanıcı verisi kaybı).
    pub fn metinden_yukle(metin: &str) -> Option<Self> {
        let metin = metin.strip_prefix('\u{feff}').unwrap_or(metin);
        let value = serde_json::from_str::<Value>(metin).ok()?;
        Self::from_backup(&value)
    }

    pub fn yukle() -> Self {
        let fallback = Self {
            settings: Settings::default(),
            history: Vec::new(),
            donemler: Vec::new(),
            cariler: Vec::new(),
            active_cari: String::new(),
            inputs: std::collections::HashMap::new(),
            widget: WidgetAyarlari::default(),
        };
        let Ok(metin) = std::fs::read_to_string(Self::dosya_yolu()) else {
            return fallback;
        };
        Self::metinden_yukle(&metin).unwrap_or(fallback)
    }

    pub fn yedek_degeri(&self) -> Value {
        let p = YedekParca {
            surum: Some(SURUM.to_string()),
            tarih: None,
            history: self
                .history
                .iter()
                .filter_map(|e| serde_json::to_value(e).ok())
                .collect(),
            donemler: self
                .donemler
                .iter()
                .filter_map(|d| serde_json::to_value(d).ok())
                .collect(),
            cariler: self
                .cariler
                .iter()
                .filter_map(|c| serde_json::to_value(c).ok())
                .collect(),
            active_cari: self.active_cari.clone(),
            ayarlar: Some(YedekAyarlari {
                show_currency_symbol: self.settings.show_currency_symbol,
                precision: self.settings.precision,
                font_scale: self.settings.font_scale,
            }),
            theme: Some(self.settings.theme.clone()),
        };
        let mut v = yedek_olustur(&p);
        if let Some(obj) = v.as_object_mut() {
            obj.insert(
                "ekstra".to_string(),
                json!({
                    "currency_symbol": self.settings.currency_symbol,
                    "guncelleme_kontrol": self.settings.guncelleme_kontrol,
                }),
            );
            let mut inputs = serde_json::Map::new();
            for (k, i) in &self.inputs {
                if let Ok(iv) = serde_json::to_value(i) {
                    inputs.insert(k.clone(), iv);
                }
            }
            obj.insert("inputs".to_string(), Value::Object(inputs));
            obj.insert(
                "widget".to_string(),
                serde_json::to_value(&self.widget).unwrap_or_else(|_| json!({})),
            );
        }
        v
    }

    /// Kalıcı kayıt. Geçici dosyaya yazar sonra atomik değiştirir; hataları
    /// yutmaz, çağıranın kullanıcıya bildirmesi için `Result` döner.
    pub fn kaydet(&self) -> Result<(), String> {
        Self::kaydet_path(&Self::dosya_yolu(), &self.yedek_degeri())
    }

    /// `kaydet`ın saf dosya yolu çeşidi: testler geçici dizinde çağırır.
    /// `gecici` hedefle aynı dizinde olmalı ki rename atomik olsun.
    pub fn kaydet_path(path: &Path, veri: &Value) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Kayıt klasörü oluşturulamadı: {e}"))?;
        }
        let metin = serde_json::to_string_pretty(veri)
            .map_err(|e| format!("Yedek JSON'u üretilemedi: {e}"))?;

        // Bozuk/okunamayan mevcut dosyanın üzerine yazmadan önce kurtarma
        // kopyası alınır; kullanıcı verisi sessizce kaybolmasın.
        if path.exists() {
            let mevcut = std::fs::read_to_string(path).unwrap_or_default();
            if Self::metinden_yukle(&mevcut).is_none() {
                let kurtarma = path.with_extension("json.bak");
                std::fs::copy(path, &kurtarma)
                    .map_err(|e| format!("Bozuk dosyanın kurtarma kopyası alınamadı: {e}"))?;
            }
        }

        let gecici = path.with_extension("json.tmp");
        // Bu noktada mevcut dosya hâlâ dokunulmadı: yazma hatasında korunur.
        std::fs::write(&gecici, &metin).map_err(|e| format!("Geçici dosya yazılamadı: {e}"))?;

        match std::fs::rename(&gecici, path) {
            Ok(()) => Ok(()),
            Err(ilk_hata) => {
                // Windows'ta hedef mevcutsa rename başarısız olur. Eski dosyayı
                // önce güvenli bir takas dosyasına kopyalarız; kopya başarısızsa
                // orijinal dosyaya hiç dokunmayız.
                let takas = path.with_extension("json.swap");
                if let Err(kopya_hata) = std::fs::copy(path, &takas) {
                    let _ = std::fs::remove_file(&gecici);
                    return Err(format!(
                        "Kayıt tamamlanamadı: takas kopyası alınamadı: {kopya_hata} ({ilk_hata})"
                    ));
                }
                let degistir = std::fs::remove_file(path)
                    .map_err(|e| e.to_string())
                    .and_then(|_| std::fs::rename(&gecici, path).map_err(|e| e.to_string()));
                match degistir {
                    Ok(()) => {
                        let _ = std::fs::remove_file(&takas); // temizlik: en iyi çaba
                        Ok(())
                    }
                    Err(hata) => {
                        let _ = std::fs::rename(&takas, path); // orijinali geri koy
                        let _ = std::fs::remove_file(&gecici);
                        Err(format!("Kayıt tamamlanamadı: {hata} ({ilk_hata})"))
                    }
                }
            }
        }
    }
}

/// Makine biçimindeki tutarı Türkçe gösterime çevirir (saf fonksiyon).
pub fn para_bicimi(machine: &str, precision: usize, sembol: &str, goster: bool) -> String {
    let deger = fh_core::parse_tr(machine).unwrap_or(Decimal::ZERO);
    let metin = fh_core::format_tr(deger, precision);
    if goster && !sembol.is_empty() {
        format!("{metin} {sembol}")
    } else {
        metin
    }
}

/// Yalnızca bu satırların değeri gerçek yüzdedir (fh-core `row_raw` ile üretir).
/// Etiketinde `%` geçen her satır yüzde değildir: `"%20 KDV"` bir para tutarı,
/// `"Adım 1 (%10)"` bir ara fiyattır. İçerik (`contains`) ile karar vermek
/// `"%200,00"` gibi yanlış çıktı üretiyordu; açık liste kullanılır.
const YUZDE_SATIRLARI: [&str; 5] = [
    "Gerçek Marj",
    "Marj Karşılığı",
    "Etkin İndirim",
    "Değişim %",
    "Efektif Oran",
];

/// Sayı olmayan (metin/sayıç) satırlar para olarak biçimlendirilmemeli.
const HAM_SATIRLAR: [&str; 2] = ["İşlem kodu", "Geçerli satır"];

/// Sonuç tablosu satırı: gerçek yüzde satırları `%`, diğerleri para olarak gösterilir.
pub fn satir_bicimi(
    label: &str,
    value: &str,
    precision: usize,
    sembol: &str,
    goster: bool,
) -> String {
    let etiket = label.trim();
    if HAM_SATIRLAR.iter().any(|s| s.eq_ignore_ascii_case(etiket)) {
        return value.to_string();
    }
    let Some(d) = fh_core::parse_tr(value) else {
        return value.to_string();
    };
    if YUZDE_SATIRLARI
        .iter()
        .any(|s| s.eq_ignore_ascii_case(etiket))
    {
        format!("%{}", fh_core::format_tr(d, precision))
    } else {
        para_bicimi(value, precision, sembol, goster)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn guncelleme_oncesi_yedek_kopyalar_ve_kaynak_yoksa_none() {
        let dir = std::env::temp_dir().join(format!("fh-yedek-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let kaynak = dir.join("veri.json");
        assert_eq!(AppState::guncelleme_oncesi_yedek(&kaynak).unwrap(), None);
        std::fs::write(&kaynak, "{\"a\":1}").unwrap();
        let hedef = AppState::guncelleme_oncesi_yedek(&kaynak).unwrap().unwrap();
        assert_eq!(std::fs::read_to_string(hedef).unwrap(), "{\"a\":1}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_NO: AtomicU64 = AtomicU64::new(0);

    fn test_dizini() -> PathBuf {
        let n = TEST_NO.fetch_add(1, Ordering::Relaxed);
        let d = std::env::temp_dir().join(format!("fh-kaydet-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn entry(id: &str) -> HistEntry {
        HistEntry {
            id: id.to_string(),
            timestamp: 1,
            mode: "kdv".to_string(),
            inputs: InputState::default(),
            ana: "1".to_string(),
            alt: String::new(),
            detay: String::new(),
        }
    }

    #[test]
    fn para_bicimi_nokta_ve_virgul() {
        assert_eq!(para_bicimi("1234567.89", 2, "₺", true), "1.234.567,89 ₺");
        assert_eq!(para_bicimi("1234567.89", 2, "₺", false), "1.234.567,89");
        assert_eq!(para_bicimi("-30", 0, "$", true), "-30 $");
    }

    #[test]
    fn satir_bicimi_yuzde_ve_para() {
        assert_eq!(satir_bicimi("Gerçek Marj", "12.5", 2, "₺", true), "%12,50");
        assert_eq!(satir_bicimi("Matrah", "1000", 2, "₺", true), "1.000,00 ₺");
        assert_eq!(satir_bicimi("İşlem kodu", "4A", 2, "₺", true), "4A");
    }

    #[test]
    fn satir_bicimi_yuzde_etiketli_para_satirlari() {
        // fh-core "% KDV" / "% Kesinti" etiketlerine para değeri yazar.
        assert_eq!(satir_bicimi("% KDV", "200", 2, "₺", true), "200,00 ₺");
        assert_eq!(satir_bicimi("% Kesinti", "150", 2, "₺", true), "150,00 ₺");
        // Gerçek yüzde satırları yüzde olarak kalmalı.
        assert_eq!(satir_bicimi("Efektif oran", "12.5", 2, "₺", true), "%12,50");
    }

    #[test]
    fn satir_bicimi_etiketinde_yuzde_olan_para_satirlari() {
        // REGRESYON: "%20 KDV" ve "Adım 1 (%10)" para tutarıdır; eski
        // `etiket.contains('%')` kuralı bunları "%200,00" / "%900,00" yapardı.
        assert_eq!(satir_bicimi("%20 KDV", "2000", 2, "₺", true), "2.000,00 ₺");
        assert_eq!(
            satir_bicimi("Adım 1 (%10)", "900", 2, "₺", true),
            "900,00 ₺"
        );
        assert_eq!(satir_bicimi("Adım 2 (%5)", "855", 2, "₺", true), "855,00 ₺");
        // Gerçek yüzde satırları etiketinde % olmasa bile yüzde kalır.
        assert_eq!(
            satir_bicimi("Marj Karşılığı", "12.5", 2, "₺", true),
            "%12,50"
        );
        assert_eq!(satir_bicimi("Etkin İndirim", "19", 2, "₺", false), "%19,00");
    }

    #[test]
    fn satir_bicimi_sayic_satiri_para_degil() {
        // Toplu hesapta "Geçerli satır: 2" sayıdır; "2,00 ₺" yanlıştır.
        assert_eq!(satir_bicimi("Geçerli satır", "2", 2, "₺", true), "2");
        assert_eq!(satir_bicimi("İşlem kodu", "4A", 2, "₺", true), "4A");
    }

    #[test]
    fn gecmis_en_fazla_100() {
        let mut h = Vec::new();
        for i in 0..120 {
            gecmis_ekle(&mut h, entry(&format!("{i}")));
        }
        assert_eq!(h.len(), 100);
        assert_eq!(h[0].id, "119");
    }

    fn ornek_entry(id: &str, mod_: &str, ana: &str) -> HistEntry {
        HistEntry {
            id: id.to_string(),
            timestamp: 1731912345000,
            mode: mod_.to_string(),
            inputs: InputState {
                birinci: "1000".into(),
                oran: "20".into(),
                ..InputState::default()
            },
            ana: ana.to_string(),
            alt: "1200".to_string(),
            detay: "KDV hesabı".to_string(),
        }
    }

    fn ornek_donem(id: &str, cari: &str) -> DonemKayit {
        DonemKayit {
            id: id.to_string(),
            year: 2026,
            month: id[5..7].parse().unwrap_or(1),
            satis: "100000".into(),
            alis: "60000".into(),
            devreden_giris: "0".into(),
            hesaplanan: "20000".into(),
            indirilecek: "12000".into(),
            net: "8000".into(),
            odenecek: true,
            kdv_rate: "20".into(),
            cari: cari.to_string(),
            closed_at: 1731912345000,
        }
    }

    fn ornek_state() -> AppState {
        let mut history = Vec::new();
        gecmis_ekle(&mut history, ornek_entry("h1", "kdv", "200"));
        gecmis_ekle(&mut history, ornek_entry("h2", "stopaj", "150"));
        let mut inputs = std::collections::HashMap::new();
        inputs.insert(
            "kdv".to_string(),
            InputState {
                birinci: "1000".into(),
                oran: "20".into(),
                secim: "haric".into(),
                ..InputState::default()
            },
        );
        AppState {
            settings: Settings {
                theme: "light".into(),
                font_scale: 1.1,
                show_currency_symbol: false,
                precision: 4,
                currency_symbol: "$".into(),
                guncelleme_kontrol: true,
            },
            history,
            donemler: vec![ornek_donem("2026-01", "c1"), ornek_donem("2026-02", "c2")],
            cariler: vec![
                Cari {
                    id: "c1".into(),
                    name: "Müşteri A".into(),
                },
                Cari {
                    id: "c2".into(),
                    name: "Müşteri B".into(),
                },
            ],
            active_cari: "c1".into(),
            inputs,
            widget: WidgetAyarlari {
                aktif: true,
                kompakt: true,
                r#mod: "stopaj".into(),
                genislik: 420.0,
                yukseklik: 360.0,
                ustte: false,
                konum: Some([150.0, 120.0]),
            },
        }
    }

    #[test]
    fn gecmis_ekle_sonra_yedekten_geri_yukle() {
        // gecmis_ekle → yedek_al → temizle_yedek → from_backup zinciri.
        let state = ornek_state();
        let yedek = state.yedek_degeri();
        let geri = AppState::from_backup(&yedek).expect("yedek geri yüklenmeli");
        assert_eq!(geri.history.len(), 2);
        assert_eq!(geri.history[0].id, "h2", "en yeni kayıt en önde kalmalı");
        assert_eq!(geri.history[0].ana, "150");
        assert_eq!(geri.history[1].ana, "200");
        assert_eq!(geri.history[0].inputs.birinci, "1000");
    }

    #[test]
    fn appstate_tam_round_trip() {
        let state = ornek_state();
        let geri = AppState::from_backup(&state.yedek_degeri()).unwrap();
        assert_eq!(geri.settings, state.settings);
        assert_eq!(geri.history, state.history);
        assert_eq!(geri.donemler, state.donemler);
        assert_eq!(geri.cariler, state.cariler);
        assert_eq!(geri.active_cari, "c1");
        assert_eq!(
            geri.widget, state.widget,
            "widget ayarları round-trip'te korunmalı"
        );
        assert_eq!(geri.inputs.get("kdv").unwrap().secim, "haric");
        assert_eq!(geri.inputs.get("kdv").unwrap().oran, "20");
    }

    #[test]
    fn p0_eski_result_alani_geri_yuklemede_kaybolmuyor() {
        // Eski Electron yedekleri "result"/"detail" alanlarını kullanır; P0 regresyonu.
        let mut yedek = ornek_state().yedek_degeri();
        yedek["history"] = json!([
            {"id": "eski", "result": "250", "timestamp": 1, "mode": "kdv",
             "inputs": "1000", "detail": "eski detay", "pinned": false}
        ]);
        let geri = AppState::from_backup(&yedek).expect("eski biçim yüklenmeli");
        assert_eq!(geri.history.len(), 1, "eski geçmiş kaydı düşürülmemeli");
        assert_eq!(geri.history[0].ana, "250");
        assert_eq!(geri.history[0].detay, "eski detay");
        assert_eq!(geri.settings.precision, 4);
    }

    #[test]
    fn eski_ayar_ve_theme_alanlari_appstate_uzerinde_korunur() {
        // Eski yedekte theme üst düzeyde değil settings içinde olabilir.
        let mut yedek = ornek_state().yedek_degeri();
        yedek["theme"] = Value::Null;
        yedek["settings"] = json!({
            "theme": "system", "showCurrencySymbol": false,
            "precision": 0, "fontScale": 0.9
        });
        yedek["activeCari"] = json!("c2");
        let geri = AppState::from_backup(&yedek).unwrap();
        assert_eq!(geri.settings.theme, "system");
        assert_eq!(geri.settings.precision, 0);
        assert_eq!(geri.settings.font_scale, 0.9);
        assert!(!geri.settings.show_currency_symbol);
        assert_eq!(geri.active_cari, "c2");
        assert_eq!(geri.settings.currency_symbol, "$"); // ekstra korunur
    }

    #[test]
    fn p0_eski_electron_donem_field_names_geri_yuklenir() {
        let mut yedek = ornek_state().yedek_degeri();
        yedek["donemler"] = json!([{
            "id": "2026-03", "year": 2026, "month": 3,
            "satis": "100000", "alis": "60000", "devredenGiris": "1000",
            "hesaplanan": "20000", "indirilecek": "12000", "net": "7000",
            "odenecek": true, "kdvRate": "20", "cari": "Müşteri A", "closedAt": 42
        }]);
        let geri = AppState::from_backup(&yedek).unwrap();
        assert_eq!(geri.donemler.len(), 1, "eski dönem kaydı düşürülmemeli");
        let donem = &geri.donemler[0];
        assert_eq!(donem.devreden_giris, "1000");
        assert_eq!(donem.kdv_rate, "20");
        assert_eq!(donem.closed_at, 42);
        assert_eq!(donem.net, "7000");
    }

    #[test]
    fn cari_filtresi_uygulama_durumu_uzerinde() {
        // app.rs::donemler_penceresi'ndeki yükleme koşuluyla aynı davranış.
        let state = ornek_state();
        let filtrele = |f: &str| -> Vec<DonemKayit> {
            state
                .donemler
                .iter()
                .filter(|d| f.is_empty() || d.cari == f)
                .cloned()
                .collect()
        };
        assert_eq!(filtrele("").len(), 2);
        assert_eq!(filtrele("c1").len(), 1);
        assert_eq!(filtrele("c1")[0].id, "2026-01");
        assert_eq!(filtrele("yok").len(), 0);
        // Filtrelenen kayıtların cari adı çözümlenebiliyor.
        let ad = state
            .cariler
            .iter()
            .find(|c| c.id == filtrele("c2")[0].cari)
            .unwrap()
            .name
            .clone();
        assert_eq!(ad, "Müşteri B");
    }

    #[test]
    fn satir_bicimi_kenar_durumlari() {
        assert_eq!(
            satir_bicimi("Efektif Oran", "12.5", 2, "₺", false),
            "%12,50"
        );
        assert_eq!(satir_bicimi("Değişim %", "-96", 2, "₺", false), "%-96,00");
        assert_eq!(satir_bicimi("Kur (1 USD)", "40", 4, "₺", true), "40,0000 ₺");
        assert_eq!(satir_bicimi("Geçersiz", "abc", 2, "₺", true), "abc");
    }

    #[test]
    fn metinden_yukle_bomlu_ve_bomsuz() {
        let state = ornek_state();
        let metin = serde_json::to_string(&state.yedek_degeri()).unwrap();
        let bomsuz = AppState::metinden_yukle(&metin).unwrap();
        let bomlu = AppState::metinden_yukle(&format!("\u{feff}{metin}")).unwrap();
        assert_eq!(bomsuz.settings, state.settings);
        assert_eq!(bomlu.settings, state.settings, "BOM'lu dosya da okunmalı");
        assert_eq!(bomlu.history.len(), state.history.len());
    }

    #[test]
    fn yedek_geri_yukleme_round_trip_bomlu() {
        let state = ornek_state();
        let metin = serde_json::to_string_pretty(&state.yedek_degeri()).unwrap();
        let geri = AppState::metinden_yukle(&format!("\u{feff}{metin}")).unwrap();
        assert_eq!(geri.settings, state.settings);
        assert_eq!(geri.history, state.history);
        assert_eq!(geri.donemler, state.donemler);
        assert_eq!(geri.cariler, state.cariler);
        assert_eq!(geri.active_cari, state.active_cari);
    }

    #[test]
    fn kaydet_yazma_hatasinda_mevcut_dosya_korunur() {
        let dizin = test_dizini();
        let path = dizin.join("veri.json");
        let onceki = "{\"uygulama\":\"finansal-hesaplayici\"}";
        std::fs::write(&path, onceki).unwrap();
        // Geçici dosya yolunu dizin yaparak yazmayı bilerek başarısız et.
        std::fs::create_dir(path.with_extension("json.tmp")).unwrap();
        let sonuc = AppState::kaydet_path(&path, &json!({"yeni": true}));
        assert!(sonuc.is_err(), "yazma hatası yutulmamalı");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), onceki);
        std::fs::remove_dir_all(dizin).unwrap();
    }

    #[test]
    fn kaydet_bozuk_dosyada_kurtarma_kopyasi_alir() {
        let dizin = test_dizini();
        let path = dizin.join("veri.json");
        std::fs::write(&path, "{bozuk json").unwrap();
        let veri = ornek_state().yedek_degeri();
        AppState::kaydet_path(&path, &veri).unwrap();
        let bak = path.with_extension("json.bak");
        assert_eq!(std::fs::read_to_string(&bak).unwrap(), "{bozuk json");
        let geri = AppState::metinden_yukle(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(geri.history.len(), 2);
        assert!(
            !path.with_extension("json.tmp").exists(),
            "geçici dosya kalmamalı"
        );
        std::fs::remove_dir_all(dizin).unwrap();
    }

    #[test]
    fn kaydet_gecerli_dosyada_kurtarma_kopyasi_olusturmaz() {
        let dizin = test_dizini();
        let path = dizin.join("veri.json");
        let veri = ornek_state().yedek_degeri();
        AppState::kaydet_path(&path, &veri).unwrap();
        AppState::kaydet_path(&path, &veri).unwrap();
        assert!(!path.with_extension("json.bak").exists());
        assert!(!path.with_extension("json.tmp").exists());
        std::fs::remove_dir_all(dizin).unwrap();
    }

    #[test]
    fn guncelleme_kontrol_yedekte_tasinir_ve_eksikte_acik() {
        let mut s = ornek_state();
        s.settings.guncelleme_kontrol = false;
        let geri = AppState::from_backup(&s.yedek_degeri()).unwrap();
        assert!(!geri.settings.guncelleme_kontrol);
        let mut v = s.yedek_degeri();
        v["ekstra"]
            .as_object_mut()
            .unwrap()
            .remove("guncelleme_kontrol");
        assert!(
            AppState::from_backup(&v)
                .unwrap()
                .settings
                .guncelleme_kontrol
        );
    }

    #[test]
    fn input_hatalari_yedege_yazilmaz() {
        let mut i = InputState::default();
        i.hatalar.push((Alan::Yil, "geçersiz sayı".into()));
        let v = serde_json::to_value(&i).unwrap();
        assert!(v.get("hatalar").is_none());
        assert!(
            serde_json::from_value::<InputState>(v)
                .unwrap()
                .hatalar
                .is_empty()
        );
    }
}
