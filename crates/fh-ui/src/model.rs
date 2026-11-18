//! Kalıcı uygulama durumu. Tüm veri %APPDATA% altında tek bir JSON dosyasında
//! tutulur; okuma `fh_data::temizle_yedek` ile sanitize edilir.

use std::path::PathBuf;

use fh_data::{DonemRecord, TemizYedek, YedekAyarlari, YedekParca, temizle_yedek, yedek_olustur};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Tek sürüm kaynağı: Cargo workspace sürümü (Kurulum/CI ile aynı değer).
pub const SURUM: &str = env!("CARGO_PKG_VERSION");
pub const GECMIS_SINIRI: usize = 100;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// "dark" | "light" | "system"
    pub theme: String,
    pub font_scale: f64,
    pub show_currency_symbol: bool,
    /// 0 | 2 | 4
    pub precision: usize,
    pub currency_symbol: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_scale: 1.0,
            show_currency_symbol: true,
            precision: 2,
            currency_symbol: "₺".to_string(),
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
        inputs: v.get("inputs").and_then(value_or::<InputState>).unwrap_or_default(),
        ana: ana.to_string(),
        alt: v.get("alt").and_then(Value::as_str).unwrap_or("").to_string(),
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
        satis: v.get("satis").and_then(Value::as_str).unwrap_or("0").to_string(),
        alis: v.get("alis").and_then(Value::as_str).unwrap_or("0").to_string(),
        devreden_giris: v
            .get("devreden_giris")
            .or_else(|| v.get("devredenGiris"))
            .and_then(Value::as_str)
            .unwrap_or("0")
            .to_string(),
        hesaplanan: v.get("hesaplanan").and_then(Value::as_str).unwrap_or("0").to_string(),
        indirilecek: v.get("indirilecek").and_then(Value::as_str).unwrap_or("0").to_string(),
        net: v.get("net").and_then(Value::as_str).unwrap_or("0").to_string(),
        odenecek: v.get("odenecek").and_then(Value::as_bool).unwrap_or(true),
        kdv_rate: v
            .get("kdv_rate")
            .or_else(|| v.get("kdvRate"))
            .and_then(Value::as_str)
            .unwrap_or("20")
            .to_string(),
        cari: v.get("cari").and_then(Value::as_str).unwrap_or("").to_string(),
        closed_at: v
            .get("closed_at")
            .or_else(|| v.get("closedAt"))
            .and_then(Value::as_i64)
            .unwrap_or(0),
    })
}

fn temiz_settings(t: &TemizYedek, sembol: String) -> Settings {
    Settings {
        theme: t.theme.clone(),
        font_scale: t.font_scale,
        show_currency_symbol: t.show_currency_symbol,
        precision: t.precision,
        currency_symbol: sembol,
    }
}

impl AppState {
    pub fn from_backup(v: &Value) -> Option<Self> {
        let t = temizle_yedek(v)?;
        let sembol = v
            .get("ekstra")
            .and_then(|e| e.get("currency_symbol"))
            .and_then(Value::as_str)
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
            settings: temiz_settings(&t, sembol),
            history: t.history.iter().filter_map(value_or_hist).collect(),
            donemler: t.donemler.iter().filter_map(value_or_donem).collect(),
            cariler: t.cariler.iter().filter_map(value_or).collect(),
            active_cari: t.active_cari,
            inputs,
        })
    }

    /// `%APPDATA%/FinansalHesaplayici/veri.json`
    pub fn dosya_yolu() -> PathBuf {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("FinansalHesaplayici").join("veri.json")
    }

    pub fn yukle() -> Self {
        let fallback = Self {
            settings: Settings::default(),
            history: Vec::new(),
            donemler: Vec::new(),
            cariler: Vec::new(),
            active_cari: String::new(),
            inputs: std::collections::HashMap::new(),
        };
        let Ok(metin) = std::fs::read_to_string(Self::dosya_yolu()) else {
            return fallback;
        };
        let Ok(value) = serde_json::from_str::<Value>(&metin) else {
            return fallback;
        };
        Self::from_backup(&value).unwrap_or(fallback)
    }

    pub fn yedek_degeri(&self) -> Value {
        let p = YedekParca {
            surum: Some(SURUM.to_string()),
            tarih: None,
            history: self.history.iter().filter_map(|e| serde_json::to_value(e).ok()).collect(),
            donemler: self.donemler.iter().filter_map(|d| serde_json::to_value(d).ok()).collect(),
            cariler: self.cariler.iter().filter_map(|c| serde_json::to_value(c).ok()).collect(),
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
            obj.insert("ekstra".to_string(), json!({"currency_symbol": self.settings.currency_symbol}));
            let mut inputs = serde_json::Map::new();
            for (k, i) in &self.inputs {
                if let Ok(iv) = serde_json::to_value(i) {
                    inputs.insert(k.clone(), iv);
                }
            }
            obj.insert("inputs".to_string(), Value::Object(inputs));
        }
        v
    }

    pub fn kaydet(&self) {
        let path = Self::dosya_yolu();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(metin) = serde_json::to_string_pretty(&self.yedek_degeri()) {
            let _ = std::fs::write(path, metin);
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

/// Etiketi `%` içerse de değeri para olan satırlar (fh-core `modes.rs` üretir).
/// Bunlar yüzde olarak biçimlendirilirse `%200,00` gibi yanlış bir çıktı oluşur.
const YUZDE_ETIKETLI_PARA_SATIRLARI: [&str; 2] = ["% KDV", "% Kesinti"];

/// Sonuç tablosu satırı: gerçek yüzde satırları `%`, diğerleri para olarak gösterilir.
pub fn satir_bicimi(label: &str, value: &str, precision: usize, sembol: &str, goster: bool) -> String {
    let Some(d) = fh_core::parse_tr(value) else {
        return value.to_string();
    };
    let etiket = label.trim();
    let yuzde_mi = !YUZDE_ETIKETLI_PARA_SATIRLARI.contains(&etiket)
        && (etiket.contains('%')
            || etiket.contains("Oran")
            || etiket.contains("Marj")
            || etiket.contains("Değişim")
            || etiket.contains("Efektif"));
    if yuzde_mi {
        format!("%{}", fh_core::format_tr(d, precision))
    } else {
        para_bicimi(value, precision, sembol, goster)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        inputs.insert("kdv".to_string(), InputState {
            birinci: "1000".into(),
            oran: "20".into(),
            secim: "haric".into(),
            ..InputState::default()
        });
        AppState {
            settings: Settings {
                theme: "light".into(),
                font_scale: 1.1,
                show_currency_symbol: false,
                precision: 4,
                currency_symbol: "$".into(),
            },
            history,
            donemler: vec![ornek_donem("2026-01", "c1"), ornek_donem("2026-02", "c2")],
            cariler: vec![
                Cari { id: "c1".into(), name: "Müşteri A".into() },
                Cari { id: "c2".into(), name: "Müşteri B".into() },
            ],
            active_cari: "c1".into(),
            inputs,
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
            state.donemler.iter()
                .filter(|d| f.is_empty() || d.cari == f)
                .cloned()
                .collect()
        };
        assert_eq!(filtrele("").len(), 2);
        assert_eq!(filtrele("c1").len(), 1);
        assert_eq!(filtrele("c1")[0].id, "2026-01");
        assert_eq!(filtrele("yok").len(), 0);
        // Filtrelenen kayıtların cari adı çözümlenebiliyor.
        let ad = state.cariler.iter().find(|c| c.id == filtrele("c2")[0].cari).unwrap().name.clone();
        assert_eq!(ad, "Müşteri B");
    }

    #[test]
    fn satir_bicimi_kenar_durumlari() {
        assert_eq!(satir_bicimi("Efektif Oran", "12.5", 2, "₺", false), "%12,50");
        assert_eq!(satir_bicimi("Değişim %", "-96", 2, "₺", false), "%-96,00");
        assert_eq!(satir_bicimi("Kur (1 USD)", "40", 4, "₺", true), "40,0000 ₺");
        assert_eq!(satir_bicimi("Geçersiz", "abc", 2, "₺", true), "abc");
    }
}
