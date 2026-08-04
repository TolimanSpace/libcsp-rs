#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern "C" {
    pub fn csp_socket(opts: u32) -> *mut csp_socket_t;
}

/// Get default CSP configuration. Bindgen doesn't pick up this header function, so we define it manually in Rust.
pub unsafe fn csp_conf_get_defaults() -> csp_conf_t {
    let mut conf: csp_conf_t = core::mem::zeroed();
    conf.version = 2;
    conf.address = 1;
    conf.hostname = b"hostname\0".as_ptr() as *const i8;
    conf.model = b"model\0".as_ptr() as *const i8;
    conf.revision = b"revision\0".as_ptr() as *const i8;
    conf.conn_dfl_so = CSP_O_NONE;
    conf.dedup = 1;
    conf
}
