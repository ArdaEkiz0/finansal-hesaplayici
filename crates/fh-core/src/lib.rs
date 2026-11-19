//! # fh-core
//!
//! Türk mali hesaplayıcının formül çekirdeği. Tüm para matematiği
//! `rust_decimal::Decimal` ile, `ROUND_HALF_UP` (28+ hane) yapılır.
//! Bozuk/eksik girdi panic üretmez, `None` döner.

mod format;
mod modes;
mod words;

pub use format::{format_tr, parse_tr};
pub use words::para_yazisi;

use serde::{Deserialize, Serialize};

/// Hesaplama girdisi. Tutarlar makine biçiminde string (`"1234.56"`) gelir.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Girdi {
    pub birinci: String,
    pub ikinci: String,
    pub oran: String,
    pub tevkifat_pay: i32,
    pub tevkifat_kod: String,
    pub yil: i32,
    pub ay: i32,
    pub gun: i32,
    pub ek: Vec<String>,
}

impl Default for Girdi {
    fn default() -> Self {
        Self {
            birinci: String::new(),
            ikinci: String::new(),
            oran: String::new(),
            tevkifat_pay: 0,
            tevkifat_kod: String::new(),
            yil: 0,
            ay: 0,
            gun: 0,
            ek: Vec::new(),
        }
    }
}

/// Mod sonucu. `ana` ve `alt` makine biçiminde; `satirlar` (etiket, değer) çiftleri.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HesapSonucu {
    pub ana: String,
    pub alt: String,
    pub satirlar: Vec<(String, String)>,
    pub detay: String,
}

/// 20 moddan birini çalıştırır. Bilinmeyen mod veya bozuk girdi → `None`.
///
/// Desteklenen modlar: `kdv, stopaj, tevkifat, fiyat, kdvCompare, margin,
/// markup, discount, percent, compound, gecikme, amortisman, doviz, batch,
/// donem, damga, emlak, kredi, basitFaiz, netBrut`.
pub fn hesapla(mod_: &str, girdi: &Girdi) -> Option<HesapSonucu> {
    modes::run(mod_, girdi)
}
