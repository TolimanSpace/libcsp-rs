#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern "C" {
    pub fn csp_socket(opts: u32) -> *mut csp_socket_t;
}

/// Get default CSP configuration. Bindgen doesn't pick up this header function, so we define it manually in Rust.
pub unsafe fn csp_conf_get_defaults() -> csp_conf_t {
    csp_conf_t {
        version: 2,
        address: 1,
        hostname: b"hostname\0" as *const u8 as *const i8,
        model: b"model\0" as *const u8 as *const i8,
        revision: b"revision\0" as *const u8 as *const i8,
        conn_dfl_so: CSP_O_NONE,
        dedup: 1,
    }
}
