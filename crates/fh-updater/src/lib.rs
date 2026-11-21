//! fh-updater — GitHub releases üzerinden güncelleme kontrolü,
//! SHA-256 bütünlük doğrulaması ve ana makale izinli (allowlist) indirme.
//!
//! Ağ olmayan ortamda `kontrol` sessizce `None` döner; hiçbir fonksiyon panic
//! üretmez. Testler tamamen çevrimdışıdır (ağ çağrısı yapılmaz).

use std::cmp::Ordering;
use std::io::{Cursor, Read};
use std::time::Duration;

use sha2::{Digest, Sha256};

/// GitHub releases API uç noktası (Electron `main.ts` ile aynı depo).
const API_URL: &str = "https://api.github.com/repos/ArdaEkiz0/finansal-hesaplayici/releases/latest";
const USER_AGENT: &str = "FinansalHesaplaci-UpdateChecker";

/// İndirmeye izin verilen hostlar. GitHub 2026'da release asset indirmelerini
/// `release-assets.githubusercontent.com`'a taşıdı — bu host olmadan son URL
/// doğrulaması tüm güncellemeleri reddediyordu. `github-releases.*` gibi eski
/// hostlar kasten dahil değil.
const IZINLI_HOSTLAR: [&str; 3] = [
    "github.com",
    "objects.githubusercontent.com",
    "release-assets.githubusercontent.com",
];

/// Tek bir güncelleme indirmesi için azami boyut (64 MiB). Gerçek kurulum
/// exe'si ~5 MB; şişirilmiş/bozuk bir yanıtın belleği doldurmasını engeller.
const MAX_INDIRME: u64 = 64 * 1024 * 1024;

/// Bir güncellemeyi tanımlayan, API'den çıkarılan bilgiler.
///
/// `url`/`sha`/`sha_url` `update.zip` içindir (bilgi amaçlı korunur).
/// Gerçek kurulum `-kurulum.exe` varlığından yapılır; doğrulanamazsa
/// güncelleme hiç sunulmaz (fail-closed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Guncelleme {
    pub version: String,
    pub url: String,
    pub sha: String,
    pub sha_url: String,
    pub notes: String,
    pub kurulum_url: String,
    pub kurulum_sha: String,
}

/// Kısa zaman aşımlı, yeniden kullanılabilir HTTP istemcisi.
fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(8))
        .timeout_read(Duration::from_secs(15))
        // GitHub indirme URL'i 302 ile objects.githubusercontent.com'a gider;
        // yönlendirme kapatılırsa indirme çalışmaz. Sınırla ve SON URL'i
        // `indir_ve_kur` içinde tekrar doğrula (allowlist atlatılamasın).
        .redirects(3)
        .build()
}

/// URL'yi ayrıştırıp yalnızca `https` + izinli host kombinasyonunu kabul eder.
/// `Url::host_str()` kullanıldığı için `https://github.com@evil.com/...`
/// gibi kullanıcı-adı (userinfo) hileleri doğru şekilde reddedilir.
fn host_izinlimi(url_str: &str) -> bool {
    let Ok(parsed) = url::Url::parse(url_str) else {
        return false;
    };
    if parsed.scheme() != "https" {
        return false;
    }
    match parsed.host_str() {
        Some(host) => {
            let host = host.to_ascii_lowercase();
            IZINLI_HOSTLAR.iter().any(|izinli| *izinli == host)
        }
        None => false,
    }
}

/// Sürüm dizesindeki sayısal parçaları çıkarır (`v5.0.0` → `[5,0,0]`).
fn surum_parcala(s: &str) -> Vec<u64> {
    let s = s.trim().trim_start_matches(|c| c == 'v' || c == 'V');
    s.split(|c: char| c == '.' || c == '-' || c == '+' || c == '_')
        .map(|p| p.parse::<u64>().unwrap_or(0))
        .collect()
}

/// İki sürümü parça parça (semver benzeri) karşılaştırır.
fn surum_karsilastir(a: &str, b: &str) -> Ordering {
    let pa = surum_parcala(a);
    let pb = surum_parcala(b);
    let n = pa.len().max(pb.len());
    for i in 0..n {
        let x = *pa.get(i).unwrap_or(&0);
        let y = *pb.get(i).unwrap_or(&0);
        match x.cmp(&y) {
            Ordering::Equal => continue,
            fark => return fark,
        }
    }
    Ordering::Equal
}

/// İçerik bakımından sabit zamanlı byte karşılaştırması. Uzunluk bilgisi sır
/// değildir (SHA-256 her zaman 32 bayt), bu yüzden yalnızca uzunluk kontrolü
/// erken çıkış yapabilir; bayt karşılaştırmasında erken çıkış yoktur.
#[inline(never)]
fn sabit_zamanli_esit(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut fark = 0u8;
    for i in 0..a.len() {
        fark |= a[i] ^ b[i];
    }
    fark == 0
}

/// Beklenen özeti (hex, isteğe bağlı `sha256:` öneki ve dosya adı) 32 byte'a
/// çözer. Geçersizse `None`.
fn ozet_coz(beklenen: &str) -> Option<[u8; 32]> {
    let tok = beklenen
        .trim()
        .split_whitespace()
        .next()?
        .to_ascii_lowercase();
    let tok = tok.strip_prefix("sha256:").unwrap_or(&tok);
    let bytes = hex::decode(tok).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Some(out)
}

/// `data` üzerinin SHA-256'sini `beklenen` hex özetiyle sabit zamanlı karşılar.
/// Önünde `sha256:` veya sonunda dosya adı olan `.sha256` içerikleri de kabul
/// edilir. Geçersiz özet / uyumsuzluk durumunda `false`.
pub fn sha_dogrula(data: &[u8], beklenen: &str) -> bool {
    let Some(beklenen_bytes) = ozet_coz(beklenen) else {
        return false;
    };
    let gercek = Sha256::digest(data);
    sabit_zamanli_esit(gercek.as_slice(), &beklenen_bytes)
}

/// GitHub releases API'sine bakıp mevcut sürümden yeni bir sürüm var mı
/// kontrol eder. Ağ hatası, hatalı JSON, doğrulanamayan güncelleme veya
/// izinli olmayan URL durumlarında `None` döner — asla panic üretmez.
pub fn kontrol(surum: &str) -> Option<Guncelleme> {
    let yanit = agent()
        .get(API_URL)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/vnd.github+json")
        .call()
        .ok()?;
    let json: serde_json::Value = yanit.into_json().ok()?;
    guncelleme_cikar(&json, surum)
}

/// Verilen adda TEK bir varlık döndürür. Aynı amaçlı iki varlık varsa `None` —
/// belirsizlik durumunda sessizce "ilk bulunan"ı seçmek yerine fail-closed.
fn tek_varlik<'a>(
    assets: &'a [serde_json::Value],
    kosul: impl Fn(&str) -> bool,
) -> Option<&'a serde_json::Value> {
    let mut eslesenler = assets.iter().filter(|a| {
        kosul(
            a.get("name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(""),
        )
    });
    let ilk = eslesenler.next()?;
    eslesenler.next().is_none().then_some(ilk)
}

/// API JSON'unu `Guncelleme`'ye çevirir. Ağ çağrısı yapmaz; saf olduğu için
/// sahte JSON ile tamamen çevrimdışı test edilebilir.
fn guncelleme_cikar(json: &serde_json::Value, surum: &str) -> Option<Guncelleme> {
    // Prerelease asla otomatik güncelleme olarak sunulmaz.
    if json
        .get("prerelease")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return None;
    }

    let tag = json.get("tag_name")?.as_str()?;
    let version = tag.trim_start_matches(|c| c == 'v' || c == 'V').to_string();
    if version.is_empty() || surum_karsilastir(&version, surum) != Ordering::Greater {
        return None;
    }

    let assets = json.get("assets")?.as_array()?;
    let zip_varlik = tek_varlik(assets, |ad| ad.ends_with(".zip"))?;
    let sha_varlik = tek_varlik(assets, |ad| ad.ends_with(".sha256"));
    let kurulum_varlik = tek_varlik(assets, |ad| ad.ends_with("-kurulum.exe"))?;

    let alan = |v: &serde_json::Value, anahtar: &str| -> String {
        v.get(anahtar)
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    let ozet = |v: &serde_json::Value| -> String {
        let raw = v
            .get("digest")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        raw.strip_prefix("sha256:")
            .unwrap_or(raw)
            .trim()
            .to_ascii_lowercase()
    };

    let url = alan(zip_varlik, "browser_download_url");
    let sha = ozet(zip_varlik);
    let sha_url = sha_varlik
        .map(|v| alan(v, "browser_download_url"))
        .unwrap_or_default();
    let kurulum_url = alan(kurulum_varlik, "browser_download_url");
    let kurulum_sha = ozet(kurulum_varlik);

    if url.is_empty() || !host_izinlimi(&url) {
        return None;
    }
    if !sha_url.is_empty() && !host_izinlimi(&sha_url) {
        return None;
    }
    // Fail closed: doğrulanabilir bir özet yoksa güncelleme sunulmaz.
    if sha.is_empty() && sha_url.is_empty() {
        return None;
    }
    // Kurulum exe'si ve GitHub digest'i zorunlu. Biri eksikse "Güncelle"
    // düğmesi gösterilmez — kullanıcıyı çalışmayacak bir işleme yönlendirme.
    if kurulum_url.is_empty() || kurulum_sha.is_empty() || !host_izinlimi(&kurulum_url) {
        return None;
    }
    if ozet_coz(&kurulum_sha).is_none() {
        return None;
    }

    Some(Guncelleme {
        version,
        url,
        sha,
        sha_url,
        notes: json
            .get("body")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string(),
        kurulum_url,
        kurulum_sha,
    })
}

/// Çalışan exe'nin dizini: kurulum klasörü veya portable klasörü.
fn hedef_dizin() -> Option<std::path::PathBuf> {
    std::env::current_exe()
        .ok()?
        .parent()
        .map(|p| p.to_path_buf())
}

/// Köprü batch dosyasının içeriği. Windows'ta çalışan exe'nin üzerine
/// yazılamadığı için kurulum ayrı bir süreçte yapılır: beklenen PID kapanır,
/// sessiz kurulum çalışır, uygulama yeniden başlatılır ve izler silinir.
///
/// Yalnızca sayısal PID ve `%~dp0` kullanılır — kullanıcı girdisi veya
/// Türkçe karakterli yol doğrudan dosyaya yazılmaz, cmd kendi yolunu genişletir.
fn kopru_icerigi(pid: u32) -> String {
    let log = "%TEMP%\\fh-guncelle.log";
    format!(
        "@echo off\r\n\
         >>\"{log}\" echo === KURULUM BASLADI ===\r\n\
         ping -n 3 127.0.0.1 >nul\r\n\
         taskkill /PID {pid} >nul 2>&1\r\n\
         >>\"{log}\" echo UYGULAMA_KAPATILDI\r\n\
         start /wait \"\" \"%~dp0fh-kurulum.exe\" /VERYSILENT /NORESTART /SUPPRESSMSGBOXES\r\n\
         >>\"{log}\" echo KURULUM_ERRORLEVEL=%errorlevel%\r\n\
         del \"%~dp0fh-kurulum.exe\" >nul 2>&1\r\n\
         start \"\" \"%~dp0Finansal Hesaplayici.exe\"\r\n\
         >>\"{log}\" echo YENIDEN_BASLATILDI\r\n\
         del \"%~f0\"\r\n"
    )
}

/// Portable köprüsü: çalışan exe kapanınca indirilen yeni exe eski dosyanın
/// üzerine `move /Y` ile yazılır ve aynı adla yeniden başlatılır. Ad yalnızca
/// doğrulanmış ASCII karakterlerden gelir (bkz. `guvenli_exe_adi`).
fn kopru_portable_icerigi(pid: u32, exe_adi: &str) -> String {
    let log = "%TEMP%\\fh-guncelle.log";
    format!(
        "@echo off\r\n\
         >>\"{log}\" echo === PORTABLE BASLADI ===\r\n\
         ping -n 3 127.0.0.1 >nul\r\n\
         taskkill /PID {pid} >nul 2>&1\r\n\
         >>\"{log}\" echo UYGULAMA_KAPATILDI\r\n\
         move /Y \"%~dp0fh-yeni.exe\" \"%~dp0{exe_adi}\" >>\"{log}\" 2>&1\r\n\
         >>\"{log}\" echo MOVE_ERRORLEVEL=%errorlevel%\r\n\
         start \"\" \"%~dp0{exe_adi}\"\r\n\
         >>\"{log}\" echo YENIDEN_BASLATILDI\r\n\
         del \"%~f0\"\r\n"
    )
}

/// exe adı köprü dosyasına yazılacağı için yalnızca güvenli karakterlere izin ver.
fn guvenli_exe_adi(ad: &str) -> bool {
    !ad.is_empty()
        && ad.len() <= 200
        && ad.chars().all(|c| c.is_ascii_alphanumeric() || "._- ".contains(c))
        && ad.to_ascii_lowercase().ends_with(".exe")
}

/// `update.zip` içinden kök dizindeki exe'yi çıkarır. Zip-slip (iç içe yol,
/// `..`) girdileri atlanır; yalnızca tek bir .exe kabul edilir.
fn zipden_exe_cikar(veri: &[u8]) -> Option<Vec<u8>> {
    let mut arsiv = zip::ZipArchive::new(Cursor::new(veri)).ok()?;
    let mut bulunan = None;
    for i in 0..arsiv.len() {
        let mut dosya = arsiv.by_index(i).ok()?;
        let ad = dosya.name();
        if ad.contains('/')
            || ad.contains('\\')
            || ad.contains("..")
            || !ad.to_ascii_lowercase().ends_with(".exe")
        {
            continue;
        }
        let mut cikti = Vec::new();
        dosya.read_to_end(&mut cikti).ok()?;
        if cikti.is_empty() || cikti.len() as u64 > MAX_INDIRME || bulunan.is_some() {
            return None; // birden fazla exe veya boş/şişkin dosya → belirsiz
        }
        bulunan = Some(cikti);
    }
    bulunan
}

/// İndir + son URL allowlist + boyut sınırı + SHA-256. Her hata `None`.
fn indir_dogrula(istemci: &ureq::Agent, url: &str, beklenen: &str) -> Option<Vec<u8>> {
    if !host_izinlimi(url) || ozet_coz(beklenen).is_none() {
        return None;
    }
    let yanit = istemci.get(url).call().ok()?;
    if !host_izinlimi(&yanit.get_url().to_string()) {
        return None;
    }
    if let Some(uzunluk) = yanit
        .header("Content-Length")
        .and_then(|v| v.parse::<u64>().ok())
    {
        if uzunluk > MAX_INDIRME {
            return None;
        }
    }
    let mut veri = Vec::new();
    let okunan = yanit
        .into_reader()
        .take(MAX_INDIRME + 1)
        .read_to_end(&mut veri);
    if okunan.is_err() || veri.len() as u64 > MAX_INDIRME || !sha_dogrula(&veri, beklenen) {
        return None;
    }
    Some(veri)
}

/// Portable kurulum mu? (exe adı `portable` içeriyorsa)
///
/// Portable modda `indir_ve_kur` `update.zip`'ten exe çıkarır ve mevcut exe'nin
/// üzerine yazar (ayrı köprü). Kurulum modundan farklı akış kullanılır.
pub fn portable_mi() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().to_ascii_lowercase())
        })
        .is_some_and(|ad| ad.contains("portable"))
}

/// Güncellemeyi indirir, SHA-256 ile doğrular ve değiştirmede köprüyü başlatır.
///
/// * Kurulum modunda `kurulum.exe` indirilir, doğrulanır ve sessiz kurulum
///   köprüsü çalıştırılır.
/// * Portable modunda `update.zip` indirilir, doğrulanır, içindeki exe
///   güvenli şekilde çıkarılır ve köprü mevcut exe'nin üzerine yazar.
///
/// Her iki yolda da yalnızca izinli hostlara HTTPS, son URL kontrolü ve 64 MiB
/// sınırı vardır. Köprü adımları `%TEMP%\fh-guncelle.log`'a yazılır.
pub fn indir_ve_kur(g: &Guncelleme) -> bool {
    let istemci = agent();
    let portable = portable_mi();

    let (veri, gecici_adi, kopru_metni) = if portable {
        // update.zip digest'i tüm içeriği kapsar; exe ayrıca zip-slip'e karşı
        // `zipden_exe_cikar` ile denetlenir.
        let beklenen = g.sha.trim().to_ascii_lowercase();
        let Some(zip_veri) = indir_dogrula(&istemci, &g.url, &beklenen) else {
            return false;
        };
        let Some(exe_veri) = zipden_exe_cikar(&zip_veri) else {
            return false;
        };
        let Some(mevcut) = std::env::current_exe().ok() else {
            return false;
        };
        let Some(ad) = mevcut
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
        else {
            return false;
        };
        if !guvenli_exe_adi(&ad) {
            return false;
        }
        (
            exe_veri,
            "fh-yeni.exe".to_string(),
            kopru_portable_icerigi(std::process::id(), &ad),
        )
    } else {
        let beklenen = g.kurulum_sha.trim().to_ascii_lowercase();
        let Some(veri) = indir_dogrula(&istemci, &g.kurulum_url, &beklenen) else {
            return false;
        };
        (
            veri,
            "fh-kurulum.exe".to_string(),
            kopru_icerigi(std::process::id()),
        )
    };

    let Some(hedef) = hedef_dizin() else {
        return false;
    };
    let gecici = hedef.join(&gecici_adi);
    let kopru = hedef.join("fh-guncelle.cmd");

    if std::fs::write(&gecici, &veri).is_err() {
        return false;
    }
    if std::fs::write(&kopru, kopru_metni).is_err() {
        let _ = std::fs::remove_file(&gecici);
        return false;
    }

    // Görünmez bir cmd penceresinde, ana süreçten bağımsız başlat.
    let baslatildi = std::process::Command::new("cmd")
        .args(["/C", "start", "", "/MIN"])
        .arg(&kopru)
        .spawn()
        .is_ok();
    if !baslatildi {
        let _ = std::fs::remove_file(&gecici);
        let _ = std::fs::remove_file(&kopru);
    }
    baslatildi
}

#[cfg(test)]
mod tests {
    use super::*;

    // "abc" için bilinen SHA-256 değeri.
    const ABC_SHA: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    // Boş veri için bilinen SHA-256 değeri.
    const BOS_SHA: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    #[test]
    fn sha_dogrula_dogru_ozet() {
        assert!(sha_dogrula(b"abc", ABC_SHA));
    }

    #[test]
    fn sha_dogrula_yanlis_ozet() {
        let yanlis = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(!sha_dogrula(b"abc", yanlis));
    }

    #[test]
    fn sha_dogrula_buyuk_harf_ve_prefix() {
        let buyuk = format!("SHA256:{}", ABC_SHA.to_ascii_uppercase());
        assert!(sha_dogrula(b"abc", &buyuk));
    }

    #[test]
    fn sha_dogrula_sha256_dosya_ici() {
        let dosya_ici = format!("{ABC_SHA}  update.zip");
        assert!(sha_dogrula(b"abc", &dosya_ici));
    }

    #[test]
    fn sha_dogrula_gecersiz_hex() {
        let gecersiz = "zz7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert!(!sha_dogrula(b"abc", gecersiz));
    }

    #[test]
    fn sha_dogrula_yanlis_uzunluk() {
        assert!(!sha_dogrula(b"abc", "ba7816bf"));
        assert!(!sha_dogrula(b"abc", ""));
    }

    #[test]
    fn sha_dogrula_bos_veri() {
        assert!(sha_dogrula(b"", BOS_SHA));
    }

    #[test]
    fn sha_dogrula_veri_degisirse_red() {
        assert!(!sha_dogrula(b"abd", ABC_SHA));
    }

    #[test]
    fn host_github_com_izinli() {
        assert!(host_izinlimi(
            "https://github.com/ArdaEkiz0/repo/releases/download/v5.0.0/a.zip"
        ));
    }

    #[test]
    fn host_objects_izinli() {
        assert!(host_izinlimi(
            "https://objects.githubusercontent.com/path/update.zip"
        ));
    }

    #[test]
    fn host_yabanci_red() {
        assert!(!host_izinlimi("https://evil.example.com/update.zip"));
        assert!(!host_izinlimi(
            "https://github-releases.githubusercontent.com/a.zip"
        ));
    }

    #[test]
    fn host_http_red() {
        assert!(!host_izinlimi("http://github.com/update.zip"));
    }

    #[test]
    fn host_userinfo_trick_red() {
        // Gerçek host evil.com olduğu için reddedilmelidir.
        assert!(!host_izinlimi("https://github.com@evil.com/update.zip"));
    }

    #[test]
    fn host_gecersiz_url_red() {
        assert!(!host_izinlimi("bu-bir-url-degil"));
        assert!(!host_izinlimi(""));
    }

    #[test]
    fn surum_karsilastir_daha_yeni() {
        assert_eq!(surum_karsilastir("5.0.1", "5.0.0"), Ordering::Greater);
        assert_eq!(surum_karsilastir("v5.1.0", "5.0.9"), Ordering::Greater);
        assert_eq!(surum_karsilastir("6.0.0", "5.9.9"), Ordering::Greater);
    }

    #[test]
    fn surum_karsilastir_esit_ve_eski() {
        assert_eq!(surum_karsilastir("5.0.0", "5.0.0"), Ordering::Equal);
        assert_eq!(surum_karsilastir("v5.0.0", "5.0.0"), Ordering::Equal);
        assert_eq!(surum_karsilastir("5.0.0", "5.0.1"), Ordering::Less);
    }

    #[test]
    fn kopru_pid_ve_kurulumu_icerir() {
        let icerik = kopru_icerigi(4242);
        assert!(icerik.contains("taskkill /PID 4242"));
        assert!(!icerik.contains("/F")); // graceful kapatma, veri kaybı riski yok
        assert!(icerik.contains("fh-kurulum.exe"));
        assert!(icerik.contains("/VERYSILENT"));
        assert!(icerik.contains("/NORESTART"));
        assert!(icerik.contains("%~dp0Finansal Hesaplayici.exe"));
    }

    #[test]
    fn kopru_sabit_veya_kullanicidan_gelen_yol_icerir() {
        let icerik = kopru_icerigi(7);
        // Köprü kendi dizinini `%~dp0` ile çözer; sabit sürücü/yol yok.
        // Sabit sürücü/yok; %TEMP% yalnızca tanılama log'u için kullanılır.
        assert!(!icerik.contains("C:\\"));
        assert!(!icerik.contains("AppData"));
        assert!(icerik.contains("%~dp0"));
    }

    #[test]
    fn kopru_sadece_ascii() {
        // cmd dosyası UTF-8 yazılır; Türkçe karakter olsaydı bozulurdu.
        assert!(kopru_icerigi(1).is_ascii());
    }

    #[test]
    fn hedef_dizin_mevcut() {
        let d = hedef_dizin().expect("current_exe dizini");
        // CI'da test binary'sinin yolu değişir; yalnızca mutlak yol ve
        // geçerli bir dizin adı beklenir (belirli bir klasör adı değil).
        assert!(d.is_absolute());
        assert!(d.file_name().is_some());
        assert!(d.components().count() >= 2);
    }

    #[test]
    fn ozet_coz_digest_bicimleri() {
        assert!(ozet_coz(ABC_SHA).is_some());
        assert!(ozet_coz(&format!("sha256:{ABC_SHA}")).is_some());
        assert!(ozet_coz(&format!("{ABC_SHA}  update.zip")).is_some());
        assert!(ozet_coz("kisa").is_none());
    }

    fn sahte_json(surum: &str) -> serde_json::Value {
        let taban = "https://github.com/ArdaEkiz0/finansal-hesaplayici/releases/download";
        serde_json::json!({
            "tag_name": format!("v{surum}"),
            "prerelease": false,
            "body": "Türkçe notlar",
            "assets": [
                {
                    "name": format!("Finansal-Hesaplayici-{surum}-kurulum.exe"),
                    "browser_download_url": format!("{taban}/v{surum}/kurulum.exe"),
                    "digest": format!("sha256:{ABC_SHA}")
                },
                {
                    "name": "update.zip",
                    "browser_download_url": format!("{taban}/v{surum}/update.zip"),
                    "digest": format!("sha256:{ABC_SHA}")
                },
                {
                    "name": "update.zip.sha256",
                    "browser_download_url": format!("{taban}/v{surum}/update.zip.sha256")
                }
            ]
        })
    }

    #[test]
    fn guncelleme_cikar_tam_json() {
        let g = guncelleme_cikar(&sahte_json("5.7.4"), "5.7.3").expect("güncelleme");
        assert_eq!(g.version, "5.7.4");
        assert_eq!(g.kurulum_sha, ABC_SHA);
        assert!(g.kurulum_url.ends_with("/kurulum.exe"));
        assert!(g.notes.contains("Türkçe"));
    }

    #[test]
    fn guncelleme_cikar_kurulum_varligi_yok() {
        let mut j = sahte_json("5.7.4");
        j["assets"]
            .as_array_mut()
            .unwrap()
            .retain(|a| !a["name"].as_str().unwrap().ends_with("-kurulum.exe"));
        assert!(guncelleme_cikar(&j, "5.7.3").is_none());
    }

    #[test]
    fn guncelleme_cikar_digest_yok() {
        let mut j = sahte_json("5.7.4");
        j["assets"][0].as_object_mut().unwrap().remove("digest");
        assert!(guncelleme_cikar(&j, "5.7.3").is_none());
    }

    #[test]
    fn guncelleme_cikar_cift_kurulum_red() {
        let mut j = sahte_json("5.7.4");
        let ikinci = j["assets"][0].clone();
        j["assets"].as_array_mut().unwrap().push(ikinci);
        assert!(guncelleme_cikar(&j, "5.7.3").is_none());
    }

    #[test]
    fn guncelleme_cikar_prerelease_red() {
        let mut j = sahte_json("5.7.4");
        j["prerelease"] = serde_json::json!(true);
        assert!(guncelleme_cikar(&j, "5.7.3").is_none());
    }

    #[test]
    fn guncelleme_cikar_eski_ve_esit_surum() {
        assert!(guncelleme_cikar(&sahte_json("5.7.3"), "5.7.3").is_none());
        assert!(guncelleme_cikar(&sahte_json("5.7.2"), "5.7.3").is_none());
    }

    #[test]
    fn guncelleme_cikar_yabanci_host_red() {
        let mut j = sahte_json("5.7.4");
        j["assets"][0]["browser_download_url"] =
            serde_json::json!("https://evil.example.com/kurulum.exe");
        assert!(guncelleme_cikar(&j, "5.7.3").is_none());
    }

    #[test]
    fn boyut_siniri_makul() {
        assert!(MAX_INDIRME >= 1024 * 1024);
        assert!(MAX_INDIRME < 1024 * 1024 * 1024);
    }

    #[test]
    fn son_url_allowlist_kurallari() {
        // Yönlendirme sonrası URL de aynı kurallara tabi. GitHub'ın güncel
        // asset host'u (release-assets) mutlaka kabul edilmeli.
        assert!(host_izinlimi("https://objects.githubusercontent.com/a.exe"));
        assert!(host_izinlimi(
            "https://release-assets.githubusercontent.com/github-production-release-asset/x?sig=1"
        ));
        assert!(!host_izinlimi("https://evil.example.com/a.exe"));
        assert!(!host_izinlimi("http://release-assets.githubusercontent.com/a.exe"));
        assert!(!host_izinlimi("https://github-releases.githubusercontent.com/a.exe"));
    }

    fn zip_yap(girdiler: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::Write;
        let mut buf = Vec::new();
        {
            let mut z = zip::ZipWriter::new(Cursor::new(&mut buf));
            for (ad, veri) in girdiler {
                z.start_file(*ad, zip::write::SimpleFileOptions::default())
                    .unwrap();
                z.write_all(veri).unwrap();
            }
            z.finish().unwrap();
        }
        buf
    }

    #[test]
    fn zipden_tek_kok_exe_cikar() {
        let z = zip_yap(&[("Finansal Hesaplayici.exe", b"EXE")]);
        assert_eq!(zipden_exe_cikar(&z), Some(b"EXE".to_vec()));
    }

    #[test]
    fn zipden_yollu_girdi_red() {
        // Zip-slip: iç içe yol veya .. içeren ad asla kabul edilmez.
        let z = zip_yap(&[("alt/kotu.exe", b"EXE")]);
        assert!(zipden_exe_cikar(&z).is_none());
        let z = zip_yap(&[("../kotu.exe", b"EXE")]);
        assert!(zipden_exe_cikar(&z).is_none());
    }

    #[test]
    fn zipden_cift_exe_red() {
        let z = zip_yap(&[("a.exe", b"A"), ("b.exe", b"B")]);
        assert!(zipden_exe_cikar(&z).is_none());
    }

    #[test]
    fn zipden_exe_yok_red() {
        let z = zip_yap(&[("icon.ico", b"ICO")]);
        assert!(zipden_exe_cikar(&z).is_none());
    }

    #[test]
    fn guvenli_exe_adi_kurallari() {
        assert!(guvenli_exe_adi("Finansal-Hesaplayici-5.7.4-portable.exe"));
        assert!(guvenli_exe_adi("Finansal Hesaplayici.exe"));
        assert!(!guvenli_exe_adi("kotu/../x.exe"));
        assert!(!guvenli_exe_adi("x.bat"));
        assert!(!guvenli_exe_adi(""));
        assert!(!guvenli_exe_adi(&"a".repeat(300)));
    }

    #[test]
    fn kopru_portable_ad_ve_log_icerir() {
        let icerik = kopru_portable_icerigi(77, "Finansal-Hesaplayici-5.7.4-portable.exe");
        assert!(icerik.contains("taskkill /PID 77"));
        assert!(icerik.contains("move /Y"));
        assert!(icerik.contains("fh-yeni.exe"));
        assert!(icerik.contains("Finansal-Hesaplayici-5.7.4-portable.exe"));
        assert!(icerik.contains("fh-guncelle.log"));
        assert!(icerik.is_ascii());
    }

    #[test]
    fn kurulum_koprusu_log_yazar() {
        let icerik = kopru_icerigi(88);
        assert!(icerik.contains("fh-guncelle.log"));
        assert!(icerik.contains("KURULUM_ERRORLEVEL"));
        assert!(icerik.contains("/VERYSILENT"));
    }
}
