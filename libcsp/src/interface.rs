#[cfg(feature = "zmq")]
use core::ptr;
use libcsp_sys::csp_iface_t;

#[cfg(feature = "zmq")]
use libcsp_sys::{
    csp_zmqhub_init, csp_zmqhub_init_w_endpoints,
    csp_zmqhub_init_w_name_endpoints_rxfilter,
};

#[cfg(feature = "socketcan")]
use libcsp_sys::{
    csp_can_add_interface, csp_can_interface_data_t, csp_can_rx, CSP_ERR_INVAL, CSP_ERR_NONE,
    CSP_ERR_TX,
};
#[cfg(feature = "socketcan")]
use socketcan::{CanFrame, CanSocket, EmbeddedFrame, ExtendedId, Id, Socket, StandardId, SocketOptions};
#[cfg(feature = "socketcan")]
use std::thread;

#[cfg(any(feature = "zmq", feature = "socketcan"))]
use crate::csp_assert;
use crate::CspError;

#[cfg(all(feature = "alloc", feature = "socketcan"))]
use alloc::{boxed::Box, format};

#[cfg(all(any(feature = "zmq", feature = "socketcan"), feature = "alloc"))]
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

#[cfg(feature = "zmq")]
impl<'a> CspZmqInterface<'a> {
    pub fn new_basic(host: &'a str, zmq_flags: u32) -> Self {
        Self::Basic { host, zmq_flags }
    }
}

#[cfg(feature = "socketcan")]
pub struct CspCanInterface<'a> {
    pub ifname: &'a str,
    pub bitrate: u32,
    pub promisc: bool,
}

#[cfg(feature = "socketcan")]
impl<'a> CspCanInterface<'a> {
    pub fn new(ifname: &'a str, bitrate: u32, promisc: bool) -> Self {
        Self {
            ifname,
            bitrate,
            promisc,
        }
    }
}

#[cfg(feature = "socketcan")]
struct CspCanContext {
    socket: CanSocket,
    iface: *mut csp_iface_t,
}

#[cfg(all(feature = "socketcan", feature = "std", feature = "alloc"))]
impl InterfaceBuilder for CspCanInterface<'_> {
    fn build(self, _address: u16) -> Result<*mut csp_iface_t, CspError> {
        let socket = CanSocket::open(self.ifname).map_err(|e| CspError {
            kind: crate::errors::CspErrorKind::Driver,
            message: format!("Failed to open CAN socket: {}", e),
        })?;

        socket.set_recv_own_msgs(true).map_err(|e| CspError {
            kind: crate::errors::CspErrorKind::Driver,
            message: format!("Failed to set RECV_OWN_MSGS: {}", e),
        })?;

        let context = Box::into_raw(Box::new(CspCanContext {
            socket,
            iface: ptr::null_mut(),
        }));

        let iface_data = Box::into_raw(Box::new(csp_can_interface_data_t {
            cfp_packet_counter: 0,
            tx_func: Some(rust_csp_can_tx_frame),
        }));

        let iface = Box::into_raw(Box::new(unsafe { core::mem::zeroed::<csp_iface_t>() }));

        unsafe {
            (*iface).name = to_owned_c_str_ptr(self.ifname);
            (*iface).addr = _address;
            (*iface).interface_data = iface_data as *mut _;
            (*iface).driver_data = context as *mut _;

            (*context).iface = iface;
        }

        unsafe {
            csp_assert!(csp_can_add_interface(iface), "Failed to add CAN interface");
        }

        let context_ptr = context as usize;
        thread::spawn(move || {
            let context = unsafe { &*(context_ptr as *const CspCanContext) };
            loop {
                match context.socket.read_frame() {
                    Ok(frame) => {
                        if let CanFrame::Data(f) = frame {
                            let id = match f.id() {
                                Id::Standard(id) => id.as_raw() as u32,
                                Id::Extended(id) => id.as_raw() as u32 | 0x80000000,
                            };
                            let data = f.data();
                            unsafe {
                                csp_can_rx(
                                    context.iface,
                                    id,
                                    data.as_ptr(),
                                    data.len() as u8,
                                    ptr::null_mut(),
                                );
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        });

        Ok(iface)
    }
}

#[cfg(feature = "socketcan")]
unsafe extern "C" fn rust_csp_can_tx_frame(
    driver_data: *mut core::ffi::c_void,
    id: u32,
    data: *const u8,
    dlc: u8,
) -> i32 {
    let context = &*(driver_data as *const CspCanContext);

    let can_id = if id & 0x80000000 != 0 {
        match ExtendedId::new(id & 0x1FFFFFFF) {
            Some(id) => Id::Extended(id),
            None => return CSP_ERR_INVAL,
        }
    } else {
        match StandardId::new(id as u16) {
            Some(id) => Id::Standard(id),
            None => return CSP_ERR_INVAL,
        }
    };

    let data_slice = core::slice::from_raw_parts(data, dlc as usize);
    let frame = match CanFrame::new(can_id, data_slice) {
        Some(f) => f,
        None => return CSP_ERR_INVAL,
    };

    match context.socket.write_frame(&frame) {
        Ok(_) => CSP_ERR_NONE as i32,
        Err(_) => CSP_ERR_TX,
    }
}

#[cfg(all(feature = "zmq", feature = "alloc"))]
impl InterfaceBuilder for CspZmqInterface<'_> {
    fn build(self, address: u16) -> Result<*mut csp_iface_t, CspError> {
        let mut return_interface = ptr::null_mut();
        let result = unsafe {
            match self {
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
            }
        };

        csp_assert!(result, "Failed to initialize ZMQ interface");

        Ok(return_interface)
    }
}
