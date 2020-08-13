// vim: tw=80

fn main() {
    let binding_config = || {
        bindgen::Builder::default()
            .header("wrapper.h")
            .whitelist_type(".*BLOSC.*")
            .whitelist_function(".*blosc.*")
            .whitelist_var(".*BLOSC.*")
    };

    let mut bindings = if let Ok(bindings) = binding_config().generate() {
        Some(bindings)
    } else {
        None
    };
    // Fallback to conda bindings
    if bindings.is_none() {
        if let Ok(conda_prefix) = env::var("CONDA_PREFIX") {
            if let Ok(bind) = binding_config()
                .clang_arg(format!("-I{}/include", conda_prefix))
                .generate()
            {
                #[cfg(windows)]
                println!("cargo:rustc-link-search=native={}/Library", conda_prefix);
                #[cfg(not(windows))]
                println!("cargo:rustc-link-search=native={}/lib", conda_prefix);
                bindings = Some(bind)
            } else {
            }
        }
    };
    let bindings = bindings.expect("Could not create bindings");
    println!("cargo:rustc-link-lib=dylib=blosc");

    let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindgen.rs"))
        .expect("Failed to write bindings");
}
