//! Windows exe kaynakları: ikon + sürüm/ürün/şirket bilgisi.
//! `rc.exe` Windows SDK ile gelir; CI (windows-latest) ve yerel makinede mevcut.

fn main() {
    println!("cargo:rerun-if-changed=../../build/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let surum = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let mut parcalar: Vec<u64> = surum
        .split('.')
        .map(|p| p.parse::<u64>().unwrap_or(0))
        .collect();
    while parcalar.len() < 4 {
        parcalar.push(0);
    }
    let ikili_surum = ((parcalar[0] & 0xffff) << 48)
        | ((parcalar[1] & 0xffff) << 32)
        | ((parcalar[2] & 0xffff) << 16)
        | (parcalar[3] & 0xffff);

    let mut kaynak = winres::WindowsResource::new();
    kaynak
        .set_icon("../../build/icon.ico")
        .set("ProductName", "Finansal Hesaplayıcı")
        .set("FileDescription", "Finansal Hesaplayıcı - Türk mali hesaplayıcı")
        .set("CompanyName", "Arda Mehmet Ekiz")
        .set("LegalCopyright", "Copyright © 2026 Arda Mehmet Ekiz")
        .set("InternalName", "Finansal_Hesaplayici.exe")
        .set("OriginalFilename", "Finansal Hesaplayici.exe")
        .set("ProductVersion", &surum)
        .set("FileVersion", &surum)
        .set_version_info(winres::VersionInfo::FILEVERSION, ikili_surum)
        .set_version_info(winres::VersionInfo::PRODUCTVERSION, ikili_surum);

    kaynak
        .compile()
        .expect("Windows exe kaynakları (ikon/sürüm) derlenemedi");
}
