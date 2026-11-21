# Finansal Hesaplayıcı — Rust

![Finansal Hesaplayıcı logosu](build/logo.png)

Türk mali hesaplamaları için sıfırdan Rust ile yazılmış, çevrimdışı çalışan masaüstü uygulaması.
20 hesaplama modu, dönem arşivi, cari kartlar, vergi takvimi, e-fatura CSV içe aktarma, güvenli yedek
al/yükle, tema ve yazı ölçeği, komut paleti, daima üstteki widget, SHA-256 doğrulamalı tek tık
güncelleme ve tek tık kurulum içerir.

## Yapı

| Crate | İş |
|---|---|
| `fh-core` | 20 hesaplama modu, `rust_decimal` para matematiği, Türkçe biçim/para yazısı |
| `fh-data` | Vergi takvimi, dönem arşivi, yedek sanitizasyonu, CSV, e-fatura ayrıştırma |
| `fh-updater` | GitHub sürüm kontrolü, SHA-256 bütünlük, host allowlist |
| `fh-ui` | eframe/egui arayüz, tema, klavye, widget, yazdırma |

Ortak API sözleşmesi: `CONTRACT.md`.

## Geliştirme

```powershell
cargo test --workspace
cargo build --release
.\target\release\Finansal Hesaplayici.exe
```

Kurulum exe'si (Inno Setup 6):

```powershell
& "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe" "/DSurum=<Cargo.toml sürümü>" "Kurulum\FH_kurulum.iss"
```

## İndirme ve güncelleme

Her sürüm dört varlıkla yayınlanır: `kurulum.exe`, `portable.exe`, `update.zip`, `update.zip.sha256`.
İndirme: `https://github.com/ArdaEkiz0/finansal-hesaplayici/releases/latest`

Uygulama açılışta arka planda GitHub releases'e bakar. Yeni sürüm varsa üst panelde uyarı ve
**İndir ve kur** düğmesi çıkar; `update.zip` indirilip GitHub API `digest` değeriyle (SHA-256)
doğrulanır, içindeki exe çıkarılıp görünmez bir köprüyle çalışan exe'nin üzerine yazılır ve
uygulama yeniden başlatılır. Uygulama içi güncelleme **installer çalıştırmaz** — Inno'nun
`unins000.exe` ikilisi konsol penceresi açıyordu. Kurulum exe'si yalnızca site'den ilk kurulum içindir.
Güncelleme kontrolünü Ayarlar → Hakkında'dan kapatabilirsin; kapalıyken hiç ağ isteği yapılmaz.
Kayıt Defteri'ndeki "Kaldır / Güvenli Kaldır" kaydı bir güncelleme sürümü geriden izleyebilir (kabul).

## Tasarım kuralları

- Para hesabı daima `Decimal`, `ROUND_HALF_UP`; gösterim `1.234.567,89`.
- Kullanıcı verisi `%APPDATA%` altında; yedek dosyaları sanitizasyondan geçer.
- Güncellemeler SHA-256 ile doğrulanır; allowlist dışı host reddedilir.
- Kod imzalama sertifikası olmadığı için SmartScreen uyarısı görünebilir.
