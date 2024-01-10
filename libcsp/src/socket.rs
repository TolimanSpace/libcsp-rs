use std::{ptr::NonNull, time::Duration};

use libcsp_sys::{
    csp_accept, csp_buffer_free, csp_conn_dport, csp_conn_dst, csp_conn_sport, csp_conn_src,
    csp_conn_t, csp_packet_t, csp_read,
};

// Timeout for receiving new packets in a single connection
const PACKET_TIMEOUT_MS: u32 = 100;

/// Represents a CSP socket.
///
/// This struct provides methods for accepting connections on the socket.
pub struct CspSocket {
    socket: *mut csp_conn_t,
}

impl CspSocket {
    /// Creates a `CspSocket` from a raw pointer to a CSP connection.
    ///
    /// # Arguments
    ///
    /// * `socket` - A raw pointer to a CSP connection.
    ///
    /// # Returns
    ///
    /// A `CspSocket` instance.
    pub(crate) fn from_ptr(socket: *mut csp_conn_t) -> Self {
        Self { socket }
    }

    /// Accepts a connection on the socket with a specified timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The maximum duration to wait for a connection.
    ///
    /// # Returns
    ///
    /// An `Option` containing a `CspConnection` if a connection is accepted within the timeout,
    /// or `None` if the timeout expires.
    pub fn accept_timeout(&self, timeout: Duration) -> Option<CspConnection> {
        let conn = unsafe { csp_accept(self.socket, timeout.as_millis() as u32) };

        if conn.is_null() {
            None
        } else {
            Some(CspConnection::new(conn))
        }
    }

    /// Repeatedly attempts to accept a connection on the socket with a default timeout of 1000 milliseconds.
    ///
    /// This method will loop until a connection is accepted.
    ///
    /// # Returns
    ///
    /// A `CspConnection` instance.
    pub fn accept(&self) -> CspConnection {
        // Loop until we get a connection
        loop {
            let conn = unsafe { csp_accept(self.socket, 1000) };

            if conn.is_null() {
                continue;
            } else {
                return CspConnection::new(conn);
            }
        }
    }
}

pub trait CspPortHandler {
    fn handle(&mut self, conn: CspConnection);
}

struct CspPortFn<F: FnMut(CspConnection), Next: CspPortHandler> {
    port: u8,
    f: F,
    inner: Next,
}

impl CspPortHandler for () {
    fn handle(&mut self, _conn: CspConnection) {}
}

impl<F: FnMut(CspConnection), Next: CspPortHandler> CspPortHandler for CspPortFn<F, Next> {
    fn handle(&mut self, conn: CspConnection) {
        if conn.dst.port == self.port {
            let _iter = (self.f)(conn);
        } else {
            self.inner.handle(conn);
        }
    }
}

#[must_use = "CspSocketBuilder must be run to accept connections"]
pub struct CspSocketBuilder<Handlers: CspPortHandler> {
    socket: CspSocket,
    handlers: Handlers,
}

impl CspSocketBuilder<()> {
    pub fn new(socket: CspSocket) -> Self {
        Self {
            socket,
            handlers: (),
        }
    }
}

impl<Handlers: CspPortHandler> CspSocketBuilder<Handlers> {
    pub fn bind_port(
        self,
        port: u8,
        f: impl FnMut(CspConnection) + 'static,
    ) -> CspSocketBuilder<impl CspPortHandler> {
        CspSocketBuilder {
            socket: self.socket,
            handlers: CspPortFn { port, f, inner: () },
        }
    }

    pub fn run(mut self) -> ! {
        loop {
            let conn = self.socket.accept();
            if conn.is_service_connection() {
                conn.handle_as_service_connection();
            } else {
                self.handlers.handle(conn);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CspConnAddress {
    address: u8,
    port: u8,
}

impl CspConnAddress {
    fn is_service_port(&self) -> bool {
        // This should optimize into a simple < comparison, but being verbose is nice for safety.
        let service_ports = [
            libcsp_sys::csp_service_port_t_CSP_CMP as u8,
            libcsp_sys::csp_service_port_t_CSP_PING as u8,
            libcsp_sys::csp_service_port_t_CSP_PS as u8,
            libcsp_sys::csp_service_port_t_CSP_MEMFREE as u8,
            libcsp_sys::csp_service_port_t_CSP_REBOOT as u8,
            libcsp_sys::csp_service_port_t_CSP_BUF_FREE as u8,
            libcsp_sys::csp_service_port_t_CSP_UPTIME as u8,
        ];

        service_ports.contains(&self.port)
    }
}

pub struct CspConnection {
    src: CspConnAddress,
    dst: CspConnAddress,
    connection: *mut csp_conn_t,
}

impl CspConnection {
    /// Internal "new" function to create a `CspConnection` from a raw pointer to a CSP connection pointer.
    fn new(connection: *mut csp_conn_t) -> Self {
        unsafe {
            Self {
                src: CspConnAddress {
                    address: csp_conn_src(connection) as u8,
                    port: csp_conn_sport(connection) as u8,
                },
                dst: CspConnAddress {
                    address: csp_conn_dst(connection) as u8,
                    port: csp_conn_dport(connection) as u8,
                },
                connection,
            }
        }
    }

    pub fn src(&self) -> CspConnAddress {
        self.src
    }

    pub fn dst(&self) -> CspConnAddress {
        self.dst
    }

    pub fn is_service_connection(&self) -> bool {
        self.dst.is_service_port()
    }

    pub fn handle_as_service_connection(self) {
        assert!(self.is_service_connection());

        loop {
            let packet = unsafe { csp_read(self.connection, PACKET_TIMEOUT_MS) };
            if packet.is_null() {
                break;
            }

            unsafe { libcsp_sys::csp_service_handler(self.connection, packet) };
        }
    }

    pub fn iter_packets(&self) -> CspConnectionPacketIter {
        CspConnectionPacketIter::new(self)
    }
}

impl Drop for CspConnection {
    fn drop(&mut self) {
        unsafe { libcsp_sys::csp_close(self.connection) };
    }
}

pub struct CspConnectionPacketIter<'a> {
    connection: *mut csp_conn_t,
    prev_packet: Option<NonNull<csp_packet_t>>,
    _marker: std::marker::PhantomData<&'a CspConnection>,
}

impl<'a> CspConnectionPacketIter<'a> {
    pub fn new(connection: &'a CspConnection) -> Self {
        Self {
            connection: connection.connection,
            prev_packet: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a> Iterator for CspConnectionPacketIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(prev_packet) = self.prev_packet.take() {
            unsafe { csp_buffer_free(prev_packet.as_ptr() as *mut std::os::raw::c_void) };
        }

        let packet = unsafe { csp_read(self.connection, PACKET_TIMEOUT_MS) };
        self.prev_packet = NonNull::new(packet);

        if packet.is_null() {
            None
        } else {
            let data = unsafe { &(*packet).__bindgen_anon_1.data as *const _ as *const u8 };
            let length = unsafe { (*packet).length };
            let slice = unsafe { std::slice::from_raw_parts(data, length as usize) };

            Some(slice)
        }
    }
}
