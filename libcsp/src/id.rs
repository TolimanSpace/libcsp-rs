use libcsp_sys::{
    csp_id_t, csp_prio_t_CSP_PRIO_CRITICAL, csp_prio_t_CSP_PRIO_HIGH, csp_prio_t_CSP_PRIO_LOW,
    csp_prio_t_CSP_PRIO_NORM,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum CspConnPriority {
    Low = csp_prio_t_CSP_PRIO_LOW as u8,
    Normal = csp_prio_t_CSP_PRIO_NORM as u8,
    High = csp_prio_t_CSP_PRIO_HIGH as u8,
    Critical = csp_prio_t_CSP_PRIO_CRITICAL as u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CspConnAddress {
    pub address: u16,
    pub port: u8,
}

impl CspConnAddress {
    pub fn new(address: u16, port: u8) -> Self {
        Self { address, port }
    }

    pub fn is_service_port(&self) -> bool {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CspId {
    pub priority: CspConnPriority,
    pub flags: u8,
    pub src: u16,
    pub dst: u16,
    pub dport: u8,
    pub sport: u8,
}

impl From<csp_id_t> for CspId {
    fn from(id: csp_id_t) -> Self {
        #[allow(non_upper_case_globals)]
        let priority = match id.pri as u32 {
            csp_prio_t_CSP_PRIO_LOW => CspConnPriority::Low,
            csp_prio_t_CSP_PRIO_NORM => CspConnPriority::Normal,
            csp_prio_t_CSP_PRIO_HIGH => CspConnPriority::High,
            csp_prio_t_CSP_PRIO_CRITICAL => CspConnPriority::Critical,
            _ => CspConnPriority::Normal,
        };
        Self {
            priority,
            flags: id.flags,
            src: id.src,
            dst: id.dst,
            dport: id.dport,
            sport: id.sport,
        }
    }
}

impl From<CspId> for csp_id_t {
    fn from(id: CspId) -> Self {
        Self {
            pri: id.priority as u8,
            flags: id.flags,
            src: id.src,
            dst: id.dst,
            dport: id.dport,
            sport: id.sport,
        }
    }
}
