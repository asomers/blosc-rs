use std::fs;

use cc::Build;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut builder = cc::Build::new();

    let compile_folder = |builder: &mut Build, folder: &str| {
        for entry in fs::read_dir(folder).unwrap() {
            let path = entry.unwrap().path();
            if let Some(extension) = path.extension() {
                if extension == "c" || extension == "cpp" {
                    builder.file(path);
                }
            }
        }
    };

    compile_folder(&mut builder, "../c-blosc/blosc");
    compile_folder(&mut builder, "../c-blosc/internal-complibs/lz4-1.9.4");
    compile_folder(&mut builder, "../c-blosc/internal-complibs/zlib-1.2.13");
    compile_folder(
        &mut builder,
        "../c-blosc/internal-complibs/zstd-1.5.4/common",
    );
    compile_folder(
        &mut builder,
        "../c-blosc/internal-complibs/zstd-1.5.4/compress",
    );
    compile_folder(
        &mut builder,
        "../c-blosc/internal-complibs/zstd-1.5.4/decompress",
    );
    compile_folder(
        &mut builder,
        "../c-blosc/internal-complibs/zstd-1.5.4/dictBuilder",
    );

    builder.includes([
        "../c-blosc/internal-complibs/lz4-1.9.4",
        "../c-blosc/internal-complibs/zlib-1.2.13",
        "../c-blosc/internal-complibs/zstd-1.5.4",
    ]);
    builder.define("HAVE_LZ4", None);
    builder.define("HAVE_ZLIB", None);
    builder.define("HAVE_ZSTD", None);

    let linklib = if cfg!(target_env = "msvc") {
        "libblosc"
    } else {
        "blosc"
    };
    builder.compile(linklib);
    //println!("cargo:root={}", root);
    //let incdir = format!("{}/include", root);
    //println!("cargo:include={}", incdir);
    //println!("cargo:library={}", linklib);
    //println!("cargo:rustc-link-search=native={}/lib", root);
    //println!("cargo:rustc-link-lib=static={}", linklib);
}
