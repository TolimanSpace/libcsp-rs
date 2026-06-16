use core::ptr;
use libcsp_sys::{csp_iface_t, csp_can_socketcan_open_and_add_interface};

#[cfg(feature = "zmq")]
use libcsp_sys::{
    csp_zmqhub_init, csp_zmqhub_init_w_endpoints,
    csp_zmqhub_init_w_name_endpoints_rxfilter,
};

use crate::csp_assert;
use crate::CspError;

#[cfg(feature = "alloc")]
use crate::utils::to_owned_c_str_ptr;

pub trait InterfaceBuilder {
    fn build(self, address: u16) -> Result<*mut csp_iface_t, CspError>;
}

#[cfg(feature = "zmq")]
pub enum CspZmqInterface<'a> {
    Basic {
        host: &'a str,
        zmq_flags: u32,
    },
    WithEndpoints {
        publish_endpoint: &'a str,
        subscribe_endpoint: &'a str,
        zmq_flags: u32,
    },
    WithNameEndpointsFilter {
        ifname: &'a str,
        addr: u16,
        rx_filter: &'a [u16],
        publish_endpoint: &'a str,
        subscribe_endpoint: &'a str,
        zmq_flags: u32,
    },
}

#[cfg(feature = "socketcan")]
pub struct CspCanInterface<'a> {
    pub device: &'a str,
    pub bitrate: u32,
    pub promisc: bool,
}

#[cfg(feature = "socketcan")]
impl<'a> CspCanInterface<'a> {
    pub fn new(device: &'a str, bitrate: u32, promisc: bool) -> Self {
        Self { device, bitrate, promisc }
    }
}

#[cfg(feature = "zmq")]
impl<'a> CspZmqInterface<'a> {
    pub fn new_basic(host: &'a str, zmq_flags: u32) -> Self {
        Self::Basic { host, zmq_flags }
    }
}

#[cfg(all(feature = "socketcan", feature = "alloc"))]
impl InterfaceBuilder for CspCanInterface<'_> {
    fn build(self, _address: u16) -> Result<*mut csp_iface_t, CspError> {
        let mut return_interface = ptr::null_mut();
        unsafe {
            let result = csp_can_socketcan_open_and_add_interface(
                to_owned_c_str_ptr(self.device),
                to_owned_c_str_ptr("CAN"),
                self.bitrate as i32,
                self.promisc,
                &mut return_interface,
            );
            if !return_interface.is_null() {
                (*return_interface).addr = _address;
                (*return_interface).netmask = 14; // Or appropriate netmask bits for CSP 2.0
            }
            csp_assert!(result, "Failed to initialize CAN interface");
        };

        Ok(return_interface)
    }
}

#[cfg(all(feature = "zmq", feature = "alloc"))]
impl InterfaceBuilder for CspZmqInterface<'_> {
    fn build(self, address: u16) -> Result<*mut csp_iface_t, CspError> {
        let mut return_interface = ptr::null_mut();
        unsafe {
            let result = match self {
                CspZmqInterface::Basic { host, zmq_flags } => csp_zmqhub_init(
                    address,
                    to_owned_c_str_ptr(host),
                    zmq_flags,
                    &mut return_interface,
                ),
                CspZmqInterface::WithEndpoints {
                    publish_endpoint,
                    subscribe_endpoint,
                    zmq_flags,
                } => csp_zmqhub_init_w_endpoints(
                    address,
                    to_owned_c_str_ptr(publish_endpoint),
                    to_owned_c_str_ptr(subscribe_endpoint),
                    zmq_flags,
                    &mut return_interface,
                ),
                CspZmqInterface::WithNameEndpointsFilter {
                    ifname,
                    addr,
                    rx_filter,
                    publish_endpoint,
                    subscribe_endpoint,
                    zmq_flags,
                } => csp_zmqhub_init_w_name_endpoints_rxfilter(
                    to_owned_c_str_ptr(ifname),
                    addr,
                    rx_filter.as_ptr(),
                    rx_filter.len() as u32,
                    to_owned_c_str_ptr(publish_endpoint),
                    to_owned_c_str_ptr(subscribe_endpoint),
                    zmq_flags,
                    &mut return_interface,
                ),
            };
            csp_assert!(result, "Failed to initialize ZMQ interface");
        };

        Ok(return_interface)
    }
}
