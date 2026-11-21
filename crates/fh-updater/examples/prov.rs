//! İzole köprü provası: sahte klasörde çalışan "eski.exe" + gerçek köprü içeriği.
//! Gerçek güncelleme akışını tetiklemez; move/yeniden-başlat mekanizmasını kanıtlar.
#[cfg(windows)]
use std::os::windows::process::CommandExt;

fn main() {
    let dir = std::path::PathBuf::from(r"C:\Users\ozel\AppData\Local\Temp\opencode\kopru-prova");
    std::fs::create_dir_all(&dir).unwrap();
    let eski = dir.join("eski.exe");
    let yeni = dir.join("fh-yeni.exe");
    let kopru = dir.join("fh-guncelle.cmd");

    // eski = notepad (WM_CLOSE'a yanıt verir), yeni = control.exe (GUI, yaşar).
    let eski_veri = std::fs::read(r"C:\Windows\System32\notepad.exe").unwrap();
    let yeni_veri = std::fs::read(r"C:\Windows\System32\control.exe").unwrap();
    std::fs::write(&eski, &eski_veri).unwrap();
    std::fs::write(&yeni, &yeni_veri).unwrap();
    println!("YENI_ONCE_HASH={}", sha2_digest(&yeni_veri));

    let calisan = std::process::Command::new(&eski).spawn().unwrap();
    let pid = calisan.id();
    std::fs::write(&kopru, fh_updater::kopru_guncelleme_icerigi(pid, "eski.exe")).unwrap();

    let mut komut = std::process::Command::new("cmd");
    komut.arg("/C").arg(&kopru);
    komut.stdout(std::process::Stdio::null());
    komut.stderr(std::process::Stdio::null());
    #[cfg(windows)]
    komut.creation_flags(0x0800_0000);
    println!("PID={pid} spawn={}", komut.spawn().is_ok());
}

fn sha2_digest(v: &[u8]) -> String {
    // sha2 crate zaten bağımlılık; kısa özet yeterli.
    use sha2::{Digest, Sha256};
    Sha256::digest(v)
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect()
}
