# Product

<!-- impeccable:product-schema 1 -->

## Platform

desktop-windows

## Users

Birincil kullanıcı: Türkçe konuşan SMMM'ler ve mali müşavir ofisleri; saniyeler içinde KDV/stopaj/tevkifat hesabı yapan, aynı anda birden fazla mükellef dosyası açık çalışan kişiler. İkincil: kendi muhasebesini yapan küçük işletme sahipleri. (Kaynak: mevcut uygulama, 20 mod, paketleme kararı — çıkarım.)

## Product Purpose

Vergi ve finans hesaplarını çevrimdışı, ondalık hassasiyetle ve Türkçe biçimde yapmak. Başarı: kullanıcı girdiyi yazdığı anda doğru sonucu görmek, verisini kaybetmemek, hesabı yazdırıp/recovery alabilmek. (Çıkarım: `fh-core`/`fh-data`.)

## Positioning

Sunucuya ihtiyaç duymayan, `rust_decimal` ile para-hassas, yerel veri saklayan bir SMMM hesap aracı; Muhasebe Paketi'ne hediye modül olarak dahil, tek başına da kurulabilir. (Doğrulanmış: repo geçmişi, v5.0.0 release notları.)

## Operating Context

Windows 10/11, kullanıcı bazlı kurulum (`%LOCALAPPDATA%\Programs`), veri `%APPDATA%` altında, 1366×768 ve üzeri ekranlar. Klavye ağırlıklı çalışma; sık kullanılan eylemler komut paletinde. Güncelleme GitHub releases üzerinden SHA-256 doğrulamalı. (Kod ve release varlıklarından çıkarım.)

## Capabilities and Constraints

20 mod (kesin liste: `fh-ui/src/modes.rs::MODLAR`); geçmiş; dönem arşivi + yıllık özet; cari kartlar; vergi takvimi; e-fatura CSV içe aktarma; yedek al/yükle; CSV dışa aktarma; koyu/açık/sistem tema; yazı ölçeği; Ctrl+K paleti; daima üstte ayrı OS penceresi widget; SHA-256 doğrulamalı tek tık güncelleyici. Tüm para matematiği `fh-core`, tüm veri kuralları `fh-data` üzerinden; arayüzde formül yok. egui 0.37 — döndürülmüş galley API'si yok. (Kod: CONTRACT.md, modül listeleri.)

## Brand Commitments

Ürün adı: Finansal Hesaplayıcı. Arayüz tamamen Türkçe. Eski Electron ürününün görsel kimliği referans alınır: koyu lacivert zemin (#0a0f1c), yüzey (#101827), birincil mavi (#2f7bff), vurgu amber (#f5a524), başarı yeşili (#10b981), hata kırmızısı (#f4536e), cam kenarlıklar. Muhasebe Paketi'nin hediye modülü. (Eski `src/index.css` ve repo geçmişinden çıkarım.)

## Evidence on Hand

Çalışan Rust uygulaması (v5.7.x), 400+ test, CI/release (4 varlık: kurulum.exe, portable.exe, update.zip, update.zip.sha256), koyu/açık tema ekran görüntüleri `C:\Users\ozel\AppData\Local\Temp\opencode\` altında. Eski Electron arayüzü git geçmişinde.

## Product Principles

1. Para doğruluğu `Decimal` ve `ROUND_HALF_UP` ile garanti edilir.
2.Çevrimdışı çalışır; kullanıcı verisi cihazda kalır.
3.Türkçe biçim ve terminoloji her yerde tutarlıdır.
4.Kullanıcı verisi yedek/güncelleme sırasında kaybolmaz.
5.Klavye ve komut paleti hızlı çalışmayı destekler.

## Accessibility & Inclusion

Yazı ölçeği A−/A+ (0.85–1.25); kontrast ve okunabilirlik yeni arayüzün doğrulama kriteri. (Kod: ayarlar.)
