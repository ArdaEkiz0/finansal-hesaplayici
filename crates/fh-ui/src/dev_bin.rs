// Yalnızca Cargo.toml'da sözleşme adını koruyan pasif hedefin kaynak dosyası.
// rustc 1.99 boşluklu crate adını reddettiği için bu hedef hiçbir zaman
// derlenmez (required-features kapalı); gerçek uygulama src/main.rs'dedir.
fn main() {}
