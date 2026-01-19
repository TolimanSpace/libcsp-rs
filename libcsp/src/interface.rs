use libcsp_sys::{
    csp_iface_t, csp_zmqhub_init, csp_zmqhub_init_w_endpoints,
    csp_zmqhub_init_w_name_endpoints_rxfilter,
};

use crate::{csp_assert, utils::to_owned_c_str_ptr, CspError};

pub trait InterfaceBuilder {
    fn build(self, address: u16) -> Result<*mut csp_iface_t, CspError>;
}

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

impl<'a> CspZmqInterface<'a> {
    pub fn new_basic(host: &'a str, zmq_flags: u32) -> Self {
        Self::Basic { host, zmq_flags }
    }
}

impl InterfaceBuilder for CspZmqInterface<'_> {
    fn build(self, address: u16) -> Result<*mut csp_iface_t, CspError> {
        let mut return_interface = std::ptr::null_mut();
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
