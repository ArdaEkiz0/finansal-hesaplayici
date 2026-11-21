//! GİB e-fatura / e-arşiv portalı ve Excel CSV'lerinden matrah çıkarımı.
//! Electron `src/data/efatura.ts` portu; yalnızca yerelde okunur (offline).

fn ayirac_sec(text: &str) -> char {
    let satirlar: Vec<&str> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(5)
        .collect();
    let ilk = satirlar.join("\n");

    // Tek sütun tespiti oyalamadan ÖNCE: ; \t | hiç yoksa ve her satır en fazla
    // bir virgül içeriyorsa virgül ondalık ayıracıdır (ör. "1234,56").
    // Böylece `1234,56` iki hücreye bölünüp 0,56 kaybolmaz.
    let tek_sutun = !satirlar.is_empty()
        && satirlar.iter().all(|l| {
            !l.chars().any(|c| matches!(c, ';' | '\t' | '|')) && l.matches(',').count() <= 1
        });
    if tek_sutun {
        return '\0'; // hiçbir satırı bölmeyen ayraç
    }

    let mut iyi = ',';
    let mut sayi = 0usize;
    for a in [';', ',', '\t', '|'] {
        let n = ilk.chars().filter(|c| *c == a).count();
        if n > sayi {
            sayi = n;
            iyi = a;
        }
    }
    iyi
}

/// Tırnaklı alanları ve `""` kaçışını anlayan basit ayraç ayrıştırıcı.
fn hucreleri_bol(line: &str, ayrac: char) -> Vec<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        if ch == '"' {
            if quoted && chars.get(i + 1) == Some(&'"') {
                cur.push('"');
                i += 2;
                continue;
            }
            quoted = !quoted;
            i += 1;
        } else if ch == ayrac && !quoted {
            out.push(cur.trim().to_string());
            cur.clear();
            i += 1;
        } else {
            cur.push(ch);
            i += 1;
        }
    }
    out.push(cur.trim().to_string());
    out
}

fn matrah_ipucu(h: &str) -> bool {
    let low = h.to_lowercase();
    let compact: String = low.chars().filter(|c| !c.is_whitespace()).collect();
    low.contains("matrah")
        || low.contains("ara toplam")
        || compact.contains("aratoplam")
        || compact.contains("mal/hizmet")
        || compact.contains("malvehizmet")
        || compact.contains("malhizmet")
}

fn kdv_ipucu(h: &str) -> bool {
    h.to_lowercase().contains("kdv")
}

/// `/\.(\d{3})(?=[^\d]|$)/g` — binlik noktalarını kaldırır, ondalık noktasını korur.
fn binlik_noktalari_sil(t: &str) -> String {
    let b = t.as_bytes();
    let mut out = String::with_capacity(t.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'.'
            && i + 3 < b.len()
            && b[i + 1].is_ascii_digit()
            && b[i + 2].is_ascii_digit()
            && b[i + 3].is_ascii_digit()
            && (i + 4 == b.len() || !b[i + 4].is_ascii_digit())
        {
            i += 1;
        } else {
            out.push(b[i] as char);
            i += 1;
        }
    }
    out
}

fn sayi_gecerli(t: &str) -> bool {
    let b = t.as_bytes();
    let mut i = 0;
    if b.first() == Some(&b'-') {
        i = 1;
    }
    let start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i == start {
        return false;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let ds = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == ds {
            return false;
        }
    }
    i == b.len()
}

/// "1.234,56" | "1234.56" | "1,234.56" -> "1234.56"
fn sayi_temizle(raw: &str) -> Option<String> {
    let mut t: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == '-')
        .collect();
    t = t.trim().to_string();
    if t.is_empty() {
        return None;
    }
    if t.contains(',') && t.contains('.') {
        let no_dots: String = t.chars().filter(|c| *c != '.').collect();
        t = no_dots;
        if let Some(p) = t.find(',') {
            t.replace_range(p..p + 1, ".");
        }
    } else if t.contains(',') {
        t = t.replace(',', ".");
    }
    if !t.contains(',') {
        t = binlik_noktalari_sil(&t);
    }
    if sayi_gecerli(&t) { Some(t) } else { None }
}

fn sayi_degeri(s: &str) -> f64 {
    s.parse::<f64>().unwrap_or(0.0)
}

fn rakam_sayisi(s: &str) -> usize {
    s.chars().filter(|c| c.is_ascii_digit()).count()
}

/// Geçersiz satırları atar; toplu moda verilecek matrahları (nokta ondalık) döner.
pub fn efatura_matrahlar(icerik: &str) -> Vec<String> {
    let temiz = icerik.strip_prefix('\u{FEFF}').unwrap_or(icerik);
    let ayrac = ayirac_sec(temiz);
    let satirlar: Vec<&str> = temiz.lines().filter(|l| !l.trim().is_empty()).collect();
    if satirlar.is_empty() {
        return Vec::new();
    }

    // GİB dışa aktarımlarında üst başlıklar (şirket adı, "E-Fatura Listesi")
    // ilk satırı işgal eder. Başlığı ilk 5 satırda ara; bulunduğu satıra kadar
    // olan tüm satırları veri sanıp atlamadan atla.
    let mut matrah_idx = None;
    let mut ilk = 0usize;
    for (i, satir) in satirlar.iter().take(5).enumerate() {
        let hucreler = hucreleri_bol(satir, ayrac);
        if let Some(idx) = hucreler
            .iter()
            .position(|h| matrah_ipucu(h) && !kdv_ipucu(h))
        {
            matrah_idx = Some(idx);
            ilk = i + 1;
            break;
        }
    }
    let header_var = matrah_idx.is_some();
    let mut rows = Vec::new();

    for line in satirlar.iter().skip(ilk) {
        let hucreler = hucreleri_bol(line, ayrac);
        if hucreler.len() == 1 && !header_var {
            if let Some(n) = sayi_temizle(&hucreler[0]) {
                // Sıfır matrah satırları diğer yollarda da atlanır; burada da
                // aynı sözleşme uygulanır.
                if sayi_degeri(&n) != 0.0 {
                    rows.push(n);
                }
            }
            continue;
        }

        let matrah_raw = if let Some(idx) = matrah_idx {
            let aday = hucreler.get(idx).map(String::as_str).unwrap_or("");
            sayi_temizle(aday)
        } else {
            // Sıfır değerleri aday listesinde BIRAK: aksi halde "0" matrahlı
            // bir satırda KDV oranı (örn. 20) tek sayısal aday sanılıp matrah
            // yerine geçiyordu. Sıfırın atlanması aşağıdaki son kontrolde yapılır.
            let sayisal: Vec<String> = hucreler.iter().filter_map(|c| sayi_temizle(c)).collect();
            if sayisal.is_empty() {
                None
            } else if sayisal.len() == 1 {
                sayisal.first().cloned()
            } else {
                // Tüm adaylar sıfır veya KDV oranı gibi küçük değerler ise
                // sıfır adayı tercih et: satır son kontrolde atlanır, %20 ise
                // yanlışlıkla matrah olarak döndürülmez.
                let sifir = sayisal.iter().find(|n| sayi_degeri(n) == 0.0);
                let sadece_kucuk = sayisal.iter().all(|n| {
                    let v = sayi_degeri(n);
                    v == 0.0 || (v.abs() > 0.0 && v.abs() <= 100.0)
                });
                if sifir.is_some() && sadece_kucuk {
                    sifir.cloned()
                } else {
                    let mut best = sayisal[0].clone();
                    for n in sayisal.iter().skip(1) {
                        if rakam_sayisi(n) > rakam_sayisi(&best) {
                            best = n.clone();
                        }
                    }
                    Some(best)
                }
            }
        };

        if let Some(m) = matrah_raw {
            if sayi_degeri(&m) != 0.0 {
                rows.push(m);
            }
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noktali_virgul_baslikli() {
        let s =
            "Tarih;Mal/Hizmet;KDV;Matrah\n2026-01-01;100,00;20;100,00\n2026-02-01;250,50;20;250,50";
        assert_eq!(efatura_matrahlar(s), vec!["100.00", "250.50"]);
    }

    #[test]
    fn basliksiz_tek_sutun() {
        let s = "1234.56\n1234.78\nabc";
        assert_eq!(efatura_matrahlar(s), vec!["1234.56", "1234.78"]);
    }

    #[test]
    fn tab_ayrac_basliksiz_cok_sutun() {
        let s = "A\t1.234,56\t20\nB\t500\tKDV";
        assert_eq!(efatura_matrahlar(s), vec!["1234.56", "500"]);
    }

    #[test]
    fn pipe_ayrac() {
        let s = "No|Matrah|KDV\n1|1200.75|240.15";
        assert_eq!(efatura_matrahlar(s), vec!["1200.75"]);
    }

    #[test]
    fn gecersiz_satirlar_atlanir() {
        let s = "Matrah;KDV\nabc;20\n0;0\n-100,25;20";
        assert_eq!(efatura_matrahlar(s), vec!["-100.25"]);
    }

    #[test]
    fn bom_kaldirilir() {
        let s = "\u{FEFF}Matrah;KDV\n1.234,56;20";
        assert_eq!(efatura_matrahlar(s), vec!["1234.56"]);
    }

    #[test]
    fn bos_icerik() {
        assert!(efatura_matrahlar("").is_empty());
        assert!(efatura_matrahlar("\n \n").is_empty());
    }

    #[test]
    fn tirnakli_alanlar() {
        let s = "A;Matrah\n\"X;Y\";\"1.000,50\"";
        assert_eq!(efatura_matrahlar(s), vec!["1000.50"]);
    }

    #[test]
    fn kdv_matrah_basligi_dislanir() {
        let s = "KDV Matrahı;Matrah\n10;200";
        assert_eq!(efatura_matrahlar(s), vec!["200"]);
    }

    #[test]
    fn binlik_nokta_ondegeri() {
        assert_eq!(sayi_temizle("1.234.567,89"), Some("1234567.89".to_string()));
        assert_eq!(sayi_temizle("1.234"), Some("1234".to_string()));
    }

    // --- P5: tek sütunlu virgüllü girdide virgül ondalık sayılır ---

    #[test]
    fn basliksiz_tek_sutun_virgullu() {
        let s = "1234,56\n999,95\nabc";
        assert_eq!(efatura_matrahlar(s), vec!["1234.56", "999.95"]);
    }

    #[test]
    fn baslikli_tek_sutun_virgullu() {
        let s = "Matrah\n1.234,56\n2.500,00";
        assert_eq!(efatura_matrahlar(s), vec!["1234.56", "2500.00"]);
    }

    #[test]
    fn ust_baslikli_gib_dosyasinda_baslik_bulunur() {
        let s = "ŞİRKET ADI A.Ş.\nE-FATURA / E-ARŞİV LİSTESİ\n\
                 Tarih;Açıklama;KDV;Matrah\n\
                 2026-01-01;Hizmet;%20;100,00\n\
                 2026-01-02;Hizmet;%20;250,50";
        assert_eq!(efatura_matrahlar(s), vec!["100.00", "250.50"]);
    }

    #[test]
    fn basliksiz_sifir_matrah_kdv_orani_sanilmasin() {
        // Başlıksız satırda matrah 0, KDV %20: 20 matrah olarak döndürülmemeli.
        let s = "A\t0\t20\nB\t500\tKDV";
        assert_eq!(efatura_matrahlar(s), vec!["500"]);
    }
}
