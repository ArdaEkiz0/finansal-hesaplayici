//! fh-updater — GitHub releases üzerinden güncelleme kontrolü,
//! SHA-256 bütünlük doğrulaması ve ana makale izinli (allowlist) indirme.
//!
//! Ağ olmayan ortamda `kontrol` sessizce `None` döner; hiçbir fonksiyon panic
//! üretmez. Testler tamamen çevrimdışıdır (ağ çağrısı yapılmaz).

use std::cmp::Ordering;
use std::io::Read;
use std::time::Duration;

use sha2::{Digest, Sha256};

/// GitHub releases API uç noktası (Electron `main.ts` ile aynı depo).
const API_URL: &str = "https://api.github.com/repos/ArdaEkiz0/finansal-hesaplayici/releases/latest";
const USER_AGENT: &str = "FinansalHesaplaci-UpdateChecker";

/// İndirmeye izin verilen tek iki host. Alt alan adları ve diğer GitHub
/// hostları (ör. `github-releases.githubusercontent.com`) kasten dahil değil.
const IZINLI_HOSTLAR: [&str; 2] = ["github.com", "objects.githubusercontent.com"];

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
    let tok = beklenen.trim().split_whitespace().next()?.to_ascii_lowercase();
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

    let tag = json.get("tag_name")?.as_str()?;
    let version = tag.trim_start_matches(|c| c == 'v' || c == 'V').to_string();
    if version.is_empty() || surum_karsilastir(&version, surum) != Ordering::Greater {
        return None;
    }

    let assets = json.get("assets")?.as_array()?;
    let mut url = String::new();
    let mut sha = String::new();
    let mut sha_url = String::new();

    let mut kurulum_url = String::new();
    let mut kurulum_sha = String::new();

    for asset in assets {
        let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let download_url = asset
            .get("browser_download_url")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let ad = name.to_ascii_lowercase();
        let ozet = asset.get("digest").and_then(|v| v.as_str()).unwrap_or("");
        let ozet = ozet
            .strip_prefix("sha256:")
            .unwrap_or(ozet)
            .trim()
            .to_ascii_lowercase();

        if ad.ends_with(".zip") && url.is_empty() {
            url = download_url.to_string();
            sha = ozet.clone();
        } else if ad.ends_with(".sha256") && sha_url.is_empty() {
            sha_url = download_url.to_string();
        } else if ad.ends_with("-kurulum.exe") && kurulum_url.is_empty() {
            // Gerçek güncelleme varlığı: NSIS/Inno kurulumu.
            kurulum_url = download_url.to_string();
            kurulum_sha = ozet;
        }
    }

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
    if kurulum_url.is_empty()
        || kurulum_sha.is_empty()
        || !host_izinlimi(&kurulum_url)
    {
        return None;
    }

    Some(Guncelleme {
        version,
        url,
        sha,
        sha_url,
        notes: json
            .get("body")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        kurulum_url,
        kurulum_sha,
    })
}

/// Çalışan exe'nin dizini: kurulum klasörü veya portable klasörü.
fn hedef_dizin() -> Option<std::path::PathBuf> {
    std::env::current_exe().ok()?.parent().map(|p| p.to_path_buf())
}

/// Köprü batch dosyasının içeriği. Windows'ta çalışan exe'nin üzerine
/// yazılamadığı için kurulum ayrı bir süreçte yapılır: beklenen PID kapanır,
/// sessiz kurulum çalışır, uygulama yeniden başlatılır ve izler silinir.
///
/// Yalnızca sayısal PID ve `%~dp0` kullanılır — kullanıcı girdisi veya
/// Türkçe karakterli yol doğrudan dosyaya yazılmaz, cmd kendi yolunu genişletir.
fn kopru_icerigi(pid: u32) -> String {
    format!(
        "@echo off\r\n\
         ping -n 3 127.0.0.1 >nul\r\n\
         taskkill /F /PID {pid} >nul 2>&1\r\n\
         start /wait \"\" \"%~dp0fh-kurulum.exe\" /VERYSILENT /NORESTART /SUPPRESSMSGBOXES\r\n\
         del \"%~dp0fh-kurulum.exe\" >nul 2>&1\r\n\
         start \"\" \"%~dp0Finansal Hesaplayici.exe\"\r\n\
         del \"%~f0\"\r\n"
    )
}

/// Güncellemeyi indirir, SHA-256 ile doğrular ve kurulumu başlatır.
///
/// Kurulum exe'si yalnızca izinli hostlardan `https` ile indirilir; özet
/// GitHub API `digest` alanından gelir ve doğrulanmadan hiçbir dosya yazılmaz.
/// Köprü dosyaları çalışan exe'nin dizinine yazılır; başarı `true` döner ve
/// arayan taraf uygulamayı kapatır (köprü yeniden başlatır).
pub fn indir_ve_kur(g: &Guncelleme) -> bool {
    if !host_izinlimi(&g.kurulum_url) {
        return false;
    }
    let beklenen = g.kurulum_sha.trim().to_ascii_lowercase();
    if ozet_coz(&beklenen).is_none() {
        return false;
    }

    let istemci = agent();
    let yanit = match istemci.get(&g.kurulum_url).call() {
        Ok(r) => r,
        Err(_) => return false,
    };
    let mut veri = Vec::new();
    if yanit.into_reader().read_to_end(&mut veri).is_err() {
        return false;
    }
    if !sha_dogrula(&veri, &beklenen) {
        return false;
    }

    let Some(hedef) = hedef_dizin() else {
        return false;
    };
    let kurulum = hedef.join("fh-kurulum.exe");
    let kopru = hedef.join("fh-guncelle.cmd");

    if std::fs::write(&kurulum, &veri).is_err() {
        return false;
    }
    if std::fs::write(&kopru, kopru_icerigi(std::process::id())).is_err() {
        let _ = std::fs::remove_file(&kurulum);
        return false;
    }

    // Görünmez bir cmd penceresinde, ana süreçten bağımsız başlat.
    std::process::Command::new("cmd")
        .args(["/C", "start", "", "/MIN"])
        .arg(&kopru)
        .spawn()
        .is_ok()
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
        assert!(host_izinlimi("https://github.com/ArdaEkiz0/repo/releases/download/v5.0.0/a.zip"));
    }

    #[test]
    fn host_objects_izinli() {
        assert!(host_izinlimi("https://objects.githubusercontent.com/path/update.zip"));
    }

    #[test]
    fn host_yabanci_red() {
        assert!(!host_izinlimi("https://evil.example.com/update.zip"));
        assert!(!host_izinlimi("https://github-releases.githubusercontent.com/a.zip"));
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
        assert!(icerik.contains("taskkill /F /PID 4242"));
        assert!(icerik.contains("fh-kurulum.exe"));
        assert!(icerik.contains("/VERYSILENT"));
        assert!(icerik.contains("/NORESTART"));
        assert!(icerik.contains("%~dp0Finansal Hesaplayici.exe"));
    }

    #[test]
    fn kopru_sabit_veya_kullanicidan_gelen_yol_icerir() {
        let icerik = kopru_icerigi(7);
        // Köprü kendi dizinini `%~dp0` ile çözer; sabit sürücü/yol yok.
        assert!(!icerik.contains("C:\\"));
        assert!(!icerik.contains("%TEMP%"));
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
}
