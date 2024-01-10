use std::time::Duration;

use libcsp_sys::{
    csp_buffer_get, csp_close, csp_conn_t, csp_connect, csp_packet_t, csp_ping,
    csp_prio_t_CSP_PRIO_CRITICAL, csp_prio_t_CSP_PRIO_HIGH, csp_prio_t_CSP_PRIO_LOW,
    csp_prio_t_CSP_PRIO_NORM, csp_send, CSP_O_NONE,
};

use crate::{errors::csp_assert, CspError, CspErrorKind, LibCspConfig};

pub struct CspClient {
    max_buffer_size: u16,
}

impl CspClient {
    pub(crate) fn new(conf: &LibCspConfig) -> Self {
        Self {
            max_buffer_size: conf.buffer_data_size,
        }
    }

    pub fn ping(&self, address: u8) -> Result<u32, CspError> {
        self.ping_timeout_size(address, Duration::from_secs(1), 100)
    }

    pub fn ping_timeout_size(
        &self,
        address: u8,
        timeout: Duration,
        size: u8,
    ) -> Result<u32, CspError> {
        unsafe {
            let result = csp_ping(
                address,
                timeout.as_millis() as u32,
                size as u32,
                CSP_O_NONE as u8,
            );

            if result < 0 {
                csp_assert!(result, "Ping failed");
                Ok(0)
            } else {
                Ok(result as u32)
            }
        }
    }

    pub fn connect(
        &self,
        address: u8,
        priority: CspConnPriority,
        port: u8,
        timeout: Duration,
    ) -> Result<CspClientConnection, CspError> {
        unsafe {
            let connection = csp_connect(
                priority as u8,
                address,
                port,
                timeout.as_millis() as u32,
                CSP_O_NONE,
            );

            if connection.is_null() {
                return Err(CspError {
                    kind: CspErrorKind::Unknown(0),
                    message: "Failed to connect".to_string(),
                });
            }

            Ok(CspClientConnection {
                max_buffer_size: self.max_buffer_size,
                connection,
            })
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum CspConnPriority {
    Low = csp_prio_t_CSP_PRIO_LOW as u8,
    Normal = csp_prio_t_CSP_PRIO_NORM as u8,
    High = csp_prio_t_CSP_PRIO_HIGH as u8,
    Critical = csp_prio_t_CSP_PRIO_CRITICAL as u8,
}

pub struct CspClientConnection {
    max_buffer_size: u16,
    connection: *mut csp_conn_t,
}

impl CspClientConnection {
    pub fn send_packet_with(
        &self,
        timeout: Duration,
        f: impl FnOnce(&mut [u8]) -> usize,
    ) -> Result<(), CspError> {
        unsafe {
            let packet = csp_buffer_get(self.max_buffer_size as usize) as *mut csp_packet_t;
            if packet.is_null() {
                return Err(CspError {
                    kind: CspErrorKind::NoBuffersAvailable,
                    message: "Failed to get CSP buffer, no buffers left in the buffer pool."
                        .to_string(),
                });
            }

            let data = &mut (*packet).__bindgen_anon_1.data as *mut _ as *mut u8;
            let slice = std::slice::from_raw_parts_mut(data, self.max_buffer_size as usize);
            let length = f(slice);
            assert!(
                length <= self.max_buffer_size as usize,
                "Returned length is too long"
            );
            (*packet).length = length as u16;

            let result = csp_send(self.connection, packet, timeout.as_millis() as u32);
            if result < 0 {
                csp_assert!(result, "Failed to send packet");
            }

            Ok(())
        }
    }

    pub fn send_packet(&self, timeout: Duration, data: &[u8]) -> Result<(), CspError> {
        assert!(
            data.len() <= self.max_buffer_size as usize,
            "Data is too long"
        );

        self.send_packet_with(timeout, |slice| {
            slice[..data.len()].copy_from_slice(data);
            data.len()
        })
    }
}

impl Drop for CspClientConnection {
    fn drop(&mut self) {
        unsafe {
            csp_close(self.connection);
        }
    }
}
