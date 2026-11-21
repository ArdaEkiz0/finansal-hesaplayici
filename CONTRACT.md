# Ortak API sözleşmesi (ajanlar bu imzalara kod yazar)

Tüm para değerleri `String` ve makine biçiminde (`"1234.56"`, negatif `"-30"`).
Kullanıcıya gösterim `fh_core::format_tr` ile yapılır (`1.234,56`).

## fh-core (crate: fh-core)
```rust
pub struct Girdi {
    pub birinci: String, pub ikinci: String, pub oran: String,
    pub tevkifat_pay: i32, pub tevkifat_kod: String,
    pub yil: i32, pub ay: i32, pub gun: i32,
    pub ek: Vec<String>,
}
pub struct HesapSonucu {
    pub ana: String,          // birincil sonuç, makine biçimi
    pub alt: String,          // ikincil sonuç (varsa), makine biçimi
    pub satirlar: Vec<(String, String)>, // (etiket, makine değeri)
    pub detay: String,        // kullanıcıya açıklama (Türkçe)
}
pub fn hesapla(mod_: &str, girdi: &Girdi) -> Option<HesapSonucu>;
pub fn format_tr(v: rust_decimal::Decimal, precision: usize) -> String; // 1.234.567,89
pub fn parse_tr(s: &str) -> Option<rust_decimal::Decimal>;  // "1.234,56" -> 1234.56
pub fn para_yazisi(v: rust_decimal::Decimal) -> String;    // "bin ikiyüz..." (Türkçe)
```
Desteklenen `mod_` değerleri (15): `kdv, stopaj, tevkifat, fiyat, kdvCompare,
margin, markup, discount, percent, compound, gecikme, amortisman, doviz, batch, donem`.
Formüllerin kaynağı: `finansal-hesaplayici/src/core/engine.ts` ve
`src/store/useStore.ts` (computeDonemNet, gecikme faizi, amortisman, döviz).
Yuvarlama `ROUND_HALF_UP`, hassasiyet 28–30. Bozuk girdi → `None` (panic yok).

## fh-data (crate: fh-data, fh-core'e bağlı)
```rust
pub struct TakvimKayit { pub id:String, pub ad:String, pub tip:String,
    pub tarih:String, pub donemi:String, pub aciklama:String }
pub fn takvim_listesi(bugun: chrono::NaiveDate) -> Vec<TakvimKayit>;
pub fn gun_kaldi(tarih: &str, bugun: chrono::NaiveDate) -> i32;

pub struct DonemRecord { pub id:String, pub year:i32, pub month:i32, pub satis:String,
    pub alis:String, pub devreden_giris:String, pub hesaplanan:String,
    pub indirilecek:String, pub net:String, pub odenecek:bool, pub kdv_rate:String,
    pub cari:String, pub closed_at:i64 }
pub struct DonemOzet { pub hesaplanan:String, pub indirilecek:String, pub net:String,
    pub odenecek_net:String, pub devreden_net:String, pub sayi:usize }
pub fn donem_ozet(list: &[DonemRecord]) -> DonemOzet;
pub fn yillara_gir(list: &[DonemRecord]) -> Vec<(i32, Vec<DonemRecord>, DonemOzet)>;
pub fn compute_donem_net(satis:&str, alis:&str, devreden:&str, oran:&str) -> Option<(String,String,String,bool)>;
pub fn kalan_devreden(list: &[DonemRecord]) -> Option<String>;
pub fn parse_donem_sayisi(s: &str) -> Option<rust_decimal::Decimal>;

pub struct TemizYedek { pub history:Vec<serde_json::Value>, pub donemler:Vec<serde_json::Value>,
    pub cariler:Vec<serde_json::Value>, pub active_cari:String,
    pub show_currency_symbol:bool, pub precision:usize, pub font_scale:f64, pub theme:String }
pub fn temizle_yedek(v: &serde_json::Value) -> Option<TemizYedek>; // limit 100/240/500
pub fn yedek_olustur(...) -> serde_json::Value;

pub fn csv_hucre(v: &str) -> String;   // enjeksiyon koruması + tirnak kaçırma
pub fn csv_metin(rows: &[Vec<String>]) -> String; // BOM + satirlar
pub fn efatura_matrahlar(icerik: &str) -> Vec<String>;
```

## fh-updater (crate: fh-updater)
```rust
pub struct Guncelleme { pub version:String, pub url:String, pub sha:String, pub sha_url:String, pub notes:String }
pub fn kontrol(surum: &str) -> Option<Guncelleme>;   // GitHub releases API, engelli ise None
pub fn sha_dogrula(data: &[u8], beklenen: &str) -> bool; // SHA-256, sabit zamanlı karşılaştırma
pub fn indir_ve_kur(g: &Guncelleme) -> bool;  // allowlist: github.com / objects.githubusercontent.com
```

## fh-ui (crate: fh-ui, binary adı `Finansal Hesaplayici`)
Yukarıdaki API'leri kullanır; doğrudanhesaplama formülü yazmaz.
eframe/egui, koyu-açık-sistem tema, yazı ölçeği, 20 mod, canlı sonuç, geçmiş,
dönem arşivi, cari, takvim, CSV/yedek, komut paleti (Ctrl+K), yazdırma, widget.
Veri yolu: `directories::ProjectDirs` → `%APPDATA%` altı.

## Dosya sınırları (çakışma yok)
- Ajan A: yalnız `crates/fh-core/**`
- Ajan B: yalnız `crates/fh-data/**`
- Ajan D: `crates/fh-updater/**`, `Kurulum/**`, `.github/workflows/**`, `scripts/**`
- Ajan C (ikinci dalga): yalnız `crates/fh-ui/**`
- Kök `Cargo.toml`, `CONTRACT.md`, `README.md`: üst ajan.
