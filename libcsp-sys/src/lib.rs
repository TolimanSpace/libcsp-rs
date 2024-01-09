#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// Get default CSP configuration. Bindgen doesn't pick up this header function, so we define it manually in Rust.
pub unsafe fn csp_conf_get_defaults() -> csp_conf_t {
    csp_conf_t {
        address: 1,
        hostname: b"hostname\0" as *const u8 as *const i8,
        model: b"model\0" as *const u8 as *const i8,
        revision: b"resvision\0" as *const u8 as *const i8,
        conn_max: 10,
        conn_queue_length: 10,
        fifo_length: 25,
        port_max_bind: 24,
        rdp_max_window: 20,
        buffers: 10,
        buffer_data_size: 256,
        conn_dfl_so: CSP_O_NONE,
    }
}
