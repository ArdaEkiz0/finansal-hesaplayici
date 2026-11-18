//! 15 modun tamamı için mod bazlı entegrasyon testleri.
//! Beklenen değerler eski Electron `src/core/engine.ts` / `src/store/useStore.ts`
//! formüllerinden türetilmiştir (git: acb0b49, Rust geçişi öncesi son sürüm).
//!
//! Bu dosya yalnızca doğrulama amaçlıdır; üretim kodunu değiştirmez.

use fh_core::{Girdi, HesapSonucu, hesapla};
use rust_decimal::Decimal;
use std::str::FromStr;

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn girdi(birinci: &str, ikinci: &str, oran: &str) -> Girdi {
    Girdi {
        birinci: birinci.to_string(),
        ikinci: ikinci.to_string(),
        oran: oran.to_string(),
        ..Girdi::default()
    }
}

fn sonuc(mod_: &str, g: &Girdi) -> HesapSonucu {
    hesapla(mod_, g).unwrap_or_else(|| panic!("{mod_} None döndü: {g:?}"))
}

fn satir(r: &HesapSonucu, etiket: &str) -> String {
    r.satirlar
        .iter()
        .find(|(k, _)| k == etiket)
        .unwrap_or_else(|| panic!("satır bulunamadı: {etiket}"))
        .1
        .clone()
}

// ---------------------------------------------------------------- KDV

#[test]
fn m01_kdv_hariciden_dahil() {
    // TS calculateKDV(1000, 20, false): tax=200, total=1200.
    let r = sonuc("kdv", &girdi("1000", "", "20"));
    assert_eq!(r.ana, "200");
    assert_eq!(r.alt, "1200");
    assert_eq!(satir(&r, "% KDV"), "200");
    assert_eq!(satir(&r, "Net (girdiğiniz)"), "1000");
    assert_eq!(satir(&r, "Toplam"), "1200");
}

#[test]
fn m02_kdv_dahilden_hariç() {
    // TS calculateKDV(1200, 20, true): net=1200/1.2=1000, tax=200.
    let mut g = girdi("1200", "dahil", "20");
    let r = sonuc("kdv", &g);
    assert_eq!(r.ana, "200");
    assert_eq!(r.alt, "1000");
    assert_eq!(satir(&r, "Net Tutar"), "1000");

    g.ikinci.clear();
    g.ek = vec!["extract".to_string()];
    assert_eq!(sonuc("kdv", &g).alt, "1000");
}

#[test]
fn m03_kdv_turkce_bicim_ve_ifade() {
    // parse_tr: "1.000,50" → 1000.50; "1000+200" → 1200.
    let r = sonuc("kdv", &girdi("1.000,50", "", "20"));
    assert_eq!(r.ana, "200.1");
    assert_eq!(r.alt, "1200.6");
    let r = sonuc("kdv", &girdi("1000+200", "", "20"));
    assert_eq!(r.ana, "240");
}

#[test]
fn m04_kdv_dahil_toplam_korunur() {
    // Yuvarlama kaçağı olmamalı: net + KDV = girilen brüt tutar.
    for (tutar, oran) in [("0.03", "20"), ("100", "20"), ("1234.56", "18"), ("999.99", "1")] {
        let g = girdi(tutar, "dahil", oran);
        let r = sonuc("kdv", &g);
        let net = d(&satir(&r, "Net Tutar"));
        let tax = d(&r.ana);
        assert_eq!(net + tax, d(tutar), "tutar={tutar} oran={oran}");
    }
}

// ---------------------------------------------------------------- Stopaj

#[test]
fn m05_stopaj_brutten_net() {
    // TS calculateStopaj(50000, 20): tax=10000, net=40000.
    let r = sonuc("stopaj", &girdi("50000", "", "20"));
    assert_eq!(r.ana, "10000");
    assert_eq!(r.alt, "40000");
    assert_eq!(satir(&r, "Net Ödenen"), "40000");
    assert_eq!(satir(&r, "Brüt (girdiğiniz)"), "50000");
}

#[test]
fn m06_stopaj_netten_brut() {
    // TS calculateStopajReverse(40000, 20): gross=50000, tax=10000.
    let g = Girdi {
        ek: vec!["netten".to_string()],
        ..girdi("40000", "", "20")
    };
    let r = sonuc("stopaj", &g);
    assert_eq!(r.ana, "10000");
    assert_eq!(r.alt, "50000");
    assert_eq!(satir(&r, "Brüt"), "50000");
}

#[test]
fn m07_stopaj_oran_15() {
    let r = sonuc("stopaj", &girdi("10000", "", "15"));
    assert_eq!(r.ana, "1500");
    assert_eq!(r.alt, "8500");
    // %100 netten → bölen sıfır, güvenli red.
    let g = Girdi {
        ek: vec!["netten".to_string()],
        ..girdi("1000", "", "100")
    };
    assert_eq!(hesapla("stopaj", &g), None);
}

// ---------------------------------------------------------------- Tevkifat

#[test]
fn m08_tevkifat_pay_5() {
    // TS calculateTevkifat(10000, 20, 5): kdv=2000, tevkifat=1000, satıcıya=11000.
    let g = Girdi {
        tevkifat_pay: 5,
        ..girdi("10000", "", "20")
    };
    let r = sonuc("tevkifat", &g);
    assert_eq!(r.ana, "1000");
    assert_eq!(r.alt, "11000");
    assert_eq!(satir(&r, "%20 KDV"), "2000");
    assert_eq!(satir(&r, "Tevkifat (5/10)"), "1000");
    assert_eq!(satir(&r, "2 No'lu Beyan"), "1000");
}

#[test]
fn m09_tevkifat_pay_10_ve_kod() {
    let g = Girdi {
        tevkifat_pay: 10,
        tevkifat_kod: "4A".to_string(),
        ..girdi("10000", "", "20")
    };
    let r = sonuc("tevkifat", &g);
    assert_eq!(r.ana, "2000");
    assert_eq!(r.alt, "10000");
    assert_eq!(satir(&r, "İşlem kodu"), "4A");
    assert!(r.detay.contains("Kod 4A"));
}

#[test]
fn m10_tevkifat_gecersiz_pay() {
    for pay in [0, 11, -1] {
        let g = Girdi {
            tevkifat_pay: pay,
            ..girdi("1000", "", "20")
        };
        assert_eq!(hesapla("tevkifat", &g), None, "pay={pay}");
    }
}

// ---------------------------------------------------------------- Fiyat

#[test]
fn m11_fiyat_haric() {
    // TS: matrah=1000, KDV%20 → 1200.
    assert_eq!(sonuc("fiyat", &girdi("1000", "", "20")).ana, "1200");
}

#[test]
fn m12_fiyat_dahil_ve_zincirleme_indirim() {
    // 1000 → %10 → 900 → %5 → 855; KDV dahil → net 855/1.2 = 712.5.
    let g = Girdi {
        ikinci: "dahil".to_string(),
        ek: vec!["10".to_string(), "5".to_string()],
        ..girdi("1000", "", "20")
    };
    let r = sonuc("fiyat", &g);
    assert_eq!(r.ana, "712.5");
    assert_eq!(satir(&r, "Matrah"), "855");
    assert_eq!(satir(&r, "Tasarruf"), "145");
    assert_eq!(satir(&r, "Adım 1 (%10)"), "900");
    assert_eq!(satir(&r, "Adım 2 (%5)"), "855");
}

#[test]
fn m13_fiyat_negatif_oran_red() {
    // KDV oranı -100 → bölen sıfır → None.
    assert_eq!(hesapla("fiyat", &girdi("1000", "", "-100")), None);
}

// ---------------------------------------------------------------- KDV karşılaştırma

#[test]
fn m14_kdv_compare() {
    // TS calculateKDVCompare(1000, 20): inject 200, extract 166.67, diff 33.33.
    let r = sonuc("kdvCompare", &girdi("1000", "", "20"));
    assert_eq!(r.ana, "33.33");
    assert_eq!(r.alt, "1200");
    assert_eq!(satir(&r, "Hariciden Dahil Toplam"), "1200");
    assert_eq!(satir(&r, "Dahilden Hariç Net"), "833.33");
    assert_eq!(satir(&r, "Dahildeki KDV"), "166.67");
    assert_eq!(satir(&r, "Haricideki KDV"), "200");
    assert_eq!(satir(&r, "KDV Farkı"), "33.33");
}

// ---------------------------------------------------------------- Marj / kâr

#[test]
fn m15_margin() {
    // TS calculateMargin(100, 20): sp=125, profit=25, mp=20.
    let r = sonuc("margin", &girdi("100", "20", ""));
    assert_eq!(r.ana, "125");
    assert_eq!(r.alt, "25");
    assert_eq!(d(&satir(&r, "Gerçek Marj")), d("20"));

    let r = sonuc("margin", &girdi("800", "25", ""));
    assert_eq!(r.ana, "1066.67");
    assert_eq!(r.alt, "266.67");
}

#[test]
fn m16_margin_gecersiz_hedef() {
    for hedef in ["100", "150", "-5"] {
        let r = sonuc("margin", &girdi("100", hedef, ""));
        assert_eq!(r.ana, "0", "hedef={hedef}");
    }
}

#[test]
fn m17_markup() {
    // TS calculateMarkup(100, 20): profit=20, sp=120.
    let g = girdi("100", "20", "");
    let r = sonuc("markup", &g);
    assert_eq!(r.ana, "120");
    assert_eq!(r.alt, "20");
    assert_eq!(satir(&r, "Maliyet"), "100");

    let r = sonuc("markup", &girdi("100", "-5", ""));
    assert_eq!(r.ana, "100");
    assert_eq!(r.alt, "0");
}

// ---------------------------------------------------------------- İndirim

#[test]
fn m18_discount_zincir() {
    // TS calculateChainedDiscount(1000, [10,10]): 810, saved 190.
    let g = Girdi {
        ek: vec!["10".to_string(), "10".to_string()],
        ..girdi("1000", "", "")
    };
    let r = sonuc("discount", &g);
    assert_eq!(r.ana, "810");
    assert_eq!(r.alt, "190");
    assert_eq!(satir(&r, "Adım 1 (%10)"), "900");
    assert_eq!(satir(&r, "Adım 2 (%10)"), "810");
    assert_eq!(d(&satir(&r, "Etkin İndirim")), d("19"));
}

#[test]
fn m19_discount_ikinci_alandan_oran() {
    let r = sonuc("discount", &girdi("500", "20", ""));
    assert_eq!(r.ana, "400");
    assert_eq!(r.alt, "100");
    assert_eq!(hesapla("discount", &girdi("1000", "", "")), None);
}

// ---------------------------------------------------------------- Yüzde

#[test]
fn m20_percent() {
    // TS calculatePercentOf(20, 500) = 100; calculatePercentChange = 2400.
    let r = sonuc("percent", &girdi("20", "500", ""));
    assert_eq!(r.ana, "100");
    assert_eq!(satir(&r, "Değişim %"), "2400");
    assert_eq!(satir(&r, "Ters Hesap"), "416.67");
}

#[test]
fn m21_percent_degisim_formulu() {
    // (new-old)/|old|*100 — eski TS calculatePercentChange birebir.
    for (eski, yeni, beklenen) in [("20", "500", "2400"), ("500", "20", "-96"), ("-20", "-30", "-50"), ("0", "500", "0")] {
        let r = sonuc("percent", &girdi(eski, yeni, ""));
        assert_eq!(satir(&r, "Değişim %"), beklenen, "{eski}→{yeni}");
    }
}

// ---------------------------------------------------------------- Bileşik faiz

#[test]
fn m22_compound_aylik() {
    // TS calculateCompoundInterest(1000, 12, 1, 12): 1000*(1+0.12/12)^12.
    let g = Girdi {
        ek: vec!["1".to_string(), "12".to_string()],
        ..girdi("1000", "12", "")
    };
    let r = sonuc("compound", &g);
    assert_eq!(r.ana, "1126.83");
    assert_eq!(r.alt, "126.83");
}

#[test]
fn m23_compound_yillik_ve_cok_yil() {
    // freq=1 → 1000*1.12 = 1120; 2 yıl %100 → 4000.
    let g = Girdi {
        ek: vec!["1".to_string(), "1".to_string()],
        ..girdi("1000", "12", "")
    };
    assert_eq!(sonuc("compound", &g).ana, "1120");

    let g = Girdi {
        ek: vec!["2".to_string(), "1".to_string()],
        ..girdi("1000", "100", "")
    };
    assert_eq!(sonuc("compound", &g).ana, "4000");
}

#[test]
fn m24_compound_gecersiz() {
    let g = Girdi {
        ek: vec!["0".to_string(), "12".to_string()],
        ..girdi("1000", "12", "")
    };
    assert_eq!(hesapla("compound", &g), None);
    let g = Girdi {
        ek: vec!["1".to_string(), "0".to_string()],
        ..girdi("1000", "12", "")
    };
    assert_eq!(hesapla("compound", &g), None);
}

// ---------------------------------------------------------------- Gecikme faizi

#[test]
fn m25_gecikme_30_gun() {
    // TS: 10000*4.5/100*30/30 = 450; total 10450.
    let g = Girdi {
        gun: 30,
        ..girdi("10000", "4.5", "")
    };
    let r = sonuc("gecikme", &g);
    assert_eq!(r.ana, "450");
    assert_eq!(r.alt, "10450");
    assert_eq!(satir(&r, "Toplam (anapara + faiz)"), "10450");
}

#[test]
fn m26_gecikme_yarim_ay_ve_varsayilan() {
    let g = Girdi {
        gun: 15,
        ..girdi("10000", "4.5", "")
    };
    assert_eq!(sonuc("gecikme", &g).ana, "225");
    // gun/ek boş → varsayılan 30 gün.
    assert_eq!(sonuc("gecikme", &girdi("10000", "4.5", "")).ana, "450");
    let g = Girdi {
        ek: vec!["-1".to_string()],
        ..girdi("10000", "4.5", "")
    };
    assert_eq!(hesapla("gecikme", &g), None);
}

// ---------------------------------------------------------------- Amortisman

#[test]
fn m27_amortisman_normal() {
    // TS calculateAmortisman(500000, 20, 5, normal): yıllık 100000, toplam 500000.
    let g = Girdi {
        yil: 5,
        ..girdi("500000", "", "20")
    };
    let r = sonuc("amortisman", &g);
    assert_eq!(r.ana, "100000");
    assert_eq!(r.alt, "500000");
    assert_eq!(satir(&r, "Yıllık ayrılan"), "100000");
    assert_eq!(satir(&r, "Toplam ayrılan"), "500000");
    assert_eq!(r.satirlar.len(), 3 + 5);
}

#[test]
fn m28_amortisman_oran_yildan() {
    let g = Girdi {
        yil: 5,
        ..girdi("500000", "", "")
    };
    assert_eq!(sonuc("amortisman", &g).ana, "100000");
}

#[test]
fn m29_amortisman_azalan() {
    // 1000 @%20, 3 yıl, azalan: 400 + 240 + 144 = 784.
    let g = Girdi {
        yil: 3,
        ek: vec![String::new(), "azalan".to_string()],
        ..girdi("1000", "", "20")
    };
    let r = sonuc("amortisman", &g);
    assert_eq!(r.alt, "784");
    assert_eq!(satir(&r, "Yıl 1 (kalan 600,00)"), "400");
    assert_eq!(satir(&r, "Yıl 2 (kalan 360,00)"), "240");
    assert_eq!(satir(&r, "Yıl 3 (kalan 216,00)"), "144");
}

#[test]
fn m30_amortisman_yil_siniri() {
    // 1..=50 aralığına clamp.
    // %1 oranında 50 yıl book'u tüketmez; yil=100 → 50'ye clamp edilir.
    let g = Girdi {
        yil: 100,
        ..girdi("1000", "", "1")
    };
    assert_eq!(sonuc("amortisman", &g).satirlar.len(), 3 + 50);
}

// ---------------------------------------------------------------- Döviz

#[test]
fn m31_doviz_to_tl() {
    let r = sonuc("doviz", &girdi("100", "40", ""));
    assert_eq!(r.ana, "4000");
    assert_eq!(r.alt, "40");
}

#[test]
fn m32_doviz_from_tl() {
    let g = Girdi {
        ek: vec!["fromTL".to_string()],
        tevkifat_kod: "EUR".to_string(),
        ..girdi("4000", "40", "")
    };
    let r = sonuc("doviz", &g);
    assert_eq!(r.ana, "100");
    assert!(r.detay.contains("satış"));
    assert!(satir(&r, "Kur (1 EUR)") == "40");
}

#[test]
fn m33_doviz_kur_sifir_red() {
    assert_eq!(hesapla("doviz", &girdi("100", "0", "")), None);
    assert_eq!(hesapla("doviz", &girdi("100", "-5", "")), None);
}

// ---------------------------------------------------------------- Toplu

#[test]
fn m34_batch_kdv() {
    let g = Girdi {
        ek: vec!["1000".to_string(), "2000".to_string(), "boz".to_string()],
        ..girdi("", "", "20")
    };
    let r = sonuc("batch", &g);
    assert_eq!(r.ana, "3600");
    assert_eq!(r.alt, "600");
    assert_eq!(satir(&r, "Toplam matrah"), "3000");
    assert!(r.detay.contains("2 geçerli satır"));
}

#[test]
fn m35_batch_stopaj_ve_tevkifat() {
    let g = Girdi {
        tevkifat_kod: "stopaj".to_string(),
        ek: vec!["1000".to_string(), "2000".to_string()],
        ..girdi("", "", "20")
    };
    let r = sonuc("batch", &g);
    assert_eq!(r.ana, "2400");
    assert_eq!(r.alt, "600");

    let g = Girdi {
        tevkifat_kod: "tevkifat".to_string(),
        tevkifat_pay: 5,
        ek: vec!["10000".to_string()],
        ..girdi("", "", "20")
    };
    let r = sonuc("batch", &g);
    assert_eq!(r.ana, "11000");
    assert_eq!(r.alt, "1000");
}

#[test]
fn m36_batch_satir_bazli_oran() {
    // Her satır kendi oranını taşıyabilir (';' veya '|' ile).
    let g = Girdi {
        ek: vec!["1000;20".to_string(), "2000|10".to_string()],
        ..girdi("", "", "20")
    };
    let r = sonuc("batch", &g);
    assert_eq!(r.ana, "3400");
    assert_eq!(r.alt, "400");
    assert_eq!(satir(&r, "Geçerli satır"), "2");
}

// ---------------------------------------------------------------- Dönem

#[test]
fn m37_donem_odenecek() {
    // 100000*20% - 60000*20% - 1000 = 20000-12000-1000 = 7000.
    let g = Girdi {
        ek: vec!["1000".to_string()],
        ..girdi("100000", "60000", "20")
    };
    let r = sonuc("donem", &g);
    assert_eq!(r.ana, "7000");
    assert_eq!(satir(&r, "Hesaplanan KDV"), "20000");
    assert_eq!(satir(&r, "İndirilecek KDV"), "12000");
    assert_eq!(satir(&r, "Önceki devreden"), "1000");
    assert!(r.detay.contains("Ödenecek KDV"));
}

#[test]
fn m38_donem_devreden() {
    let g = Girdi {
        ek: vec!["0".to_string()],
        ..girdi("50000", "60000", "20")
    };
    let r = sonuc("donem", &g);
    assert_eq!(r.ana, "2000");
    assert!(r.detay.contains("Devreden KDV"));
    assert_eq!(r.alt, "0");
}

#[test]
fn m39_donem_eksik_oran_red() {
    assert_eq!(hesapla("donem", &girdi("100000", "60000", "")), None);
}

// ---------------------------------------------------------------- Genel sözleşme

#[test]
fn m40_tum_modlar_bozuk_girdide_panic_uretmiyor() {
    // Bozuk girdi hiçbir modda panic üretmemeli; ya None ya da sonuç.
    let bozuk = Girdi {
        birinci: "abc".into(),
        ikinci: "???".into(),
        oran: "".into(),
        ..Girdi::default()
    };
    for m in [
        "kdv", "stopaj", "tevkifat", "fiyat", "kdvCompare", "margin", "markup",
        "discount", "percent", "compound", "gecikme", "amortisman", "doviz", "batch", "donem",
    ] {
        let _ = hesapla(m, &bozuk);
    }
}

#[test]
fn m41_mod_adi_buyuk_kucuk_harf_duyarsiz() {
    let g = girdi("1000", "", "20");
    let beklenen = sonuc("kdv", &g);
    assert_eq!(sonuc("KDV", &g), beklenen);
    assert_eq!(sonuc("  Kdv  ", &g), beklenen);
}

#[test]
fn m42_bilinmeyen_mod_red() {
    assert_eq!(hesapla("yok", &girdi("1", "2", "3")), None);
    assert_eq!(hesapla("", &girdi("1", "2", "3")), None);
}
