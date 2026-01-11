use std::env;
use std::path::PathBuf;

pub fn main() {
    let libcsp = pkg_config::probe_library("libcsp").expect("libcsp not found via pkg-config");
    let zmq = pkg_config::probe_library("libzmq").expect("libzmq not found via pkg-config");

    for path in libcsp.link_paths {
        println!("cargo:rustc-link-search=native={}", path.to_str().unwrap());
    }
    for lib in libcsp.libs {
        println!("cargo:rustc-link-lib=dylib={}", lib);
    }

    for path in zmq.link_paths {
        println!("cargo:rustc-link-search=native={}", path.to_str().unwrap());
    }
    for lib in zmq.libs {
        println!("cargo:rustc-link-lib=dylib={}", lib);
    }

    println!("cargo:rerun-if-changed=wrapper.h");

    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(&[
            #[cfg(feature = "zmq")]
            "-DCSP_RS_ZMQ",
            #[cfg(feature = "socketcan")]
            "-DCSP_RS_SOCKETCAN",
            #[cfg(feature = "usart")]
            "-DCSP_RS_USART",
        ])
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()));

    for path in libcsp.include_paths {
        builder = builder.clang_arg(format!("-I{}", path.to_str().unwrap()));
    }
    for path in zmq.include_paths {
        builder = builder.clang_arg(format!("-I{}", path.to_str().unwrap()));
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
