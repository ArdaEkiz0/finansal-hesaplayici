//! Widget ayarları / kalıcılık / tema / komut paleti saf testleri.
//! GUI açılmaz; kaynak modülleri `#[path]` ile dahil edilir.
//! `commands` modülü `crate::modes` kullandığı için `modes` de dahil edilir.

#[path = "../src/model.rs"]
mod model;
#[path = "../src/modes.rs"]
mod modes;
#[path = "../src/commands.rs"]
mod commands;
#[path = "../src/theme.rs"]
mod theme;

use commands::{Eylem, filtrele, komutlar, turkce_kucuk};
use model::{
    AppState, Settings, WIDGET_GENISLIK_ARALIGI, WIDGET_YUKSEKLIK_ARALIGI, WidgetAyarlari,
};
use serde_json::{Value, json};

fn ornek_widget() -> WidgetAyarlari {
    WidgetAyarlari {
        aktif: true,
        kompakt: true,
        r#mod: "stopaj".into(),
        genislik: 420.0,
        yukseklik: 360.0,
        ustte: false,
    }
}

fn state(w: WidgetAyarlari) -> AppState {
    AppState {
        settings: Settings::default(),
        history: Vec::new(),
        donemler: Vec::new(),
        cariler: Vec::new(),
        active_cari: String::new(),
        inputs: Default::default(),
        widget: w,
    }
}

#[test]
fn widget_varsayilan_degerleri() {
    let w = WidgetAyarlari::default();
    assert!(!w.aktif);
    assert!(!w.kompakt);
    assert_eq!(w.r#mod, "kdv");
    assert_eq!(w.genislik, 340.0);
    assert_eq!(w.yukseklik, 300.0);
    assert!(w.ustte);
}

#[test]
fn widget_serde_round_trip_ve_mod_anahtari() {
    let w = ornek_widget();
    let v = serde_json::to_value(&w).unwrap();
    // Raw identifier olmasına karşın serde anahtarı "mod" olmalı.
    assert_eq!(v["mod"], "stopaj");
    assert!(v.get("r#mod").is_none());
    assert_eq!(serde_json::from_value::<WidgetAyarlari>(v).unwrap(), w);
    let metin = serde_json::to_string(&w).unwrap();
    assert_eq!(serde_json::from_str::<WidgetAyarlari>(&metin).unwrap(), w);
}

#[test]
fn widget_yedek_round_trip_bomsuz() {
    let w = ornek_widget();
    let yedek = state(w.clone()).yedek_degeri();
    assert_eq!(yedek["widget"]["aktif"], true);
    assert_eq!(yedek["widget"]["mod"], "stopaj");
    let metin = serde_json::to_string(&yedek).unwrap();
    assert!(!metin.starts_with('\u{feff}'));
    let geri = AppState::metinden_yukle(&metin).unwrap();
    assert_eq!(geri.widget, w);
}

#[test]
fn widget_yedek_round_trip_bomlu() {
    let w = ornek_widget();
    let metin = serde_json::to_string(&state(w.clone()).yedek_degeri()).unwrap();
    let geri = AppState::metinden_yukle(&format!("\u{feff}{metin}")).unwrap();
    assert_eq!(geri.widget, w, "BOM'lu dosyada widget ayarları kaybolmamalı");
}

#[test]
fn widget_eksik_alanlar_varsayilana_duser() {
    let mut yedek = state(ornek_widget()).yedek_degeri();
    yedek["widget"] = json!({"aktif": true, "genislik": 500.0});
    let w = AppState::from_backup(&yedek).unwrap().widget;
    assert!(w.aktif, "gelen alan korunmalı");
    assert_eq!(w.genislik, 500.0);
    assert!(!w.kompakt);
    assert_eq!(w.r#mod, "kdv");
    assert_eq!(w.yukseklik, 300.0);
    assert!(w.ustte);

    // widget anahtarı hiç yoksa tam varsayılan.
    let mut yedek2 = state(ornek_widget()).yedek_degeri();
    yedek2.as_object_mut().unwrap().remove("widget");
    assert_eq!(AppState::from_backup(&yedek2).unwrap().widget, WidgetAyarlari::default());
}

#[test]
fn widget_bozuk_alanlar_varsayilana_duser() {
    let mut yedek = state(ornek_widget()).yedek_degeri();
    yedek["widget"] = json!({
        "aktif": "evet", "kompakt": 1, "mod": 42,
        "genislik": "genis", "yukseklik": true, "ustte": Value::Null
    });
    assert_eq!(AppState::from_backup(&yedek).unwrap().widget, WidgetAyarlari::default());
}

#[test]
fn widget_boyut_sinirlari() {
    let (gen_lo, gen_hi) = WIDGET_GENISLIK_ARALIGI;
    let (yuk_lo, yuk_hi) = WIDGET_YUKSEKLIK_ARALIGI;
    let oku = |g: Value, y: Value| {
        let mut yedek = state(ornek_widget()).yedek_degeri();
        yedek["widget"] = json!({"genislik": g, "yukseklik": y});
        AppState::from_backup(&yedek).unwrap().widget
    };
    // Aralık uçları dahil kabul edilir.
    let w = oku(json!(gen_lo), json!(yuk_lo));
    assert_eq!(w.genislik, gen_lo);
    assert_eq!(w.yukseklik, yuk_lo);
    let w = oku(json!(gen_hi), json!(yuk_hi));
    assert_eq!(w.genislik, gen_hi);
    assert_eq!(w.yukseklik, yuk_hi);
    // Dışındaki değerler varsayılana düşer.
    let w = oku(json!(gen_lo - 1.0), json!(yuk_hi + 1.0));
    assert_eq!(w.genislik, 340.0);
    assert_eq!(w.yukseklik, 300.0);
    let w = oku(json!(-5.0), json!(0.0));
    assert_eq!(w.genislik, 340.0);
    assert_eq!(w.yukseklik, 300.0);
}

#[test]
fn widget_gecersiz_yapilar_varsayilana_duser() {
    for v in [Value::Null, json!("widget"), json!(42), json!([])] {
        assert_eq!(WidgetAyarlari::from_value(&v), WidgetAyarlari::default());
        let mut yedek = state(ornek_widget()).yedek_degeri();
        yedek["widget"] = v;
        assert_eq!(AppState::from_backup(&yedek).unwrap().widget, WidgetAyarlari::default());
    }
    // Boş/uzun mod adı da varsayılana düşer.
    for m in ["", "   ", "x".repeat(40).as_str()] {
        let w = WidgetAyarlari::from_value(&json!({"mod": m}));
        assert_eq!(w.r#mod, "kdv");
    }
}

#[test]
fn komut_paleti_widget_kimlikleri_gecerli() {
    let l = komutlar();
    let widgetlar: Vec<_> = l.iter().filter(|c| turkce_kucuk(&c.ad).contains("widget")).collect();
    assert_eq!(widgetlar.len(), 3, "palette 3 widget girdisi olmalı");
    for c in widgetlar {
        assert_eq!(c.eylem, Eylem::Widget, "{} bilinmeyen widget eylemi", c.ad);
        assert_eq!(c.kategori, "Görünüm");
    }
    let bulunan = filtrele(&l, "widget");
    assert_eq!(bulunan.len(), 3);
    assert!(bulunan.iter().all(|&i| l[i].eylem == Eylem::Widget));
}

#[test]
fn komut_paleti_tum_kimlikleri_gecerli() {
    // Eylem kapalı bir enum: yeni varyant eklenirse bu eşleşme derlenmez.
    let l = komutlar();
    for c in &l {
        match c.eylem {
            Eylem::Mod(id) => assert!(modes::mod_def(id).is_some(), "geçersiz mod: {id}"),
            Eylem::Tema(t) => assert!(["dark", "light", "system"].contains(&t)),
            Eylem::Widget => assert_eq!(c.kategori, "Görünüm"),
            Eylem::FontBuyut
            | Eylem::FontKucult
            | Eylem::YedekAl
            | Eylem::YedekYukle
            | Eylem::GecmisCsv
            | Eylem::EfaturaIce
            | Eylem::Takvim
            | Eylem::Gecmis
            | Eylem::GecmisTemizle
            | Eylem::Ayarlar
            | Eylem::Cariler
            | Eylem::Donemler
            | Eylem::Yazdir => {}
        }
    }
    for c in &l {
        assert_eq!(l.iter().filter(|x| x.ad == c.ad).count(), 1, "yinelenen ad: {}", c.ad);
    }
}

#[test]
fn theme_widget_yardimcilari_paletle_tutarly() {
    let koyu = theme::palet(true);
    let acik = theme::palet(false);
    // Palet kimliği bozulmadı.
    assert_ne!(koyu.arka, acik.arka);
    assert_eq!(koyu.birincil.r(), 0x3b);
    assert_eq!(acik.birincil.r(), 0x25);
    // Widget yardımcıları mevcut paletten türetilir.
    assert_ne!(theme::widget_kart(true), koyu.arka);
    assert_ne!(theme::widget_kart(true), koyu.yuzey);
    assert_ne!(theme::widget_kart(false), acik.yuzey);
    assert_eq!(theme::widget_vurgu_cizgi(true).color, koyu.aksan);
    assert_eq!(theme::widget_vurgu_cizgi(false).color, acik.aksan);
    assert_eq!(theme::widget_kenar(true).color, koyu.kenar);
    assert_eq!(theme::widget_hover(true).1, koyu.birincil);
    assert_eq!(theme::widget_hover(false).1, acik.birincil);
}
