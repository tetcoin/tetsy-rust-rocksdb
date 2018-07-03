extern crate cc;
extern crate cmake;

use std::env;

use cc::Build;
use cmake::Config;

fn build_lz4() {
    Build::new()
        .include("lz4/lib")
        .opt_level(3)
        .file("lz4/lib/lz4.c")
        .file("lz4/lib/lz4frame.c")
        .file("lz4/lib/lz4hc.c")
        .file("lz4/lib/xxhash.c")
        .compile("liblz4.a");
}

fn main() {
    let mut cfg = Config::new("rocksdb");
    cfg.define("CMAKE_VERBOSE_MAKEFILE", "ON")
        .register_dep("SNAPPY")
        .define("WITH_SNAPPY", "ON")
        .register_dep("LZ4")
        .define("WITH_LZ4", "ON")
        .build_target("rocksdb");

    let snappy = env::var_os("DEP_SNAPPY_INCLUDE").expect("DEP_SNAPPY_INCLUDE is set in snappy.");
    let lz4 = env::current_dir().unwrap().join("lz4/lib");


    if cfg!(target_env = "msvc") {
        cfg.env("SNAPPY_INCLUDE", snappy);
        cfg.env("LZ4_INCLUDE", lz4);

        println!("cargo:rustc-link-lib=dylib={}", "rpcrt4");
        println!("cargo:rustc-link-lib=dylib={}", "shlwapi");
    } else {
        cfg.define("SNAPPY_INCLUDE_DIR", snappy)
            .define("SNAPPY_LIBRARIES", "/dev/null");
        cfg.define("LZ4_INCLUDE_DIR", lz4)
            .define("LZ4_LIBRARIES", "/dev/null");
    }

    build_lz4();

    let out = cfg.build();

    let mut build = out.join("build");

    if cfg!(target_os = "windows") {
        let profile = match &*env::var("PROFILE").unwrap_or("debug".to_owned()) {
            "bench" | "release" => "Release",
            _ => "Debug",
        };
        build = build.join(profile);
    }

    println!("cargo:rustc-link-search=native={}", build.display());
    println!("cargo:rustc-link-lib=static=rocksdb");
    println!("cargo:rustc-link-lib=static=snappy");
    println!("cargo:rustc-link-lib=static=lz4");

    // https://github.com/alexcrichton/cc-rs/blob/ca70fd32c10f8cea805700e944f3a8d1f97d96d4/src/lib.rs#L891
    if cfg!(any(target_os = "macos", target_os = "freebsd", target_os = "openbsd")) {
        println!("cargo:rustc-link-lib=c++");
    } else if cfg!(not(target_env = "msvc")) {
        println!("cargo:rustc-link-lib=stdc++");
    }
}
