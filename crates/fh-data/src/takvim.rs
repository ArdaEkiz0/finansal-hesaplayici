//! Statik vergi takvimi — masaüstü offline. Kural: özel duyuru yoksa standart VUK süreleri.
//! Electron `src/data/takvim.ts` birebir portu (düzeltilmiş 4. çeyrek hatası dahil).

use chrono::{Datelike, NaiveDate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TakvimKayit {
    pub id: String,
    pub ad: String,
    pub tip: String,
    /// ISO "2026-05-28"
    pub tarih: String,
    pub donemi: String,
    pub aciklama: String,
}

const AY_KISA: [&str; 13] = [
    "Aralık", "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", "Temmuz", "Ağustos",
    "Eylül", "Ekim", "Kasım", "Aralık",
];

fn iso(y: i32, m1: u32, gun: u32) -> String {
    format!("{y:04}-{m1:02}-{gun:02}")
}

fn artik_yil(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn ay_sonu(y: i32, m1: u32) -> u32 {
    match m1 {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if artik_yil(y) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn donem_adi(m1: u32, y: i32) -> String {
    format!("{} {}", AY_KISA[m1 as usize], y)
}

/// Bir yılın tüm son gün kayıtları: her ay için takip eden dönemin beyanları.
fn kelimeleri_olustur(y: i32) -> Vec<TakvimKayit> {
    let mut out = Vec::new();
    for m1 in 1..=12u32 {
        let pm = m1 - 1;
        let py = if pm == 0 { y - 1 } else { y };
        let pm_ad = donem_adi(if pm == 0 { 12 } else { pm }, py);

        out.push(TakvimKayit {
            id: format!("kdv-{y}-{m1}"),
            ad: "KDV Beyannamesi".to_string(),
            tip: "kdv".to_string(),
            tarih: iso(y, m1, 28),
            donemi: pm_ad.clone(),
            aciklama: format!("{pm_ad} KDV beyannamesi verilir ve KDV ödemesi yapılır"),
        });

        out.push(TakvimKayit {
            id: format!("mpd-{y}-{m1}"),
            ad: "Muhtasar & Prim Hizmet".to_string(),
            tip: "mpd".to_string(),
            tarih: iso(y, m1, 26),
            donemi: pm_ad.clone(),
            aciklama: format!("{pm_ad} gelir tevkifatı ve SGK prim beyannamesi"),
        });

        out.push(TakvimKayit {
            id: format!("dv-{y}-{m1}"),
            ad: "Damga Vergisi".to_string(),
            tip: "dV".to_string(),
            tarih: iso(y, m1, 26),
            donemi: pm_ad.clone(),
            aciklama: format!("{pm_ad} damga vergisi beyannamesi"),
        });

        out.push(TakvimKayit {
            id: format!("babs-{y}-{m1}"),
            ad: "Ba-Bs Formları".to_string(),
            tip: "babs".to_string(),
            tarih: iso(y, m1, ay_sonu(y, m1)),
            donemi: pm_ad.clone(),
            aciklama: format!("{pm_ad} Ba/Bs formu bilgileri"),
        });

        // Geçici vergi: 3 aylık dönem sonu Mart/Haziran/Eylül/Aralık → beyan: 2 ay sonranın 15'i.
        // pm=0 (Aralık dönemi, m1=1) → Şubat 15: Electron'da düzeltilmiş davranış, aynen korunur.
        if pm == 3 || pm == 6 || pm == 9 || pm == 0 {
            let beyan_ay = if m1 + 1 <= 12 { m1 + 1 } else { 1 };
            let beyan_yil = if beyan_ay == 1 { y + 1 } else { y };
            let ceyrek_ad = match pm {
                0 => format!("4. çeyrek (Ekim–Aralık) {py}"),
                3 => format!("1. çeyrek (Ocak–Mart) {py}"),
                6 => format!("2. çeyrek (Nisan–Haziran) {py}"),
                9 => format!("3. çeyrek (Temmuz–Eylül) {py}"),
                _ => String::new(),
            };
            out.push(TakvimKayit {
                id: format!("gvv-{beyan_yil}-{beyan_ay}"),
                ad: "Geçici Vergi".to_string(),
                tip: "gVV".to_string(),
                tarih: iso(beyan_yil, beyan_ay, 15),
                donemi: ceyrek_ad,
                aciklama: "Geçici vergi beyannamesi; vergi beyandan itibaren 7 gün içinde ödenir"
                    .to_string(),
            });
        }
    }
    out
}

/// Bugünden sonrası: sıralı yaklaşan takvim (bu yıl + gelecek yıl şablonu).
pub fn takvim_listesi(bugun: NaiveDate) -> Vec<TakvimKayit> {
    let y = bugun.year();
    let mut sablon = kelimeleri_olustur(y);
    sablon.extend(kelimeleri_olustur(y + 1));
    let bugun_key = iso(bugun.year(), bugun.month(), bugun.day());
    let mut out: Vec<TakvimKayit> = sablon
        .into_iter()
        .filter(|k| k.tarih.as_str() >= bugun_key.as_str())
        .collect();
    out.sort_by(|a, b| a.tarih.cmp(&b.tarih));
    out
}

/// Bugün ile tarih arasındaki gün farkı (negatif olabilir). Bozuk tarih → 0.
pub fn gun_kaldi(tarih: &str, bugun: NaiveDate) -> i32 {
    let t = NaiveDate::parse_from_str(tarih, "%Y-%m-%d")
        .ok()
        .or_else(|| {
            tarih
                .get(0..10)
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
        });
    match t {
        Some(d) => d.signed_duration_since(bugun).num_days() as i32,
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn kdv_ayin_28inde() {
        let all = kelimeleri_olustur(2026);
        let k = all.iter().find(|k| k.id == "kdv-2026-5").unwrap();
        assert_eq!(k.tarih, "2026-05-28");
        assert_eq!(k.tip, "kdv");
        assert_eq!(k.donemi, "Nisan 2026");
    }

    #[test]
    fn muhtasar_ayin_26sinda() {
        let all = kelimeleri_olustur(2026);
        let k = all.iter().find(|k| k.id == "mpd-2026-3").unwrap();
        assert_eq!(k.tarih, "2026-03-26");
    }

    #[test]
    fn damga_ayin_26sinda() {
        let all = kelimeleri_olustur(2026);
        let k = all.iter().find(|k| k.id == "dv-2026-7").unwrap();
        assert_eq!(k.tarih, "2026-07-26");
        assert_eq!(k.tip, "dV");
    }

    #[test]
    fn babs_artik_yil_subat_29() {
        let all = kelimeleri_olustur(2024);
        let k = all.iter().find(|k| k.id == "babs-2024-2").unwrap();
        assert_eq!(k.tarih, "2024-02-29");
    }

    #[test]
    fn babs_normal_yil_subat_28() {
        let all = kelimeleri_olustur(2023);
        let k = all.iter().find(|k| k.id == "babs-2023-2").unwrap();
        assert_eq!(k.tarih, "2023-02-28");
    }

    #[test]
    fn babs_otuz_bir_cekuyen_ay() {
        let all = kelimeleri_olustur(2026);
        let k = all.iter().find(|k| k.id == "babs-2026-8").unwrap();
        assert_eq!(k.tarih, "2026-08-31");
    }

    #[test]
    fn gecici_vergi_birinci_ceyrek_mayis_15() {
        let all = kelimeleri_olustur(2026);
        let g = all.iter().find(|k| k.tip == "gVV" && k.tarih == "2026-05-15").unwrap();
        assert_eq!(g.donemi, "1. çeyrek (Ocak–Mart) 2026");
        assert_eq!(g.id, "gvv-2026-5");
    }

    #[test]
    fn gecici_vergi_dortuncu_ceyrek_subat_15_hatasi_korunuyor() {
        let all = kelimeleri_olustur(2026);
        let g = all.iter().find(|k| k.tip == "gVV" && k.tarih == "2026-02-15").unwrap();
        assert_eq!(g.donemi, "4. çeyrek (Ekim–Aralık) 2025");
    }

    #[test]
    fn takvim_listesi_gecmisi_atar_ve_siralar() {
        let list = takvim_listesi(d("2026-05-20"));
        assert!(!list.is_empty());
        assert!(list.iter().all(|k| k.tarih.as_str() >= "2026-05-20"));
        for w in list.windows(2) {
            assert!(w[0].tarih <= w[1].tarih);
        }
        assert_eq!(list[0].tarih, "2026-05-26");
    }

    #[test]
    fn gun_kaldi_pozitif_ve_negatif() {
        assert_eq!(gun_kaldi("2026-05-28", d("2026-05-20")), 8);
        assert_eq!(gun_kaldi("2026-05-20", d("2026-05-28")), -8);
        assert_eq!(gun_kaldi("2026-05-20", d("2026-05-20")), 0);
    }

    #[test]
    fn gun_kaldi_bosuk_tarih() {
        assert_eq!(gun_kaldi("bozuk", d("2026-05-20")), 0);
    }

    #[test]
    fn yil_sonu_aralik_donemi_onceki_yili_gosterir() {
        let all = kelimeleri_olustur(2026);
        let k = all.iter().find(|k| k.id == "kdv-2026-1").unwrap();
        assert_eq!(k.donemi, "Aralık 2025");
    }
}
