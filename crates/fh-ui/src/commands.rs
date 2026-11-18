//! Ctrl+K komut paleti. Komut listesi ve filtreleme saf tutulur (test edilebilir).

#[derive(Debug, Clone, PartialEq)]
pub enum Eylem {
    Mod(&'static str),
    Tema(&'static str),
    FontBuyut,
    FontKucult,
    YedekAl,
    YedekYukle,
    GecmisCsv,
    EfaturaIce,
    Takvim,
    Gecmis,
    GecmisTemizle,
    Ayarlar,
    Cariler,
    Donemler,
    Widget,
    Yazdir,
}

#[derive(Debug, Clone)]
pub struct Komut {
    pub ad: String,
    pub kategori: &'static str,
    pub eylem: Eylem,
}

fn k(id: &'static str, ad: &str) -> Komut {
    Komut { ad: format!("{ad} moduna geç"), kategori: "Modlar", eylem: Eylem::Mod(id) }
}

pub fn komutlar() -> Vec<Komut> {
    let v = vec![
        k("kdv", "KDV"), k("stopaj", "Stopaj"), k("tevkifat", "Tevkifat"),
        k("fiyat", "Fiyat etiketi"), k("kdvCompare", "KDV karşılaştırma"),
        k("discount", "İndirim"), k("margin", "Marj"), k("markup", "Kâr oranı"),
        k("percent", "Yüzde"), k("compound", "Bileşik faiz"), k("gecikme", "Gecikme faizi"),
        k("amortisman", "Amortisman"), k("doviz", "Döviz"), k("batch", "Toplu"),
        k("donem", "Dönem KDV"),
        Komut { ad: "Koyu temaya geç".into(), kategori: "Görünüm", eylem: Eylem::Tema("dark") },
        Komut { ad: "Açık temaya geç".into(), kategori: "Görünüm", eylem: Eylem::Tema("light") },
        Komut { ad: "Sistem temasına geç".into(), kategori: "Görünüm", eylem: Eylem::Tema("system") },
        Komut { ad: "Yazıyı büyüt (A+)".into(), kategori: "Görünüm", eylem: Eylem::FontBuyut },
        Komut { ad: "Yazıyı küçült (A−)".into(), kategori: "Görünüm", eylem: Eylem::FontKucult },
        Komut { ad: "Mini widget'ı aç/kapat".into(), kategori: "Görünüm", eylem: Eylem::Widget },
        Komut { ad: "Yedek al".into(), kategori: "Veri", eylem: Eylem::YedekAl },
        Komut { ad: "Yedek yükle".into(), kategori: "Veri", eylem: Eylem::YedekYukle },
        Komut { ad: "Geçmişi CSV dışa aktar".into(), kategori: "Veri", eylem: Eylem::GecmisCsv },
        Komut { ad: "e-Fatura CSV içe aktar".into(), kategori: "Veri", eylem: Eylem::EfaturaIce },
        Komut { ad: "Vergi takvimini aç".into(), kategori: "Paneller", eylem: Eylem::Takvim },
        Komut { ad: "Geçmişi aç".into(), kategori: "Paneller", eylem: Eylem::Gecmis },
        Komut { ad: "Geçmişi temizle".into(), kategori: "Veri", eylem: Eylem::GecmisTemizle },
        Komut { ad: "Dönem arşivini aç".into(), kategori: "Paneller", eylem: Eylem::Donemler },
        Komut { ad: "Cari kartları aç".into(), kategori: "Paneller", eylem: Eylem::Cariler },
        Komut { ad: "Ayarları aç".into(), kategori: "Paneller", eylem: Eylem::Ayarlar },
        Komut { ad: "Yazdır / Özet önizleme".into(), kategori: "Veri", eylem: Eylem::Yazdir },
    ];
    v
}

/// Türkçe karakterleri ASCII karşılığına indirger; aramayı Türkçe kullanıcı için
/// duyarsız kılar (ş→s, ı→i, ğ→g, ü→u, ö→o, ç→c).
pub fn turkce_kucuk(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'İ' | 'I' | 'ı' | 'i' => 'i',
            'Ş' | 'ş' => 's',
            'Ğ' | 'ğ' => 'g',
            'Ü' | 'ü' => 'u',
            'Ö' | 'ö' => 'o',
            'Ç' | 'ç' => 'c',
            _ => c.to_ascii_lowercase(),
        })
        .collect()
}

/// Sorguya uyan komutların listedeki indislerini döner (saf).
pub fn filtrele(liste: &[Komut], sorgu: &str) -> Vec<usize> {
    let q = turkce_kucuk(sorgu.trim());
    liste
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            if q.is_empty() {
                return true;
            }
            let ad = turkce_kucuk(&c.ad);
            let kat = turkce_kucuk(c.kategori);
            ad.contains(&q) || kat.contains(&q)
        })
        .map(|(i, _)| i)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turkce_karakterler_normalize() {
        assert_eq!(turkce_kucuk("ŞİİR ÇĞÜÖ"), "siir cguo");
        assert_eq!(turkce_kucuk("Işık"), "isik");
    }

    #[test]
    fn en_az_12_komut() {
        assert!(komutlar().len() >= 12);
    }

    #[test]
    fn filtre_bos_sorga_tumunu_doner() {
        let l = komutlar();
        assert_eq!(filtrele(&l, "").len(), l.len());
        assert_eq!(filtrele(&l, "   ").len(), l.len());
    }

    #[test]
    fn filtre_turkce_duyarsiz() {
        let l = komutlar();
        let r = filtrele(&l, "koyu tema");
        assert!(!r.is_empty());
        assert_eq!(l[r[0]].eylem, Eylem::Tema("dark"));
        assert!(!filtrele(&l, "tema").is_empty());
        assert!(filtrele(&l, "zxqw").is_empty());
    }

    #[test]
    fn filtre_kategori_ile() {
        let l = komutlar();
        assert!(filtrele(&l, "gorunum").len() >= 5);
    }

    #[test]
    fn yazdir_komutu_var() {
        let l = komutlar();
        assert!(l.iter().any(|c| c.eylem == Eylem::Yazdir));
        assert!(!filtrele(&l, "yazdir").is_empty());
    }
}
