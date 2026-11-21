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
    format!(
        "@echo off\r\n\
         ping -n 3 127.0.0.1 >nul\r\n\
         taskkill /PID {pid} >nul 2>&1\r\n\
         start /wait \"\" \"%~dp0fh-kurulum.exe\" /VERYSILENT /NORESTART /SUPPRESSMSGBOXES\r\n\
         del \"%~dp0fh-kurulum.exe\" >nul 2>&1\r\n\
         start \"\" \"%~dp0Finansal Hesaplayici.exe\"\r\n\
         del \"%~f0\"\r\n"
    )
}

/// Portable kurulum mu? (exe adı `portable` içeriyorsa)
///
/// Portable sürümde köprü kurulu exe adını başlatır ve salt-okunur klasöre
/// yazma başarısız olabilir; otomatik güncelleme sağlıklı değildir. Bu yüzden
/// portable modda güncelleme reddedilir, kullanıcıya kurulum indirmesi söylenir.
pub fn portable_mi() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().to_ascii_lowercase())
        })
        .is_some_and(|ad| ad.contains("portable"))
}

/// Güncellemeyi indirir, SHA-256 ile doğrular ve kurulumu başlatır.
///
/// Kurulum exe'si yalnızca izinli hostlardan `https` ile indirilir; özet
/// GitHub API `digest` alanından gelir ve doğrulanmadan hiçbir dosya yazılmaz.
/// Köprü dosyaları çalışan exe'nin dizinine yazılır; başarı `true` döner ve
/// arayan taraf uygulamayı kapatır (köprü yeniden başlatır).
pub fn indir_ve_kur(g: &Guncelleme) -> bool {
    // Portable sürüm otomatik güncellenemez (bkz. portable_mi).
    if portable_mi() || !host_izinlimi(&g.kurulum_url) {
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
    // Yönlendirme sonrası GERÇEK URL de allowlist'te olmalı — ilk URL
    // github.com olsa bile 302 başka bir hosta götürebilir.
    let son_url = yanit.get_url().to_string();
    if !host_izinlimi(&son_url) {
        return false;
    }
    // Bildirilen boyut sınırı aşıyorsa hiç okumaya başlama.
    if let Some(uzunluk) = yanit
        .header("Content-Length")
        .and_then(|v| v.parse::<u64>().ok())
    {
        if uzunluk > MAX_INDIRME {
            return false;
        }
    }

    // Akış sırasında da sınırla: Content-Length yoksa veya yalan söylüyorsa
    // belleği doldurmasın. MAX+1 bayt okunduysa fazla var demektir.
    let mut veri = Vec::new();
    let okunan = yanit
        .into_reader()
        .take(MAX_INDIRME + 1)
        .read_to_end(&mut veri);
    if okunan.is_err() || veri.len() as u64 > MAX_INDIRME {
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
    let baslatildi = std::process::Command::new("cmd")
        .args(["/C", "start", "", "/MIN"])
        .arg(&kopru)
        .spawn()
        .is_ok();
    if !baslatildi {
        // Köprü çalışmadıysa iz bırakma: arayan yine kapanacak, dosyalar
        // kurulum dizininde kalmasın.
        let _ = std::fs::remove_file(&kurulum);
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
        // Yönlendirme sonrası URL de aynı kurallara tabi.
        assert!(host_izinlimi("https://objects.githubusercontent.com/a.exe"));
        assert!(!host_izinlimi("https://evil.example.com/a.exe"));
        assert!(!host_izinlimi("http://objects.githubusercontent.com/a.exe"));
    }
}
