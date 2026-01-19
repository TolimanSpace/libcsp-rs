use std::{sync::Mutex, thread, time::Duration};

use interface::InterfaceBuilder;
use libcsp_sys::*;
use once_cell::sync::Lazy;
use utils::to_owned_c_str_ptr;

pub mod interface;

mod id;
pub use id::*;

mod route;
pub use route::*;
mod connection;
pub use connection::*;

mod socket;
pub use socket::*;
mod port;
pub use port::*;
mod client;
pub use client::*;

mod errors;
use errors::csp_assert;
pub use errors::{CspError, CspErrorKind};

mod utils;

static GLOBAL_LIBCSP_INSTANCE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

pub struct LibCspBuilder<'a> {
    debug_channels: &'a [CspDebugChannel],
    config: LibCspConfig,
}

impl<'a> LibCspBuilder<'a> {
    pub fn new(config: LibCspConfig) -> Self {
        Self {
            debug_channels: CspDebugChannel::up_to_error(),
            config,
        }
    }

    /// Sets the debug channels for the global LibCSP instance.
    ///
    /// LibCSP treats each debug channel individually, for example
    /// if you set the `Info` channel, you will only see `Info` messages
    /// without `Error` and `Warn` messages.
    ///
    /// Use `CspDebugChannel::all()` to set all channels, or `CspDebugChannel::up_to(..)`
    /// to set all channels up to a certain level.
    pub fn debug_channels(mut self, channels: &'a [CspDebugChannel]) -> Self {
        self.debug_channels = channels;
        self
    }

    pub fn build(self) -> LibCspInstance {
        // This line can only be run once throughout the lifetime of the process.
        // The global instance lock is aquired within and never released.
        let guard_result = GLOBAL_LIBCSP_INSTANCE_LOCK.try_lock();
        let guard = match guard_result {
            Ok(guard) => guard,
            Err(_) => panic!("Only one LibCSP instance can be created per process"),
        };

        // Leak the guard, so it's never dropped.
        Box::leak(Box::new(guard));

        unsafe {
            // Initialize buffers
            csp_buffer_init();
        }

        unsafe {
            // Set the config for the global instance.
            let config = self.config.to_csp_conf_t();
            csp_conf = config;
            csp_init();
            
            // Add loopback route
            csp_rtable_set(self.config.address, -1, std::ptr::addr_of_mut!(csp_if_lo), CSP_NO_VIA_ADDRESS as u16);
        }

        unsafe {
            // Initialize the background router task
            // TODO: Which parameters are actually needed here?
            // csp_route_start_task(500, 0);

            thread::spawn(|| loop {
                csp_route_work();
            });
        }

        LibCspInstance::new(self.config)
    }
}

/// A global LibCSP instance. There can only be one per process,
/// due to the structure of the underlying C library.
pub struct LibCspInstance {
    config: LibCspConfig,
}

impl LibCspInstance {
    // Private new function, doesn't initialize the config. Other things are initialized by the builder.
    fn new(config: LibCspConfig) -> Self {
        Self { config }
    }

    /// Associates a route with an interface and adds it to the route table on the global LibCSP instance.
    pub fn add_interface_route(
        &self,
        route: Route,
        interface: impl InterfaceBuilder,
    ) -> Result<(), CspError> {
        let int = interface.build(self.config.address)?;
        unsafe {
            let result = csp_rtable_set(route.address, route.netmask, int, route.via);
            csp_assert!(result, "Failed to add interface");
        }

        Ok(())
    }

    pub fn open_server_socket(&self, port: CspPort) -> Result<CspSocket, CspError> {
        unsafe {
            // In LibCSP v2.0, we must provide the memory for the socket.
            let socket_ptr = Box::into_raw(Box::new(std::mem::zeroed::<csp_socket_t>()));
            
            csp_bind(socket_ptr, port.as_u8());
            csp_listen(socket_ptr, self.config.connection_backlog);

            Ok(CspSocket::from_ptr(
                socket_ptr,
                self.config.service_timeout.as_millis() as u32,
            ))
        }
    }

    pub fn server_sync_socket_builder(&self) -> Result<CspSocketBuilder<'_, ()>, CspError> {
        let socket = self.open_server_socket(CspPort::any_port())?;
        Ok(CspSocketBuilder::new(socket))
    }

    pub fn client(&self) -> CspClient {
        CspClient::new(&self.config)
    }

    pub fn print_conn_table(&self) {
        unsafe {
            csp_conn_print_table();
        }
    }

    pub fn print_iflist(&self) {
        unsafe {
            csp_iflist_print();
        }
    }

    pub fn print_rtable(&self) {
        unsafe {
            csp_rtable_print();
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CspDebugChannel {
    /// Error
    Error = 0,
    /// Warning
    Warn = 1,
    /// Informational
    Info = 2,
    /// Buffer, e.g. csp_packet get/free
    Buffer = 3,
    /// Packet routing
    Packet = 4,
    /// Protocol, i.e. RDP
    Protocol = 5,
    /// Locking, i.e. semaphore
    Lock = 6,
}

impl CspDebugChannel {
    pub fn all() -> &'static [Self; 7] {
        &[
            Self::Error,
            Self::Warn,
            Self::Info,
            Self::Buffer,
            Self::Packet,
            Self::Protocol,
            Self::Lock,
        ]
    }

    pub fn up_to(self) -> &'static [Self] {
        &Self::all()[..=self as usize]
    }

    pub fn up_to_error() -> &'static [Self] {
        Self::up_to(Self::Error)
    }

    pub fn up_to_warn() -> &'static [Self] {
        Self::up_to(Self::Warn)
    }

    pub fn up_to_info() -> &'static [Self] {
        Self::up_to(Self::Info)
    }
}

pub struct LibCspConfig {
    pub address: u16,
    pub hostname: String,
    pub model: String,
    pub revision: String,
    pub dedup: u8,
    pub conn_dfl_so: u32,
    pub connection_backlog: usize,

    /// Packet timeout on service messages that are handled internally
    pub service_timeout: Duration,
}

impl LibCspConfig {
    pub fn new(address: u16) -> Self {
        Self {
            address,
            ..Default::default()
        }
    }

    /// Refer to the LibCSP documentation
    pub fn details(
        self,
        hostname: impl Into<String>,
        model: impl Into<String>,
        revision: impl Into<String>,
    ) -> Self {
        Self {
            hostname: hostname.into(),
            model: model.into(),
            revision: revision.into(),
            ..self
        }
    }

    /// Refer to the LibCSP documentation
    pub fn dedup(self, dedup: u8) -> Self {
        Self { dedup, ..self }
    }

    /// Refer to the LibCSP documentation
    pub fn conn_dfl_so(self, conn_dfl_so: u32) -> Self {
        Self {
            conn_dfl_so,
            ..self
        }
    }

    fn to_csp_conf_t(&self) -> csp_conf_t {
        csp_conf_t {
            version: 2,
            address: self.address,
            hostname: to_owned_c_str_ptr(&self.hostname),
            model: to_owned_c_str_ptr(&self.model),
            revision: to_owned_c_str_ptr(&self.revision),
            conn_dfl_so: self.conn_dfl_so,
            dedup: self.dedup,
        }
    }
}

impl Default for LibCspConfig {
    fn default() -> Self {
        Self {
            address: 1,
            hostname: "{hostname unspecified}".to_string(),
            model: "{model unspecified}".to_string(),
            revision: "{resvision unspecified}".to_string(),
            dedup: 1,
            conn_dfl_so: CSP_O_NONE,
            connection_backlog: 64,
            service_timeout: Duration::from_millis(100),
        }
    }
}
