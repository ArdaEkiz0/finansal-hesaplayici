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
    DonemOzet, DonemRecord, compute_donem_net, donem_ozet, kalan_devreden, parse_donem_sayisi,
    yillara_gir,
};
pub use efatura::efatura_matrahlar;
pub use takvim::{TakvimKayit, gun_kaldi, takvim_listesi};
pub use yedek::{TemizYedek, YedekAyarlari, YedekParca, temizle_yedek, yedek_olustur};
