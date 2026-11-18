//! Yedek dosyası (JSON) sanitizasyonu — dışarıdan gelen veride güven zinciri:
//! bilinmeyen alanları at, bozuk kayıtları filtrele, sınırlara kilitle.
//! Electron `src/lib/yedek.ts` portu.

use chrono::SecondsFormat;
use serde_json::{Value, json};

pub const SINIR_HISTORY: usize = 100;
pub const SINIR_DONEMLER: usize = 240;
pub const SINIR_CARILER: usize = 500;

#[derive(Debug, Clone, PartialEq)]
pub struct TemizYedek {
    pub history: Vec<Value>,
    pub donemler: Vec<Value>,
    pub cariler: Vec<Value>,
    pub active_cari: String,
    pub show_currency_symbol: bool,
    pub precision: usize,
    pub font_scale: f64,
    pub theme: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct YedekAyarlari {
    pub show_currency_symbol: bool,
    pub precision: usize,
    pub font_scale: f64,
}

/// `yedek_olustur` için parça; alanlar opsiyonel, varsayılanlar TS ile aynı.
#[derive(Debug, Clone, Default)]
pub struct YedekParca {
    pub surum: Option<String>,
    pub tarih: Option<String>,
    pub history: Vec<Value>,
    pub donemler: Vec<Value>,
    pub cariler: Vec<Value>,
    pub active_cari: String,
    pub ayarlar: Option<YedekAyarlari>,
    pub theme: Option<String>,
}

fn arr<'a>(v: &'a Value, key: &str) -> Vec<&'a Value> {
    v.get(key)
        .and_then(|x| x.as_array())
        .map(|a| a.iter().collect())
        .unwrap_or_default()
}

/// Geçmiş kaydı iki biçimde de kabul et:
/// - Rust UI (fh-ui `HistEntry`): `"ana"` alanı,
/// - eski Electron yedekleri: `"result"` alanı.
/// `id` / `mode` / `timestamp` doğrulaması iki biçimde de aynen korunur.
fn gecerli_gecmis(e: &Value) -> bool {
    let sonuc_var = e.get("ana").is_some_and(Value::is_string)
        || e.get("result").is_some_and(Value::is_string);
    e.is_object()
        && e.get("id").is_some_and(Value::is_string)
        && sonuc_var
        && e.get("timestamp").is_some_and(Value::is_number)
        && e.get("mode").is_some_and(Value::is_string)
}

fn donem_id_gecerli(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 7
        && b[4] == b'-'
        && b[0..4].iter().all(|c| c.is_ascii_digit())
        && b[5..7].iter().all(|c| c.is_ascii_digit())
}

fn gecerli_donem(e: &Value) -> bool {
    e.is_object()
        && e.get("id")
            .and_then(Value::as_str)
            .is_some_and(donem_id_gecerli)
        && e.get("year").is_some_and(Value::is_number)
        && e.get("month").is_some_and(Value::is_number)
        && e.get("hesaplanan").is_some_and(Value::is_string)
        && e.get("indirilecek").is_some_and(Value::is_string)
        && e.get("net").is_some_and(Value::is_string)
        && e.get("odenecek").is_some_and(|v| v.is_boolean())
}

fn gecerli_cari(e: &Value) -> bool {
    e.is_object()
        && e.get("id").is_some_and(Value::is_string)
        && e.get("name").is_some_and(Value::is_string)
}

/// Geçerli bir Finansal Hesaplayici yedeği ise temizlenmiş halini, değilse None.
pub fn temizle_yedek(v: &Value) -> Option<TemizYedek> {
    if !v.is_object() {
        return None;
    }
    let yedek_izi = ["history", "donemler", "cariler"]
        .iter()
        .any(|k| v.get(*k).is_some_and(Value::is_array))
        || v.get("uygulama").and_then(Value::as_str) == Some("finansal-hesaplayici");
    if !yedek_izi {
        return None;
    }

    let s = v.get("settings").filter(|x| x.is_object());
    // Eski Electron yedeklerinde alan adları camelCase, bazı el yazımı
    // yedeklerde ise snake_case olabilir. Üç biçim de aynı şekilde okunur.
    let ayar = |ad: &str| s.and_then(|x| x.get(ad));
    let show_currency_symbol = ayar("showCurrencySymbol")
        .or_else(|| ayar("show_currency_symbol"))
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let precision = ayar("precision")
        .and_then(Value::as_f64)
        .filter(|n| n.fract() == 0.0 && matches!(*n as i64, 0 | 2 | 4))
        .map(|n| n as usize)
        .unwrap_or(2);
    let font_scale = ayar("fontScale")
        .or_else(|| ayar("font_scale"))
        .and_then(Value::as_f64)
        .filter(|f| f.is_finite() && *f >= 0.85 && *f <= 1.25)
        .unwrap_or(1.0);
    let theme = v
        .get("theme")
        .and_then(Value::as_str)
        .or_else(|| ayar("theme").and_then(Value::as_str))
        .filter(|t| ["dark", "light", "system"].contains(t))
        .unwrap_or("dark")
        .to_string();

    let history = arr(v, "history")
        .into_iter()
        .filter(|e| gecerli_gecmis(e))
        .take(SINIR_HISTORY)
        .cloned()
        .collect();
    let donemler = arr(v, "donemler")
        .into_iter()
        .filter(|e| gecerli_donem(e))
        .take(SINIR_DONEMLER)
        .cloned()
        .collect();
    let cariler = arr(v, "cariler")
        .into_iter()
        .filter(|e| gecerli_cari(e))
        .take(SINIR_CARILER)
        .cloned()
        .collect();

    Some(TemizYedek {
        history,
        donemler,
        cariler,
        active_cari: v
            .get("activeCari")
            .or_else(|| v.get("active_cari"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        show_currency_symbol,
        precision,
        font_scale,
        theme,
    })
}

/// Mevcut state'ten yedek sözlüğü üretir (Settings dışa aktarma biçimi).
pub fn yedek_olustur(parca: &YedekParca) -> Value {
    let tarih = parca
        .tarih
        .clone()
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true));
    let settings = match parca.ayarlar {
        Some(a) => json!({
            "showCurrencySymbol": a.show_currency_symbol,
            "precision": a.precision,
            "fontScale": a.font_scale,
        }),
        None => Value::Null,
    };
    json!({
        "uygulama": "finansal-hesaplayici",
        "surum": parca.surum,
        "tarih": tarih,
        "history": parca.history,
        "donemler": parca.donemler,
        "cariler": parca.cariler,
        "activeCari": parca.active_cari,
        "settings": settings,
        "theme": parca.theme.as_deref().unwrap_or("dark"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gecmis(id: &str) -> Value {
        json!({"id": id, "result": "1", "timestamp": 1, "mode": "kdv", "ekstra": 9})
    }
    fn donem(id: &str) -> Value {
        json!({"id": id, "year": 2026, "month": 1, "hesaplanan": "1",
               "indirilecek": "0", "net": "1", "odenecek": true})
    }
    fn cari(id: &str) -> Value {
        json!({"id": id, "name": "Müşteri"})
    }

    #[test]
    fn yedek_olmadigi_ic_none() {
        assert!(temizle_yedek(&json!({"a": 1})).is_none());
        assert!(temizle_yedek(&Value::Null).is_none());
    }

    #[test]
    fn uygulama_imzasi_yeterli() {
        let t = temizle_yedek(&json!({"uygulama": "finansal-hesaplayici"})).unwrap();
        assert!(t.history.is_empty());
        assert_eq!(t.theme, "dark");
    }

    #[test]
    fn bozuk_kayitlar_ayiklanir() {
        let v = json!({
            "history": [gecmis("a"), json!({"id": "x"}), json!("bozuk"), gecmis("b")],
            "donemler": [donem("2026-01"), json!({"id": "2026-1"}), json!({"id": "2026-02"})],
            "cariler": [cari("c"), json!({"id": "sadece-id"}), cari("d")],
        });
        let t = temizle_yedek(&v).unwrap();
        assert_eq!(t.history.len(), 2);
        assert_eq!(t.donemler.len(), 1);
        assert_eq!(t.cariler.len(), 2);
    }

    #[test]
    fn sınırlar_100_240_500() {
        let h: Vec<Value> = (0..150).map(|i| gecmis(&format!("h{i}"))).collect();
        let d: Vec<Value> = (0..300).map(|i| donem(&format!("2026-{:02}", i % 12 + 1))).collect();
        let c: Vec<Value> = (0..600).map(|i| cari(&format!("c{i}"))).collect();
        let t = temizle_yedek(&json!({"history": h, "donemler": d, "cariler": c})).unwrap();
        assert_eq!(t.history.len(), 100);
        assert_eq!(t.donemler.len(), 240);
        assert_eq!(t.cariler.len(), 500);
    }

    #[test]
    fn bozuk_ayarlar_varsayilana_duser() {
        let t = temizle_yedek(&json!({
            "history": [],
            "settings": {"showCurrencySymbol": "evet", "precision": 3, "fontScale": 9},
            "theme": "mavi",
            "activeCari": 42,
        })).unwrap();
        assert!(t.show_currency_symbol);
        assert_eq!(t.precision, 2);
        assert_eq!(t.font_scale, 1.0);
        assert_eq!(t.theme, "dark");
        assert_eq!(t.active_cari, "");
    }

    #[test]
    fn gecerli_ayarlar_korunur() {
        let t = temizle_yedek(&json!({
            "cariler": [],
            "activeCari": "c1",
            "settings": {"showCurrencySymbol": false, "precision": 4, "fontScale": 1.1},
            "theme": "light",
        })).unwrap();
        assert!(!t.show_currency_symbol);
        assert_eq!(t.precision, 4);
        assert_eq!(t.font_scale, 1.1);
        assert_eq!(t.theme, "light");
        assert_eq!(t.active_cari, "c1");
    }

    #[test]
    fn eski_ve_snake_case_ayar_adlari_okunur() {
        // Eski Electron: tema settings içinde, diğer alanlar camelCase.
        let eski = temizle_yedek(&json!({
            "history": [],
            "settings": {"showCurrencySymbol": false, "precision": 4,
                          "fontScale": 1.2, "theme": "light"},
            "activeCari": "eski"
        })).unwrap();
        assert_eq!(eski.theme, "light");
        assert_eq!(eski.font_scale, 1.2);
        assert_eq!(eski.precision, 4);
        assert!(!eski.show_currency_symbol);
        assert_eq!(eski.active_cari, "eski");

        // El yazımı yedekler: snake_case alan adları.
        let yeni = temizle_yedek(&json!({
            "cariler": [],
            "settings": {"show_currency_symbol": false, "precision": 0,
                          "font_scale": 0.9, "theme": "system"},
            "active_cari": "yeni"
        })).unwrap();
        assert_eq!(yeni.theme, "system");
        assert_eq!(yeni.font_scale, 0.9);
        assert_eq!(yeni.precision, 0);
        assert!(!yeni.show_currency_symbol);
        assert_eq!(yeni.active_cari, "yeni");
    }

    #[test]
    fn font_scale_sinirlari() {
        let low = temizle_yedek(&json!({"history": [], "settings": {"fontScale": 0.5}})).unwrap();
        let high = temizle_yedek(&json!({"history": [], "settings": {"fontScale": 2.0}})).unwrap();
        assert_eq!(low.font_scale, 1.0);
        assert_eq!(high.font_scale, 1.0);
    }

    #[test]
    fn yedek_olustur_bicimi() {
        let p = YedekParca {
            surum: Some("5.0.0".to_string()),
            tarih: Some("2026-11-18T00:00:00.000Z".to_string()),
            history: vec![gecmis("a")],
            donemler: vec![donem("2026-01")],
            cariler: vec![cari("c")],
            active_cari: "c".to_string(),
            ayarlar: Some(YedekAyarlari {
                show_currency_symbol: true,
                precision: 2,
                font_scale: 1.0,
            }),
            theme: Some("dark".to_string()),
        };
        let y = yedek_olustur(&p);
        assert_eq!(y["uygulama"], "finansal-hesaplayici");
        assert_eq!(y["surum"], "5.0.0");
        assert_eq!(y["tarih"], "2026-11-18T00:00:00.000Z");
        assert_eq!(y["history"][0]["id"], "a");
        assert_eq!(y["donemler"].as_array().unwrap().len(), 1);
        assert_eq!(y["cariler"].as_array().unwrap().len(), 1);
        assert_eq!(y["activeCari"], "c");
        assert_eq!(y["settings"]["precision"], 2);
        assert_eq!(y["theme"], "dark");
        let temiz = temizle_yedek(&y).unwrap();
        assert_eq!(temiz.history.len(), 1);
    }

    #[test]
    fn yedek_olustur_varsayilanlari() {
        let y = yedek_olustur(&YedekParca::default());
        assert_eq!(y["theme"], "dark");
        assert_eq!(y["activeCari"], "");
        assert!(y["tarih"].as_str().unwrap().ends_with('Z'));
        assert!(y["settings"].is_null());
    }

    // --- P0: "ana" / "result" alan uyumluluğu ---

    #[test]
    fn ui_bicimi_ana_kaydi_korunur() {
        // fh-ui HistEntry: {id, timestamp, mode, inputs, ana, alt, detay}
        let ui = json!({"id": "u1", "timestamp": 1731912345000i64, "mode": "kdv",
                        "inputs": {}, "ana": "200", "alt": "1200", "detay": "..."});
        let t = temizle_yedek(&json!({"history": [ui]})).unwrap();
        assert_eq!(t.history.len(), 1);
        assert_eq!(t.history[0]["ana"], "200");
    }

    #[test]
    fn eski_electron_bicimi_result_kaydi_korunur() {
        let eski = json!({"id": "e1", "result": "1", "timestamp": 1, "mode": "kdv"});
        let t = temizle_yedek(&json!({"history": [eski]})).unwrap();
        assert_eq!(t.history.len(), 1);
        assert_eq!(t.history[0]["result"], "1");
    }

    #[test]
    fn iki_bicim_birlikte_veya_tek_baslikli_bozuk_kayit_ayiklanir() {
        let ui = json!({"id": "u1", "ana": "5", "timestamp": 2, "mode": "kdv"});
        let eski = json!({"id": "e1", "result": "6", "timestamp": 3, "mode": "stopaj"});
        let eksik_ana = json!({"id": "x", "timestamp": 4, "mode": "kdv"});
        let yanlis_tip = json!({"id": "y", "ana": 7, "timestamp": 5, "mode": "kdv"});
        let eksik_id = json!({"ana": "8", "timestamp": 6, "mode": "kdv"});
        let eksik_ts = json!({"id": "z", "ana": "9", "mode": "kdv"});
        let eksik_mode = json!({"id": "q", "ana": "10", "timestamp": 7});
        let t = temizle_yedek(&json!({"history": [
            ui, eski, eksik_ana, yanlis_tip, eksik_id, eksik_ts, eksik_mode, json!("bozuk")
        ]})).unwrap();
        assert_eq!(t.history.len(), 2, "yalnızca iki geçerli biçim kalmalı");
        assert_eq!(t.history[0]["ana"], "5");
        assert_eq!(t.history[1]["result"], "6");
    }

    #[test]
    fn ui_bicimi_round_trip() {
        let ui = json!({"id": "rt", "timestamp": 99, "mode": "percent",
                        "inputs": {"birinci": "20"}, "ana": "100", "alt": "416.67",
                        "detay": "detay", "ekstra": 9});
        let y = yedek_olustur(&YedekParca {
            history: vec![ui.clone()],
            ..YedekParca::default()
        });
        let temiz = temizle_yedek(&y).unwrap();
        assert_eq!(temiz.history.len(), 1);
        assert_eq!(temiz.history[0], ui, "alanlar birebir geri gelmeli");
        assert_eq!(temiz.history[0]["ana"], "100");
    }
}
