use fh_core::{Girdi, format_tr, hesapla, para_yazisi, parse_tr};
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

fn ana(mod_: &str, g: &Girdi) -> String {
    hesapla(mod_, g)
        .unwrap_or_else(|| panic!("{mod_} None döndü"))
        .ana
}

fn alt(mod_: &str, g: &Girdi) -> String {
    hesapla(mod_, g)
        .unwrap_or_else(|| panic!("{mod_} None döndü"))
        .alt
}

// --- Biçimlendirme / ayrıştırma ---

#[test]
fn t01_format_tr_grouping() {
    assert_eq!(format_tr(d("1234567.89"), 2), "1.234.567,89");
}

#[test]
fn t02_format_tr_negative() {
    assert_eq!(format_tr(d("-1234.5"), 2), "-1.234,50");
}

#[test]
fn t03_format_tr_zero_precision() {
    assert_eq!(format_tr(d("1234567"), 0), "1.234.567");
}

#[test]
fn t04_format_tr_rounds_half_up() {
    assert_eq!(format_tr(d("2.675"), 2), "2,68");
}

#[test]
fn t05_parse_tr_grouped() {
    assert_eq!(parse_tr("1.234,56"), Some(d("1234.56")));
}

#[test]
fn t06_parse_tr_machine() {
    assert_eq!(parse_tr("1234.56"), Some(d("1234.56")));
}

#[test]
fn t07_parse_tr_negative() {
    assert_eq!(parse_tr("-30"), Some(d("-30")));
}

#[test]
fn t08_parse_tr_bad() {
    for s in ["", "  ", "abc", "1.2.3,4,5", "-", ","] {
        assert_eq!(parse_tr(s), None, "{s}");
    }
}

#[test]
fn t09_para_yazisi_examples() {
    assert_eq!(para_yazisi(d("0")), "sıfır lira");
    assert_eq!(
        para_yazisi(d("1234.56")),
        "bin ikiyüz otuz dört lira elli altı kuruş"
    );
}

#[test]
fn t10_para_yazisi_buyuk() {
    assert_eq!(para_yazisi(d("1000000")), "bir milyon lira");
}

// --- KDV ---

#[test]
fn t11_kdv_tax() {
    let g = girdi("1000", "", "20");
    let r = hesapla("kdv", &g).unwrap();
    assert_eq!(r.ana, "200");
    assert_eq!(r.alt, "1200");
    assert!(r.detay.contains("KDV"));
}

#[test]
fn t12_kdv_extract() {
    let mut g = girdi("1200", "dahil", "20");
    assert_eq!(ana("kdv", &g), "200");
    assert_eq!(alt("kdv", &g), "1000");
    g.ek = vec!["extract".to_string()];
    assert_eq!(ana("kdv", &g), "200");
}

#[test]
fn t13_kdv_rate_one() {
    assert_eq!(ana("kdv", &girdi("1000", "", "1")), "10");
}

#[test]
fn t14_kdv_empty_input() {
    assert_eq!(hesapla("kdv", &Girdi::default()), None);
    assert_eq!(hesapla("kdv", &girdi("1000", "", "")), None);
}

// --- Stopaj ---

#[test]
fn t15_stopaj_basit() {
    let r = hesapla("stopaj", &girdi("50000", "", "20")).unwrap();
    assert_eq!(r.ana, "10000"); // kesinti
    assert_eq!(r.alt, "40000"); // net
}

#[test]
fn t16_stopaj_reverse() {
    let g = girdi("40000", "", "20");
    let r = hesapla(
        "stopaj",
        &Girdi {
            ek: vec!["netten".to_string()],
            ..g
        },
    )
    .unwrap();
    assert_eq!(r.ana, "10000");
    assert_eq!(r.alt, "50000");
}

#[test]
fn t17_stopaj_rate_100_reverse_none() {
    let g = girdi("1000", "", "100");
    assert_eq!(
        hesapla(
            "stopaj",
            &Girdi {
                ek: vec!["netten".to_string()],
                ..g
            }
        ),
        None
    );
}

// --- Tevkifat ---

#[test]
fn t18_tevkifat_bilinen() {
    let g = Girdi {
        tevkifat_pay: 5,
        ..girdi("10000", "", "20")
    };
    let r = hesapla("tevkifat", &g).unwrap();
    assert_eq!(r.ana, "1000");
    assert_eq!(r.alt, "11000");
    assert!(
        r.satirlar
            .iter()
            .any(|(k, v)| k.contains("KDV") && v == "2000")
    );
}

#[test]
fn t19_tevkifat_pay_gecersiz() {
    let g = Girdi {
        tevkifat_pay: 0,
        ..girdi("1000", "", "20")
    };
    assert_eq!(hesapla("tevkifat", &g), None);
}

// --- Fiyat / KDV karşılaştırma ---

#[test]
fn t20_fiyat() {
    assert_eq!(ana("fiyat", &girdi("1000", "", "20")), "1200");
}

#[test]
fn t21_fiyat_indirimli() {
    let g = Girdi {
        ek: vec!["10".to_string()],
        ..girdi("1000", "", "20")
    };
    assert_eq!(ana("fiyat", &g), "1080");
}

#[test]
fn t22_kdv_compare() {
    let r = hesapla("kdvCompare", &girdi("1000", "", "20")).unwrap();
    assert_eq!(r.ana, "33.33");
    assert_eq!(r.alt, "1200");
}

// --- Marj / kâr / indirim / yüzde ---

#[test]
fn t23_margin() {
    let g = girdi("100", "20", "");
    let r = hesapla("margin", &g).unwrap();
    assert_eq!(r.ana, "125");
    assert_eq!(r.alt, "25");
}

#[test]
fn t24_margin_100() {
    let r = hesapla("margin", &girdi("100", "100", "")).unwrap();
    assert_eq!(r.ana, "0");
}

#[test]
fn t25_markup() {
    let r = hesapla("markup", &girdi("100", "20", "")).unwrap();
    assert_eq!(r.ana, "120");
    assert_eq!(r.alt, "20");
}

#[test]
fn t26_discount_zincir() {
    let g = Girdi {
        ek: vec!["10".to_string(), "10".to_string()],
        ..girdi("1000", "", "")
    };
    let r = hesapla("discount", &g).unwrap();
    assert_eq!(r.ana, "810");
    assert_eq!(r.alt, "190");
}

#[test]
fn t27_discount_bos() {
    assert_eq!(hesapla("discount", &girdi("1000", "", "")), None);
}

#[test]
fn t28_percent() {
    assert_eq!(ana("percent", &girdi("20", "500", "")), "100");
}

#[test]
fn t29_percent_reverse() {
    let r = hesapla("percent", &girdi("20", "500", "")).unwrap();
    assert!(r.satirlar.iter().any(|(_, v)| v == "416.67"));
}

// --- Bileşik faiz / gecikme / amortisman ---

#[test]
fn t30_compound() {
    let g = Girdi {
        ek: vec!["1".to_string(), "12".to_string()],
        ..girdi("1000", "12", "")
    };
    let r = hesapla("compound", &g).unwrap();
    assert_eq!(r.ana, "1126.83");
    assert_eq!(r.alt, "126.83");
}

#[test]
fn t31_compound_yil_sifir() {
    let g = Girdi {
        yil: 0,
        ..girdi("1000", "12", "")
    };
    assert_eq!(hesapla("compound", &g), None);
}

#[test]
fn t32_gecikme_bilinen() {
    let g = Girdi {
        gun: 30,
        ..girdi("10000", "4.5", "")
    };
    let r = hesapla("gecikme", &g).unwrap();
    assert_eq!(r.ana, "450");
    assert_eq!(r.alt, "10450");
}

#[test]
fn t33_gecikme_yarim_ay() {
    let g = Girdi {
        gun: 15,
        ..girdi("10000", "4.5", "")
    };
    assert_eq!(ana("gecikme", &g), "225");
}

#[test]
fn t34_amortisman_bilinen() {
    let g = Girdi {
        yil: 5,
        ..girdi("500000", "", "20")
    };
    let r = hesapla("amortisman", &g).unwrap();
    assert_eq!(r.ana, "100000");
    assert_eq!(r.alt, "500000");
}

#[test]
fn t35_amortisman_oran_yildan() {
    let g = Girdi {
        yil: 5,
        ..girdi("500000", "", "")
    };
    assert_eq!(ana("amortisman", &g), "100000");
}

#[test]
fn t36_amortisman_azalan() {
    let g = Girdi {
        yil: 3,
        ek: vec![String::new(), "azalan".to_string()],
        ..girdi("1000", "", "20")
    };
    assert_eq!(alt("amortisman", &g), "784");
}

// --- Döviz / toplu / dönem ---

#[test]
fn t37_doviz_to_tl() {
    assert_eq!(ana("doviz", &girdi("100", "40", "")), "4000");
}

#[test]
fn t38_doviz_from_tl() {
    let g = Girdi {
        ek: vec!["fromTL".to_string()],
        ..girdi("4000", "40", "")
    };
    assert_eq!(ana("doviz", &g), "100");
}

#[test]
fn t39_doviz_kur_sifir() {
    assert_eq!(hesapla("doviz", &girdi("100", "0", "")), None);
}

#[test]
fn t40_batch() {
    let g = Girdi {
        ek: vec!["1000".to_string(), "2000".to_string(), "boz".to_string()],
        ..girdi("", "", "20")
    };
    let r = hesapla("batch", &g).unwrap();
    assert_eq!(r.ana, "3600");
    assert_eq!(r.alt, "600");
    assert!(r.detay.contains("2 geçerli satır"));
}

#[test]
fn t41_donem_bilinen() {
    let g = Girdi {
        ek: vec!["1000".to_string()],
        ..girdi("100000", "60000", "20")
    };
    assert_eq!(ana("donem", &g), "7000");
}

#[test]
fn t42_donem_devreden() {
    let g = Girdi {
        ek: vec!["0".to_string()],
        ..girdi("50000", "60000", "20")
    };
    assert_eq!(ana("donem", &g), "2000");
    assert!(hesapla("donem", &g).unwrap().detay.contains("Devreden"));
}

#[test]
fn t43_bilinmeyen_mod() {
    assert_eq!(hesapla("yok", &girdi("1", "2", "3")), None);
}

#[test]
fn t45_compound_tasmada_panic_yok() {
    // rust_decimal::powd/exp taşmada panic atar; checked karşılıkları None döner.
    let g = Girdi {
        ek: vec!["200".into(), "1".into()],
        ..girdi("1000", "200", "")
    };
    assert_eq!(hesapla("compound", &g), None);
    // Çok büyük üs: uzun döngü/taşma yerine hızlı red.
    let g = Girdi {
        ek: vec!["1000000000".into(), "12".into()],
        ..girdi("1000", "12", "")
    };
    assert_eq!(hesapla("compound", &g), None);
}

#[test]
fn t46_compound_negatif_taban_cift_us() {
    // -150% yıllık, yılda 1 bileşme, 2 yıl → (1-1.5)^2 = 0.25 → 250.
    // Eski power() negatif tabanı 0 sanıp 0 döndürüyordu.
    let g = Girdi {
        ek: vec!["2".into(), "1".into()],
        ..girdi("1000", "-150", "")
    };
    assert_eq!(ana("compound", &g), "250");
}

#[test]
fn t47_batch_tevkifat_gecersiz_pay_red() {
    for pay in [0, 11, -1] {
        let g = Girdi {
            tevkifat_kod: "tevkifat".into(),
            tevkifat_pay: pay,
            ek: vec!["1000".into()],
            ..girdi("", "", "20")
        };
        assert_eq!(hesapla("batch", &g), None, "pay={pay}");
    }
    let g = Girdi {
        tevkifat_kod: "tevkifat".into(),
        tevkifat_pay: 5,
        ek: vec!["10000".into()],
        ..girdi("", "", "20")
    };
    assert_eq!(ana("batch", &g), "11000");
}

#[test]
fn t44_tum_modlar_bos_girdi() {
    for m in [
        "kdv",
        "stopaj",
        "tevkifat",
        "fiyat",
        "kdvCompare",
        "margin",
        "markup",
        "discount",
        "percent",
        "compound",
        "gecikme",
        "amortisman",
        "doviz",
        "batch",
        "donem",
    ] {
        assert_eq!(
            hesapla(m, &Girdi::default()),
            None,
            "{m} boş girdide None dönmeli"
        );
    }
}
