use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let vendor = PathBuf::from("vendor/tilem");

    // Collect all .c sources: core files + per-model subdirs.
    let mut sources: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(&vendor).expect("vendor/tilem missing") {
        let path = entry.unwrap().path();
        if path.extension().map(|e| e == "c").unwrap_or(false) {
            sources.push(path);
        } else if path.is_dir() {
            for sub in fs::read_dir(&path).unwrap() {
                let sp = sub.unwrap().path();
                if sp.extension().map(|e| e == "c").unwrap_or(false) {
                    sources.push(sp);
                }
            }
        }
    }

    let mut build = cc::Build::new();
    build
        .include(&vendor)
        .define("PACKAGE_VERSION", Some("\"2.0\""))
        .define("_GNU_SOURCE", None)
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-sign-compare")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .flag_if_supported("-Wno-unused-but-set-variable")
        .flag_if_supported("-Wno-unused-variable")
        .flag_if_supported("-Wno-misleading-indentation")
        .flag_if_supported("-Wno-parentheses")
        .flag_if_supported("-Wno-maybe-uninitialized")
        .flag_if_supported("-Wno-pointer-sign")
        .flag_if_supported("-Wno-implicit-function-declaration")
        .flag_if_supported("-Wno-deprecated-non-prototype")
        .flag_if_supported("-Wno-strict-prototypes")
        .flag_if_supported("-std=gnu99");
    for s in &sources {
        build.file(s);
    }
    build.compile("tilemcore");

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");
    for s in &sources {
        println!("cargo:rerun-if-changed={}", s.display());
    }

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", vendor.display()))
        .allowlist_function("tilem_.*")
        .allowlist_type("Tilem.*")
        .allowlist_var("TILEM_.*")
        // Host clang runs on macOS; struct layout tests it generates are
        // anchored to host sizes and can spuriously fail when rustc targets
        // aarch64-linux-musl.  The C core sizes are the source of truth at
        // runtime, so skip the compile-time assertions.
        .layout_tests(false)
        .generate()
        .expect("bindgen failed");

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("tilem_bindings.rs");
    bindings.write_to_file(&out).expect("writing bindings");
}
