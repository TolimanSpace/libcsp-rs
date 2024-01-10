use libcsp_sys::CSP_NO_VIA_ADDRESS;

/// Represents a route for the CSP protocol network.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Route {
    pub address: u8,
    pub netmask: u8,
    pub via: u8,
}

impl Route {
    /// Creates a new Route instance with the specified address, a netmask of 255 (all bits),
    /// and a via address of CSP_NO_VIA_ADDRESS.
    ///
    /// # Arguments
    ///
    /// * `address` - The address value.
    ///
    /// # Returns
    ///
    /// A new Route instance.
    pub fn new(address: u8) -> Self {
        Self {
            address,
            netmask: 255, // Assume maximal bits
            via: CSP_NO_VIA_ADDRESS as u8,
        }
    }

    /// Sets the netmask number for the Route. The netmask is an arbitrary bit mask, with 0xFF being all bits.
    ///
    /// # Arguments
    ///
    /// * `netmask` - The netmask value.
    ///
    /// # Returns
    ///
    /// The modified Route instance.
    ///
    /// # Panics
    ///
    /// Panics if the address is not valid for the netmask.
    pub fn netmask(mut self, netmask: u8) -> Self {
        assert!(netmask <= 8);
        assert!(self.address & (!netmask) == 0);

        self.netmask = netmask;
        self
    }

    /// Sets the netmask bit number for the Route. The bit number is akin to CIDR notation, with 8 being all bits.
    ///
    /// # Arguments
    ///
    /// * `netmask_bits` - The netmask bit number.
    ///
    /// # Returns
    ///
    /// The modified Route instance.
    ///
    /// # Panics
    ///
    /// Panics if the netmask bit number is greater than 8 or if the address is not valid for the netmask.
    pub fn netmask_bits(mut self, netmask_bits: u8) -> Self {
        let num_bits = 8 - netmask_bits;
        let netmask = 0xFF << num_bits;

        assert!(netmask_bits <= 8);
        assert!(self.address & netmask == 0);

        self.netmask = netmask;
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
    ///
    /// # Panics
    ///
    /// Panics if the via address is equal to CSP_NO_VIA_ADDRESS.
    pub fn via(mut self, via: u8) -> Self {
        assert!(via != CSP_NO_VIA_ADDRESS as u8);

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
