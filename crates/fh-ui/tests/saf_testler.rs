//! fh-ui saf testleri. fh-ui bir binary crate olduğu için kaynak modülleri
//! `#[path]` ile doğrudan dahil edilir; hiçbir test GUI penceresi açmaz.

#[path = "../src/model.rs"]
mod model;
#[path = "../src/theme.rs"]
mod theme;

use model::{AppState, PENCERE_AYARLARI, SURUM, Settings};
use serde_json::{Value, json};

fn ornek() -> AppState {
    let mut history = Vec::new();
    model::gecmis_ekle(
        &mut history,
        model::HistEntry {
            id: "h1".into(),
            timestamp: 1,
            mode: "kdv".into(),
            inputs: model::InputState::default(),
            ana: "200".into(),
            alt: "1200".into(),
            detay: "KDV hesabı".into(),
        },
    );
    AppState {
        settings: Settings {
            theme: "light".into(),
            font_scale: 1.1,
            show_currency_symbol: false,
            precision: 4,
            currency_symbol: "$".into(),
        },
        history,
        donemler: Vec::new(),
        cariler: vec![model::Cari { id: "c1".into(), name: "Müşteri A".into() }],
        active_cari: "c1".into(),
        inputs: Default::default(),
    }
}

#[test]
fn surum_etiketi_workspace_ile_ayni() {
    // Tek sürüm kaynağı: Cargo workspace (Kurulum/CI ile aynı değer).
    assert_eq!(SURUM, env!("CARGO_PKG_VERSION"));
    // İkinci kontrol: kök Cargo.toml'daki workspace sürümüyle aynı olmalı
    // (sabit sürüm yazmak bir sonraki yayında testi kırıyor).
    let kok = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    let metin = std::fs::read_to_string(kok).expect("kök Cargo.toml okunamadı");
    let workspace_surum = metin
        .lines()
        .find(|l| l.trim_start().starts_with("version = "))
        .and_then(|l| l.split('"').nth(1))
        .expect("workspace sürümü bulunamadı");
    assert_eq!(SURUM, workspace_surum);
}

#[test]
fn tema_cozumleme_dark_light_system() {
    assert!(theme::koyu_mu("dark"));
    assert!(theme::koyu_mu("system")); // sistem tema algılanamıyorsa koyu
    assert!(!theme::koyu_mu("light"));

    let coz = |tema: Value| {
        AppState::metinden_yukle(
            &json!({"uygulama": "finansal-hesaplayici", "theme": tema}).to_string(),
        )
        .unwrap()
        .settings
        .theme
    };
    assert_eq!(coz(json!("dark")), "dark");
    assert_eq!(coz(json!("light")), "light");
    assert_eq!(coz(json!("system")), "system");
    assert_eq!(coz(json!("mavi")), "dark", "bozuk tema varsayılana düşmeli");
    assert_eq!(coz(Value::Null), "dark", "eksik tema varsayılana düşmeli");
}

#[test]
fn font_scale_sinir_ve_varsayilan() {
    let coz = |fs: Value| {
        AppState::metinden_yukle(
            &json!({"uygulama": "finansal-hesaplayici", "settings": {"fontScale": fs}})
                .to_string(),
        )
        .unwrap()
        .settings
        .font_scale
    };
    assert_eq!(coz(json!(0.9)), 0.9);
    assert_eq!(coz(json!(1.1)), 1.1);
    assert_eq!(coz(json!(0.5)), 1.0, "alt sınırın altı varsayılana düşmeli");
    assert_eq!(coz(json!(2.0)), 1.0, "üst sınırın üstü varsayılana düşmeli");
    assert_eq!(coz(json!("bozuk")), 1.0);
    assert_eq!(coz(Value::Null), 1.0);
}

#[test]
fn ayar_json_bomsuz_okunur() {
    let metin = serde_json::to_string(&ornek().yedek_degeri()).unwrap();
    assert!(!metin.starts_with('\u{feff}'));
    let geri = AppState::metinden_yukle(&metin).expect("BOM'suz JSON okunmalı");
    assert_eq!(geri.settings, ornek().settings);
    assert_eq!(geri.history.len(), 1);
    assert_eq!(geri.cariler[0].name, "Müşteri A");
}

#[test]
fn ayar_json_bomlu_okunur() {
    // Windows/editör kaynaklı BOM yüzünden kullanıcı dosyası kaybolmamalı.
    let metin = format!("\u{feff}{}", serde_json::to_string(&ornek().yedek_degeri()).unwrap());
    let geri = AppState::metinden_yukle(&metin).expect("BOM'lu JSON okunmalı");
    assert_eq!(geri.settings, ornek().settings);
    assert_eq!(geri.history.len(), 1, "BOM yüzünden geçmiş kaybolmamalı");
    assert_eq!(geri.cariler[0].id, "c1");
}

#[test]
fn yedek_geri_yukleme_round_trip() {
    let state = ornek();
    let metin = serde_json::to_string_pretty(&state.yedek_degeri()).unwrap();
    let geri = AppState::metinden_yukle(&metin).unwrap();
    assert_eq!(geri.settings, state.settings);
    assert_eq!(geri.history, state.history);
    assert_eq!(geri.cariler, state.cariler);
    assert_eq!(geri.active_cari, state.active_cari);
}

#[test]
fn bozuk_ayarlar_guvenli_varsayilana_duser() {
    let mut yedek = ornek().yedek_degeri();
    yedek["settings"] = json!({"showCurrencySymbol": "evet", "precision": 3, "fontScale": 9});
    yedek["theme"] = json!("mavi");
    yedek["ekstra"] = json!({"currency_symbol": "   "});
    let geri = AppState::from_backup(&yedek).unwrap();
    assert_eq!(geri.settings.theme, "dark");
    assert_eq!(geri.settings.precision, 2);
    assert_eq!(geri.settings.font_scale, 1.0);
    assert!(geri.settings.show_currency_symbol);
    assert_eq!(geri.settings.currency_symbol, "₺");
}

#[test]
fn pencere_ayarlari_acilis_varsayilanlari() {
    assert!(PENCERE_AYARLARI.maksimum_acilis, "uygulama ekrana sığarak açılmalı");
    assert!(!PENCERE_AYARLARI.persist_window, "bozuk 14x14 kaydı geri gelmemeli");
    assert!(PENCERE_AYARLARI.inner_size[0] >= PENCERE_AYARLARI.min_inner_size[0]);
    assert!(PENCERE_AYARLARI.inner_size[1] >= PENCERE_AYARLARI.min_inner_size[1]);
    assert!(
        PENCERE_AYARLARI.min_inner_size[0] <= 800.0 && PENCERE_AYARLARI.min_inner_size[1] <= 600.0,
        "küçük ekranlarda taşma olmamalı"
    );
}
