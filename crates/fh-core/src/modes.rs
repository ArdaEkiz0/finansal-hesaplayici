//! 15 modun formülleri. Tüm para matematiği `Decimal`, yuvarlama HALF_UP.
//!
//! Girdi eşlemesi (bkz. CONTRACT.md):
//! - kdv: birinci=tutar, oran=KDV %, ikinci/ek="dahil"/"extract" bayrağı
//! - stopaj: birinci=brüt, oran=stopaj %, ek[0]="netten"/"reverse"
//! - tevkifat: birinci=matrah, oran=KDV %, tevkifat_pay=pay/10, tevkifat_kod=kod
//! - fiyat: birinci=etiket, oran=KDV %, ek=indirim oranları
//! - kdvCompare: birinci=tutar, oran=KDV %
//! - margin/markup: birinci=maliyet, ikinci=hedef oran
//! - discount: birinci=orijinal, ek=indirim oranları
//! - percent: birinci=yüzde, ikinci=değer
//! - compound: birinci=anapara, ikinci=yıllık %, ek[0]=yıl, ek[1]=yılda bileşme
//! - gecikme: birinci=anapara, ikinci=aylık %, gun=gün
//! - amortisman: birinci=maliyet, oran/ikinci=oran (boşsa 100/yıl), yil=yıl, ek[1]=method
//! - doviz: birinci=tutar, ikinci=kur, ek[0]/tevkifat_kod=yön (toTL|fromTL)
//! - batch: ek=satırlar, tevkifat_kod=alt mod, oran=varsayılan oran
//! - donem: birinci=satış, ikinci=aliş, oran=KDV %, ek[0]=devreden

use crate::format::{dp2, dp4, format_tr, machine, parse_tr};
use crate::{Girdi, HesapSonucu};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{Decimal, MathematicalOps};

fn dec(n: i64) -> Decimal {
    Decimal::from(n)
}

fn amount(s: &str) -> Option<Decimal> {
    parse_tr(s)
}

fn rate(g: &Girdi) -> Option<Decimal> {
    parse_tr(&g.oran).or_else(|| parse_tr(&g.ikinci))
}

fn flag(s: &str) -> bool {
    matches!(
        s.trim().to_lowercase().as_str(),
        "1" | "true"
            | "evet"
            | "extract"
            | "dahil"
            | "netten"
            | "reverse"
            | "ters"
            | "cikar"
            | "çıkar"
            | "-1"
    )
}

fn any_flag(g: &Girdi) -> bool {
    flag(&g.ikinci) || flag(&g.tevkifat_kod) || g.ek.iter().any(|s| flag(s))
}

fn ek(g: &Girdi, index: usize) -> Option<Decimal> {
    g.ek.get(index).and_then(|s| parse_tr(s))
}

fn out(
    ana: Decimal,
    alt: Option<Decimal>,
    satirlar: Vec<(String, String)>,
    detay: String,
) -> HesapSonucu {
    HesapSonucu {
        ana: machine(ana),
        alt: alt.map(machine).unwrap_or_default(),
        satirlar,
        detay,
    }
}

fn row(label: impl Into<String>, value: Decimal) -> (String, String) {
    (label.into(), machine(dp2(value)))
}

fn row_raw(label: String, value: String) -> (String, String) {
    (label, value)
}

/// Zincirleme indirim: her adım bir önceki fiyattan hesaplanır.
fn chained(
    original: Decimal,
    discounts: &[Decimal],
) -> (Decimal, Decimal, Vec<(Decimal, Decimal)>) {
    let mut price = original;
    let mut steps = Vec::with_capacity(discounts.len());
    for &d in discounts {
        price -= price * d / dec(100);
        let price = dp2(price);
        steps.push((d, price));
    }
    let price = dp2(price);
    (price, dp2(original - price), steps)
}

fn power(base: Decimal, exponent: Decimal) -> Option<Decimal> {
    if base <= Decimal::ZERO {
        if exponent < Decimal::ZERO || exponent.fract() != Decimal::ZERO {
            return None;
        }
        return Some(if exponent.is_zero() {
            Decimal::ONE
        } else {
            Decimal::ZERO
        });
    }
    if let Some(n) = exponent.to_u32() {
        if exponent == dec(n as i64) {
            return Some(base.powd(dec(n as i64)));
        }
    }
    // Kesirli üs: ln/exp ile tam Decimal aritmetiği.
    Some((exponent * base.ln()).exp())
}

pub(super) fn run(mod_: &str, g: &Girdi) -> Option<HesapSonucu> {
    match mod_.trim().to_ascii_lowercase().as_str() {
        "kdv" => kdv(g),
        "stopaj" => stopaj(g),
        "tevkifat" => tevkifat(g),
        "fiyat" => fiyat(g),
        "kdvcompare" => kdv_compare(g),
        "margin" => margin(g),
        "markup" => markup(g),
        "discount" => discount(g),
        "percent" => percent(g),
        "compound" => compound(g),
        "gecikme" => gecikme(g),
        "amortisman" => amortisman(g),
        "doviz" => doviz(g),
        "batch" => batch(g),
        "donem" => donem(g),
        _ => None,
    }
}

fn kdv(g: &Girdi) -> Option<HesapSonucu> {
    let a = amount(&g.birinci)?;
    let r = rate(g)?;
    let extract = any_flag(g);
    let divisor = Decimal::ONE + r / dec(100);
    if divisor <= Decimal::ZERO {
        return None;
    }
    if extract {
        let net = dp2(a / divisor);
        let tax = dp2(a - net);
        Some(out(
            tax,
            Some(net),
            vec![
                row("% KDV", tax),
                row("Net Tutar", net),
                row("Brüt (girdiğiniz)", a),
            ],
            format!(
                "Dahilden Hariç — %{} KDV • KDV {} • Net {}",
                machine(r),
                format_tr(tax, 2),
                format_tr(net, 2)
            ),
        ))
    } else {
        let tax = dp2(a * r / dec(100));
        let total = dp2(a + tax);
        Some(out(
            tax,
            Some(total),
            vec![
                row("% KDV", tax),
                row("Net (girdiğiniz)", a),
                row("Toplam", total),
            ],
            format!(
                "Hariciden Dahil — %{} KDV • KDV {} • Toplam {}",
                machine(r),
                format_tr(tax, 2),
                format_tr(total, 2)
            ),
        ))
    }
}

fn stopaj(g: &Girdi) -> Option<HesapSonucu> {
    let gross = amount(&g.birinci)?;
    let r = rate(g)?;
    let reverse = flag(&g.tevkifat_kod)
        || g.ek
            .iter()
            .any(|s| matches!(s.trim().to_lowercase().as_str(), "netten" | "reverse" | "1"));
    if reverse {
        let divisor = Decimal::ONE - r / dec(100);
        if divisor <= Decimal::ZERO {
            return None;
        }
        let brut = gross / divisor;
        let cut = dp2(brut - gross);
        Some(out(
            cut,
            Some(dp2(brut)),
            vec![
                row("% Kesinti", cut),
                row("Brüt", brut),
                row("Net (girdiğiniz)", gross),
            ],
            format!(
                "Netten Brüt — %{} Stopaj • Kesinti {}",
                machine(r),
                format_tr(cut, 2)
            ),
        ))
    } else {
        let cut = dp2(gross * r / dec(100));
        let net = dp2(gross - cut);
        Some(out(
            cut,
            Some(net),
            vec![
                row("% Kesinti", cut),
                row("Net Ödenen", net),
                row("Brüt (girdiğiniz)", gross),
            ],
            format!(
                "Brütten Net — %{} Stopaj • Kesinti {} • Net {}",
                machine(r),
                format_tr(cut, 2),
                format_tr(net, 2)
            ),
        ))
    }
}

fn tevkifat(g: &Girdi) -> Option<HesapSonucu> {
    let matrah = amount(&g.birinci)?;
    let oran = rate(g)?;
    let pay = g.tevkifat_pay;
    if !(1..=10).contains(&pay) {
        return None;
    }
    let kdv = dp2(matrah * oran / dec(100));
    let tev = dp2(kdv * dec(pay as i64) / dec(10));
    let saticiya = dp2(matrah + kdv - tev);
    let mut rows = vec![
        row("Matrah", matrah),
        row(format!("%{} KDV", machine(oran)), kdv),
        row(format!("Tevkifat ({pay}/10)"), tev),
        row("Satıcıya Ödenen", saticiya),
        row("2 No'lu Beyan", tev),
    ];
    if !g.tevkifat_kod.trim().is_empty() {
        rows.push(row_raw("İşlem kodu".to_string(), g.tevkifat_kod.clone()));
    }
    let code = if g.tevkifat_kod.is_empty() {
        String::new()
    } else {
        format!(" • Kod {}", g.tevkifat_kod)
    };
    Some(out(
        tev,
        Some(saticiya),
        rows,
        format!(
            "Tevkifat {pay}/10 — %{} KDV{code} • Tevkifat {} • Satıcıya {}",
            machine(oran),
            format_tr(tev, 2),
            format_tr(saticiya, 2)
        ),
    ))
}

fn fiyat(g: &Girdi) -> Option<HesapSonucu> {
    let etiket = amount(&g.birinci)?;
    let oran = rate(g)?;
    let extract = flag(&g.ikinci) || flag(&g.tevkifat_kod);
    let discounts: Vec<Decimal> = g.ek.iter().filter_map(|s| parse_tr(s)).collect();
    let (matrah, saved, steps) = chained(etiket, &discounts);
    let divisor = Decimal::ONE + oran / dec(100);
    if divisor <= Decimal::ZERO {
        return None;
    }
    let (sonuc, tax, label) = if extract {
        let net = dp2(matrah / divisor);
        (net, dp2(matrah - net), "Ödenecek")
    } else {
        let tax = dp2(matrah * oran / dec(100));
        (dp2(matrah + tax), tax, "Toplam")
    };
    let mut rows = vec![
        row(label, sonuc),
        row("Etiket", etiket),
        row("Matrah", matrah),
    ];
    if !discounts.is_empty() {
        rows.push(row("Tasarruf", saved));
        for (i, (d, p)) in steps.iter().enumerate() {
            rows.push(row(&format!("Adım {} (%{})", i + 1, machine(*d)), *p));
        }
    }
    rows.push(row(format!("%{} KDV", machine(oran)), tax));
    Some(out(
        sonuc,
        Some(tax),
        rows,
        format!(
            "Fiyat — {} • %{} KDV • {}",
            if extract {
                "KDV dahil etiket"
            } else {
                "KDV hariç matrah"
            },
            machine(oran),
            format_tr(sonuc, 2)
        ),
    ))
}

fn kdv_compare(g: &Girdi) -> Option<HesapSonucu> {
    let a = amount(&g.birinci)?;
    let oran = rate(g)?;
    let divisor = Decimal::ONE + oran / dec(100);
    if divisor <= Decimal::ZERO {
        return None;
    }
    let extract_net = dp2(a / divisor);
    let extract_tax = dp2(a - extract_net);
    let inject_tax = dp2(a * oran / dec(100));
    let inject_total = dp2(a + inject_tax);
    let diff = dp2(inject_tax - extract_tax);
    Some(out(
        diff,
        Some(inject_total),
        vec![
            row("Hariciden Dahil Toplam", inject_total),
            row("Dahilden Hariç Net", extract_net),
            row("Dahildeki KDV", extract_tax),
            row("Haricideki KDV", inject_tax),
            row("KDV Farkı", diff),
        ],
        format!(
            "Karşılaştırma — %{} • Dahil KDV {} • Hariç KDV {} • Fark {}",
            machine(oran),
            format_tr(extract_tax, 2),
            format_tr(inject_tax, 2),
            format_tr(diff, 2)
        ),
    ))
}

fn margin(g: &Girdi) -> Option<HesapSonucu> {
    let cost = amount(&g.birinci)?;
    let target = parse_tr(&g.ikinci).or_else(|| parse_tr(&g.oran))?;
    let divisor = Decimal::ONE - target / dec(100);
    if target >= dec(100) || target < Decimal::ZERO || divisor <= Decimal::ZERO {
        return Some(out(
            Decimal::ZERO,
            Some(Decimal::ZERO),
            vec![
                row("Satış Fiyatı", Decimal::ZERO),
                row("Maliyet", cost),
                row("Kâr", Decimal::ZERO),
            ],
            "Geçersiz hedef marj".to_string(),
        ));
    }
    let sp = dp2(cost / divisor);
    let profit = dp2(sp - cost);
    let mp = if sp.is_zero() {
        Decimal::ZERO
    } else {
        dp4(profit / sp * dec(100))
    };
    Some(out(
        sp,
        Some(profit),
        vec![
            row("Satış Fiyatı", sp),
            row("Maliyet", cost),
            row("Kâr", profit),
            row_raw("Gerçek Marj".to_string(), machine(mp)),
        ],
        format!("Satış Fiyatı — Marj: {}%", machine(mp)),
    ))
}

fn markup(g: &Girdi) -> Option<HesapSonucu> {
    let cost = amount(&g.birinci)?;
    let target = parse_tr(&g.ikinci).or_else(|| parse_tr(&g.oran))?;
    if target < Decimal::ZERO {
        return Some(out(
            dp2(cost),
            Some(Decimal::ZERO),
            vec![
                row("Satış Fiyatı", cost),
                row("Maliyet", cost),
                row("Kâr", Decimal::ZERO),
            ],
            "Negatif kâr oranı".to_string(),
        ));
    }
    let profit = dp2(cost * target / dec(100));
    let sp = dp2(cost + profit);
    let mp = if sp.is_zero() {
        Decimal::ZERO
    } else {
        dp4(profit / sp * dec(100))
    };
    Some(out(
        sp,
        Some(profit),
        vec![
            row("Satış Fiyatı", sp),
            row("Maliyet", cost),
            row("Kâr", profit),
            row_raw("Marj Karşılığı".to_string(), machine(mp)),
        ],
        format!("Satış Fiyatı — Kâr Oranı: {}%", machine(target)),
    ))
}

fn discount(g: &Girdi) -> Option<HesapSonucu> {
    let original = amount(&g.birinci)?;
    let mut rates: Vec<Decimal> = g.ek.iter().filter_map(|s| parse_tr(s)).collect();
    if rates.is_empty() {
        rates.extend(parse_tr(&g.ikinci));
    }
    if rates.is_empty() {
        return None;
    }
    let (final_price, saved, steps) = chained(original, &rates);
    let effective = if original.is_zero() {
        Decimal::ZERO
    } else {
        dp4(saved / original * dec(100))
    };
    let mut rows = vec![
        row("Son Fiyat", final_price),
        row("Orijinal", original),
        row("Tasarruf", saved),
        row_raw("Etkin İndirim".to_string(), machine(effective)),
    ];
    for (i, (d, p)) in steps.iter().enumerate() {
        rows.push(row(&format!("Adım {} (%{})", i + 1, machine(*d)), *p));
    }
    Some(out(
        final_price,
        Some(saved),
        rows,
        format!(
            "İndirimler: {} • Son fiyat {}",
            rates
                .iter()
                .map(|d| format!("%{}", machine(*d)))
                .collect::<Vec<_>>()
                .join(" → "),
            format_tr(final_price, 2)
        ),
    ))
}

fn percent(g: &Girdi) -> Option<HesapSonucu> {
    let p = amount(&g.birinci)?;
    let value = parse_tr(&g.ikinci).or_else(|| parse_tr(&g.oran))?;
    let of = dp4(value * p / dec(100));
    // TS calculatePercentChange(oldValue=p, newValue=value): (value−p)/|p|*100, p==0 koruması.
    let change = if p.is_zero() {
        Decimal::ZERO
    } else {
        dp2((value - p) / p.abs() * dec(100))
    };
    let reverse = if p.is_zero() || p == dec(-100) {
        dp2(value)
    } else {
        dp2(value / (p / dec(100) + Decimal::ONE))
    };
    Some(out(
        of,
        Some(change),
        vec![
            row("Sonuç", of),
            row_raw("Değişim %".to_string(), machine(change)),
            row("Ters Hesap", reverse),
        ],
        format!(
            "%{} × {} = {}",
            machine(p),
            format_tr(value, 2),
            format_tr(of, 2)
        ),
    ))
}

fn compound(g: &Girdi) -> Option<HesapSonucu> {
    let principal = amount(&g.birinci)?;
    let annual = parse_tr(&g.ikinci).or_else(|| parse_tr(&g.oran))?;
    let years = ek(g, 0).or_else(|| {
        if g.yil > 0 {
            Some(dec(g.yil as i64))
        } else {
            None
        }
    })?;
    if years <= Decimal::ZERO {
        return None;
    }
    let freq = ek(g, 1)
        .or_else(|| {
            if g.ay > 0 {
                Some(dec(g.ay as i64))
            } else {
                None
            }
        })
        .unwrap_or(dec(12));
    if freq <= Decimal::ZERO {
        return None;
    }
    let r = annual / dec(100);
    let n = freq;
    let base = Decimal::ONE + r / n;
    let factor = power(base, n * years)?;
    let fv = dp2(principal * factor);
    let interest = dp2(fv - principal);
    let effective = if principal.is_zero() {
        Decimal::ZERO
    } else {
        dp4((fv / principal - Decimal::ONE) * dec(100))
    };
    Some(out(
        fv,
        Some(interest),
        vec![
            row("Vade Sonu", fv),
            row("Ana Para", principal),
            row("Faiz", interest),
            row_raw("Efektif Oran".to_string(), machine(effective)),
        ],
        format!(
            "Vade Sonu — {} yıl, yılda {} bileşik • Faiz {}",
            machine(years),
            machine(freq),
            format_tr(interest, 2)
        ),
    ))
}

fn gecikme(g: &Girdi) -> Option<HesapSonucu> {
    let principal = amount(&g.birinci)?;
    let rate = parse_tr(&g.ikinci).or_else(|| parse_tr(&g.oran))?;
    let days = ek(g, 0)
        .or_else(|| {
            if g.gun > 0 {
                Some(dec(g.gun as i64))
            } else {
                None
            }
        })
        .unwrap_or(dec(30));
    if days < Decimal::ZERO {
        return None;
    }
    let interest = dp2(principal * rate / dec(100) * days / dec(30));
    let total = dp2(principal + interest);
    Some(out(
        interest,
        Some(total),
        vec![
            row("Gecikme Faizi", interest),
            row("Anapara", principal),
            row("Toplam (anapara + faiz)", total),
        ],
        format!(
            "Gecikme — %{} aylık, {} gün • Faiz {} • Toplam {}",
            machine(rate),
            machine(days),
            format_tr(interest, 2),
            format_tr(total, 2)
        ),
    ))
}

fn amortisman(g: &Girdi) -> Option<HesapSonucu> {
    let cost = amount(&g.birinci)?;
    let years = ek(g, 0)
        .and_then(|d| d.to_i32())
        .or(if g.yil > 0 { Some(g.yil) } else { None })
        .unwrap_or(5)
        .clamp(1, 50);
    let years_dec = dec(years as i64);
    let oran = parse_tr(&g.oran)
        .or_else(|| parse_tr(&g.ikinci))
        .unwrap_or(dec(100) / years_dec);
    let declining =
        g.ek.get(1)
            .map(|s| s.trim().eq_ignore_ascii_case("azalan"))
            .unwrap_or(false)
            || g.tevkifat_kod.trim().eq_ignore_ascii_case("azalan");
    let r = oran / dec(100);
    let annual = cost * r;
    let mut book = cost;
    let mut rows = Vec::new();
    let mut total = Decimal::ZERO;
    for y in 1..=years {
        let dep = if declining {
            (book * r * dec(2)).min(book)
        } else {
            annual.min(book)
        };
        if dep <= Decimal::ZERO {
            break;
        }
        book -= dep;
        let dep = dp2(dep);
        total += dep;
        rows.push((
            format!(
                "Yıl {y} (kalan {})",
                format_tr(dp2(book.max(Decimal::ZERO)), 2)
            ),
            machine(dep),
        ));
        if book <= Decimal::ZERO {
            break;
        }
    }
    let yearly = rows
        .first()
        .map(|r| parse_tr(&r.1).unwrap_or(Decimal::ZERO))
        .unwrap_or(Decimal::ZERO);
    let total = dp2(total);
    let mut out_rows = vec![
        row("Yıllık ayrılan", yearly),
        row("Toplam ayrılan", total),
        row("Maliyet", cost),
    ];
    out_rows.extend(rows);
    Some(out(
        yearly,
        Some(total),
        out_rows,
        format!(
            "Amortisman — {}, %{} • Yıllık {} • Toplam {}",
            if declining {
                "Azalan bakiyeler"
            } else {
                "Normal"
            },
            machine(oran),
            format_tr(yearly, 2),
            format_tr(total, 2)
        ),
    ))
}

fn doviz(g: &Girdi) -> Option<HesapSonucu> {
    let a = amount(&g.birinci)?;
    let kur = parse_tr(&g.ikinci).or_else(|| parse_tr(&g.oran))?;
    if kur <= Decimal::ZERO {
        return None;
    }
    let direction = g.ek.first().map(String::as_str).unwrap_or(&g.tevkifat_kod);
    let to_tl = !direction.trim().eq_ignore_ascii_case("fromTL")
        && !direction.trim().eq_ignore_ascii_case("TLden")
        && !direction.trim().eq_ignore_ascii_case("fromtl");
    let result = dp2(if to_tl { a * kur } else { a / kur });
    let currency = if g.tevkifat_kod.trim().is_empty() {
        "DÖVİZ"
    } else {
        g.tevkifat_kod.trim()
    };
    Some(out(
        result,
        Some(kur),
        vec![
            row("Sonuç", result),
            row("Girdi", a),
            row_raw(format!("Kur (1 {currency})"), machine(kur)),
        ],
        format!(
            "{currency} → TL ({}) • Kur {} • Sonuç {}",
            if to_tl { "alış" } else { "satış" },
            machine(kur),
            format_tr(result, 2)
        ),
    ))
}

fn batch(g: &Girdi) -> Option<HesapSonucu> {
    let mode = if g.tevkifat_kod.trim().is_empty() {
        "kdv".to_string()
    } else {
        g.tevkifat_kod.trim().to_ascii_lowercase()
    };
    let default_rate = parse_tr(&g.oran);
    let lines: Vec<String> = if g.ek.is_empty() {
        g.birinci.lines().map(|s| s.to_string()).collect()
    } else {
        g.ek.clone()
    };
    let mut rows = Vec::new();
    let mut base_sum = Decimal::ZERO;
    let mut tax_sum = Decimal::ZERO;
    let mut main_sum = Decimal::ZERO;
    let mut valid = 0usize;
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (value_str, line_rate) = if let Some((a, b)) = line.split_once(';') {
            (a, parse_tr(b))
        } else if let Some((a, b)) = line.split_once('|') {
            (a, parse_tr(b))
        } else {
            (line, None)
        };
        let value = match parse_tr(value_str) {
            Some(v) => v,
            None => continue,
        };
        let r = match line_rate.or(default_rate) {
            Some(r) => r,
            None => continue,
        };
        valid += 1;
        base_sum += value;
        let (main, tax) = match mode.as_str() {
            "stopaj" => {
                let t = dp2(value * r / dec(100));
                (dp2(value - t), t)
            }
            "tevkifat" => {
                let kdv = dp2(value * r / dec(100));
                let t = dp2(kdv * dec(g.tevkifat_pay as i64) / dec(10));
                (dp2(value + kdv - t), t)
            }
            _ => {
                let t = dp2(value * r / dec(100));
                (dp2(value + t), t)
            }
        };
        tax_sum += tax;
        main_sum += main;
        rows.push((format!("{} → {}", line, format_tr(main, 2)), machine(main)));
    }
    if valid == 0 {
        return None;
    }
    rows.push(("Geçerli satır".to_string(), valid.to_string()));
    rows.push(row("Toplam matrah", dp2(base_sum)));
    rows.push(row("Toplam vergi", dp2(tax_sum)));
    rows.push(row("Toplam sonuç", dp2(main_sum)));
    Some(out(
        dp2(main_sum),
        Some(dp2(tax_sum)),
        rows,
        format!(
            "Toplu hesap ({mode}) — {valid} geçerli satır • Toplam {}",
            format_tr(dp2(main_sum), 2)
        ),
    ))
}

fn donem(g: &Girdi) -> Option<HesapSonucu> {
    let satis = amount(&g.birinci)?;
    let alis = amount(&g.ikinci).unwrap_or(Decimal::ZERO);
    let oran = parse_tr(&g.oran)?;
    let devreden = ek(g, 0)
        .or_else(|| parse_tr(&g.tevkifat_kod))
        .unwrap_or(Decimal::ZERO);
    let hesaplanan = dp2(satis * oran / dec(100));
    let indirilecek = dp2(alis * oran / dec(100));
    let net = dp2(hesaplanan - indirilecek - devreden);
    let odenecek = net >= Decimal::ZERO;
    let main = net.abs();
    Some(out(
        main,
        Some(devreden),
        vec![
            row(
                if odenecek {
                    "Ödenecek KDV"
                } else {
                    "Devreden KDV"
                },
                main,
            ),
            row("Satış matrahı", satis),
            row("Hesaplanan KDV", hesaplanan),
            row("Alış matrahı", alis),
            row("İndirilecek KDV", indirilecek),
            row("Önceki devreden", devreden),
        ],
        format!(
            "Dönem — %{} • {} • {}",
            machine(oran),
            if odenecek {
                "Ödenecek KDV"
            } else {
                "Devreden KDV"
            },
            format_tr(main, 2)
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn degisim(birinci: &str, ikinci: &str) -> String {
        let g = Girdi {
            birinci: birinci.to_string(),
            ikinci: ikinci.to_string(),
            ..Girdi::default()
        };
        run("percent", &g)
            .unwrap()
            .satirlar
            .into_iter()
            .find(|(k, _)| k == "Değişim %")
            .unwrap()
            .1
    }

    #[test]
    fn yuzde_degisim_formulu() {
        // TS calculatePercentChange(old=p=20, new=value=500) → (500-20)/|20|*100
        assert_eq!(degisim("20", "500"), "2400");
        assert_eq!(degisim("500", "20"), "-96");
        assert_eq!(degisim("-20", "-30"), "-50");
    }

    #[test]
    fn yuzde_degisim_p_sifir_korumasi() {
        assert_eq!(degisim("0", "500"), "0");
    }
}
