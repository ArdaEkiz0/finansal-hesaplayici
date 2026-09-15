# Finansal Hesaplaci

Electron ile yazilmis profesyonel Turk mali hesaplayici masaustu uygulamasi.

## Ozellikler

- **KDV Hesaplama** (1%, 10%, 20%)
- **Stopaj Hesaplama** (1/3/5/7/10/15/20%)
- **Kar Marji** ve **Kar Orani**
- **Zincirli Indirim**
- **Yuzde Hesaplama**
- **Bileşik Faiz**
- **KDV Karsilastirma**

### Teknik

- Electron + React + TypeScript
- Tailwind CSS v4 (glassmorphism dark tema)
- Zustand (durum yonetimi)
- decimal.js (hassasiyet)
- Turk sayi formati: `1.234.567,89`
- Offline calisir (sunucu gerekmez)

## Kurulum

```bash
# Bagimliliklari yukle
npm install

# Uygulamayi baslat
npm run dev

# Build al
npm run build
```

## Masaustu Kisayolu

`kur.bat` dosyasini calistirin — otomatik build alir ve masaustune kisayol olusturur.

## Kullanim

- **Sayi tuslari**: Deger girme
- **Tab**: Bir sonraki alana gec
- **Enter**: Kaydet
- **F1**: Yardim
- **F2**: Ayarlar
