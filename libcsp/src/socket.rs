use std::{ptr::NonNull, time::Duration};

use libcsp_sys::{
    csp_accept, csp_socket_close, csp_socket_t,
};

use crate::CspConnection;

/// Represents a CSP socket.
///
/// This struct provides methods for accepting connections on the socket.
pub struct CspSocket {
    service_timeout_ms: u32,
    socket: NonNull<csp_socket_t>,
}

impl CspSocket {
    /// Creates a `CspSocket` from a raw pointer to a CSP socket.
    ///
    /// # Arguments
    ///
    /// * `socket` - A raw pointer to a CSP socket.
    ///
    /// # Returns
    ///
    /// A `CspSocket` instance.
    pub(crate) fn from_ptr(socket: *mut csp_socket_t, service_timeout_ms: u32) -> Self {
        Self {
            service_timeout_ms,
            socket: NonNull::new(socket).expect("Socket pointer cannot be null"),
        }
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
        let conn = unsafe { csp_accept(self.socket.as_ptr(), timeout.as_millis() as u32) };

        if conn.is_null() {
            None
        } else {
            Some(CspConnection::new(conn, self.service_timeout_ms))
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
            let conn = unsafe { csp_accept(self.socket.as_ptr(), 1000) };

            if conn.is_null() {
                continue;
            } else {
                return CspConnection::new(conn, self.service_timeout_ms);
            }
        }
    }
}

impl Drop for CspSocket {
    fn drop(&mut self) {
        unsafe {
            csp_socket_close(self.socket.as_ptr());
            // The memory was allocated with Box::into_raw in lib.rs
            let _ = Box::from_raw(self.socket.as_ptr());
        }
    }
}

pub trait CspPortHandler {
    fn handle(&mut self, conn: CspConnection);
}

struct CspPortFn<'a, F: 'a + FnMut(CspConnection), Next: CspPortHandler> {
    port: u8,
    f: F,
    inner: Next,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl CspPortHandler for () {
    fn handle(&mut self, _conn: CspConnection) {}
}

impl<'a, F: 'a + FnMut(CspConnection), Next: CspPortHandler> CspPortHandler
    for CspPortFn<'a, F, Next>
{
    fn handle(&mut self, conn: CspConnection) {
        if conn.dst.port == self.port {
            let _iter = (self.f)(conn);
        } else {
            self.inner.handle(conn);
        }
    }
}

#[must_use = "CspSocketBuilder must be run to accept connections"]
pub struct CspSocketBuilder<'a, Handlers: CspPortHandler> {
    socket: CspSocket,
    handlers: Handlers,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl CspSocketBuilder<'static, ()> {
    pub fn new(socket: CspSocket) -> Self {
        Self {
            socket,
            handlers: (),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, Handlers: 'a + CspPortHandler> CspSocketBuilder<'a, Handlers> {
    pub fn bind_port<'b, F: 'b + FnMut(CspConnection)>(
        self,
        port: u8,
        f: F,
    ) -> CspSocketBuilder<'b, impl 'b + CspPortHandler>
    where
        'a: 'b,
    {
        CspSocketBuilder {
            socket: self.socket,
            handlers: CspPortFn {
                port,
                f,
                inner: self.handlers,
                _marker: std::marker::PhantomData,
            },
            _marker: std::marker::PhantomData,
        }
    }

    pub fn run_sync(mut self) -> ! {
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


