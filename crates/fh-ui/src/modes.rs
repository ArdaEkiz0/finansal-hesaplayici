//! 15 modun alan tanımları ve `Girdi` eşlemesi. Formül içermez; hesaplama
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
];

pub fn mod_def(id: &str) -> Option<&'static ModeDef> {
    MODLAR.iter().find(|m| m.id == id)
}

pub fn mod_etiketi(id: &str) -> &'static str {
    mod_def(id).map(|m| m.etiket).unwrap_or("Bilinmeyen mod")
}

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
    fn onbes_mod_tanli() {
        assert_eq!(MODLAR.len(), 15);
        for id in ["kdv", "stopaj", "tevkifat", "fiyat", "kdvCompare", "margin", "markup",
                   "discount", "percent", "compound", "gecikme", "amortisman", "doviz", "batch", "donem"] {
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
        assert_eq!(gruplar().len(), 4);
    }
}
