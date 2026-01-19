use libcsp_sys::CSP_NO_VIA_ADDRESS;

/// Represents a route for the CSP protocol network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Route {
    pub address: u16,
    pub netmask: i32,
    pub via: u16,
}

impl Route {
    /// Creates a new Route instance with the specified address, a netmask of -1 (all bits),
    /// and a via address of CSP_NO_VIA_ADDRESS.
    ///
    /// # Arguments
    ///
    /// * `address` - The address value.
    ///
    /// # Returns
    ///
    /// A new Route instance.
    pub fn new(address: u16) -> Self {
        Self {
            address,
            netmask: -1, // Assume maximal bits
            via: CSP_NO_VIA_ADDRESS as u16,
        }
    }

    /// Sets the netmask number for the Route.
    ///
    /// # Arguments
    ///
    /// * `netmask` - The netmask value.
    ///
    /// # Returns
    ///
    /// The modified Route instance.
    pub fn netmask(mut self, netmask: i32) -> Self {
        self.netmask = netmask;
        self
    }

    /// Sets the netmask bit number for the Route.
    ///
    /// # Arguments
    ///
    /// * `netmask_bits` - The netmask bit number.
    ///
    /// # Returns
    ///
    /// The modified Route instance.
    pub fn netmask_bits(mut self, netmask_bits: u8) -> Self {
        self.netmask = netmask_bits as i32;
        self
    }

    /// Sets the via address for the Route.
    ///
    /// # Arguments
    ///
    /// * `via` - The via address value.
    ///
    /// # Returns
    ///
    /// The modified Route instance.
    pub fn via(mut self, via: u16) -> Self {
        self.via = via;
        self
    }

    /// Returns a default Route instance with address 0 and netmask 0.
    ///
    /// # Returns
    ///
    /// A default Route instance.
    pub fn default_address() -> Self {
        Self::new(0).netmask(0)
    }
}
