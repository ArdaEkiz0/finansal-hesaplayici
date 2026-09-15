<p align="center">
  <img src="public/logo.svg" alt="Finansal Hesaplaci" width="120">
</p>

<h1 align="center">Finansal Hesaplaci</h1>

<p align="center">
  Turk mali hesaplamalari icin masaustu uygulamasi<br>
  Electron + React + TypeScript ile yazildi.<br>
  <small>Developer: Arda M. Ekiz</small>
</p>

## Ne Yapar?

KDV, stopaj, kar marji, bilesek faiz gibi gunluk mali hesaplamalari kolaylastirir.
Ozellikle Turk vergi sistemindeki oranlari (KDV %1/%10/%20, stopaj %1-%20) destekler.
Tum hesaplamalar offline calisir, internet gerekmez.

## Modlar

- **KDV** — KDV ekleme / dahil etme
- **Stopaj** — Vergi kesintisi hesaplama
- **Kar Marji** — Marjdan satis fiyati bulma
- **Kar Orani** — Kardan satis fiyati bulma
- **Zincirli Indirim** — Ust uste yuzde indirim
- **Yuzde** — Yuzde bazli islemler
- **Bilesik Faiz** — Yillik faiz hesaplama
- **KDV Karsilastirma** — Dahil mi / haric mi karsilastir

## Kurulum

`kur.bat` dosyasini calistirin:

- Node.js yoksa otomatik yukler
- Electron'u indirir
- npm install + build yapar
- Masaustune kisayol birakir

Tek yapman gereken bu. Nodes yoksa bile otomatik kuruyor.

## Gelistirme

```bash
npm install
npm run dev
```

Vite ile hot-reload calisir.

## Yapi

```
src/
  App.tsx              # Ana layout
  main.tsx             # Entry point
  index.css            # Tailwind tema
  core/
    engine.ts          # Matematik fonksiyonlari
  store/
    useStore.ts        # Zustand state
  hooks/
    useKeyboard.ts     # Klavye kisayollari
  lib/
    persist.ts         # LocalStorage kayit
  components/
    Display.tsx        # Giris alani + sonuc
    Keypad.tsx         # Sayi tuslari
    ModeSelector.tsx   # Mod secici
    History.tsx        # Gecmis sidebar
    Settings.tsx       # Ayarlar paneli
    Help.tsx           # Klavye kisayollari
    TitleBar.tsx       # Electron baslik cubugu
    Notification.tsx   # Toast bildirim
    ErrorBoundary.tsx  # Hata yakalama

electron/
  main.ts             # Electron main process
  preload.ts          # IPC guvenli kopru
```

## Teknik

- **decimal.js** — virgullu sayilarda hassasiyet kaybi olmamasi icin
- **Zustand** — state yonetimi (context kullanimi gerekmedi)
- **Turk sayi formati** — `1.234.567,89` (nokta ayirici, virgul ondalik)
- **Yerel fontlar** — Inter ve JetBrains Mono dosyayla birlikte geliyor
- **Glassmorphism** — koyu tema, cam efektli kartlar
- **Tek instance** — ayni anda iki pencere acilmaz
- **Gecmis** — LocalStorage'da saklanir, silinmez

## Lisans

MIT
