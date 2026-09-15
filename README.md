<p align="center">
  <h1 align="center">Finansal Hesaplaci</h1>
  <p align="center">
    Electron ile yazilmis profesyonel Turk mali hesaplayici masaustu uygulamasi
  </p>
  <p align="center">
    <img src="https://img.shields.io/badge/Electron-36.9.5-47848F?logo=electron" alt="Electron">
    <img src="https://img.shields.io/badge/React-19-61DAFB?logo=react" alt="React">
    <img src="https://img.shields.io/badge/TypeScript-6-3178C6?logo=typescript" alt="TypeScript">
    <img src="https://img.shields.io/badge/Tailwind_CSS-4.3-06B6D4?logo=tailwindcss" alt="Tailwind">
    <img src="https://img.shields.io/badge/License-MIT-green" alt="License">
  </p>
</p>

---

### Gerekli
- Internet baglantisi yok — tamamen offline calisir
- Windows 10/11

### Ozellikler

| Mod | Aciklama |
|-----|----------|
| **KDV** | %1, %10, %20 oranlarinda KDV hesaplama |
| **Stopaj** | %1, %3, %5, %7, %10, %15, %20 oranlarinda stopaj |
| **Kar Marji** | Kar marjindan satis fiyati hesaplama |
| **Kar Orani** | Kar oranindan satis fiyati hesaplama |
| **Zincirli Indirim** | Ust uste indirim hesaplama |
| **Yuzde** | Yuzde orani ile islemler |
| **Bilesik Faiz** | Yillik bilesek faiz hesaplama |
| **KDV Karsilastirma** | Farkli KDV oranlarini karsilastirma |

### Kurulum

**Kolay kurulum** — `kur.bat` dosyasini calistirin, her sey otomatik yuklenir:

```
1. Repositoryyi indirin
2. kur.bat dosyasini calistirin
3. Masaustunde olusan kisayol ile acin
```

`kur.bat` su islemleri otomatik yapar:
- Node.js yukler (yoksa)
- Electron indirir (yoksa)
- Bagimliliklari yukler
- Build alir
- Masaustu kisayolu olusturur

### Gelendirme

```bash
# Bagimliliklari yukle
npm install

# Gelistirme modunda baslat
npm run dev

# Build al
npm run build
```

### Kullanim

| Tus | Islev |
|-----|-------|
| **Sayi tuslari** | Deger girme |
| **Tab** | Bir sonraki alana gec |
| **Enter** | Kaydet |
| **F1** | Yardim |
| **F2** | Ayarlar |

### Teknik Detaylar

- **decimal.js** — hassasiyetli matematik islemleri
- **Zustand** — durum yonetimi
- **Turk sayi formati** — `1.234.567,89`
- **Yerel fontlar** — internet bagimsiz calisir
- **Glassmorphism** — karanlik tema, animasyonlu gecisler
- **Tek instance** — ayni anda birden fazla pencere acilmaz
- **Gecmis** — islemler LocalStorage'da saklanir

### Lisans

MIT License
