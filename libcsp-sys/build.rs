use std::env;
use std::path::PathBuf;

pub fn main() {
    println!("cargo:rustc-link-lib=static={}", "csp");
    println!("cargo:rustc-link-lib=dylib={}", "zmq");
    println!(
        "cargo:rustc-link-search=native={}",
        "/home/arduano/programming/spiralblue/libcsp-rs/libcsp-sys/libcsp/builddir"
    );

    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_args(&["-I/home/arduano/programming/spiralblue/libcsp-rs/libcsp-sys/libcsp/include"])
        .clang_args(&["-I/home/arduano/programming/spiralblue/libcsp-rs/libcsp-sys/libcsp/builddir/include"])
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
