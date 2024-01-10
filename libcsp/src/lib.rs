use std::sync::Mutex;

use libcsp_sys::*;
use once_cell::sync::Lazy;
use utils::to_owned_c_str_ptr;

mod interface;
pub use interface::*;
mod route;
pub use route::*;
mod socket;
pub use socket::*;
mod port;
pub use port::*;

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
            // Set the debug channels before initializing the config
            for channel in self.debug_channels {
                csp_debug_set_level(*channel as u32, true);
            }
        }

        unsafe {
            // Set the config for the global instance.
            let config = self.config.to_csp_conf_t();
            csp_init(&config);
        }

        unsafe {
            // Initialize the background router task
            // TODO: Which parameters are actually needed here?
            csp_route_start_task(500, 0);
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
        address: Route,
        interface: impl InterfaceBuilder,
    ) -> Result<(), CspError> {
        let int = interface.build(self.config.address)?;
        unsafe {
            let result = csp_rtable_set(address.address, address.netmask, int, address.via);
            csp_assert!(result, "Failed to add interface");
        }

        Ok(())
    }

    pub fn open_server_socket(&self, port: CspPort) -> Result<CspSocket, CspError> {
        unsafe {
            let socket_ptr = csp_socket(CSP_SO_NONE);
            csp_bind(socket_ptr, port.as_u8());
            csp_listen(socket_ptr, self.config.connection_backlog);

            Ok(CspSocket::from_ptr(socket_ptr))
        }
    }

    pub fn server_socket_builder(&self) -> Result<CspSocketBuilder<()>, CspError> {
        let socket = self.open_server_socket(CspPort::any_port())?;
        Ok(CspSocketBuilder::new(socket))
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
    pub address: u8,
    pub hostname: String,
    pub model: String,
    pub revision: String,
    pub conn_max: u8,
    pub conn_queue_length: u8,
    pub fifo_length: u8,
    pub port_max_bind: u8,
    pub rdp_max_window: u8,
    pub buffers: u16,
    pub buffer_data_size: u16,
    pub conn_dfl_so: u32,
    pub connection_backlog: usize,
}

impl LibCspConfig {
    pub fn new(address: u8) -> Self {
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
    pub fn conn_max(self, conn_max: u8) -> Self {
        Self { conn_max, ..self }
    }

    /// Refer to the LibCSP documentation
    pub fn conn_queue_length(self, conn_queue_length: u8) -> Self {
        Self {
            conn_queue_length,
            ..self
        }
    }

    /// Refer to the LibCSP documentation
    pub fn fifo_length(self, fifo_length: u8) -> Self {
        Self {
            fifo_length,
            ..self
        }
    }

    /// Refer to the LibCSP documentation
    pub fn port_max_bind(self, port_max_bind: u8) -> Self {
        Self {
            port_max_bind,
            ..self
        }
    }

    /// Refer to the LibCSP documentation
    pub fn rdp_max_window(self, rdp_max_window: u8) -> Self {
        Self {
            rdp_max_window,
            ..self
        }
    }

    /// Refer to the LibCSP documentation
    pub fn buffers(self, buffers: u16) -> Self {
        Self { buffers, ..self }
    }

    /// Refer to the LibCSP documentation
    pub fn buffer_data_size(self, buffer_data_size: u16) -> Self {
        Self {
            buffer_data_size,
            ..self
        }
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
            address: self.address,
            hostname: to_owned_c_str_ptr(&self.hostname),
            model: to_owned_c_str_ptr(&self.model),
            revision: to_owned_c_str_ptr(&self.revision),
            conn_max: self.conn_max,
            conn_queue_length: self.conn_queue_length,
            fifo_length: self.fifo_length,
            port_max_bind: self.port_max_bind,
            rdp_max_window: self.rdp_max_window,
            buffers: self.buffers,
            buffer_data_size: self.buffer_data_size,
            conn_dfl_so: self.conn_dfl_so,
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
            conn_max: 10,
            conn_queue_length: 10,
            fifo_length: 25,
            port_max_bind: 24,
            rdp_max_window: 20,
            buffers: 10,
            buffer_data_size: 256,
            conn_dfl_so: CSP_O_NONE,
            connection_backlog: 10,
        }
    }
}
