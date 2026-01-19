use std::time::Duration;

use libcsp_sys::{
    csp_connect, csp_ping, CSP_O_NONE,
};

use crate::{
    errors::csp_assert, CspConnAddress, CspConnPriority, CspError, CspErrorKind, LibCspConfig,
    CspConnection,
};

pub struct CspClient {}

impl CspClient {
    pub(crate) fn new(_conf: &LibCspConfig) -> Self {
        Self {}
    }

    pub fn ping(&self, address: u16) -> Result<u32, CspError> {
        self.ping_timeout_size(address, Duration::from_secs(1), 100)
    }

    pub fn ping_timeout_size(
        &self,
        address: u16,
        timeout: Duration,
        size: u32,
    ) -> Result<u32, CspError> {
        unsafe {
            let result = csp_ping(
                address,
                timeout.as_millis() as u32,
                size,
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
        address: CspConnAddress,
        priority: CspConnPriority,
        timeout: Duration,
    ) -> Result<CspConnection, CspError> {
        self.connect_opts(address, priority, timeout, CSP_O_NONE)
    }

    pub fn connect_opts(
        &self,
        address: CspConnAddress,
        priority: CspConnPriority,
        timeout: Duration,
        opts: u32,
    ) -> Result<CspConnection, CspError> {
        unsafe {
            let connection = csp_connect(
                priority as u8,
                address.address,
                address.port,
                timeout.as_millis() as u32,
                opts,
            );

            if connection.is_null() {
                return Err(CspError {
                    kind: CspErrorKind::Unknown(0),
                    message: "Failed to connect".to_string(),
                });
            }

            Ok(CspConnection::new(connection, 1000)) // Default service timeout
        }
    }
}
