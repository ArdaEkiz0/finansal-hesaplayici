use crate::format::{dp2, machine};
use rust_decimal::Decimal;

const ONES: [&str; 10] = [
    "", "bir", "iki", "üç", "dört", "beş", "altı", "yedi", "sekiz", "dokuz",
];
const TENS: [&str; 10] = [
    "", "on", "yirmi", "otuz", "kırk", "elli", "altmış", "yetmiş", "seksen", "doksan",
];
const SCALES: [&str; 5] = ["", "bin", "milyon", "milyar", "trilyon"];

fn three_digits(n: u32) -> String {
    let h = n / 100;
    let t = (n % 100) / 10;
    let o = n % 10;
    let mut s = String::new();
    if h > 0 {
        if h == 1 {
            s.push_str("yüz");
        } else {
            s.push_str(ONES[h as usize]);
            s.push_str("yüz");
        }
    }
    if t > 0 {
        if !s.is_empty() {
            s.push(' ');
        }
        s.push_str(TENS[t as usize]);
    }
    if o > 0 {
        if !s.is_empty() {
            s.push(' ');
        }
        s.push_str(ONES[o as usize]);
    }
    s
}

fn collapse_spaces(s: String) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `1234.56` → `"bin ikiyüz otuz dört lira elli dört kuruş"`.
pub fn para_yazisi(v: Decimal) -> String {
    let value = dp2(v);
    let negative = value.is_sign_negative() && !value.is_zero();
    let text = machine(value.abs());
    let (int_str, dec_str) = match text.split_once('.') {
        Some((a, b)) => (a, b),
        None => (text.as_str(), "0"),
    };
    let int = match int_str.parse::<u64>() {
        Ok(n) => n,
        Err(_) => return format!("{} lira", machine(value)),
    };
    let kurus = dec_str
        .chars()
        .take(2)
        .collect::<String>()
        .parse::<u32>()
        .unwrap_or(0);

    if int == 0 && kurus == 0 {
        return "sıfır lira".to_string();
    }

    let mut words = String::new();
    if int == 0 {
        // yalnızca kuruş
    } else {
        let mut remaining = int;
        let mut gi = 0usize;
        let mut parts: Vec<String> = Vec::new();
        while remaining > 0 && gi < SCALES.len() {
            let tri = (remaining % 1000) as u32;
            if tri > 0 {
                let mut w = three_digits(tri);
                if gi == 1 && tri == 1 {
                    w.clear(); // "bir bin" değil "bin"
                }
                let piece = if w.is_empty() {
                    SCALES[gi].to_string()
                } else {
                    format!("{w} {}", SCALES[gi])
                };
                parts.insert(0, piece);
            }
            remaining /= 1000;
            gi += 1;
        }
        words.push_str(&parts.join(" "));
    }

    let mut out = String::new();
    if negative {
        out.push_str("eksi ");
    }
    out.push_str(&words);
    if !out.ends_with(' ') && !out.is_empty() {
        out.push(' ');
    }
    out.push_str("lira");
    if kurus > 0 {
        out.push(' ');
        out.push_str(&three_digits(kurus));
        out.push_str(" kuruş");
    }
    collapse_spaces(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn words_examples() {
        assert_eq!(para_yazisi(Decimal::ZERO), "sıfır lira");
        assert_eq!(para_yazisi(Decimal::from_str("1.00").unwrap()), "bir lira");
        assert_eq!(
            para_yazisi(Decimal::from_str("1234.56").unwrap()),
            "bin ikiyüz otuz dört lira elli altı kuruş"
        );
    }

    #[test]
    fn words_negative() {
        assert_eq!(para_yazisi(Decimal::from(-30)), "eksi otuz lira");
    }
}
