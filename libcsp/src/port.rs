/// Represents a LibCSP protocol port.
///
/// Ports are identified by a unique number ranging from 0 to 254, with 255 representing "any port".
///
/// # Examples
///
/// Creating a port with a specific number:
///
/// ```
/// use libcsp::CspPort;
///
/// let port = CspPort::port(42);
/// ```
///
/// Creating a port that represents "any port":
///
/// ```
/// use libcsp::CspPort;
///
/// let port = CspPort::any_port();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CspPort {
    port: u8,
}

impl CspPort {
    /// Creates a new `CspPort` with the specified port number.
    ///
    /// # Arguments
    ///
    /// * `port` - The port number to assign to the `CspPort`.
    ///
    /// # Panics
    ///
    /// This function will panic if the `port` value is equal to 255, which is reserved for "any port".
    ///
    /// # Examples
    ///
    /// ```
    /// use libcsp::CspPort;
    /// let port = CspPort::port(10);
    /// assert_eq!(port.as_u8(), 10);
    /// ```
    pub fn port(port: u8) -> Self {
        assert!(port != 255);

        Self { port }
    }

    /// Creates a new `CspPort` for any port number.
    ///
    /// # Examples
    ///
    /// ```
    /// use libcsp::CspPort;
    /// let port = CspPort::any_port();
    /// assert_eq!(port.as_u8(), 255);
    /// ```
    pub fn any_port() -> Self {
        Self { port: 255 }
    }

    /// Returns the port number as a `u8`.
    ///
    /// # Examples
    ///
    /// ```
    /// use libcsp::CspPort;
    /// let port = CspPort::port(10);
    /// assert_eq!(port.as_u8(), 10);
    /// ```
    pub fn as_u8(&self) -> u8 {
        self.port
    }
}
