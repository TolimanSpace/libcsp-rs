use std::{ops::Deref, ptr::NonNull, time::Duration, io::Write};
use libcsp_sys::{
    csp_buffer_free, csp_conn_dport, csp_conn_dst, csp_conn_sport, csp_conn_src,
    csp_conn_t, csp_packet_t, csp_read, csp_send, csp_buffer_get, csp_buffer_data_size,
};

use crate::{CspConnAddress, CspError, CspErrorKind, CspId};

pub struct CspConnection {
    pub src: CspConnAddress,
    pub dst: CspConnAddress,
    service_timeout_ms: u32,
    max_buffer_size: u16,
    pub(crate) connection: *mut csp_conn_t,
}

unsafe impl Send for CspConnection {}

impl CspConnection {
    /// Internal "new" function to create a `CspConnection` from a raw pointer to a CSP connection pointer.
    pub(crate) fn new(connection: *mut csp_conn_t, service_timeout_ms: u32) -> Self {
        unsafe {
            Self {
                src: CspConnAddress {
                    address: csp_conn_src(connection) as u16,
                    port: csp_conn_sport(connection) as u8,
                },
                dst: CspConnAddress {
                    address: csp_conn_dst(connection) as u16,
                    port: csp_conn_dport(connection) as u8,
                },
                service_timeout_ms,
                max_buffer_size: csp_buffer_data_size() as u16,
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

            unsafe { libcsp_sys::csp_service_handler(packet) };
        }
    }

    pub fn iter_packets(self, timeout: Duration) -> CspConnectionPacketIter {
        CspConnectionPacketIter::new(self, timeout)
    }

    pub fn into_reader(self, timeout: Duration) -> CspConnectionPacketReader {
        CspConnectionPacketReader::new(self, timeout)
    }

    pub fn into_writer(self) -> CspConnectionWriter {
        CspConnectionWriter {
            connection: self,
            packet: None,
            pos: 0,
        }
    }

    pub fn send_packet(&self, data: &[u8]) -> Result<(), CspError> {
        if data.len() > self.max_buffer_size as usize {
            return Err(CspError {
                kind: CspErrorKind::Inval,
                message: format!(
                    "Data length {} exceeds maximum buffer size {}",
                    data.len(),
                    self.max_buffer_size
                ),
            });
        }

        self.send_packet_with(|slice| {
            slice[..data.len()].copy_from_slice(data);
            data.len()
        })
    }

    pub fn send_packet_with<F>(&self, f: F) -> Result<(), CspError>
    where
        F: FnOnce(&mut [u8]) -> usize,
    {
        unsafe {
            let packet = csp_buffer_get(self.max_buffer_size as usize) as *mut csp_packet_t;
            if packet.is_null() {
                return Err(CspError {
                    kind: CspErrorKind::Nomem,
                    message: "Failed to get CSP buffer".to_string(),
                });
            }

            let data_ptr = &mut (*packet).__bindgen_anon_2.data as *mut _ as *mut u8;
            let slice = std::slice::from_raw_parts_mut(data_ptr, self.max_buffer_size as usize);
            let length = f(slice);

            if length > self.max_buffer_size as usize {
                csp_buffer_free(packet as *mut _);
                return Err(CspError {
                    kind: CspErrorKind::Inval,
                    message: "Data length exceeds maximum buffer size".to_string(),
                });
            }

            (*packet).length = length as u16;

            csp_send(self.connection, packet);
            // In v2.0 csp_send returns void, but we might want to check for errors if possible.
            // Actually, in some v2.0 versions it returns void, in others it might return int.
            // Let's check bindings.rs again.
        }

        Ok(())
    }
}

impl Drop for CspConnection {
    fn drop(&mut self) {
        unsafe { libcsp_sys::csp_close(self.connection) };
    }
}

pub struct CspConnectionPacketIter {
    connection: CspConnection,
    timeout_ms: u32,
}

unsafe impl Send for CspConnectionPacketIter {}

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
    pub fn id(&self) -> CspId {
        unsafe { (*self.packet.as_ptr()).id.into() }
    }

    pub fn as_slice(&self) -> &[u8] {
        let data =
            unsafe { &(*self.packet.as_ptr()).__bindgen_anon_2.data as *const _ as *const u8 };
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
    connection: CspConnection,
    timeout_ms: u32,
    packet: PacketReaderState,
    pos: usize,
}

enum PacketReaderState {
    NoPacket,
    Packet(CspPacket),
    Finished,
}

unsafe impl Send for CspConnectionPacketReader {}

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
                            self.packet = PacketReaderState::Packet(CspPacket { packet });
                            packet
                        }
                        None => {
                            self.packet = PacketReaderState::Finished;
                            return Ok(read);
                        }
                    }
                }
                PacketReaderState::Packet(packet) => packet.packet,
                PacketReaderState::Finished => return Ok(read),
            };

            let slice = unsafe {
                let data = &(*next_packet.as_ptr()).__bindgen_anon_2.data as *const _ as *const u8;
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

pub struct CspConnectionWriter {
    connection: CspConnection,
    packet: Option<NonNull<csp_packet_t>>,
    pos: usize,
}

impl std::io::Write for CspConnectionWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut remaining_buf = buf;
        let mut written = 0;
        loop {
            if remaining_buf.len() == 0 {
                return Ok(written);
            }

            let packet = match self.packet {
                Some(packet) => packet.as_ptr(),
                None => {
                    let packet = unsafe {
                        let packet = csp_buffer_get(self.connection.max_buffer_size as usize)
                            as *mut csp_packet_t;
                        if packet.is_null() {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Failed to get CSP buffer, no buffers left in the buffer pool.",
                            ));
                        }
                        (*packet).length = self.connection.max_buffer_size;
                        NonNull::new_unchecked(packet)
                    };
                    self.packet = Some(packet);
                    packet.as_ptr()
                }
            };

            let slice = unsafe {
                let data = &mut (*packet).__bindgen_anon_2.data as *mut _ as *mut u8;
                let slice = std::slice::from_raw_parts_mut(data, (*packet).length as usize);
                slice
            };

            let remaining_slice = &mut slice[self.pos..];

            let to_write = std::cmp::min(remaining_slice.len(), remaining_buf.len());
            remaining_slice[..to_write].copy_from_slice(&remaining_buf[..to_write]);
            self.pos += to_write;

            remaining_buf = &remaining_buf[to_write..];
            written += to_write;

            if self.pos == slice.len() {
                self.flush()?;
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let Some(packet) = self.packet.take() else {
            return Ok(());
        };

        unsafe {
            (*packet.as_ptr()).length = self.pos as u16;
            csp_send(self.connection.connection, packet.as_ptr());
        }

        self.pos = 0;
        Ok(())
    }
}

impl Drop for CspConnectionWriter {
    fn drop(&mut self) {
        self.flush().ok();
    }
}
