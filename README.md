<p align="center">
  <img src="public/logo.svg" alt="Finansal Hesaplayıcı" width="120">
</p>

<h1 align="center">Finansal Hesaplayıcı</h1>

<p align="center">
  Türk vergi ve mali hesapları için tamamen çevrimdışı masaüstü uygulaması<br>
  Electron + React + TypeScript • decimal.js hassasiyeti • Aurora Ledger arayüzü<br>
  <small>Developer: Arda M. Ekiz</small>
</p>

## İndir

En güncel sürüm: [Releases](https://github.com/ArdaEkiz0/finansal-hesaplayici/releases) sayfasından indir.

| Dosya | Ne zaman? |
|---|---|
| `Finansal-Hesaplayici-X.Y.Z-kurulum.exe` | Kalıcı kurulum: Başlat menüsü + masaüstü kısayolu, Denetim Masası'ndan kaldırılabilir |
| `Finansal-Hesaplayici-X.Y.Z-portable.exe` | Kurulum istemeyen tek dosya — USB'den çalıştır |

Uygulama içinde **Ayarlar → Güncelleme → Denetle** ile kendi kendini güncelleyebilir.

> Not: Sürüm imzasızdır; Windows SmartScreen ilk çalıştırmada "更多信息 → Çalıştır" diyebilir.

## Neler yapar?

- **16 hesap modu** — KDV (hariç/dahil, %1/%10/%20 + özel oran), Dönem KDV, Stopaj, Tevkifat, Gecikme faizi, Amortisman, Döviz (TCMB), Marj, Kâr oranı, İndirim, Yüzde, Bileşik, Fiyat, Karşılaştır, Toplu
- **Dönem arşivi + devreden zinciri** — ay kapanışları, geçmiş dönem devreden KDV otomatik taşınır
- **Cari kartlar** — dönem ve kayıtlar cari bazında izlenir
- **E-fatura CSV içe aktarma** — matrah/KDV sütunlarını tanıyıp toplar
- **Vergi takvimi** — KDV (28), Muhtasar & Prim (26), Damga (26), Ba-Bs ve geçici vergi son günleri, "kaç gün kaldı" rozetleriyle
- **Beyan özeti** — dönemden yazdırılabilir KDV özeti kartı (ön kontrol aracı, resmi belge değildir)
- **Yedek al / geri yükle** — tüm veriler tek JSON; bilgisayar değişimi için birebir
- **Yüzücü widget** — ayrı, boyutlandırılabilir mini hesap makinesi (her zaman üstte)
- **Komut paleti** — `Ctrl+K` ile her mod ve panele tek tıkla
- **Tema** — Koyu / Açık / Sistem + yazı ölçeği (A−/A+)

Tüm hesaplar **çevrimdışı** çalışır; yalnızca döviz kuru ve güncelleme denetimi internet ister.

## Geliştirme

```bash
npm install
npm run dev          # tarayıcı önizleme (localhost:5173)
npm test             # vitest — 42 test
npm run build        # tsc + vite
npm run dist         # portable + kurulum exe (release/)
```

Etiket push'la yayın: `git tag vX.Y.Z; git push origin vX.Y.Z` → CI test eder, paketleri, GitHub Release'e yükler.

## Proje yapısı

```
src/
  core/engine.ts       # decimal.js hesaplama motoru + TR sayı formatı
  data/                # oranlar (tax), dönem (donem), e-fatura, vergi takvimi (takvim)
  store/useStore.ts    # Zustand — tek durum deposu, localStorage'a yazar
  components/          # Aurora Ledger arayüzü (mod seçici, keypad, paneller, modallar)
  lib/persist.ts       # kalıcılık (sürüm anahtarı v2)
electron/
  main.ts              # pencere, widget, güncelleyici, TCMB kuru
  preload.ts           # güvenli IPC köprüsü
tests/                 # vitest — SMMM senaryoları, dönem, e-fatura, takvim
docs/index.html        # tanıtım sayfası
```

## Teknik notlar

- **decimal.js** — para hesaplarında kayan nokta hatası yok
- **TR sayı formatı** — `1.234.567,89`
- **Güncel oran denetimi** — yıl değişince oranlar için tazelik uyarısı
- **Tek örnek** — aynı anda iki ana pencere açılmaz
- **Kod imzası yok** — sertifika alınırsa SmartScreen uyarısı kalkar

## Lisans

MIT
