// vim: tw=80

fn main() {
    if std::env::var_os("CARGO_FEATURE_STATIC").is_some() {
        let blosc_lib = std::env::var("DEP_BLOSCSRC_LIBRARY").unwrap();
        let blosc_root = std::env::var("DEP_BLOSCSRC_ROOT").unwrap();
        println!("cargo:rustc-link-search=native={}/lib", blosc_root);
        println!("cargo:rustc-link-lib=static={}", blosc_lib);
    } else {
        println!("cargo:rustc-link-lib=blosc");
    }
}
