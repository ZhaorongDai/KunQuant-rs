use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let kunquant_dir = PathBuf::from(&manifest_dir).join("KunQuant");
    let cpp_dir = kunquant_dir.join("cpp");

    // Look for KunRuntime library in multiple possible locations
    let possible_lib_paths = vec![
        kunquant_dir.join("build"),
        kunquant_dir.join("build/lib"),
        // Check if installed via pip in virtual environment
        PathBuf::from(&manifest_dir)
            .join("kunquant-env/lib/python3.12/site-packages/KunQuant/runner"),
    ];

    for lib_path in &possible_lib_paths {
        if lib_path.exists() {
            println!("cargo:rustc-link-search=native={}", lib_path.display());
        }
    }

    // Tell cargo to tell rustc to link the KunRuntime library
    println!("cargo:rustc-link-lib=dylib=KunRuntime");

    // Tell cargo to invalidate the built crate whenever the C++ source changes
    println!("cargo:rerun-if-changed={}", cpp_dir.display());

    // Add include path for the C headers
    println!("cargo:include={}", cpp_dir.display());
}
