use std::env;
use std::path::PathBuf;

use bindgen::{MacroTypeVariation, RustEdition};

#[cfg(not(feature = "vendored"))]
fn main() {
    let include_dirs = probe_installed().unwrap_or_default();
    do_bindgen(include_dirs);
}

fn do_bindgen(include_dirs: Vec<PathBuf>) {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let bindgen_file = out_dir.join("bindgen.rs");

    let mut builder = bindgen::Builder::default();
    if let Some(dir) = include_dirs.first() {
        println!("cargo:rerun-if-changed={}", dir.display());
        builder = builder.clang_arg(format!("-I{}", dir.display()));
    }

    println!("cargo:rerun-if-changed=wrapper.h");
    let bindings = builder
        .use_core()
        .header("wrapper.h")
        .rust_edition(RustEdition::Edition2024)
        .default_macro_constant_type(MacroTypeVariation::Signed)
        .allowlist_file(".*[[:punct:]]mimalloc.*\\.h")
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(&bindgen_file)
        .expect("Couldn't write bindings!");
}

#[cfg(not(feature = "vendored"))]
fn probe_installed() -> Option<Vec<PathBuf>> {
    if let Ok(lib) = pkg_config::Config::new()
        .print_system_libs(false)
        .probe("mimalloc")
    {
        return Some(lib.include_paths);
    }

    if let Ok(lib) = vcpkg::Config::new()
        .emit_includes(true)
        .find_package("mimalloc")
    {
        return Some(lib.include_paths);
    }

    println!("cargo:rustc-link-lib=mimalloc");
    None
}

#[cfg(feature = "vendored")]
fn main() {
    use std::path::Path;

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = Path::new(&crate_dir);
    let mimalloc_src_dir = src_dir.join("mimalloc");
    let include_dir = mimalloc_src_dir.join("include");

    println!("cargo:rerun-if-changed={}", mimalloc_src_dir.display());
    let mut cmake_config = cmake::Config::new(mimalloc_src_dir);
    cmake_config
        .define("MI_BUILD_SHARED", "OFF")
        .define("MI_BUILD_STATIC", "ON")
        .define("MI_BUILD_TESTS", "OFF")
        .define("MI_OPT_ARCH", "ON")
        .define("MI_OPT_SIMD", "ON")
        .define("MI_INSTALL_TOPLEVEL", "ON");
    #[cfg(target_env = "musl")]
    cmake_config.define("MI_LIBC_MUSL", "ON");
    #[cfg(feature = "secure")]
    cmake_config.define("MI_SECURE", "ON");
    #[cfg(target_env = "msvc")]
    select_msvc_crt(&mut cmake_config);

    let mimalloc_install_root = cmake_config.build();
    let lib_search_dir = Path::new(&mimalloc_install_root).join("lib");
    // set link options
    println!(
        "cargo:rustc-link-search=native={}",
        lib_search_dir.display()
    );
    let mut libname = String::from("mimalloc");
    #[cfg(feature = "secure")]
    libname.push_str("-secure");
    if cmake_config.get_profile() == "Debug" {
        libname.push_str("-debug");
    }
    println!("cargo:rustc-link-lib=static={libname}");

    do_bindgen(vec![include_dir]);
}

#[cfg(all(feature = "vendored", target_env = "msvc"))]
fn select_msvc_crt(cmake_config: &mut cmake::Config) {
    let linkage = env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default();
    if linkage.contains("crt-static") {
        cmake_config.define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreaded");
    } else {
        cmake_config.define("CMAKE_MSVC_RUNTIME_LIBRARY", "MultiThreadedDLL");
    }
}
