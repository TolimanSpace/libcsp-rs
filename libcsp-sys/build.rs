use std::env;
use std::path::PathBuf;

pub fn main() {
    println!("cargo:rustc-link-lib=dylib={}", "csp");
    println!("cargo:rustc-link-lib=dylib={}", "zmq");

    // Get LIBCSP_DIR from environment variable, or use default
    // let lib_dir = env::var("LIBCSP_DIR").unwrap_or("/usr/local".to_string());

    // println!("cargo:rustc-link-search=native={lib_dir}/lib");

    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(&[
            #[cfg(feature = "zmq")]
            "-DCSP_RS_ZMQ",
            #[cfg(feature = "socketcan")]
            "-DCSP_RS_SOCKETCAN",
            #[cfg(feature = "usart")]
            "-DCSP_RS_USART",
        ])
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
