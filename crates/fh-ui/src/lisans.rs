//! Lisans/deneme kapısı — Finansal Hesaplayıcı standalone (LISANS_SOZLESMESI.md §10).
//!
//! Kasa modeli: pakete hediye, tek başına satılık. Üç katman: satılmış anahtar
//! (Ed25519, `FH-` ONEK) → saat-geri koruması → 15 günlük yerel deneme.
//! Ağ çağırmaz (§6 FH disiplini: ağ yalnız kullanıcı eyleminde); karar
//! çekirdeği `muhasebe-lisans` crate'idir (Python hattıyla altın vektör kilitli).
//!
//! Bağlayıcı değişmezler (§8, §10.3): deneme tabanı uzatılamaz (kurulum günü
//! dosya doğum tarihi tabanı yalnız geriye çeker — kaydı silmek sıfırlamaz);
//! saat >30 dk geriden gelirse kapalı; anahtar imzası bozuksa denemeye
//! DÜŞÜLMEZ, kapı kapalı.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, Utc};
use eframe::egui;
use serde::{Deserialize, Serialize};

use muhasebe_lisans::anahtar::imzali_anahtar_dogrula;
use muhasebe_lisans::karar::{DenemeDurumu, deneme_karari};
use muhasebe_lisans::sabitler::{DENEME_GUN, KDV_SATIS_ACIK_ANAHTARI, SAAT_SAPMA_TOLERANS_SN};

/// Satılmış anahtar öneki (atölye `FH-` üretir; sözleşme §10.1).
pub const ONEK: &str = "FH";

/// `lisans.json` içeriği (`%APPDATA%\FinansalHesaplayici\` — veri.json ile aynı klasör).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Kayit {
    #[serde(default)]
    pub deneme_baslangic: Option<String>,
    #[serde(default)]
    pub son_gorulen_ms: Option<i64>,
    #[serde(default)]
    pub anahtar: Option<String>,
}

/// Kapı kararı. `Kilitli` kullanıcıya gösterilecek Türkçe metni taşır.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kapi {
    Lisans,
    Deneme { kalan_gun: i64 },
    Kilitli { baslik: String, gerekce: String },
}

fn kilit(baslik: &str, gerekce: &str) -> Kapi {
    Kapi::Kilitli {
        baslik: baslik.into(),
        gerekce: gerekce.into(),
    }
}

const SATIR: &str = "Satın alma: ardaekiz72@gmail.com · 0546 208 38 54";

/// SAF karar fonksiyonu — disk/saat enjekte edilir, her şey test edilebilir.
/// `(kapı, güncellenmiş kayıt)` döner; çağıran kaydı yalnız değiştiyse yazar.
pub fn kapi_karari(
    kayit: &Kayit,
    taban: Option<&str>,
    bugun: &str,
    simdi_ms: i64,
) -> (Kapi, Kayit) {
    kapi_karari_ile(kayit, taban, bugun, simdi_ms, KDV_SATIS_ACIK_ANAHTARI)
}

/// Aynı karar, açık anahtar enjekte edilerek — imzalı anahtar uçtan uca
/// test edilebilsin diye (üretim daima gömülü sabiti kullanır).
pub fn kapi_karari_ile(
    kayit: &Kayit,
    taban: Option<&str>,
    bugun: &str,
    simdi_ms: i64,
    acik_anahtar_b64: &str,
) -> (Kapi, Kayit) {
    // 1) Satılmış anahtar: tek güven kaynağı imza. Hatalı anahtar denemeye
    //    DÜŞÜRMEZ (kayıtla oynayarak bedava deneme devşirilemesin).
    if let Some(a) = kayit.anahtar.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        let kilitli = |t: &str, g: &str| (kilit(t, &format!("{g} {SATIR}")), kayit.clone());
        return match imzali_anahtar_dogrula(a, ONEK, acik_anahtar_b64, bugun) {
            Ok(_) => (Kapi::Lisans, kayit.clone()),
            Err(e) if e.0 == "sure-doldu" => {
                kilitli("Lisans süresi dolmuş", "Kayıtlı anahtarın süresi doldu.")
            }
            Err(_) => kilitli(
                "Lisans doğrulanamadı",
                "Kayıtlı anahtar geçersiz ya da tahrif edilmiş.",
            ),
        };
    }

    // 2) Saat-geri koruması: kayıt bugüne dek en son NEYİ gördüyse o çepadır;
    //    saat çepadan >30 dk gerideyse kapı kapanır.
    if let Some(t) = kayit.son_gorulen_ms {
        if simdi_ms < t - SAAT_SAPMA_TOLERANS_SN * 1000 {
            return (
                kilit(
                    "Sistem saati geriden geliyor",
                    "Deneme sayacı saat değişikliğine kapandı. Bilgisayar saatini \
                     düzelttikten sonra yeniden açın.",
                ),
                kayit.clone(),
            );
        }
    }

    // 3) Deneme: ilk açılış bugündür; taban yalnız GERİYE çekilir (kurulum
    //    günü = dosya doğum tarihi) — kural crate'te, altın vektörlerde.
    let mut yeni = kayit.clone();
    if yeni.deneme_baslangic.is_none() {
        yeni.deneme_baslangic = Some(bugun.to_string());
    }
    let karar = match deneme_karari(
        yeni.deneme_baslangic.as_deref(),
        taban,
        bugun,
        DENEME_GUN,
        true,
    ) {
        Ok(k) => k,
        Err(_) => {
            return (
                kilit("Lisans hatası", "Deneme durumu okunamadı."),
                kayit.clone(),
            );
        }
    };
    // Zaman damgası yalnız ileri gider (MP son_sunucu_ms ilkesiyle aynı).
    yeni.son_gorulen_ms = Some(yeni.son_gorulen_ms.unwrap_or(i64::MIN).max(simdi_ms));
    let kapi = match karar.durum {
        DenemeDurumu::Kilitli => kilit(
            "Deneme süresi doldu",
            "Finansal Hesaplayıcı'nın 15 günlük denemesi bitti.",
        ),
        _ => Kapi::Deneme {
            kalan_gun: karar.kalan_gun,
        },
    };
    (kapi, yeni)
}

/// `%APPDATA%\FinansalHesaplayici\lisans.json` — model::dosya_yolu ile aynı kök.
pub fn kayit_yolu() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| std::env::temp_dir());
    base.join("FinansalHesaplayici").join("lisans.json")
}

fn kayit_oku(yol: &Path) -> Kayit {
    std::fs::read(yol)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

/// Atomik yazım: .tmp + rename (EK lease kalıbı).
fn kayit_yaz(yol: &Path, kayit: &Kayit) -> std::io::Result<()> {
    if let Some(dir) = yol.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = yol.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(kayit).unwrap())?;
    std::fs::rename(&tmp, yol)
}

fn dosya_tarihi(p: &Path) -> Option<String> {
    let m = std::fs::metadata(p).ok()?;
    let t = m.created().or_else(|_| m.modified()).ok()?;
    Some(DateTime::<Local>::from(t).format("%Y-%m-%d").to_string())
}

/// Kurulum-günü tabanı: kayıt dosyasının VE exe'nin doğum tarihinden EN ESKİSİ.
/// Kayıt silinip yeniden yaratılırsa exe doğumu eski tabanı taşımaya devam eder.
fn taban_tarihi(kayit_yolu: &Path) -> Option<String> {
    let exe = std::env::current_exe().ok().and_then(|p| dosya_tarihi(&p));
    match (dosya_tarihi(kayit_yolu), exe) {
        // YYYY-MM-DD sözel sıralama = kronolojik sıralama.
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    }
}

/// Üretim girişi: diski okur, karar verir, kayıt değiştiyse yazar.
/// Yazma hatası kapıyı KAPATMAZ (salt-okunur diskte dürüst kullanıcı cezalanmaz;
/// exe-doğum tabanı suiistimali zaten sınırlar).
pub fn kapi_uygula() -> Kapi {
    let yol = kayit_yolu();
    let kayit = kayit_oku(&yol);
    let taban = taban_tarihi(&yol);
    let bugun = Local::now().format("%Y-%m-%d").to_string();
    let simdi = Utc::now().timestamp_millis();
    let (kapi, yeni) = kapi_karari(&kayit, taban.as_deref(), &bugun, simdi);
    if yeni != kayit {
        let _ = kayit_yaz(&yol, &yeni);
    }
    kapi
}

/// Kilit ekranının ürün temasını veri.json'dan okuması (okunamazsa varsayılan).
pub(crate) fn kilit_tema_ayarlari() -> (String, f64) {
    let varsayilan = (String::new(), 1.0);
    let base = dirs::config_dir().unwrap_or_else(|| std::env::temp_dir());
    let yol = base.join("FinansalHesaplayici").join("veri.json");
    let Ok(metin) = std::fs::read_to_string(&yol) else {
        return varsayilan;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(metin.trim_start_matches('\u{feff}'))
    else {
        return varsayilan;
    };
    let tema = v
        .get("settings")
        .and_then(|s| s.get("theme"))
        .and_then(|t| t.as_str())
        .map(str::to_string)
        .unwrap_or_default();
    let olcek = v
        .get("settings")
        .and_then(|s| s.get("font_scale"))
        .and_then(|o| o.as_f64())
        .filter(|o| (0.5..=3.0).contains(o))
        .unwrap_or(1.0);
    (tema, olcek)
}

/// Uygulama kabuğu: denemeyse en alta ince şerit ekler, gerisi FhApp'in.
/// (Üst bant değil: FH'nin özelleştirilmiş başlık çubuğu en üste sahip.)
pub struct Kirpik<A> {
    pub ic: A,
    pub deneme_kalan: Option<i64>,
}

impl<A: eframe::App> eframe::App for Kirpik<A> {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // eframe 0.37: kök Ui'yi framework verir. Şerit en üste ayrılır,
        // sonra FhApp kalan alanı kullanır (FhApp'in kendi üst çubuğu alta kayar).
        if let Some(gun) = self.deneme_kalan {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(format!("Deneme sürümü — {gun} gün kaldı")).strong(),
                );
                ui.label(SATIR);
            });
            ui.separator();
        }
        self.ic.ui(ui, frame);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.ic.save(storage);
    }
}

/// Kilit ekranı: ana pencere hiç kurulmaz; yalnız gerekçe + iletişim.
pub struct KilitEkrani {
    pub baslik: String,
    pub gerekce: String,
}

impl eframe::App for KilitEkrani {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.vertical_centered(|ui| {
            ui.add_space(48.0);
            ui.label(egui::RichText::new(&self.baslik).heading().strong());
            ui.add_space(12.0);
            ui.label(&self.gerekce);
            ui.label(SATIR);
            ui.add_space(24.0);
            if ui.button("Kapat").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    const AN: i64 = 1_763_000_000_000; // 2026-11-21 civarı, sabit "şimdi"

    #[test]
    fn yeni_kurulum_deneme_baslar_15_gun() {
        let (kap, yeni) = kapi_karari(&Kayit::default(), None, "2026-11-21", AN);
        assert_eq!(kap, Kapi::Deneme { kalan_gun: 15 });
        assert_eq!(yeni.deneme_baslangic.as_deref(), Some("2026-11-21"));
        assert_eq!(yeni.son_gorulen_ms, Some(AN));
    }

    #[test]
    fn taban_elle_ileri_yazilamaz_kurulum_kazanir() {
        // Kullanıcı kaydı açıp başlangıcı bugüne çekmeye çalıştı:
        let kayit = Kayit {
            deneme_baslangic: Some("2026-11-21".into()),
            ..Default::default()
        };
        let (kap, _) = kapi_karari(&kayit, Some("2026-11-01"), "2026-11-21", AN);
        assert!(matches!(kap, Kapi::Kilitli { .. }), "eski taban kazanmalı: {kap:?}");
    }

    #[test]
    fn kayit_silinince_exe_dogumu_taban_olur() {
        // Dosya silindi → taze kayıt + eski exe doğumu → deneme BİTİRİLİR.
        let (kap, _) = kapi_karari(&Kayit::default(), Some("2026-10-01"), "2026-11-21", AN);
        assert!(matches!(kap, Kapi::Kilitli { .. }));
    }

    #[test]
    fn saat_gerisi_kapati() {
        let kayit = Kayit {
            deneme_baslangic: Some("2026-11-18".into()),
            son_gorulen_ms: Some(AN + 3_600_000), // saat bir saat ilerisi görüldü
            anahtar: None,
        };
        let (kap, _) = kapi_karari(&kayit, None, "2026-11-21", AN);
        assert!(matches!(kap, Kapi::Kilitli { .. }), "saat geri → kilitli: {kap:?}");
    }

    #[test]
    fn sahte_anahtar_denemeye_dusurmez_kilitler() {
        let kayit = Kayit {
            anahtar: Some("FH-ABCD-EFGH-JKLM-NPQR.2027-12-31.1.TAAAAA".into()),
            ..Default::default()
        };
        let (kap, _) = kapi_karari(&kayit, None, "2026-11-21", AN);
        match kap {
            Kapi::Kilitli { baslik, .. } => assert_eq!(baslik, "Lisans doğrulanamadı"),
            other => panic!("kilitli beklenirdi: {other:?}"),
        }
    }

    /// Sözleşme §5 biçiminde imzalı anahtar üretir: `KEY.bitis.maxdev.b64(imza)`,
    /// imzalanan yük = ilk üç bölümün noktalı birleşimi (lisans.py:1090).
    fn anahtar_uret(seed: u8, bitis: &str) -> (String, String) {
        use ed25519_dalek::{Signer, SigningKey};
        use muhasebe_lisans::b64::kodla_urlsafe;
        let sk = SigningKey::from_bytes(&[seed; 32]);
        let anahtar = format!("{}-ABCD-EFGH-JKLM-NPQR", ONEK);
        let yuk = format!("{anahtar}.{bitis}.1");
        let imza = sk.sign(yuk.as_bytes());
        (
            format!("{yuk}.{}", kodla_urlsafe(&imza.to_bytes())),
            kodla_urlsafe(&sk.verifying_key().to_bytes()),
        )
    }

    #[test]
    fn gercek_imzali_anahtar_kapiyi_acar_baska_anahtar_acmaz() {
        let (metin, acik) = anahtar_uret(1, "2027-12-31");
        let kayit = Kayit {
            anahtar: Some(metin.clone()),
            ..Default::default()
        };
        // Doğru açık anahtarla: TAM AÇIK (deneme sayacı işlemez).
        let (kap, _) = kapi_karari_ile(&kayit, None, "2026-11-21", AN, &acik);
        assert_eq!(kap, Kapi::Lisans);
        // Başkasının imzaladığı aynı metin gömülü KDV anahtarıyla AÇMAZ.
        let (kap, _) = kapi_karari(&kayit, None, "2026-11-21", AN);
        assert!(matches!(kap, Kapi::Kilitli { .. }), "yabancı imza reddedilmeli");
    }

    #[test]
    fn suresi_dolan_anahtar_denemeye_donusmez() {
        // İmzası geçerli ama süre çoktan dolmuş (grece 14 gün çok dışında) →
        // "süresi dolmuş"; DENEME dalına düşmek YOK (bedava tur yok).
        let (metin, acik) = anahtar_uret(2, "2025-01-01");
        let kayit = Kayit {
            anahtar: Some(metin),
            ..Default::default()
        };
        let (kap, _) = kapi_karari_ile(&kayit, None, "2026-11-21", AN, &acik);
        match kap {
            Kapi::Kilitli { baslik, .. } => assert_eq!(baslik, "Lisans süresi dolmuş"),
            other => panic!("süre dolumu kilitler: {other:?}"),
        }
    }

    #[test]
    fn sinir_gun_kalan_sifir_deneme() {
        // Canlı .pyd altın vektörü: 2026-11-06 + 15 gün → 2026-11-21, kalan 0 = deneme.
        let kayit = Kayit {
            deneme_baslangic: Some("2026-11-06".into()),
            ..Default::default()
        };
        let (kap, _) = kapi_karari(&kayit, None, "2026-11-21", AN);
        assert_eq!(kap, Kapi::Deneme { kalan_gun: 0 });
    }

    #[test]
    fn kayit_yaz_oku_roundtrip_atomik() {
        let dir = std::env::temp_dir().join(format!(
            "fh-lisans-{}-rt",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let yol = dir.join("lisans.json");
        let kayit = Kayit {
            deneme_baslangic: Some("2026-11-21".into()),
            son_gorulen_ms: Some(AN),
            anahtar: None,
        };
        kayit_yaz(&yol, &kayit).unwrap();
        assert_eq!(kayit_oku(&yol), kayit);
        // Bozuk JSON → panik yok, varsayılan kayıt (taban yine de korur).
        std::fs::write(&yol, b"{bozuk").unwrap();
        assert_eq!(kayit_oku(&yol), Kayit::default());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
