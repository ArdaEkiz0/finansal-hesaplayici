//! Yeni 5 mod (damga, emlak, kredi, basitFaiz, netBrut) için sözleşme testleri.
//! Tüm para matematiği Decimal / ROUND_HALF_UP, 2 hane; makine biçiminde karşılaştırılır.

use fh_core::{Girdi, hesapla};

fn girdi(birinci: &str, ikinci: &str, oran: &str) -> Girdi {
    Girdi {
        birinci: birinci.to_string(),
        ikinci: ikinci.to_string(),
        oran: oran.to_string(),
        ..Girdi::default()
    }
}

fn satir(mod_: &str, g: &Girdi, etiket: &str) -> String {
    hesapla(mod_, g)
        .unwrap_or_else(|| panic!("{mod_} None döndü: {g:?}"))
        .satirlar
        .into_iter()
        .find(|(k, _)| k == etiket)
        .unwrap_or_else(|| panic!("satır bulunamadı: {etiket}"))
        .1
}

// ---------------------------------------------------------------- Damga vergisi

#[test]
fn y01_damga_bilinen_deger() {
    // 100000 × 9.48‰ / 1000 = 948,00
    let g = girdi("100000", "", "9.48");
    let r = hesapla("damga", &g).unwrap();
    assert_eq!(r.ana, "948");
    assert_eq!(satir("damga", &g, "Damga vergisi"), "948");
}

#[test]
fn y02_damga_turkce_bicim() {
    // 1.000.000 × 9,48‰ / 1000 = 9480
    let g = girdi("1.000.000", "", "9,48");
    assert_eq!(hesapla("damga", &g).unwrap().ana, "9480");
}

#[test]
fn y03_damga_bozuk_girdi_red() {
    assert_eq!(hesapla("damga", &Girdi::default()), None);
    assert_eq!(hesapla("damga", &girdi("100000", "", "")), None);
    assert_eq!(hesapla("damga", &girdi("abc", "", "9.48")), None);
    assert_eq!(hesapla("damga", &girdi("100000", "", "abc")), None);
}

// ---------------------------------------------------------------- Emlak vergisi

#[test]
fn y04_emlak_bilinen_deger() {
    // 500000 × 1‰ / 1000 = 500,00
    let g = girdi("500000", "", "1");
    let r = hesapla("emlak", &g).unwrap();
    assert_eq!(r.ana, "500");
    assert_eq!(satir("emlak", &g, "Emlak vergisi"), "500");
}

#[test]
fn y05_emlak_hata_durumlari() {
    assert_eq!(hesapla("emlak", &Girdi::default()), None);
    assert_eq!(hesapla("emlak", &girdi("500000", "", "")), None);
    assert_eq!(hesapla("emlak", &girdi("500000", "", "abc")), None);
}

// ---------------------------------------------------------------- Kredi

#[test]
fn y06_kredi_bilinen_deger() {
    // P=100000, aylık %1, 12 ay → taksit 8884.88 (annüite formülü),
    // toplam = taksit*n = 106618.56, faiz = 6618.56.
    // NOT: sözleşmedeki "8878.49" örneği verilen formülle tutarlı değildir;
    // formülün tam sonucu 8884.8788... → HALF_UP 8884.88'dir.
    let g = Girdi {
        ay: 12,
        ..girdi("100000", "", "1")
    };
    let r = hesapla("kredi", &g).unwrap();
    assert_eq!(r.ana, "8884.88");
    assert_eq!(satir("kredi", &g, "Aylık taksit"), "8884.88");
    assert_eq!(satir("kredi", &g, "Toplam ödeme"), "106618.56");
    assert_eq!(satir("kredi", &g, "Toplam faiz"), "6618.56");
    assert_eq!(satir("kredi", &g, "Anapara"), "100000");
}

#[test]
fn y07_kredi_faiz_sifir() {
    // r==0 → P/n
    let g = Girdi {
        ay: 12,
        ..girdi("120000", "", "0")
    };
    let r = hesapla("kredi", &g).unwrap();
    assert_eq!(r.ana, "10000");
    assert_eq!(satir("kredi", &g, "Toplam faiz"), "0");
}

#[test]
fn y08_kredi_vade_sinirlari() {
    for ay in [0, -1, 361, 1000] {
        let g = Girdi {
            ay,
            ..girdi("100000", "", "1")
        };
        assert_eq!(hesapla("kredi", &g), None, "ay={ay}");
    }
    assert_eq!(hesapla("kredi", &Girdi::default()), None);
    assert_eq!(
        hesapla(
            "kredi",
            &Girdi {
                ay: 12,
                ..girdi("", "", "1")
            }
        ),
        None
    );
}

#[test]
fn y09_kredi_yuvarlama_half_up() {
    // r==0, P=100, n=3 → 33.333… → 33.33; toplam = 33.33*3 = 99.99
    let g = Girdi {
        ay: 3,
        ..girdi("100", "", "0")
    };
    let r = hesapla("kredi", &g).unwrap();
    assert_eq!(r.ana, "33.33");
    assert_eq!(satir("kredi", &g, "Toplam ödeme"), "99.99");
}

// ---------------------------------------------------------------- Basit faiz

#[test]
fn y10_basit_faiz_bilinen_deger() {
    // 100000 × %10 × 365/365 = 10000
    let g = Girdi {
        gun: 365,
        ..girdi("100000", "", "10")
    };
    let r = hesapla("basitFaiz", &g).unwrap();
    assert_eq!(r.ana, "10000");
    assert_eq!(r.alt, "110000");
    assert_eq!(satir("basitFaiz", &g, "Faiz tutarı"), "10000");
}

#[test]
fn y11_basit_faiz_kismi_gun() {
    // 100000 × %10 × 180/365 = 4931.506… → 4931.51
    let g = Girdi {
        gun: 180,
        ..girdi("100000", "", "10")
    };
    let r = hesapla("basitFaiz", &g).unwrap();
    assert_eq!(r.ana, "4931.51");
    assert_eq!(r.alt, "104931.51");
}

#[test]
fn y12_basit_faiz_gun_sinirlari() {
    for gun in [0, -1, 3651] {
        let g = Girdi {
            gun,
            ..girdi("100000", "", "10")
        };
        assert_eq!(hesapla("basitFaiz", &g), None, "gun={gun}");
    }
    assert_eq!(hesapla("basitFaiz", &Girdi::default()), None);
}

// ---------------------------------------------------------------- Netten brüt

#[test]
fn y13_net_brut_bilinen_deger() {
    // net 80000 / (1 - 0.20) = 100000, stopaj = 20000
    let g = girdi("80000", "", "20");
    let r = hesapla("netBrut", &g).unwrap();
    assert_eq!(r.ana, "100000");
    assert_eq!(r.alt, "20000");
    assert_eq!(satir("netBrut", &g, "Brüt tutar"), "100000");
    assert_eq!(satir("netBrut", &g, "Stopaj"), "20000");
}

#[test]
fn y14_net_brut_oran_sinirlari() {
    // oran == 100 → sıfıra bölme; 100 üstü ve negatif oran → None
    for oran in ["100", "150", "-1"] {
        assert_eq!(
            hesapla("netBrut", &girdi("80000", "", oran)),
            None,
            "oran={oran}"
        );
    }
    assert_eq!(hesapla("netBrut", &Girdi::default()), None);
    assert_eq!(hesapla("netBrut", &girdi("80000", "", "")), None);
}

// ---------------------------------------------------------------- Genel sözleşme

#[test]
fn y15_yeni_modlar_bozuk_girdide_panic_uretmiyor() {
    let bozuk = Girdi {
        birinci: "abc".into(),
        ikinci: "???".into(),
        oran: "xyz".into(),
        ay: -5,
        gun: -5,
        ..Girdi::default()
    };
    for m in ["damga", "emlak", "kredi", "basitFaiz", "netBrut"] {
        let _ = hesapla(m, &bozuk);
    }
}

#[test]
fn y16_mod_adi_buyuk_kucuk_harf_duyarsiz() {
    let d = girdi("100000", "", "9.48");
    assert_eq!(hesapla("DAMGA", &d), hesapla("damga", &d));
    assert_eq!(
        hesapla("  Emlak  ", &girdi("500000", "", "1")),
        hesapla("emlak", &girdi("500000", "", "1"))
    );

    let k = Girdi {
        ay: 12,
        ..girdi("100000", "", "1")
    };
    assert_eq!(hesapla("Kredi", &k), hesapla("kredi", &k));

    let f = Girdi {
        gun: 365,
        ..girdi("100000", "", "10")
    };
    assert_eq!(hesapla("BasitFaiz", &f), hesapla("basitFaiz", &f));

    let n = girdi("80000", "", "20");
    assert_eq!(hesapla("NETBRUT", &n), hesapla("netBrut", &n));
}
