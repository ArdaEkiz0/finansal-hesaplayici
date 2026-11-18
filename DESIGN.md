# DESIGN — Finansal Hesaplayıcı (v5.2.0)

Operate yüzeyi. Kullanıcı "bilet + kapı paneli" düzenini reddetti; yerine **premium finans konsolu** kuruldu. Ekran doldurulur, boş alan bırakılmaz; tipografi ve hiyerarşi öne çıkar.

## Yön

Koyu: derin lacivert (`#0a0f1c`) zemin, `#101827` yüzey, ince ızgara dokusu, mavi eylem + cyan seçim şeridi, amber uyarı, yeşil doğrulama. Açık: sıcak kâğıt (`#faf8f4`) zemin, koyu mürekkep (`#18202f`), aynı mavi/cyan vurgu — yalnız renk terslemesi değil, ayrı yüzey/gölge/kenar dili. İki tema da gerçek ekranda doğrulandı: `C:\Users\ozel\AppData\Local\Temp\opencode\v52-real-dark.png` ve `v52-real-light.png`.

## Tipografi

Ürün başlığı 26–28, mod başlığı 23, gövde/alan etiketi 12–14.5, alanlar 36px yüksekliğinde monospace, sonuçta ana tutar 48px monospace + cyan alt çizgi. Sayılar sabit genişlikli. A−/A+ yazı ölçeği 0.85–1.25 çalışır. egui 0.37'de döndürülmüş yazı yok.

## Yapı

- **Üst:** kimlik bloğu (ad + sürüm + "çevrimdışı finans konsolu") ve araç düğmeleri (⌘K, A+, A−, Widget, Ayarlar, Takvim, Cari, Dönemler, Geçmiş). Güncelleme bildirimi burada.
- **Sol:** 15 mod, dört grup halinde dikey navigasyon; aktif satırda cyan şerit + mavi zemin; kaydırma belirgin, yatay kırpma yok.
- **Orta:** geniş çalışma alanı; mod başlığı, durum satırı, etiketli 36px alanlar, yedek/CSV eylemleri, "çalışma notu" kartı (alan sayısı, biçim ipucu, kısayol).
- **Sağ:** sonuç alanı; 48px ana tutar, okunur döküm tablosu, doğrulama/birim şeridi, kopyala/yazdır/kaydet. Boş durum öğretici metinli.
- **Alt:** tam genişlikte kayıt paneli (Geçmiş / Dönem / Takvim), okunur tablo, bilgilendirici boş durum, CSV/Temizle eylemleri. Kalan yükseklik tamamen bu panele verilir; altta boşluk kalmaz.
- **Dar (<1040px):** üçüncü sütun kapanır, sonuç çalışma alanının altına iner.

## Durumlar

Tüm kontrollerde default / hover / focus / seçim / devre dışı ayrımı; 6px köşe, 1px hairline kenar, yumuşak gölge. Amber yalnız son-gün uyarısı, yeşil doğrulama, kırmızı hata. Hareket yalnızca durum geçişlerinde kısa geçiş.

## Finish review

248 test (15 modun tamamı, yedek/CSV/takvim/ayarlar geri uyumu, `compound` panic düzeltmesi) ve release derlemesi temiz. Koyu + açık ekran görüntüleri okundu: taşma/kırpma yok, paneller ekranı dolduruyor, temalar ayrı kimlik. Bilinen sınırlar: sistem teması koyuya düşer, widget uygulama içi panel, native yazdırma yerine metin önizleme.
