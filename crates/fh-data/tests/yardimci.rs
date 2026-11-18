//! fh-data yardımcı işlevlerinin uçtan uca doğrulaması.
//! Yedek alma → temizleme → geri yükleme (P0 regresyon), CSV enjeksiyonu,
//! e-fatura içe aktarma, dönem yıllık özeti ve vergi takvimi.

use chrono::NaiveDate;
use fh_data::{
    DonemRecord, YedekAyarlari, YedekParca, csv_hucre, csv_metin, donem_ozet, efatura_matrahlar,
    gun_kaldi, kalan_devreden, takvim_listesi, temizle_yedek, yedek_olustur, yillara_gir,
};
use rust_decimal::{Decimal, RoundingStrategy};
use serde_json::{Value, json};
use std::str::FromStr;

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn gecmis(id: &str, alan: &str) -> Value {
    let mut v = json!({"id": id, "timestamp": 1731912345000i64, "mode": "kdv", "inputs": {}, "alt": "1200", "detay": "KDV"});
    v[alan] = json!("200");
    v
}

fn donem(id: &str) -> Value {
    json!({"id": id, "year": 2026, "month": 1, "satis": "1000", "alis": "400",
           "devreden_giris": "0", "hesaplanan": "200", "indirilecek": "80", "net": "120",
           "odenecek": true, "kdv_rate": "20", "cari": "c1", "closed_at": 1})
}

fn kayit(id: &str, yil: i32, ay: i32, hes: &str, ind: &str, net: &str, odenecek: bool, cari: &str) -> DonemRecord {
    DonemRecord {
        id: id.to_string(),
        year: yil,
        month: ay,
        satis: "0".into(),
        alis: "0".into(),
        devreden_giris: "0".into(),
        hesaplanan: hes.into(),
        indirilecek: ind.into(),
        net: net.into(),
        odenecek,
        kdv_rate: "20".into(),
        cari: cari.into(),
        closed_at: 0,
    }
}

// ------------------------------------------------------- Yedek round-trip (P0)

#[test]
fn y01_yedek_al_temizle_geri_yukle_round_trip() {
    // UI "ana" biçimi + eski Electron "result" biçimi birlikte; hiçbir kayıt kaybolmamalı.
    let parca = YedekParca {
        surum: Some("5.0.0".into()),
        tarih: Some("2026-11-18T00:00:00.000Z".into()),
        history: vec![gecmis("u1", "ana"), gecmis("e1", "result")],
        donemler: vec![donem("2026-01"), donem("2026-02")],
        cariler: vec![json!({"id": "c1", "name": "Müşteri A"})],
        active_cari: "c1".into(),
        ayarlar: Some(YedekAyarlari {
            show_currency_symbol: false,
            precision: 4,
            font_scale: 1.1,
        }),
        theme: Some("light".into()),
    };
    let yedek = yedek_olustur(&parca);
    let temiz = temizle_yedek(&yedek).expect("geçerli yedek");

    assert_eq!(temiz.history.len(), 2);
    assert_eq!(temiz.history[0]["ana"], "200");
    assert_eq!(temiz.history[1]["result"], "200");
    assert_eq!(temiz.donemler.len(), 2);
    assert_eq!(temiz.donemler[0]["net"], "120");
    assert_eq!(temiz.cariler.len(), 1);
    assert_eq!(temiz.cariler[0]["name"], "Müşteri A");
    assert_eq!(temiz.active_cari, "c1");
    assert!(!temiz.show_currency_symbol);
    assert_eq!(temiz.precision, 4);
    assert_eq!(temiz.font_scale, 1.1);
    assert_eq!(temiz.theme, "light");
}

#[test]
fn y02_p0_ana_ve_result_alanlari_korunur() {
    // P0 veri kaybı regresyonu: "ana" alanı eklenmeden önce yüklenen eski
    // yedeklerdeki "result" alanı da, yeni UI kayıtlarındaki "ana" da aynen dönmeli.
    let ui = json!({"id": "yeni", "ana": "150", "timestamp": 2, "mode": "kdv"});
    let eski = json!({"id": "eski", "result": "250", "timestamp": 1, "mode": "stopaj"});
    let yedek = yedek_olustur(&YedekParca {
        history: vec![ui, eski],
        ..YedekParca::default()
    });
    let temiz = temizle_yedek(&yedek).unwrap();
    assert_eq!(temiz.history.len(), 2);
    assert!(temiz.history.iter().any(|e| e["ana"] == "150"));
    assert!(temiz.history.iter().any(|e| e["result"] == "250"));
}

#[test]
fn y03_yabanci_yedek_red_ve_sanitizasyon() {
    assert!(temizle_yedek(&json!({"rastgele": 1})).is_none());
    let t = temizle_yedek(&json!({
        "uygulama": "finansal-hesaplayici",
        "history": [json!({"id": "x", "timestamp": "sayı-değil", "mode": "kdv", "ana": "1"})],
        "settings": {"precision": 3, "fontScale": 99},
        "theme": "mavi"
    })).unwrap();
    assert!(t.history.is_empty(), "bozuk timestamp ayıklanmalı");
    assert_eq!(t.precision, 2);
    assert_eq!(t.font_scale, 1.0);
    assert_eq!(t.theme, "dark");
}

// ------------------------------------------------------- CSV dışa aktarma

#[test]
fn y04_csv_enjeksiyon_ve_bicim() {
    // app.rs::gecmis_csv başlıklarıyla aynı biçim.
    let satirlar = vec![
        vec!["Tarih".into(), "Mod".into(), "Sonuç".into(), "Detay".into()],
        vec!["2026-11-18 10:30".into(), "KDV".into(), "200".into(),
             "=HYPERLINK(\"http://kötü\",\"tıkla\")".into()],
        vec!["2026-11-18 11:00".into(), "Stopaj".into(), "1.000,50".into(),
             "iki, virgül ve \"tırnak\"".into()],
    ];
    let metin = csv_metin(&satirlar);
    assert!(metin.starts_with('\u{FEFF}'), "Excel için BOM şart");
    assert!(metin.contains("\"'=HYPERLINK(\"\"http://kötü\"\",\"\"tıkla\"\")\""), "formül apostroflanmalı");
    assert!(metin.contains("\"iki, virgül ve \"\"tırnak\"\"\""), "tırnaklar kaçırılmalı");
    assert_eq!(metin.matches('\n').count(), 2);
}

#[test]
fn y05_csv_hucre_enjeksiyon_onekleri() {
    for p in ["=1+1", "+SUM(A1)", "-30", "@cmd", "\tX", "\rX"] {
        assert!(csv_hucre(p).starts_with("\"'"), "{p}");
    }
    assert_eq!(csv_hucre("normal"), "\"normal\"");
    assert_eq!(csv_satir_uzunlugu(&["a".into(), "b".into()]), "\"a\",\"b\"");
}

#[test]
fn y05b_csv_enjeksiyon_bosluk_atlatmali() {
    // Excel baştaki boşlukları yok sayar; " =1+1" de formül olarak çalışır.
    for p in [" =1+1", "   @SUM(1)", "\n=2+2", " \r+HYPERLINK(\"x\")"] {
        let h = csv_hucre(p);
        assert!(h.starts_with("\"'"), "{p:?} → {h}");
    }
    assert_eq!(csv_hucre("  güvenli"), "\"  güvenli\"");
}

fn csv_satir_uzunlugu(hucreler: &[String]) -> String {
    fh_data::csv_satir(hucreler)
}

// ------------------------------------------------------- e-fatura içe aktarma

#[test]
fn y06_efatura_gib_csv_ice_aktarma() {
    // BOM + noktalı virgül + binlik nokta + virgüllü ondalık + tırnaklı alan.
    let icerik = "\u{FEFF}Tarih;Açıklama;KDV;Matrah\n\
                  2026-01-01;Hizmet;%20;1.234,56\n\
                  2026-01-02;\"Mal, çeşit\";%20;2.500,00\n\
                  2026-01-03;Hizmet;%20;0\n";
    let matrahlar = efatura_matrahlar(icerik);
    assert_eq!(matrahlar, vec!["1234.56", "2500.00"], "sıfır matrah atlanmalı");

    // İçe aktarılan matrahlar toplu moda beslenir: %20 KDV → 4481.47.
    let toplam: Decimal = matrahlar.iter().map(|s| d(s)).sum();
    let kdv = (toplam * d("20") / d("100"))
        .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    assert_eq!(kdv, d("746.91"));
    assert_eq!(toplam + kdv, d("4481.47"));
}

#[test]
fn y07_efatura_basliksiz_ve_tek_sutun() {
    assert_eq!(efatura_matrahlar("1000\n2000.50\nabc"), vec!["1000", "2000.50"]);
    assert_eq!(efatura_matrahlar(""), Vec::<String>::new());
}

#[test]
fn y07b_efatura_baslik_paritesi_electron() {
    // Electron efatura.ts MATRAH_IPUCU / findIndex ile birebir aynı davranış:
    // "Mal/Hizmet" ilk eşleştiği için matrah sütunu olarak seçilir (leaded header
    // varsa açıklama sütunundaki sayılar okunur). Bu bir Rust regresyonu değil,
    // eski uygulamanın kasten korunan davranışıdır.
    let s = "Tarih;Mal/Hizmet;KDV;Matrah\n2026-01-01;Hizmet;%20;1234,56";
    assert_eq!(efatura_matrahlar(s), Vec::<String>::new());
    // "Mal/Hizmet" yoksa "Matrah" doğru seçilir.
    let s2 = "Tarih;Açıklama;KDV;Matrah\n2026-01-01;Hizmet;%20;1234,56";
    assert_eq!(efatura_matrahlar(s2), vec!["1234.56"]);
}

// ------------------------------------------------------- Dönem yıllık özet

#[test]
fn y08_donem_yillik_ozet_12_ay() {
    let mut kayitlar = Vec::new();
    for ay in 1..=12 {
        let (hes, ind, net, odenecek) = if ay <= 6 {
            ("1000", "400", "600", true)
        } else {
            ("500", "800", "-300", false)
        };
        kayitlar.push(kayit(&format!("2026-{ay:02}"), 2026, ay, hes, ind, net, odenecek, "c1"));
    }
    let gruplar = yillara_gir(&kayitlar);
    assert_eq!(gruplar.len(), 1);
    assert_eq!(gruplar[0].0, 2026);
    assert_eq!(gruplar[0].1.len(), 12);

    let o = &gruplar[0].2;
    assert_eq!(o.sayi, 12);
    assert_eq!(o.hesaplanan, "9000");
    assert_eq!(o.indirilecek, "7200");
    assert_eq!(o.net, "1800");
    assert_eq!(o.odenecek_net, "3600");
    assert_eq!(o.devreden_net, "1800");
    // En yeni dönem en önde.
    assert_eq!(gruplar[0].1[0].id, "2026-12");
    assert_eq!(donem_ozet(&kayitlar), *o);
}

#[test]
fn y09_donemler_arasi_yillik_ayrim() {
    let kayitlar = vec![
        kayit("2025-12", 2025, 12, "100", "50", "50", true, ""),
        kayit("2026-01", 2026, 1, "200", "80", "120", true, "c2"),
        kayit("2026-02", 2026, 2, "300", "100", "200", true, "c2"),
    ];
    let g = yillara_gir(&kayitlar);
    assert_eq!(g.len(), 2);
    assert_eq!(g[0].0, 2026);
    assert_eq!(g[0].2.net, "320");
    assert_eq!(g[1].0, 2025);
    assert_eq!(g[1].2.net, "50");
}

// ------------------------------------------------------- Cari filtresi

#[test]
fn y10_cari_filtresi_donemleri_ayirir() {
    // app.rs::donemler_penceresi ile aynı yükleme: filtre boşsa tümü,
    // doluysa yalnızca o cariye ait kayıtlar.
    let kayitlar = vec![
        kayit("2026-01", 2026, 1, "200", "80", "120", true, "c1"),
        kayit("2026-02", 2026, 2, "50", "90", "-40", false, "c2"),
        kayit("2026-03", 2026, 3, "300", "100", "200", true, "c1"),
    ];
    let filtrele = |f: &str| -> Vec<DonemRecord> {
        kayitlar.iter().filter(|d| f.is_empty() || d.cari == f).cloned().collect()
    };
    assert_eq!(filtrele("").len(), 3);
    assert_eq!(filtrele("c1").len(), 2);
    assert_eq!(filtrele("c2").len(), 1);
    assert_eq!(filtrele("yok").len(), 0);
    assert_eq!(donem_ozet(&filtrele("c1")).net, "320");
    assert_eq!(kalan_devreden(&filtrele("c2")).unwrap(), "40");
}

// ------------------------------------------------------- Vergi takvimi

#[test]
fn y11_gecici_vergi_dortuncu_ceyrek_subat_15() {
    for yil in [2024, 2025, 2026, 2027] {
        let bugun = NaiveDate::from_ymd_opt(yil, 1, 1).unwrap();
        let liste = takvim_listesi(bugun);
        let gvv = liste
            .iter()
            .find(|k| k.tip == "gVV" && k.tarih == format!("{yil}-02-15"))
            .unwrap_or_else(|| panic!("{yil} Şubat 15 geçici vergi kaydı yok"));
        assert_eq!(gvv.donemi, format!("4. çeyrek (Ekim–Aralık) {}", yil - 1));
        assert_eq!(gvv.id, format!("gvv-{yil}-2"));
    }
}

#[test]
fn y12_gecici_vergi_diger_ceyrekler() {
    let bugun = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let liste = takvim_listesi(bugun);
    for (tarih, donem) in [
        ("2026-05-15", "1. çeyrek (Ocak–Mart) 2026"),
        ("2026-08-15", "2. çeyrek (Nisan–Haziran) 2026"),
        ("2026-11-15", "3. çeyrek (Temmuz–Eylül) 2026"),
    ] {
        let k = liste.iter().find(|k| k.tip == "gVV" && k.tarih == tarih)
            .unwrap_or_else(|| panic!("{tarih} kaydı yok"));
        assert_eq!(k.donemi, donem);
    }
    // Takvim sıralı ve geçmiş tarihler atılmış.
    for w in liste.windows(2) {
        assert!(w[0].tarih <= w[1].tarih);
    }
    assert!(liste.iter().all(|k| k.tarih.as_str() >= "2026-01-01"));
    assert_eq!(gun_kaldi("2026-02-15", bugun), 45);
}
