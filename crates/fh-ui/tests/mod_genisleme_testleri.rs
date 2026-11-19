//! v5.6 mod genişletmesi için saf testler: 20 mod, kimlik benzersizliği,
//! yeni modların grup/alan eşlemesi, kaydırma sınırı sabiti ve widget listesi.
//! GUI açılmaz; kaynak modülleri `#[path]` ile dahil edilir.

#[path = "../src/model.rs"]
mod model;
#[path = "../src/modes.rs"]
mod modes;
#[path = "../src/commands.rs"]
mod commands;

use model::InputState;
use modes::{MODLAR, MOD_NAV_MIN_HEIGHT, mod_def};

#[test]
fn modlar_20_tanlidir() {
    assert_eq!(MODLAR.len(), 20);
    for id in [
        "kdv", "stopaj", "tevkifat", "fiyat", "kdvCompare", "margin", "markup",
        "discount", "percent", "compound", "gecikme", "amortisman", "doviz", "batch", "donem",
        "damga", "emlak", "kredi", "basitFaiz", "netBrut",
    ] {
        assert!(mod_def(id).is_some(), "{id} modu tanımlı olmalı");
    }
}

#[test]
fn mod_idleri_benzersizdir() {
    for (i, m) in MODLAR.iter().enumerate() {
        assert!(!m.id.is_empty(), "boş mod id'si");
        assert_eq!(
            MODLAR.iter().filter(|x| x.id == m.id).count(),
            1,
            "yinelenen mod id'si: {}",
            m.id
        );
        assert_eq!(i, MODLAR.iter().position(|x| x.id == m.id).unwrap());
    }
}

#[test]
fn yeni_modlarin_grup_ve_alan_eslemesi() {
    let beklenen = [
        ("damga", "Damga Vergisi", "KDV & Stopaj", &["Birinci", "Oran"][..]),
        ("emlak", "Emlak Vergisi", "KDV & Stopaj", &["Birinci", "Oran"][..]),
        ("kredi", "Kredi Taksiti", "Faiz & Kredi", &["Birinci", "Oran", "Ay"][..]),
        ("basitFaiz", "Basit Faiz", "Faiz & Kredi", &["Birinci", "Oran", "Gun"][..]),
        ("netBrut", "Net → Brüt", "Kâr & Yüzde", &["Birinci", "Oran"][..]),
    ];
    for (id, etiket, grup, alanlar) in beklenen {
        let m = mod_def(id).unwrap_or_else(|| panic!("{id} bulunamadı"));
        assert_eq!(m.etiket, etiket, "{id} etiketi");
        assert_eq!(m.grup, grup, "{id} grubu");
        let gercek: Vec<_> = m.alanlar.iter().map(|f| format!("{:?}", f.alan)).collect();
        assert_eq!(gercek, alanlar, "{id} alan eşlemesi");
        assert!(m.alanlar.iter().all(|f| !f.etiket.is_empty()), "{id} boş alan etiketi");
    }
}

#[test]
fn yeni_modlarin_alan_etiketleri() {
    let damga = mod_def("damga").unwrap();
    assert_eq!(damga.alanlar[0].etiket, "Matrah");
    assert_eq!(damga.alanlar[1].etiket, "Oran (‰)");

    let emlak = mod_def("emlak").unwrap();
    assert_eq!(emlak.alanlar[0].etiket, "Matrah");
    assert_eq!(emlak.alanlar[1].etiket, "Oran (‰)");

    let kredi = mod_def("kredi").unwrap();
    assert_eq!(kredi.alanlar[0].etiket, "Anapara");
    assert_eq!(kredi.alanlar[1].etiket, "Aylık faiz (%)");
    assert_eq!(kredi.alanlar[2].etiket, "Vade (ay)");

    let basit = mod_def("basitFaiz").unwrap();
    assert_eq!(basit.alanlar[0].etiket, "Anapara");
    assert_eq!(basit.alanlar[1].etiket, "Yıllık faiz (%)");
    assert_eq!(basit.alanlar[2].etiket, "Gün sayısı");

    let net = mod_def("netBrut").unwrap();
    assert_eq!(net.alanlar[0].etiket, "Net tutar");
    assert_eq!(net.alanlar[1].etiket, "Stopaj oranı (%)");
}

#[test]
fn kaydirma_yukseklik_sabiti_gecerlidir() {
    assert_eq!(MOD_NAV_MIN_HEIGHT, 120.0);
    assert!(MOD_NAV_MIN_HEIGHT.is_finite());
    assert!(MOD_NAV_MIN_HEIGHT > 0.0);
    // max() davranışı: mevcut yükseklik düşük olsa bile sınır 120'nin altına inmez.
    assert!((50.0_f32).max(MOD_NAV_MIN_HEIGHT) >= MOD_NAV_MIN_HEIGHT);
}

#[test]
fn gruplar_yeni_gruplari_icerir() {
    let g = modes::gruplar();
    assert_eq!(g.len(), 5);
    assert!(g.contains(&"KDV & Stopaj"));
    assert!(g.contains(&"Faiz & Kredi"));
    assert!(g.contains(&"Kâr & Yüzde"));
    assert!(g.contains(&"Faiz & Zaman"));
    assert!(g.contains(&"Diğer"));
}

#[test]
fn widget_mod_listesi_tum_20_modu_icerir() {
    // Widget seçicisi MODLAR'ı doğrudan iterasyonla kullanır; özel liste yok.
    // Bu yüzden 20 modun tamamı etiketli ve seçilebilir olmalı.
    let ids: Vec<_> = MODLAR.iter().map(|m| m.id).collect();
    assert_eq!(ids.len(), 20);
    for id in [
        "damga", "emlak", "kredi", "basitFaiz", "netBrut",
    ] {
        assert!(ids.contains(&id), "widget listesinde {id} eksik");
        assert!(!modes::mod_etiketi(id).is_empty());
    }
    assert_eq!(modes::mod_etiketi("yok"), "Bilinmeyen mod");
}

#[test]
fn komut_paleti_20_modu_kapsar() {
    let l = commands::komutlar();
    let mod_komutlari: Vec<_> = l
        .iter()
        .filter_map(|c| match c.eylem {
            commands::Eylem::Mod(id) => Some(id),
            _ => None,
        })
        .collect();
    assert_eq!(mod_komutlari.len(), 20);
    for m in MODLAR {
        assert!(
            mod_komutlari.contains(&m.id),
            "{} komut paletinde eksik",
            m.id
        );
    }
}

#[test]
fn yeni_modlar_build_girdi_alanlarini_tasir() {
    let mut i = InputState {
        birinci: "1000".into(),
        oran: "2".into(),
        ay: "12".into(),
        gun: "30".into(),
        ..InputState::default()
    };
    for id in ["damga", "emlak", "netBrut"] {
        let g = modes::build_girdi(id, &i);
        assert_eq!(g.birinci, "1000", "{id} birinci alanı");
        assert_eq!(g.oran, "2", "{id} oran alanı");
    }
    let g = modes::build_girdi("kredi", &i);
    assert_eq!(g.ay, 12, "kredi vade alanı");
    let g = modes::build_girdi("basitFaiz", &i);
    assert_eq!(g.gun, 30, "basitFaiz gün alanı");
    // Boş sayısal alanlar 0'a düşer (varsayılan davranış korunur).
    i.ay.clear();
    i.gun.clear();
    assert_eq!(modes::build_girdi("kredi", &i).ay, 0);
    assert_eq!(modes::build_girdi("basitFaiz", &i).gun, 0);
}
