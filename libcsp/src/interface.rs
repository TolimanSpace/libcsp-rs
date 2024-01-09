use libcsp_sys::{csp_iface_t, csp_zmqhub_init};

use crate::utils::to_owned_c_str_ptr;

pub struct CspInterface {
    iface: *mut csp_iface_t,
}

impl CspInterface {
    pub fn new_zmq(address: u8, host: &str, flags: u32) -> Self {
        let mut return_interface = std::ptr::null_mut();
        let result = unsafe {
            csp_zmqhub_init(
                address,
                to_owned_c_str_ptr(host),
                flags,
                &mut return_interface,
            )
        };

        return Self {
            iface: return_interface,
        };
    }
}

impl Drop for CspInterface {
    fn drop(&mut self) {
        // Warn that the interface is being dropped while LibCSP doesn't support disposing of interfaces.
        // This is a memory leak.
        eprintln!("Dropping CspInterface while LibCSP doesn't support disposing of interfaces. This is a memory leak.");
    }
}
