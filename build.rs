use std::env;
use std::path::PathBuf;

use bindgen::{MacroTypeVariation, RustEdition};

fn main() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let bindgen_file = out_dir.join("bindgen.rs");
    let include_dirs = probe_installed().unwrap_or_default();

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

fn probe_installed() -> Option<Vec<PathBuf>> {
    if let Ok(lib) = vcpkg::Config::new()
        .emit_includes(true)
        .find_package("mimalloc")
    {
        return Some(lib.include_paths);
    }

    None
}
