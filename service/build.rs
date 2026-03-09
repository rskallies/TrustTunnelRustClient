// Tell Cargo where to find the vpn_easy import library.
//
// MSVC builds expect vpn_easy.lib; MinGW builds expect libvpn_easy.a.
// Set VPN_EASY_LIB_DIR to the directory containing either file.
fn main() {
    if let Ok(dir) = std::env::var("VPN_EASY_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", dir);
    }
    println!("cargo:rerun-if-env-changed=VPN_EASY_LIB_DIR");
}
