//! Denetim bulguları için saf karar testleri. GUI açılmaz; kaynak modülleri
//! `#[path]` ile dahil edilir.

#[path = "../src/commands.rs"]
mod commands;
#[path = "../src/kararlar.rs"]
mod kararlar;
#[path = "../src/model.rs"]
mod model;
#[path = "../src/modes.rs"]
mod modes;

use commands::{Eylem, komutlar};
use kararlar::{
    GuncellemeKarari, KisayolKararlari, WidgetEtki, ag_istegi_yapilir, guncelleme_karari,
    kisayol_kararlari, palet_secim_duzelt, widget_etki,
};
use model::{AppState, InputState, Settings, WidgetAyarlari};
use modes::{build_girdi, parse_hatalari, varsayilanlari_doldur};

/// app.rs::komut_calistir ile aynı etki uygulaması (saf karar üzerinden).
fn widget_uygula(w: &mut WidgetAyarlari, e: &Eylem) -> bool {
    match widget_etki(e) {
        Some(WidgetEtki::Aktiflik) => {
            w.aktif = !w.aktif;
            true
        }
        Some(WidgetEtki::Sadeleştir) => {
            w.kompakt = !w.kompakt;
            true
        }
        Some(WidgetEtki::Ustte) => {
            w.ustte = !w.ustte;
            true
        }
        None => false,
    }
}

#[test]
fn kisayol_dagitimi_cakismiyor() {
    assert_eq!(
        kisayol_kararlari(true, false, false),
        KisayolKararlari {
            palet_degistir: true,
            widget_degistir: false,
            pencereleri_kapat: false,
            yok: false,
        }
    );
    assert!(kisayol_kararlari(false, false, false).yok);
    let k = kisayol_kararlari(true, true, true);
    assert!(k.palet_degistir && k.widget_degistir && k.pencereleri_kapat);
}

#[test]
fn komut_calistirma_tek_alan_degistirir() {
    let eylemler = [
        Eylem::WidgetAcKapat,
        Eylem::WidgetSadeleştir,
        Eylem::WidgetÜsteSabitle,
    ];
    for e in &eylemler {
        let mut w = WidgetAyarlari::default();
        let onceki = (w.aktif, w.kompakt, w.ustte);
        assert!(widget_uygula(&mut w, e), "{e:?} widget komutu olmalı");
        let sonraki = (w.aktif, w.kompakt, w.ustte);
        let degisen = (onceki.0 != sonraki.0) as u8
            + (onceki.1 != sonraki.1) as u8
            + (onceki.2 != sonraki.2) as u8;
        assert_eq!(degisen, 1, "{e:?} birden fazla alanı değiştirdi");
    }
    let mut w = WidgetAyarlari::default();
    for e in &eylemler {
        widget_uygula(&mut w, e);
    }
    assert!(w.aktif && w.kompakt && !w.ustte);

    // Paletteki üç komut bu üç eyleme bağlı, başka eylem yok.
    let l = komutlar();
    for e in &eylemler {
        assert_eq!(l.iter().filter(|c| c.eylem == *e).count(), 1);
    }
    assert!(!widget_uygula(&mut w, &Eylem::Ayarlar));
}

#[test]
fn guncelleme_durum_makinesi_tek_sefer_bildirir() {
    assert_eq!(
        guncelleme_karari(true, false, None, false),
        GuncellemeKarari::YeniSurum
    );
    assert_eq!(
        guncelleme_karari(true, true, Some(false), false),
        GuncellemeKarari::IndirmeBasarisiz
    );
    assert_eq!(
        guncelleme_karari(true, true, Some(false), true),
        GuncellemeKarari::Yok
    );
    assert_eq!(
        guncelleme_karari(true, true, Some(true), false),
        GuncellemeKarari::IndirmeBasarili
    );
}

#[test]
fn palet_secimi_filtre_degisiminde_sifirlanir() {
    assert_eq!(palet_secim_duzelt(12, 40, false), 12);
    assert_eq!(palet_secim_duzelt(12, 40, true), 0);
    assert_eq!(
        palet_secim_duzelt(12, 2, false),
        1,
        "eski seçim clamp edilmeli"
    );
    assert_eq!(palet_secim_duzelt(7, 0, false), 0);
}

#[test]
fn ag_opt_out_yedekte_korunur() {
    let mut state = AppState {
        settings: Settings::default(),
        history: Vec::new(),
        donemler: Vec::new(),
        cariler: Vec::new(),
        active_cari: String::new(),
        inputs: Default::default(),
        widget: WidgetAyarlari::default(),
    };
    assert!(ag_istegi_yapilir(state.settings.guncelleme_kontrol));
    state.settings.guncelleme_kontrol = false;
    let geri = AppState::from_backup(&state.yedek_degeri()).unwrap();
    assert!(!ag_istegi_yapilir(geri.settings.guncelleme_kontrol));
}

#[test]
fn bozuk_tamsayi_alan_hata_bayragi_uretilir() {
    let mut i = InputState::default();
    varsayilanlari_doldur(&mut i, "tevkifat");
    i.birinci = "10000".into();
    i.oran = "20".into();
    i.tevkifat_pay = "bozuk".into();
    let hatalar = parse_hatalari(&i);
    assert!(!hatalar.is_empty(), "geçersiz tamsayı yakalanmalı");
    assert_eq!(hatalar[0].0, model::Alan::TevkifatPay);
    // build_girdi eski davranışı korur (0); UI hata bayrağıyla sonucu bastırır.
    assert_eq!(build_girdi("tevkifat", &i).tevkifat_pay, 0);
    i.tevkifat_pay = "5".into();
    assert!(parse_hatalari(&i).is_empty());
}
