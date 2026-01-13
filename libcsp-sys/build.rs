use std::env;
use std::path::PathBuf;

pub fn main() {
    println!("cargo:rustc-link-lib=dylib={}", "csp");
    println!("cargo:rustc-link-lib=dylib={}", "zmq");

    // Get LIBCSP_DIR from environment variable, or use default
    // let lib_dir = env::var("LIBCSP_DIR").unwrap_or("/usr/local".to_string());

    // println!("cargo:rustc-link-search=native={lib_dir}/lib");

    println!("cargo:rerun-if-changed=wrapper.h");

    let libcsp = pkg_config::probe_library("libcsp").expect("Could not find libcsp via pkg-config");

    // Print paths for debugging if the build fails
    for path in &libcsp.include_paths {
        println!("cargo:warning=Found libcsp include path: {}", path.display());
    }

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        // This is important: tell bindgen to use the include paths from pkg-config
        .clang_args(
            libcsp.include_paths.iter().map(|path| format!("-I{}", path.to_string_lossy()))
        );

    // Add feature-based defines
    if cfg!(feature = "zmq") { builder = builder.clang_arg("-DCSP_RS_ZMQ"); }
    if cfg!(feature = "socketcan") { builder = builder.clang_arg("-DCSP_RS_SOCKETCAN"); }
    if cfg!(feature = "usart") { builder = builder.clang_arg("-DCSP_RS_USART"); }

    // Also include standard include paths from the system/nix environment
    if let Ok(c_include_path) = std::env::var("C_INCLUDE_PATH") {
        for path in std::env::split_paths(&c_include_path) {
            builder = builder.clang_arg(format!("-I{}", path.to_string_lossy()));
        }
    }

    let bindings = builder
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}