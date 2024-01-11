use std::{ops::Deref, ptr::NonNull, time::Duration};

use libcsp_sys::{
    csp_accept, csp_buffer_free, csp_conn_dport, csp_conn_dst, csp_conn_sport, csp_conn_src,
    csp_conn_t, csp_packet_t, csp_read,
};

/// Represents a CSP socket.
///
/// This struct provides methods for accepting connections on the socket.
pub struct CspSocket {
    service_timeout_ms: u32,
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
    pub(crate) fn from_ptr(socket: *mut csp_conn_t, service_timeout_ms: u32) -> Self {
        Self {
            service_timeout_ms,
            socket,
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
        let conn = unsafe { csp_accept(self.socket, timeout.as_millis() as u32) };

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
            let conn = unsafe { csp_accept(self.socket, 1000) };

            if conn.is_null() {
                continue;
            } else {
                return CspConnection::new(conn, self.service_timeout_ms);
            }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CspConnAddress {
    pub address: u8,
    pub port: u8,
}

impl CspConnAddress {
    pub fn new(address: u8, port: u8) -> Self {
        Self { address, port }
    }

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
    service_timeout_ms: u32,
    connection: *mut csp_conn_t,
}

unsafe impl Send for CspConnection {}

impl CspConnection {
    /// Internal "new" function to create a `CspConnection` from a raw pointer to a CSP connection pointer.
    fn new(connection: *mut csp_conn_t, service_timeout_ms: u32) -> Self {
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
                service_timeout_ms,
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
            let packet = unsafe { csp_read(self.connection, self.service_timeout_ms) };
            if packet.is_null() {
                break;
            }

            unsafe { libcsp_sys::csp_service_handler(self.connection, packet) };
        }
    }

    pub fn iter_packets(self, timeout: Duration) -> CspConnectionPacketIter {
        CspConnectionPacketIter::new(self, timeout)
    }

    pub fn into_reader(self, timeout: Duration) -> CspConnectionPacketReader {
        CspConnectionPacketReader::new(self, timeout)
    }
}

impl Drop for CspConnection {
    fn drop(&mut self) {
        unsafe { libcsp_sys::csp_close(self.connection) };
    }
}

pub struct CspConnectionPacketIter {
    connection: CspConnection, // Keeping the connection alive (making sure it's not dropped)
    timeout_ms: u32,
}

impl CspConnectionPacketIter {
    pub fn new(connection: CspConnection, timeout: Duration) -> Self {
        Self {
            connection: connection,
            timeout_ms: timeout.as_millis() as u32,
        }
    }
}

impl Iterator for CspConnectionPacketIter {
    type Item = CspPacket;

    fn next(&mut self) -> Option<Self::Item> {
        let packet = unsafe { csp_read(self.connection.connection, self.timeout_ms) };
        let packet = NonNull::new(packet)?;
        Some(CspPacket { packet })
    }
}

pub struct CspPacket {
    packet: NonNull<csp_packet_t>,
}

unsafe impl Send for CspPacket {}

impl CspPacket {
    pub fn as_slice(&self) -> &[u8] {
        let data =
            unsafe { &(*self.packet.as_ptr()).__bindgen_anon_1.data as *const _ as *const u8 };
        let length = unsafe { (*self.packet.as_ptr()).length };
        unsafe { std::slice::from_raw_parts(data, length as usize) }
    }
}

impl Deref for CspPacket {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl Drop for CspPacket {
    fn drop(&mut self) {
        unsafe { csp_buffer_free(self.packet.as_ptr() as *mut std::os::raw::c_void) };
    }
}

pub struct CspConnectionPacketReader {
    connection: CspConnection, // Keeping the connection alive (making sure it's not dropped)
    timeout_ms: u32,
    packet: PacketReaderState,
    pos: usize,
}

enum PacketReaderState {
    NoPacket,
    Packet(NonNull<csp_packet_t>),
    Finished,
}

impl CspConnectionPacketReader {
    pub fn new(connection: CspConnection, timeout: Duration) -> Self {
        Self {
            connection,
            timeout_ms: timeout.as_millis() as u32,
            packet: PacketReaderState::NoPacket,
            pos: 0,
        }
    }
}

impl<'a> std::io::Read for CspConnectionPacketReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut read = 0;
        let mut remaining_buf = buf;

        loop {
            if remaining_buf.len() == 0 {
                return Ok(read);
            }

            let next_packet = match &self.packet {
                PacketReaderState::NoPacket => {
                    let packet = unsafe { csp_read(self.connection.connection, self.timeout_ms) };
                    let packet = NonNull::new(packet);

                    match packet {
                        Some(packet) => {
                            self.packet = PacketReaderState::Packet(packet);
                            packet
                        }
                        None => {
                            self.packet = PacketReaderState::Finished;
                            return Ok(read);
                        }
                    }
                }
                PacketReaderState::Packet(packet) => *packet,
                PacketReaderState::Finished => return Ok(read),
            };

            let slice = unsafe {
                let data = &(*next_packet.as_ptr()).__bindgen_anon_1.data as *const _ as *const u8;
                let length = (*next_packet.as_ptr()).length;
                std::slice::from_raw_parts(data, length as usize)
            };
            let remaining_packet = &slice[self.pos..];

            let to_read = std::cmp::min(remaining_buf.len(), remaining_packet.len());

            remaining_buf[..to_read].copy_from_slice(&remaining_packet[..to_read]);
            read += to_read;
            remaining_buf = &mut remaining_buf[to_read..];

            self.pos += to_read;

            if self.pos >= slice.len() {
                self.pos = 0;
                self.packet = PacketReaderState::NoPacket;
            }
        }
    }
}
