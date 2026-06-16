use std::env;
use std::path::PathBuf;

pub fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let libcsp_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("libcsp");

    let dst = cmake::Config::new(&libcsp_path)
        .define("CSP_POSIX", "1")
        .define("CSP_USE_RDP", "ON")
        .define("CSP_USE_HMAC", "ON")
        .define("CSP_USE_PROMISC", "ON")
        .define("CSP_USE_DEDUP", "ON")
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=csp");

    if cfg!(feature = "zmq") {
        let zmq = pkg_config::probe_library("libzmq").expect("libzmq not found via pkg-config");
        for path in &zmq.link_paths {
            println!("cargo:rustc-link-search=native={}", path.to_str().unwrap());
        }
        for lib in &zmq.libs {
            println!("cargo:rustc-link-lib=dylib={}", lib);
        }
    }

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed={}", libcsp_path.display());

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .use_core()
        .clang_arg(format!("-I{}/include", dst.display()))
        .clang_arg(format!("-I{}", libcsp_path.join("include").display()));

    // Add feature-based defines
    if cfg!(feature = "zmq") { builder = builder.clang_arg("-DCSP_RS_ZMQ"); }
    if cfg!(feature = "socketcan") { builder = builder.clang_arg("-DCSP_RS_SOCKETCAN"); }
    if cfg!(feature = "usart") { builder = builder.clang_arg("-DCSP_RS_USART"); }

    // We avoid manually adding C_INCLUDE_PATH to clang arguments because
    // it can cause conflicts with the internal headers we want to use.
    // Clang will already pick up C_INCLUDE_PATH from the environment.

    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_type("csp_.*")
        .allowlist_var("csp_.*")
        .allowlist_var("CSP_.*")
        .allowlist_function("csp_.*")
        .allowlist_type("nexthop_t")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}