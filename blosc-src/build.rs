fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let mut cfg = cmake::Config::new("c-blosc");

    for option in &[
        "BUILD_SHARED",
        "BUILD_TESTS",
        "BUILD_FUZZERS",
        "BUILD_BENCHMARKS",
    ] {
        cfg.define(option, "OFF");
    }

    if !cfg!(target_feature = "sse2") {
        cfg.define("DEACTIVATE_SSE2", "OFF");
    }
    if !cfg!(target_feature = "avx") {
        cfg.define("DEACTIVATE_AVX", "OFF");
    }

    let dst = cfg.build();
    println!("cargo:root={}", dst.display());
    let incdir = format!("{}/include", dst.display());
    println!("cargo:include={}", incdir);
    println!(
        "cargo:library={}",
        if cfg!(target_env = "msvc") {
            "libblosc"
        } else {
            "blosc"
        }
    );
}
