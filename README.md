# Finansal Hesaplayıcı — Rust

![Finansal Hesaplayıcı logosu](build/logo.png)

Türk mali hesaplamaları için sıfırdan Rust ile yazılmış, çevrimdışı çalışan masaüstü uygulaması.
Electron sürümünün (v4.8.0) tüm özelliklerini karşılar: 15 hesaplama modu, dönem arşivi, cari kartlar,
vergi takvimi, e-fatura CSV içe aktarma, güvenli yedek al/yükle, tema ve yazı ölçeği, komut paleti,
güncelleyici (SHA-256 doğrulamalı) ve tek tık kurulum.

## Yapı

| Crate | İş |
|---|---|
| `fh-core` | 15 hesaplama modu, `rust_decimal` para matematiği, Türkçe biçim/para yazısı |
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
& "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe" "Kurulum\FH_kurulum.iss"
```

## Tasarım kuralları

- Para hesabı daima `Decimal`, `ROUND_HALF_UP`; gösterim `1.234.567,89`.
- Kullanıcı verisi `%APPDATA%` altında; yedek dosyaları sanitizasyondan geçer.
- Güncellemeler SHA-256 ile doğrulanır; allowlist dışı host reddedilir.
- Kod imzalama sertifikası olmadığı için SmartScreen uyarısı görünebilir.
