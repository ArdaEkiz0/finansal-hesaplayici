//! 20 modun alan tanımları ve `Girdi` eşlemesi. Formül içermez; hesaplama
//! yalnızca `fh_core::hesapla` tarafından yapılır.

use fh_core::Girdi;

use crate::model::{Alan, InputState};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    Metin,
    CokSatir,
    Secim,
    PaySecim,
}

#[derive(Debug, Clone, Copy)]
pub struct FieldDef {
    pub alan: Alan,
    pub etiket: &'static str,
    pub ipucu: &'static str,
    pub tur: FieldKind,
    pub secenekler: &'static [(&'static str, &'static str)],
}

#[derive(Debug, Clone, Copy)]
pub struct ModeDef {
    pub id: &'static str,
    pub etiket: &'static str,
    pub grup: &'static str,
    pub alanlar: &'static [FieldDef],
}

const SEC_DAHIL: &[(&str, &str)] = &[("KDV hariç", "haric"), ("KDV dahil", "dahil")];
const SEC_STOPAJ: &[(&str, &str)] = &[("Brütten net", "brut"), ("Netten brüt", "netten")];
const SEC_AMORTISMAN: &[(&str, &str)] =
    &[("Normal (doğrusal)", "normal"), ("Azalan bakiyeler", "azalan")];
const SEC_DOVIZ: &[(&str, &str)] = &[("Döviz → TL", "toTL"), ("TL → Döviz", "fromTL")];
const SEC_BATCH: &[(&str, &str)] = &[("KDV", "kdv"), ("Stopaj", "stopaj"), ("Tevkifat", "tevkifat")];
pub const SEC_PAY: &[(&str, &str)] = &[
    ("1/10", "1"), ("2/10", "2"), ("3/10", "3"), ("4/10", "4"), ("5/10", "5"),
    ("6/10", "6"), ("7/10", "7"), ("8/10", "8"), ("9/10", "9"), ("10/10", "10"),
];

/// Mod navigasyon panelindeki dikey kaydırma alanının asgari görünür yüksekliği (px).
/// `app.rs::mod_nav_panel` bu değerle `ScrollArea::max_height` sınırını ayarlar;
/// sınır konulmazsa fare tekerleğiyle kaydırma çalışmaz.
#[allow(dead_code)] // eski ScrollArea alt sınırı; testlerde korunur
pub const MOD_NAV_MIN_HEIGHT: f32 = 120.0;

pub const MODLAR: &[ModeDef] = &[
    ModeDef {
        id: "kdv", etiket: "KDV Hesaplama", grup: "KDV & Stopaj",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Tutar", ipucu: "Örn. 1.234,56", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Oran, etiket: "KDV oranı (%)", ipucu: "20", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Secim, etiket: "Hesaplama yönü", ipucu: "", tur: FieldKind::Secim, secenekler: SEC_DAHIL },
        ],
    },
    ModeDef {
        id: "stopaj", etiket: "Stopaj / Tevkifat", grup: "KDV & Stopaj",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Brüt tutar", ipucu: "Örn. 10.000", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Oran, etiket: "Stopaj oranı (%)", ipucu: "15", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Secim, etiket: "Hesaplama yönü", ipucu: "", tur: FieldKind::Secim, secenekler: SEC_STOPAJ },
        ],
    },
    ModeDef {
        id: "tevkifat", etiket: "KDV Tevkifatı", grup: "KDV & Stopaj",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Matrah", ipucu: "Örn. 5.000", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Oran, etiket: "KDV oranı (%)", ipucu: "20", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::TevkifatPay, etiket: "Tevkifat payı", ipucu: "", tur: FieldKind::PaySecim, secenekler: SEC_PAY },
            FieldDef { alan: Alan::TevkifatKod, etiket: "İşlem kodu (opsiyonel)", ipucu: "Örn. 4A", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "fiyat", etiket: "Fiyat Etiketi", grup: "KDV & Stopaj",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Etiket fiyatı", ipucu: "Örn. 1.200", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Oran, etiket: "KDV oranı (%)", ipucu: "20", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Secim, etiket: "Etiket KDV dahil mi?", ipucu: "", tur: FieldKind::Secim, secenekler: SEC_DAHIL },
            FieldDef { alan: Alan::Ek1, etiket: "İndirim oranları (%)", ipucu: "Virgülle: 10, 5", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "kdvCompare", etiket: "KDV Karşılaştırma", grup: "KDV & Stopaj",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Tutar", ipucu: "Örn. 1.000", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Oran, etiket: "KDV oranı (%)", ipucu: "20", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "discount", etiket: "Zincirleme İndirim", grup: "KDV & Stopaj",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Orijinal fiyat", ipucu: "Örn. 2.000", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Ek1, etiket: "İndirim oranları (%)", ipucu: "Virgülle: 20, 10, 5", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "margin", etiket: "Marj → Satış Fiyatı", grup: "Kâr & Yüzde",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Maliyet", ipucu: "Örn. 800", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Ikinci, etiket: "Hedef marj (%)", ipucu: "25", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "markup", etiket: "Kâr Oranı → Fiyat", grup: "Kâr & Yüzde",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Maliyet", ipucu: "Örn. 800", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Ikinci, etiket: "Kâr oranı (%)", ipucu: "30", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "percent", etiket: "Yüzde Hesaplama", grup: "Kâr & Yüzde",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Yüzde (%)", ipucu: "Örn. 18", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Ikinci, etiket: "Değer", ipucu: "Örn. 4.500", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "compound", etiket: "Bileşik Faiz", grup: "Faiz & Zaman",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Anapara", ipucu: "Örn. 10.000", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Ikinci, etiket: "Yıllık faiz (%)", ipucu: "45", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Ek1, etiket: "Yıl", ipucu: "Örn. 3", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Ek2, etiket: "Yılda bileşme", ipucu: "Örn. 12", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "gecikme", etiket: "Gecikme Faizi", grup: "Faiz & Zaman",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Anapara", ipucu: "Örn. 5.000", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Ikinci, etiket: "Aylık faiz (%)", ipucu: "3,5", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Gun, etiket: "Gün sayısı", ipucu: "Örn. 45", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "amortisman", etiket: "Amortisman", grup: "Faiz & Zaman",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Maliyet", ipucu: "Örn. 100.000", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Oran, etiket: "Yıllık oran (%)", ipucu: "Boş bırakırsan otomatik", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Yil, etiket: "Yıl", ipucu: "Örn. 5", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Secim, etiket: "Yöntem", ipucu: "", tur: FieldKind::Secim, secenekler: SEC_AMORTISMAN },
        ],
    },
    ModeDef {
        id: "doviz", etiket: "Döviz Çevirme", grup: "Diğer",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Tutar", ipucu: "Örn. 1.000", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Ikinci, etiket: "Kur", ipucu: "Örn. 34,50", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Secim, etiket: "Yön", ipucu: "", tur: FieldKind::Secim, secenekler: SEC_DOVIZ },
            FieldDef { alan: Alan::TevkifatKod, etiket: "Para birimi", ipucu: "USD, EUR, GBP...", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "batch", etiket: "Toplu Hesaplama", grup: "Diğer",
        alanlar: &[
            FieldDef { alan: Alan::Secim, etiket: "Alt mod", ipucu: "", tur: FieldKind::Secim, secenekler: SEC_BATCH },
            FieldDef { alan: Alan::Oran, etiket: "Varsayılan oran (%)", ipucu: "20", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Ek1, etiket: "Satırlar", ipucu: "Her satır: tutar;oran", tur: FieldKind::CokSatir, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "donem", etiket: "Dönem KDV", grup: "Diğer",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Satış matrahı", ipucu: "Örn. 100.000", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Ikinci, etiket: "Alış matrahı", ipucu: "Örn. 40.000", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Oran, etiket: "KDV oranı (%)", ipucu: "20", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Ek1, etiket: "Önceki devreden", ipucu: "Örn. 1.500", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    // --- v5.6: 5 yeni mod (fh-core formülleriyle eşleşir) ---
    ModeDef {
        id: "damga", etiket: "Damga Vergisi", grup: "KDV & Stopaj",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Matrah", ipucu: "Örn. 10.000", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Oran, etiket: "Oran (‰)", ipucu: "Örn. 0,948", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "emlak", etiket: "Emlak Vergisi", grup: "KDV & Stopaj",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Matrah", ipucu: "Örn. 100.000", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Oran, etiket: "Oran (‰)", ipucu: "Örn. 1", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "kredi", etiket: "Kredi Taksiti", grup: "Faiz & Kredi",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Anapara", ipucu: "Örn. 100.000", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Oran, etiket: "Aylık faiz (%)", ipucu: "Örn. 3,5", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Ay, etiket: "Vade (ay)", ipucu: "1–360", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "basitFaiz", etiket: "Basit Faiz", grup: "Faiz & Kredi",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Anapara", ipucu: "Örn. 10.000", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Oran, etiket: "Yıllık faiz (%)", ipucu: "Örn. 45", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Gun, etiket: "Gün sayısı", ipucu: "1–3650", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
    ModeDef {
        id: "netBrut", etiket: "Net → Brüt", grup: "Kâr & Yüzde",
        alanlar: &[
            FieldDef { alan: Alan::Birinci, etiket: "Net tutar", ipucu: "Örn. 8.500", tur: FieldKind::Metin, secenekler: &[] },
            FieldDef { alan: Alan::Oran, etiket: "Stopaj oranı (%)", ipucu: "Örn. 15", tur: FieldKind::Metin, secenekler: &[] },
        ],
    },
];

pub fn mod_def(id: &str) -> Option<&'static ModeDef> {
    MODLAR.iter().find(|m| m.id == id)
}

pub fn mod_etiketi(id: &str) -> &'static str {
    mod_def(id).map(|m| m.etiket).unwrap_or("Bilinmeyen mod")
}

#[allow(dead_code)] // eski gruplu çizim yardımcısı; testlerde korunur
pub fn gruplar() -> Vec<&'static str> {
    let mut out = Vec::new();
    for m in MODLAR {
        if !out.contains(&m.grup) {
            out.push(m.grup);
        }
    }
    out
}

/// Boş seçim alanlarını modun ilk seçeneğiyle doldurur.
pub fn varsayilanlari_doldur(inp: &mut InputState, mod_id: &str) {
    if let Some(def) = mod_def(mod_id) {
        for f in def.alanlar {
            let v = inp.get_mut(f.alan);
            if v.is_empty() {
                if let Some((_, value)) = f.secenekler.first() {
                    v.push_str(value);
                }
            }
        }
    }
    if inp.tevkifat_pay.is_empty() {
        inp.tevkifat_pay.push_str("1");
    }
}

fn virgulden_bol(s: &str) -> Vec<String> {
    s.split([',', ';', '|'])
        .map(str::trim)
        .filter(|x| !x.is_empty())
        .map(str::to_string)
        .collect()
}

fn tamsayi(s: &str) -> i32 {
    s.trim().parse::<i32>().unwrap_or(0)
}

/// UI alanlarından `fh_core::Girdi` üretir — formül yok, yalnızca eşleme.
pub fn build_girdi(mod_id: &str, inp: &InputState) -> Girdi {
    let mut g = Girdi {
        birinci: inp.birinci.clone(),
        ikinci: inp.ikinci.clone(),
        oran: inp.oran.clone(),
        tevkifat_pay: tamsayi(&inp.tevkifat_pay),
        tevkifat_kod: inp.tevkifat_kod.clone(),
        yil: tamsayi(&inp.yil),
        ay: tamsayi(&inp.ay),
        gun: tamsayi(&inp.gun),
        ek: Vec::new(),
    };
    match mod_id {
        "kdv" => {
            if inp.secim == "dahil" {
                g.ikinci = "dahil".to_string();
            }
        }
        "stopaj" => {
            if inp.secim == "netten" {
                g.tevkifat_kod = "netten".to_string();
            }
        }
        "fiyat" => {
            if inp.secim == "dahil" {
                g.ikinci = "dahil".to_string();
            }
            g.ek = virgulden_bol(&inp.ek1);
        }
        "discount" => g.ek = virgulden_bol(&inp.ek1),
        "compound" => g.ek = vec![inp.ek1.clone(), inp.ek2.clone()],
        "amortisman" => {
            if inp.secim == "azalan" {
                g.tevkifat_kod = "azalan".to_string();
            }
        }
        "doviz" => {
            g.ek = vec![if inp.secim == "fromTL" { "fromTL" } else { "toTL" }.to_string()];
        }
        "batch" => {
            g.tevkifat_kod = inp.secim.clone();
            g.ek = inp.ek1.lines().map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
        }
        "donem" => g.ek = vec![inp.ek1.clone()],
        _ => {}
    }
    g
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp() -> InputState {
        let mut i = InputState::default();
        varsayilanlari_doldur(&mut i, "kdv");
        i
    }

    #[test]
    fn yirmi_mod_tanli() {
        assert_eq!(MODLAR.len(), 20);
        for id in ["kdv", "stopaj", "tevkifat", "fiyat", "kdvCompare", "margin", "markup",
                   "discount", "percent", "compound", "gecikme", "amortisman", "doviz", "batch", "donem",
                   "damga", "emlak", "kredi", "basitFaiz", "netBrut"] {
            assert!(mod_def(id).is_some(), "{id}");
        }
    }

    #[test]
    fn varsayilan_secim_dahil_haric() {
        assert_eq!(inp().secim, "haric");
    }

    #[test]
    fn girdi_kdv_dahil_bayragi() {
        let mut i = inp();
        i.birinci = "1200".into();
        i.oran = "20".into();
        let g = build_girdi("kdv", &i);
        assert_eq!(g.ikinci, "");
        i.secim = "dahil".into();
        let g = build_girdi("kdv", &i);
        assert_eq!(g.ikinci, "dahil");
    }

    #[test]
    fn girdi_tevkifat_payi() {
        let mut i = inp();
        i.tevkifat_pay = "7".into();
        assert_eq!(build_girdi("tevkifat", &i).tevkifat_pay, 7);
        i.tevkifat_pay = "bozuk".into();
        assert_eq!(build_girdi("tevkifat", &i).tevkifat_pay, 0);
    }

    #[test]
    fn girdi_doviz_yonu_ve_para_birimi() {
        let mut i = inp();
        i.secim = "fromTL".into();
        i.tevkifat_kod = "EUR".into();
        let g = build_girdi("doviz", &i);
        assert_eq!(g.ek, vec!["fromTL"]);
        assert_eq!(g.tevkifat_kod, "EUR");
    }

    #[test]
    fn girdi_batch_satirlari() {
        let mut i = inp();
        i.ek1 = "100;20\n200\n\n 300|10 ".into();
        let g = build_girdi("batch", &i);
        assert_eq!(g.ek, vec!["100;20", "200", "300|10"]);
    }

    #[test]
    fn girdi_uzerinden_hesaplama_kdv() {
        let mut i = inp();
        i.birinci = "100".into();
        i.oran = "20".into();
        let r = fh_core::hesapla("kdv", &build_girdi("kdv", &i)).unwrap();
        assert_eq!(r.ana, "20");
        assert_eq!(r.alt, "120");
    }

    #[test]
    fn girdi_stopaj_netten_kodu() {
        let mut i = inp();
        i.secim = "netten".into();
        assert_eq!(build_girdi("stopaj", &i).tevkifat_kod, "netten");
    }

    #[test]
    fn mod_etiketi_ve_gruplar() {
        assert_eq!(mod_etiketi("kdvCompare"), "KDV Karşılaştırma");
        assert_eq!(mod_etiketi("yok"), "Bilinmeyen mod");
        assert_eq!(gruplar().len(), 5);
        assert!(gruplar().contains(&"Faiz & Kredi"));
    }

    // --- Alan eşlemesi → fh_core uçtan uca (20 modun tamamı) ---

    fn ui(mod_id: &str, birinci: &str, ikinci: &str, oran: &str) -> InputState {
        let mut i = InputState {
            birinci: birinci.to_string(),
            ikinci: ikinci.to_string(),
            oran: oran.to_string(),
            ..InputState::default()
        };
        varsayilanlari_doldur(&mut i, mod_id);
        i
    }

    fn hesapla_ui(mod_id: &str, i: &InputState) -> fh_core::HesapSonucu {
        fh_core::hesapla(mod_id, &build_girdi(mod_id, i))
            .unwrap_or_else(|| panic!("{mod_id} UI→core None döndü: {i:?}"))
    }

    #[test]
    fn ui_kdv() {
        let i = ui("kdv", "1000", "", "20");
        let r = hesapla_ui("kdv", &i);
        assert_eq!((r.ana.as_str(), r.alt.as_str()), ("200", "1200"));
        let mut i = ui("kdv", "1200", "", "20");
        i.secim = "dahil".into();
        let r = hesapla_ui("kdv", &i);
        assert_eq!((r.ana.as_str(), r.alt.as_str()), ("200", "1000"));
    }

    #[test]
    fn ui_stopaj() {
        let r = hesapla_ui("stopaj", &ui("stopaj", "50000", "", "20"));
        assert_eq!((r.ana.as_str(), r.alt.as_str()), ("10000", "40000"));
        let mut i = ui("stopaj", "40000", "", "20");
        i.secim = "netten".into();
        let r = hesapla_ui("stopaj", &i);
        assert_eq!((r.ana.as_str(), r.alt.as_str()), ("10000", "50000"));
    }

    #[test]
    fn ui_tevkifat() {
        let mut i = ui("tevkifat", "10000", "", "20");
        i.tevkifat_pay = "5".into();
        let r = hesapla_ui("tevkifat", &i);
        assert_eq!((r.ana.as_str(), r.alt.as_str()), ("1000", "11000"));
    }

    #[test]
    fn ui_fiyat() {
        let r = hesapla_ui("fiyat", &ui("fiyat", "1000", "", "20"));
        assert_eq!(r.ana, "1200");
        let mut i = ui("fiyat", "1000", "", "20");
        i.ek1 = "10,5".into();
        assert_eq!(hesapla_ui("fiyat", &i).ana, "1026");
        let mut i = ui("fiyat", "1200", "", "20");
        i.secim = "dahil".into();
        assert_eq!(hesapla_ui("fiyat", &i).ana, "1000");
    }

    #[test]
    fn ui_kdv_compare() {
        let r = hesapla_ui("kdvCompare", &ui("kdvCompare", "1000", "", "20"));
        assert_eq!((r.ana.as_str(), r.alt.as_str()), ("33.33", "1200"));
    }

    #[test]
    fn ui_margin_ve_markup() {
        let r = hesapla_ui("margin", &ui("margin", "100", "20", ""));
        assert_eq!((r.ana.as_str(), r.alt.as_str()), ("125", "25"));
        let r = hesapla_ui("markup", &ui("markup", "100", "20", ""));
        assert_eq!((r.ana.as_str(), r.alt.as_str()), ("120", "20"));
    }

    #[test]
    fn ui_discount() {
        let mut i = ui("discount", "1000", "", "");
        i.ek1 = "10,10".into();
        let r = hesapla_ui("discount", &i);
        assert_eq!((r.ana.as_str(), r.alt.as_str()), ("810", "190"));
    }

    #[test]
    fn ui_percent() {
        let r = hesapla_ui("percent", &ui("percent", "20", "500", ""));
        assert_eq!(r.ana, "100");
    }

    #[test]
    fn ui_compound() {
        let mut i = ui("compound", "1000", "12", "");
        i.ek1 = "1".into();
        i.ek2 = "12".into();
        let r = hesapla_ui("compound", &i);
        assert_eq!((r.ana.as_str(), r.alt.as_str()), ("1126.83", "126.83"));
    }

    #[test]
    fn ui_gecikme() {
        let mut i = ui("gecikme", "10000", "4.5", "");
        i.gun = "30".into();
        let r = hesapla_ui("gecikme", &i);
        assert_eq!((r.ana.as_str(), r.alt.as_str()), ("450", "10450"));
    }

    #[test]
    fn ui_amortisman() {
        let mut i = ui("amortisman", "500000", "", "20");
        i.yil = "5".into();
        let r = hesapla_ui("amortisman", &i);
        assert_eq!((r.ana.as_str(), r.alt.as_str()), ("100000", "500000"));
        let mut i = ui("amortisman", "1000", "", "20");
        i.yil = "3".into();
        i.secim = "azalan".into();
        let r = hesapla_ui("amortisman", &i);
        assert_eq!(r.alt, "784");
    }

    #[test]
    fn ui_doviz() {
        let mut i = ui("doviz", "100", "40", "");
        i.tevkifat_kod = "USD".into();
        assert_eq!(hesapla_ui("doviz", &i).ana, "4000");
        let mut i = ui("doviz", "4000", "40", "");
        i.secim = "fromTL".into();
        assert_eq!(hesapla_ui("doviz", &i).ana, "100");
    }

    #[test]
    fn ui_batch() {
        let mut i = ui("batch", "", "", "20");
        i.ek1 = "1000\n2000\nboz".into();
        let r = hesapla_ui("batch", &i);
        assert_eq!((r.ana.as_str(), r.alt.as_str()), ("3600", "600"));
        i.secim = "stopaj".into();
        assert_eq!(hesapla_ui("batch", &i).ana, "2400");
        i.secim = "tevkifat".into();
        i.tevkifat_pay = "5".into();
        i.ek1 = "10000".into();
        assert_eq!(hesapla_ui("batch", &i).ana, "11000");
    }

    #[test]
    fn ui_donem() {
        let mut i = ui("donem", "100000", "60000", "20");
        i.ek1 = "1000".into();
        let r = hesapla_ui("donem", &i);
        assert_eq!(r.ana, "7000");
    }

    #[test]
    fn ui_tum_modlar_varsayilan_alanlarla_sonuc_verir() {
        // Her mod, varsayılan seçimler doldurulduğunda makul girdiyle Some döner.
        for m in MODLAR {
            let mut i = ui(m.id, "1000", "20", "20");
            i.ek1 = "1".into(); // compound yılı / fiyat indirimi / donem devredeni
            i.ek2 = "12".into(); // compound bileşme sıklığı
            i.gun = "30".into();
            i.yil = "5".into();
            i.ay = "12".into(); // kredi vadesi
            assert!(
                fh_core::hesapla(m.id, &build_girdi(m.id, &i)).is_some(),
                "{} varsayılan alanlarla çalışmalı",
                m.id
            );
        }
    }
}
