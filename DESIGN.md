# DESIGN — Finansal Hesaplayıcı (v5.1.0)

Operate yüzeyi: SMMM'nin masaüstünde hızlı, güvenilir ve Türkçe hesap yapması. Görsel dünya, HCV kataloğundan gelen "bilet + kapı paneli" yönüdür (seed:a400ba98b7ca3361): koyu zemin, ince çizgiler, tablo rakamları, renk yalnızca duruma ayrılmıştır.

## Palet

| Rol | Koyu | Açık |
|---|---|---|
| Zemin | `#0a0f1c` | `#f6f8fc` |
| Yüzey | `#101827` | `#ffffff` |
| Birincil metin | `#eef2f8` | `#101729` |
| İkincil metin | `#c6cfdd` | `#3d4a61` |
| Eylem / seçim | `#2f7bff` | `#2563eb` |
| Uyarı / son gün | `#f5a524` | `#f0a62e` |
| Doğrulama | `#10b981` | `#10b981` |
| Hata | `#f4536e` | `#f4536e` |
| Ayırıcı | rgba(148,175,255,.10) | rgba(16,23,42,.10) |

## Tipografi

Tek aile, sabit ölçek: başlık 19, mod cipi 12, gövde 13, etiket 11, tablo 11, bilet ana tutarı 30 (sabit genişlikli rakamlar). Yazı ölçeği A−/A+ ile 0.85–1.25 aralığında değişir. Döndürülmüş yazı yok (egui 0.37 kısıtı).

## Kompozisyon

- **Üst kimlik şeridi:** ürün adı + sürüm + çevrimdışı rozeti; sıralı araç düğmeleri (⌘K, A+, A−, Widget, Ayarlar, Takvim, Cari, Dönemler, Geçmiş); altında yatay kaydırmalı, gruplu 15 mod şeridi.
- **Sol girdi paneli:** mod başlığı, "Hazır" durum noktası, etiketli geniş girişler, hesaplama yönü seçici, yedek al/yükle eylemleri.
- **Sonuç bileti:** "SONUÇ BİLETİ" etiketi, büyük ana tutar, perforasyon ayırıcı, hesap dökümü tablosu, not kutusu; kopyala / yazdır-önizle / geçmişe kaydet eylemleri.
- **Kapı paneli:** biletin altında kesin yükseklikte; GEÇMİŞ / DÖNEM / TAKVİM sekmeleri, tablo başlıkları (tarih, mod, tutar, detay), boş durum öğretici metni, yaklaşan son gün için amber rozet.

## Durumlar ve etkileşim

Tüm düğmelerin varsayılan/hover/devre dışı durumları tutarlı. Sonuç yokken eylemler devre dışı; boş geçmişte öğretici metin; hata durumunda onarıcı mesaj; güncelleme indirilirken yükleniyor çizgisi. Klavye: Ctrl+K komut paleti, enter ile hesaplama. Motion: yalnızca durum geçişlerinde kısa geçişler; sahne açılışı yok.

## Uyarlama

1366×768 ve üzeri: sol panel ~%34, sonuç bileti + kapı paneli kalan alan. Dar (<1040px) ekranda araç etiketleri kısalır, mod şeridi yatay kaydırılır; 600px altında bilet dökümü kaydırma gerektirir (taşma yok).

## Finish review

Gerçek pencere ekran görüntüsü `C:\Users\ozel\AppData\Local\Temp\opencode\gui-v51d.png` incelendi: üst araç çubuğu taşmıyor, kapı paneli ve sekmeleri görünür, bilet okunaklı, kontrast yeterli. 238 test (15 mod dahil) ve release derlemesi temiz. Bilinen sınırlar: sistem teması koyuya düşer, widget uygulama içi paneldir, native yazdırma yerine metin önizleme vardır. Karar: tasarım sözleşmesi karşılandı, yayına uygun.
