extern crate cc;
extern crate cmake;

use std::env;

use cc::Build;
use cmake::Config;

fn lz4() {
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
		.define("WITH_GFLAGS", "OFF")
		.build_target("rocksdb");

	if cfg!(target_env = "msvc") {
		cfg.env("SNAPPY_INCLUDE", env::var_os("DEP_SNAPPY_INCLUDE").expect("DEP_SNAPPY_INCLUDE is set in snappy."));
	} else {
		cfg.define("SNAPPY_INCLUDE_DIR", env::var_os("DEP_SNAPPY_INCLUDE").expect("DEP_SNAPPY_INCLUDE is set in snappy."))
			.define("SNAPPY_LIBRARIES", "/dev/null"); // cmake requires defining this but we don't need it

		let src = env::current_dir().unwrap().join("lz4/lib");

		cfg.define("LZ4_INCLUDE_DIR", src)
			.define("LZ4_LIBRARIES", "/dev/null"); // cmake requires defining this but we don't need it
	}

	let out = cfg.build();

	let mut build = out.join("build");

	if cfg!(target_os = "windows") {
		let profile = match &*env::var("PROFILE").unwrap_or("debug".to_owned()) {
			"bench" | "release" => "Release",
			_ => "Debug",
		};
		build = build.join(profile);
	}

	lz4();

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
