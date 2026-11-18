//! Dönem arşivi hesapları — Electron `src/data/donem.ts` portu.
//! Tüm tutarlar makine biçiminde String; ROUND_HALF_UP, 2 ondalık.

use std::collections::BTreeMap;
use std::str::FromStr;

use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;

#[derive(Debug, Clone, PartialEq)]
pub struct DonemRecord {
    pub id: String,
    pub year: i32,
    pub month: i32,
    pub satis: String,
    pub alis: String,
    /// Döneme girilen önceki devreden
    pub devreden_giris: String,
    pub hesaplanan: String,
    pub indirilecek: String,
    /// pozitif: ödenecek, negatif: devreden
    pub net: String,
    pub odenecek: bool,
    pub kdv_rate: String,
    pub cari: String,
    pub closed_at: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DonemOzet {
    pub hesaplanan: String,
    pub indirilecek: String,
    pub net: String,
    pub odenecek_net: String,
    pub devreden_net: String,
    pub sayi: usize,
}

/// Saklanmış dönem sayısını güvenle Decimal'e çevirir (ifade değerlendirmez).
pub fn parse_donem_sayisi(s: &str) -> Option<Decimal> {
    let t = s.trim();
    if t.is_empty() {
        return None;
    }
    // rust_decimal Decimal hiçbir zaman NaN/Inf tutmaz; ayrıştırma başarısı yeterli.
    Decimal::from_str(&t.replace(',', ".")).ok()
}

fn parse(v: &str) -> Decimal {
    parse_donem_sayisi(v).unwrap_or(Decimal::ZERO)
}

fn round2(d: Decimal) -> Decimal {
    d.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero)
}

/// Makine biçimi: gereksiz sıfırlar atılır ("1234.56", "-30", "0").
fn fmt(d: Decimal) -> String {
    let mut s = d.to_string();
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

/// Dönem KDV neti: hesaplanan − indirilecek − girilen devreden.
pub fn compute_donem_net(
    satis: &str,
    alis: &str,
    devreden: &str,
    oran: &str,
) -> Option<(String, String, String, bool)> {
    let s = parse_donem_sayisi(satis);
    let a = parse_donem_sayisi(alis);
    if s.is_none() && a.is_none() {
        return None;
    }
    let rate = parse_donem_sayisi(oran)?;
    let d = parse_donem_sayisi(devreden).unwrap_or(Decimal::ZERO);
    let yuz = Decimal::from(100);

    let hesaplanan = round2(s.unwrap_or(Decimal::ZERO) * rate / yuz);
    let indirilecek = round2(a.unwrap_or(Decimal::ZERO) * rate / yuz);
    let net = round2(hesaplanan - indirilecek - d);
    let odenecek = net >= Decimal::ZERO;

    Some((fmt(hesaplanan), fmt(indirilecek), fmt(net), odenecek))
}

/// Son kapanan dönemden gelen devreden KDV (mutlak değer); önceki dönem ödendiyse 0.
/// Kayıt yoksa null. Liste en yeni dönem önce sıralı varsayılır (list[0]).
pub fn kalan_devreden(list: &[DonemRecord]) -> Option<String> {
    let son = list.first()?;
    if son.odenecek {
        return Some("0".to_string());
    }
    let n = parse_donem_sayisi(&son.net)?;
    Some(fmt(n.abs()))
}

/// Kayıtları toplayıp özet çıkarır (bozuk alanlar 0 sayılır).
pub fn donem_ozet(list: &[DonemRecord]) -> DonemOzet {
    let mut hesaplanan = Decimal::ZERO;
    let mut indirilecek = Decimal::ZERO;
    let mut net = Decimal::ZERO;
    let mut odenecek_net = Decimal::ZERO;
    let mut devreden_net = Decimal::ZERO;

    for r in list {
        let h = parse(&r.hesaplanan);
        let i = parse(&r.indirilecek);
        let n = parse(&r.net);
        hesaplanan += h;
        indirilecek += i;
        net += n;
        if r.odenecek {
            odenecek_net += n;
        } else {
            devreden_net += n.abs();
        }
    }

    DonemOzet {
        hesaplanan: fmt(hesaplanan),
        indirilecek: fmt(indirilecek),
        net: fmt(net),
        odenecek_net: fmt(odenecek_net),
        devreden_net: fmt(devreden_net),
        sayi: list.len(),
    }
}

/// Kayıtları yılına göre gruplar; en yeni yıl önce, yıl içinde en yeni dönem önce.
pub fn yillara_gir(list: &[DonemRecord]) -> Vec<(i32, Vec<DonemRecord>, DonemOzet)> {
    let mut by_year: BTreeMap<i32, Vec<DonemRecord>> = BTreeMap::new();
    for r in list {
        by_year.entry(r.year).or_default().push(r.clone());
    }
    by_year
        .into_iter()
        .rev()
        .map(|(yil, mut kayitlar)| {
            kayitlar.sort_by(|a, b| b.id.cmp(&a.id));
            let ozet = donem_ozet(&kayitlar);
            (yil, kayitlar, ozet)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(id: &str, year: i32, month: i32, hes: &str, ind: &str, net: &str, odenecek: bool) -> DonemRecord {
        DonemRecord {
            id: id.to_string(),
            year,
            month,
            satis: "0".to_string(),
            alis: "0".to_string(),
            devreden_giris: "0".to_string(),
            hesaplanan: hes.to_string(),
            indirilecek: ind.to_string(),
            net: net.to_string(),
            odenecek,
            kdv_rate: "20".to_string(),
            cari: String::new(),
            closed_at: 0,
        }
    }

    #[test]
    fn parse_donem_sayisi_virgul_ve_nokta() {
        assert_eq!(parse_donem_sayisi("1234,56").unwrap().to_string(), "1234.56");
        assert_eq!(parse_donem_sayisi(" 1234.56 ").unwrap().to_string(), "1234.56");
        assert_eq!(parse_donem_sayisi("-30").unwrap().to_string(), "-30");
        // TS: virgül noktaya çevrilir; "1.234,56" → "1.234.56" → geçersiz.
        assert!(parse_donem_sayisi("1.234,56").is_none());
    }

    #[test]
    fn parse_donem_sayisi_bosuk_ve_bozuk() {
        assert!(parse_donem_sayisi("").is_none());
        assert!(parse_donem_sayisi("abc").is_none());
        assert!(parse_donem_sayisi("1+2").is_none());
    }

    #[test]
    fn compute_donem_net_temel() {
        let (h, i, n, od) = compute_donem_net("1000", "400", "0", "20").unwrap();
        assert_eq!(h, "200");
        assert_eq!(i, "80");
        assert_eq!(n, "120");
        assert!(od);
    }

    #[test]
    fn compute_donem_net_devreden_dusulur() {
        let (_, _, n, od) = compute_donem_net("1000", "400", "150", "20").unwrap();
        assert_eq!(n, "-30");
        assert!(!od);
    }

    #[test]
    fn compute_donem_net_ikinonda_none() {
        assert!(compute_donem_net("", "", "0", "20").is_none());
        assert!(compute_donem_net("x", "y", "0", "20").is_none());
    }

    #[test]
    fn compute_donem_net_yanlis_oran_none() {
        assert!(compute_donem_net("100", "50", "0", "x").is_none());
    }

    #[test]
    fn compute_donem_net_half_up() {
        let (h, _, _, _) = compute_donem_net("1.025", "0", "0", "100").unwrap();
        assert_eq!(h, "1.03");
        let (_, i, _, _) = compute_donem_net("0", "1.005", "0", "100").unwrap();
        assert_eq!(i, "1.01");
    }

    #[test]
    fn kalan_devreden_ilk_kayittan() {
        let list = vec![
            rec("2026-03", 2026, 3, "0", "0", "-45.50", false),
            rec("2026-02", 2026, 2, "0", "0", "-10", false),
        ];
        assert_eq!(kalan_devreden(&list).unwrap(), "45.5");
    }

    #[test]
    fn kalan_devreden_odendiyse_sifir() {
        let list = vec![rec("2026-03", 2026, 3, "0", "0", "12", true)];
        assert_eq!(kalan_devreden(&list).unwrap(), "0");
    }

    #[test]
    fn kalan_devreden_kayit_yoksa_none() {
        assert!(kalan_devreden(&[]).is_none());
    }

    #[test]
    fn donem_ozet_toplamlari() {
        let list = vec![
            rec("2026-03", 2026, 3, "200", "80", "120", true),
            rec("2026-02", 2026, 2, "50", "90", "-50", false),
        ];
        let o = donem_ozet(&list);
        assert_eq!(o.hesaplanan, "250");
        assert_eq!(o.indirilecek, "170");
        assert_eq!(o.net, "70");
        assert_eq!(o.odenecek_net, "120");
        assert_eq!(o.devreden_net, "50");
        assert_eq!(o.sayi, 2);
    }

    #[test]
    fn donem_ozet_bos_liste() {
        let o = donem_ozet(&[]);
        assert_eq!(o.net, "0");
        assert_eq!(o.sayi, 0);
    }

    #[test]
    fn donem_ozet_bozuk_tutarlar_sifir_sayilir() {
        let list = vec![rec("2026-01", 2026, 1, "x", "y", "q", true)];
        let o = donem_ozet(&list);
        assert_eq!(o.hesaplanan, "0");
        assert_eq!(o.net, "0");
    }

    #[test]
    fn yillara_gir_yil_ve_id_sirasi() {
        let list = vec![
            rec("2025-11", 2025, 11, "1", "1", "1", true),
            rec("2026-02", 2026, 2, "1", "1", "1", true),
            rec("2026-01", 2026, 1, "1", "1", "1", true),
        ];
        let g = yillara_gir(&list);
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].0, 2026);
        assert_eq!(g[0].1[0].id, "2026-02");
        assert_eq!(g[1].0, 2025);
        assert_eq!(g[0].2.sayi, 2);
    }

    #[test]
    fn yillara_gir_bos() {
        assert!(yillara_gir(&[]).is_empty());
    }
}
