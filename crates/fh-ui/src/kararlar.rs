//! Saf karar fonksiyonları: kısayol dağıtımı, komut etkileri, palet seçimi
//! ve güncelleme durumu makinesi. GUI'ye bağımlı değildir; birim testleri
//! ekran gerektirmeden çalışır.

use crate::commands::Eylem;

/// Bir karede algılanan kısayolların ürettiği kararlar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KisayolKararlari {
    /// Ctrl+K: komut paletini aç/kapat.
    pub palet_degistir: bool,
    /// Ctrl+Shift+W: mini widget'ı aç/kapat.
    pub widget_degistir: bool,
    /// Esc: açık pencereleri kapat.
    pub pencereleri_kapat: bool,
    /// Kısayol yoksa hiçbir şey yapılmaz.
    pub yok: bool,
}

/// Kısayol bayraklarını çakışmasız kararlara çevirir (saf).
/// Aynı karede birden fazla kısayol gelirse hepsi uygulanabilir; Esc
/// yalnızca diğerleri yokken "yok" kararı üretmez.
pub fn kisayol_kararlari(ctrl_k: bool, ctrl_shift_w: bool, esc: bool) -> KisayolKararlari {
    KisayolKararlari {
        palet_degistir: ctrl_k,
        widget_degistir: ctrl_shift_w,
        pencereleri_kapat: esc,
        yok: !ctrl_k && !ctrl_shift_w && !esc,
    }
}

/// Widget komutlarının kalıcı duruma etkisi. Üç komut üç ayrı alana bağlanır;
/// hepsi aynı `Eylem::Widget`'a bağlı olduğunda "sadeleştir" düğmesi yanlışlıkla
/// widget'ı kapatıyordu (denetim bulgusu).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetEtki {
    Aktiflik,
    Sadeleştir,
    Ustte,
}

/// Bir eylemin widget etkisini döner; widget komutu değilse `None`.
pub fn widget_etki(eylem: &Eylem) -> Option<WidgetEtki> {
    match eylem {
        Eylem::WidgetAcKapat => Some(WidgetEtki::Aktiflik),
        Eylem::WidgetSadeleştir => Some(WidgetEtki::Sadeleştir),
        Eylem::WidgetÜsteSabitle => Some(WidgetEtki::Ustte),
        _ => None,
    }
}

/// Güncelleme akışının tek seferlik bildirim kararları.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuncellemeKarari {
    Yok,
    YeniSurum,
    IndirmeBasarili,
    IndirmeBasarisiz,
}

/// Arka plan thread'lerinin paylaşımlı durumundan tek bir eylem çıkarır (saf).
/// Bildirimler yalnızca bir kez üretilir; `bildirildi` bayrakları tekrarı engeller.
pub fn guncelleme_karari(
    yeni_surum: bool,
    guncelleme_bildirildi: bool,
    indirme: Option<bool>,
    indirme_bildirildi: bool,
) -> GuncellemeKarari {
    if !guncelleme_bildirildi && yeni_surum {
        return GuncellemeKarari::YeniSurum;
    }
    if !indirme_bildirildi {
        if let Some(basari) = indirme {
            return if basari {
                GuncellemeKarari::IndirmeBasarili
            } else {
                GuncellemeKarari::IndirmeBasarisiz
            };
        }
    }
    GuncellemeKarari::Yok
}

/// Ağ isteğine izin var mı? Kullanıcı güncelleme kontrolünü kapattıysa
/// açılışta ve el ile yapılan kontrollerde hiçbir ağ isteği yapılmaz.
pub fn ag_istegi_yapilir(guncelleme_kontrol: bool) -> bool {
    guncelleme_kontrol
}

/// Komut paletinde filtre değişince seçimi sıfırlar, aynı filtrede aralığa
/// sınırlar. Eski `sec.min(n-1)` davranışı, liste küçüldüğünde yanlış komutu
/// çalıştırıyordu (denetim bulgusu).
pub fn palet_secim_duzelt(sec: usize, n: usize, sorgu_degisti: bool) -> usize {
    if sorgu_degisti || n == 0 {
        0
    } else {
        sec.min(n - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kisayol_yok_karari() {
        assert_eq!(
            kisayol_kararlari(false, false, false),
            KisayolKararlari {
                palet_degistir: false,
                widget_degistir: false,
                pencereleri_kapat: false,
                yok: true
            }
        );
    }

    #[test]
    fn kisayol_tek_baslar() {
        assert_eq!(kisayol_kararlari(true, false, false).palet_degistir, true);
        assert_eq!(kisayol_kararlari(false, true, false).widget_degistir, true);
        assert_eq!(
            kisayol_kararlari(false, false, true).pencereleri_kapat,
            true
        );
        assert!(!kisayol_kararlari(true, false, false).yok);
    }

    #[test]
    fn kisayol_birlesimi_ve_esc() {
        let k = kisayol_kararlari(true, true, true);
        assert!(k.palet_degistir && k.widget_degistir && k.pencereleri_kapat);
        assert!(!k.yok);
    }

    #[test]
    fn widget_komutlari_uc_ayri_etki() {
        assert_eq!(
            widget_etki(&Eylem::WidgetAcKapat),
            Some(WidgetEtki::Aktiflik)
        );
        assert_eq!(
            widget_etki(&Eylem::WidgetSadeleştir),
            Some(WidgetEtki::Sadeleştir)
        );
        assert_eq!(
            widget_etki(&Eylem::WidgetÜsteSabitle),
            Some(WidgetEtki::Ustte)
        );
        let etkiler = [
            widget_etki(&Eylem::WidgetAcKapat),
            widget_etki(&Eylem::WidgetSadeleştir),
            widget_etki(&Eylem::WidgetÜsteSabitle),
        ];
        for (i, a) in etkiler.iter().enumerate() {
            for b in etkiler.iter().skip(i + 1) {
                assert_ne!(a, b, "widget komutları aynı etkiye bağlanmamalı");
            }
        }
    }

    #[test]
    fn widget_olmayan_komut_etki_yok() {
        assert_eq!(widget_etki(&Eylem::Ayarlar), None);
        assert_eq!(widget_etki(&Eylem::FontBuyut), None);
        assert_eq!(widget_etki(&Eylem::Mod("kdv")), None);
    }

    #[test]
    fn guncelleme_yeni_surum_bir_kez_bildirilir() {
        assert_eq!(
            guncelleme_karari(true, false, None, false),
            GuncellemeKarari::YeniSurum
        );
        assert_eq!(
            guncelleme_karari(true, true, None, false),
            GuncellemeKarari::Yok
        );
        assert_eq!(
            guncelleme_karari(false, false, None, false),
            GuncellemeKarari::Yok
        );
    }

    #[test]
    fn guncelleme_indirme_sonucu_bir_kez_bildirilir() {
        assert_eq!(
            guncelleme_karari(false, true, Some(true), false),
            GuncellemeKarari::IndirmeBasarili
        );
        assert_eq!(
            guncelleme_karari(false, true, Some(false), false),
            GuncellemeKarari::IndirmeBasarisiz
        );
        assert_eq!(
            guncelleme_karari(false, true, Some(false), true),
            GuncellemeKarari::Yok
        );
        assert_eq!(
            guncelleme_karari(false, true, None, false),
            GuncellemeKarari::Yok
        );
    }

    #[test]
    fn guncelleme_yeni_surum_indirmeyi_maskeler() {
        // Aynı karede iki koşul da oluşursa yeni sürüm bildirimi önce gelir;
        // indirme kararı ertesi karede üretilir (tek bildirim/kare kuralı).
        assert_eq!(
            guncelleme_karari(true, false, Some(true), false),
            GuncellemeKarari::YeniSurum
        );
    }

    #[test]
    fn ag_istegi_opt_out() {
        assert!(ag_istegi_yapilir(true));
        assert!(!ag_istegi_yapilir(false));
    }

    #[test]
    fn palet_secim_sorgu_degisiminde_sifirlanir() {
        assert_eq!(palet_secim_duzelt(9, 20, true), 0);
        assert_eq!(palet_secim_duzelt(9, 20, false), 9);
    }

    #[test]
    fn palet_secim_liste_kuculince_clamp() {
        assert_eq!(palet_secim_duzelt(15, 3, false), 2);
        assert_eq!(palet_secim_duzelt(0, 0, false), 0);
        assert_eq!(palet_secim_duzelt(15, 0, false), 0);
    }
}
