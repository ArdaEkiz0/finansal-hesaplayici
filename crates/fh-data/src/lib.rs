//! fh-data — statik vergi takvimi, dönem arşivi, yedek sanitizasyonu, CSV ve e-fatura.
//!
//! Tüm para değerleri makine biçiminde `String` ("1234.56") döner; kullanıcıya
//! gösterim fh-core::format_tr işidir. fh-core'e doğrudan bağımlılık yoktur:
//! sözleşmedeki parse/yuvarlama yardımcıları bu crate içinde private tutulur.

pub mod csv;
pub mod donem;
pub mod efatura;
pub mod takvim;
pub mod yedek;

pub use csv::{csv_hucre, csv_metin, csv_satir};
pub use donem::{
    compute_donem_net, kalan_devreden, parse_donem_sayisi, donem_ozet, yillara_gir, DonemOzet,
    DonemRecord,
};
pub use efatura::efatura_matrahlar;
pub use takvim::{gun_kaldi, takvim_listesi, TakvimKayit};
pub use yedek::{temizle_yedek, yedek_olustur, TemizYedek, YedekAyarlari, YedekParca};
