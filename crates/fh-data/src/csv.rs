//! Paylaşılan CSV üretimi — enjeksiyon korumalı hücreler, BOM.
//! Electron `src/lib/csv.ts` portu.

/// Excel/Sheets'te formül olarak çalışabilecek değerleri metne çevirir, tırnaklar kaçırılır.
pub fn csv_hucre(v: &str) -> String {
    let mut s = v.to_string();
    if matches!(
        s.chars().next(),
        Some('=' | '+' | '-' | '@' | '\t' | '\r')
    ) {
        s.insert(0, '\'');
    }
    format!("\"{}\"", s.replace('"', "\"\""))
}

pub fn csv_satir(cells: &[String]) -> String {
    cells.iter().map(|c| csv_hucre(c)).collect::<Vec<_>>().join(",")
}

/// Satırları CSV metnine çevirir; BOM Excel'de Türkçe karakterler için şart.
pub fn csv_metin(rows: &[Vec<String>]) -> String {
    let mut out = String::from("\u{FEFF}");
    for (i, row) in rows.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        out.push_str(&csv_satir(row));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_hucre_tirnaklanir() {
        assert_eq!(csv_hucre("merhaba"), "\"merhaba\"");
        assert_eq!(csv_hucre(""), "\"\"");
    }

    #[test]
    fn formul_enjeksiyonu_apostroflanir() {
        for p in ["=1+1", "+SUM(A1)", "-30", "@cmd", "\tX", "\rX"] {
            assert!(csv_hucre(p).starts_with("\"'"), "{p}");
        }
        assert_eq!(csv_hucre("=1+1"), "\"'=1+1\"");
    }

    #[test]
    fn cift_tirnak_kacirilir() {
        assert_eq!(csv_hucre(r#"a"b"#), "\"a\"\"b\"");
        assert_eq!(csv_hucre("=A\"1"), "\"'=A\"\"1\"");
    }

    #[test]
    fn csv_metin_bom_ve_satirlar() {
        let rows = vec![
            vec!["ad".to_string(), "tutar".to_string()],
            vec!["Ayşe".to_string(), "1234.56".to_string()],
        ];
        let s = csv_metin(&rows);
        assert!(s.starts_with('\u{FEFF}'));
        assert_eq!(
            s,
            "\u{FEFF}\"ad\",\"tutar\"\n\"Ayşe\",\"1234.56\""
        );
    }

    #[test]
    fn csv_metin_bos() {
        assert_eq!(csv_metin(&[]), "\u{FEFF}");
    }
}
