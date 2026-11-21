use rust_decimal::{Decimal, RoundingStrategy};
use std::str::FromStr;

/// decimal.js `ROUND_HALF_UP` karşılığı: eşit mesafede sıfırdan uzağa.
pub(crate) const HALF_UP: RoundingStrategy = RoundingStrategy::MidpointAwayFromZero;

pub(crate) fn dp(v: Decimal, places: u32) -> Decimal {
    v.round_dp_with_strategy(places, HALF_UP)
}

pub(crate) fn dp2(v: Decimal) -> Decimal {
    dp(v, 2)
}

pub(crate) fn dp4(v: Decimal) -> Decimal {
    dp(v, 4)
}

/// Makine biçimi: trailing sıfırlar atılır, `-0` → `0`.
pub(crate) fn machine(v: Decimal) -> String {
    let s = v.to_string();
    if !s.contains('.') {
        return if s == "-0" { "0".to_string() } else { s };
    }
    let t = s.trim_end_matches('0').trim_end_matches('.');
    if t.is_empty() || t == "-0" || t == "-" {
        "0".to_string()
    } else {
        t.to_string()
    }
}

fn group(digits: &str) -> String {
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push('.');
        }
        out.push(ch);
    }
    out
}

/// `1234567.89, 2` → `"1.234.567,89"`. Negatifler `-` önekiyle gelir.
pub fn format_tr(v: Decimal, precision: usize) -> String {
    let p = precision.min(28) as u32;
    let rounded = dp(v, p);
    if rounded.is_zero() {
        // -0,00 üretmeyelim.
        return if p == 0 {
            "0".to_string()
        } else {
            format!("0,{:0width$}", 0, width = p as usize)
        };
    }
    let neg = rounded.is_sign_negative();
    let text = format!("{}", rounded.abs());
    let (int_part, dec_part) = match text.split_once('.') {
        Some((a, b)) => (a, b),
        None => (text.as_str(), ""),
    };
    let head = group(if int_part.is_empty() { "0" } else { int_part });
    let mut out = if neg { format!("-{head}") } else { head };
    if p > 0 {
        let mut frac = dec_part.to_string();
        while frac.len() < p as usize {
            frac.push('0');
        }
        frac.truncate(p as usize);
        out.push(',');
        out.push_str(&frac);
    }
    out
}

/// `^\d{1,3}(\.\d{3})+$` — virgül yoksa noktalar binlik ayraçtır (`10.000` → 10000).
/// `10,5` veya `10.5,50` gibi girdilerde virgül bulunduğu için bu yol kullanılmaz.
fn binlik_nokta_mi(s: &str) -> bool {
    let parcalar: Vec<&str> = s.split('.').collect();
    parcalar.len() >= 2
        && !parcalar[0].is_empty()
        && parcalar[0].len() <= 3
        && parcalar[0].bytes().all(|b| b.is_ascii_digit())
        // 4-hane precision sözleşmesi: "0.001" / "0.0001" gibi 1–4 ondalıklı
        // değerler binlik değil ONDALIKTIR. Binlik gösterimde tam kısım asla
        // "0" ile başlamaz; bu kontrol olmadan "0.001" → "0001" → 1 oluyordu.
        && !parcalar[0].starts_with('0')
        && parcalar[1..]
            .iter()
            .all(|p| p.len() == 3 && p.bytes().all(|b| b.is_ascii_digit()))
}

/// İşaretsiz tek sayıyı Türk biçiminden çözer (köprü: eski parse_tr gövdesi).
fn parse_sayi(body: &str) -> Option<Decimal> {
    if body.is_empty() {
        return None;
    }
    let dots = body.matches('.').count();
    let commas = body.matches(',').count();
    let normalized = if dots > 0 && commas > 0 {
        // Sağdaki ayırıcı ondalık; diğeri binlik.
        if body.rfind('.') > body.rfind(',') {
            body.replace(',', "")
        } else {
            body.replace('.', "").replace(',', ".")
        }
    } else if commas == 1 {
        body.replace(',', ".")
    } else if commas > 1 {
        return None;
    } else if binlik_nokta_mi(body) {
        // YENİ: virgül yoksa ve 10.000 / 1.234.567 biçimindeyse binlik.
        body.replace('.', "")
    } else if dots > 1 {
        body.replace('.', "") // 12.34.567 gibi eski hoşgörülü binlik
    } else {
        body.to_string()
    };

    let mut digit_count = 0usize;
    let mut seen_dot = false;
    for ch in normalized.chars() {
        if ch == '.' {
            if seen_dot {
                return None;
            }
            seen_dot = true;
        } else if ch.is_ascii_digit() {
            digit_count += 1;
        } else {
            return None;
        }
    }
    if digit_count == 0 {
        return None;
    }
    Decimal::from_str(&normalized).ok()
}

/// TS `evaluateExpression` portu: yalnızca dört işlem, parantez, birli ± ve `%`.
/// Keyfi kod (harf, `;`, `<` vb.) asla kabul edilmez; uzunluk sınırı vardır.
const MAX_IFADE_UZUNLUGU: usize = 256;

struct IfadeAyrıştırıcı<'a> {
    karakterler: &'a [char],
    konum: usize,
}

impl<'a> IfadeAyrıştırıcı<'a> {
    fn bak(&self) -> Option<char> {
        self.karakterler.get(self.konum).copied()
    }

    fn ilerle(&mut self) {
        self.konum += 1;
    }

    /// ifade := terim (('+' | '-') terim)*
    fn ifade(&mut self) -> Option<Decimal> {
        let mut deger = self.terim()?;
        loop {
            match self.bak() {
                Some('+') => {
                    self.ilerle();
                    deger += self.terim()?;
                }
                Some('-') => {
                    self.ilerle();
                    deger -= self.terim()?;
                }
                _ => return Some(deger),
            }
        }
    }

    /// terim := çarpan (('*' | '/') çarpan)*
    fn terim(&mut self) -> Option<Decimal> {
        let mut deger = self.carpan()?;
        loop {
            match self.bak() {
                Some('*') => {
                    self.ilerle();
                    deger *= self.carpan()?;
                }
                Some('/') => {
                    self.ilerle();
                    let bolen = self.carpan()?;
                    if bolen.is_zero() {
                        return None; // TS: NaN → null
                    }
                    deger /= bolen;
                }
                _ => return Some(deger),
            }
        }
    }

    /// çarpan := ('+' | '-') çarpan | '(' ifade ')' | sayı ['%']
    fn carpan(&mut self) -> Option<Decimal> {
        match self.bak() {
            Some('+') => {
                self.ilerle();
                self.carpan()
            }
            Some('-') => {
                self.ilerle();
                Some(-self.carpan()?)
            }
            Some('(') => {
                self.ilerle();
                let deger = self.ifade()?;
                if self.bak() != Some(')') {
                    return None;
                }
                self.ilerle();
                Some(deger)
            }
            Some(c) if c.is_ascii_digit() || c == '.' || c == ',' => {
                let bas = self.konum;
                while matches!(self.bak(), Some(c) if c.is_ascii_digit() || c == '.' || c == ',') {
                    self.ilerle();
                }
                let token: String = self.karakterler[bas..self.konum].iter().collect();
                let mut deger = parse_sayi(&token)?;
                if self.bak() == Some('%') {
                    self.ilerle();
                    deger /= Decimal::from(100);
                }
                Some(deger)
            }
            _ => None,
        }
    }
}

fn evaluate_expression(s: &str) -> Option<Decimal> {
    if s.is_empty() || s.len() > MAX_IFADE_UZUNLUGU {
        return None;
    }
    if !s.chars().all(|c| {
        c.is_ascii_digit() || matches!(c, '.' | ',' | '+' | '-' | '*' | '/' | '(' | ')' | '%')
    }) {
        return None;
    }
    let karakterler: Vec<char> = s.chars().collect();
    let mut p = IfadeAyrıştırıcı {
        karakterler: &karakterler,
        konum: 0,
    };
    let deger = p.ifade()?;
    // Decimal sonlu olduğundan (sıfıra bölme yukarıda reddedildi) TS'deki
    // isFinite kontrolüne gerek yok; yalnızca tüm karakterlerin tüketildiğini denetle.
    if p.konum != karakterler.len() {
        return None;
    }
    Some(deger)
}

/// `"1.234,56"` → `1234.56`; `"100+50*2"` → `200`; `"-30"` → `-30`.
/// Bozuk/boş girdide, geçersiz ifadede veya sıfıra bölmede `None`.
pub fn parse_tr(s: &str) -> Option<Decimal> {
    let compact: String = s
        .trim()
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\u{a0}' && *c != '\u{202f}')
        .collect();
    if compact.is_empty() {
        return None;
    }
    // Aritmetik işareç varsa güvenli ifade değerlendiricisine geç (TS parseAmount).
    if compact
        .chars()
        .any(|c| matches!(c, '+' | '-' | '*' | '/' | '(' | ')' | '%'))
    {
        return evaluate_expression(&compact);
    }
    parse_sayi(&compact)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_basic() {
        assert_eq!(format_tr(Decimal::from(1234567), 2), "1.234.567,00");
    }

    #[test]
    fn format_negative() {
        assert_eq!(format_tr(Decimal::from(-30), 2), "-30,00");
    }

    #[test]
    fn format_zero_precision() {
        assert_eq!(format_tr(Decimal::from(1234567), 0), "1.234.567");
    }

    #[test]
    fn parse_traditional() {
        assert_eq!(
            parse_tr("1.234,56"),
            Some(Decimal::from_str("1234.56").unwrap())
        );
    }

    #[test]
    fn parse_machine_and_bad() {
        assert_eq!(
            parse_tr("1234.56"),
            Some(Decimal::from_str("1234.56").unwrap())
        );
        assert_eq!(parse_tr(""), None);
        assert_eq!(parse_tr("abc"), None);
    }

    // --- P1: güvenli aritmetik ifadeler ---

    #[test]
    fn parse_ifade_temel() {
        assert_eq!(parse_tr("100+50*2"), Some(Decimal::from(200)));
        assert_eq!(parse_tr("(100+50)*2"), Some(Decimal::from(300)));
        assert_eq!(parse_tr("100-30-20"), Some(Decimal::from(50)));
        assert_eq!(parse_tr("10/4"), Some(Decimal::from_str("2.5").unwrap()));
    }

    #[test]
    fn parse_ifade_unary_ve_yuzde() {
        assert_eq!(parse_tr("-30"), Some(Decimal::from(-30)));
        assert_eq!(parse_tr("-(20+10)"), Some(Decimal::from(-30)));
        assert_eq!(parse_tr("50%"), Some(Decimal::from_str("0.5").unwrap()));
        assert_eq!(
            parse_tr("200+10%"),
            Some(Decimal::from_str("200.1").unwrap())
        );
    }

    #[test]
    fn parse_ifade_girdi_bicimleri() {
        // Virgül ondalık; ifade içinde de Türk biçimi korunur.
        assert_eq!(
            parse_tr("1.234,56+2"),
            Some(Decimal::from_str("1236.56").unwrap())
        );
        assert_eq!(parse_tr("10,5*2"), Some(Decimal::from(21)));
        assert_eq!(
            parse_tr("-1.234,56"),
            Some(Decimal::from_str("-1234.56").unwrap())
        );
    }

    #[test]
    fn parse_ifade_güvenli_red() {
        let uzun = "1+".repeat(200); // uzunluk sınırı (256) aşımı
        for s in [
            "1/0",
            "1+",
            "(1+2",
            "2(3)",
            "1;2",
            "alert(1)",
            "1<2",
            "1**2",
            uzun.as_str(),
        ] {
            assert_eq!(parse_tr(s), None, "ifade reddedilmeli: {s}");
        }
    }

    // --- P4: nokta binlik ayraç ---

    #[test]
    fn parse_nokta_binlik() {
        assert_eq!(parse_tr("10.000"), Some(Decimal::from(10000)));
        assert_eq!(parse_tr("1.234.567"), Some(Decimal::from(1234567)));
        assert_eq!(parse_tr("1.234"), Some(Decimal::from(1234)));
    }

    #[test]
    fn parse_binlik_olmayan_ondegerler() {
        assert_eq!(parse_tr("10,5"), Some(Decimal::from_str("10.5").unwrap()));
        assert_eq!(
            parse_tr("10.5,50"),
            Some(Decimal::from_str("105.50").unwrap())
        );
        assert_eq!(
            parse_tr("1234.56"),
            Some(Decimal::from_str("1234.56").unwrap())
        );
        assert_eq!(
            parse_tr("12345.678"),
            Some(Decimal::from_str("12345.678").unwrap())
        );
    }

    // --- P0 regresyon: 0 ile başlayan ondalıklar binlik sanılmasın ---

    #[test]
    fn parse_ondalik_basli_sifir() {
        assert_eq!(parse_tr("0.001"), Some(Decimal::from_str("0.001").unwrap()));
        assert_eq!(parse_tr("0.01"), Some(Decimal::from_str("0.01").unwrap()));
        assert_eq!(
            parse_tr("0.0001"),
            Some(Decimal::from_str("0.0001").unwrap())
        );
        assert_eq!(parse_tr("0.5"), Some(Decimal::from_str("0.5").unwrap()));
    }

    #[test]
    fn parse_binlik_ve_ondalik_ayrimi_korunur() {
        // Mevcut sözleşme bozulmasın: 1.234 binlik, 1234.56 ondalık.
        assert_eq!(parse_tr("1.234"), Some(Decimal::from(1234)));
        assert_eq!(parse_tr("10.000"), Some(Decimal::from(10000)));
        assert_eq!(parse_tr("1.234.567"), Some(Decimal::from(1234567)));
        assert_eq!(
            parse_tr("1234.56"),
            Some(Decimal::from_str("1234.56").unwrap())
        );
    }
}
