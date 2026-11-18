---
version: 1
slug: "crates-fh-ui-src-app-rs"
primary_target: "crates/fh-ui/src/app.rs"
related_targets: []
---

## Scope — `crates/fh-ui` (Windows masaüstü, eframe/egui 0.37); Operate

## Direction contract

THESIS: Hesaplama, bölümlere ayrılmış bir "bilet" ve canlı bir "kapı paneli" olarak okunur; kullanıcı tutarı yazdıkça bilet güncellenir, geçmiş/dönem kayıtları değiştiğinde panelde yeniden sıralanır ve uyarı rengi bir kez yanıp kalır. Kategori varsayılanı olan "sol menü + eşit kartlar + mavi vurgu" düzenini reddeder; tanıdık native kontrolleri korur.

OWN-WORLD: Mat lacivert zemin (#0a0f1c), yüzey (#101827), incehairline ayırıcılar, tablo rakamlarında sabit genişlikli rakamlar, tek uyarı rengi amber (#f5a524), seçimde mavi (#2f7bff), doğrulama/utt durumunda yeşil (#10b981). Renk yalnızca durum, seçim ve eylem için; dekorasyon için değil.

STORY: Kullanıcı modu seçer, tutarı yazar, sonucu bilette büyük rakamla okur; alt panel kayıtları tarihe göre sıralar, yaklaşan vergi son günü amber rozetle işaretler.

FIRST VIEWPORT: 1366x768. Üstte ürün adı + mod şeridi; solda girdi alanı (etiket + geniş input), sağda sonuç bileti (büyük ana tutar, alt satırlar, kopyala/kaydet); altta tablo biçimli kapı paneli (geçmiş/dönem/takvim sekmeleri). Boş durum öğretici, hata durumu onarıcı metinli.

FORM: Seyahat belgesi + canlı kapı paneli (boarding-pass / gate-board); seed:a400ba98b7ca3361.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance.
