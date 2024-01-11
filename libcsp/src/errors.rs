#[repr(i32)]
#[derive(Debug)]
pub enum CspErrorKind {
    Nomem = -1,
    Inval = -2,
    Timedout = -3,
    Used = -4,
    Notsup = -5,
    Busy = -6,
    Already = -7,
    Reset = -8,
    Nobufs = -9,
    Tx = -10,
    Driver = -11,
    Again = -12,
    Hmac = -100,
    Xtea = -101,
    Crc32 = -102,
    Sfp = -103,
    Unknown(i32) = 1,
    NoBuffersAvailable = 2,
    FailedToSend = 3,
}

impl std::error::Error for CspErrorKind {}

impl std::fmt::Display for CspErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CspErrorKind::Nomem => write!(f, "No memory available"),
            CspErrorKind::Inval => write!(f, "Invalid argument"),
            CspErrorKind::Timedout => write!(f, "Timed out"),
            CspErrorKind::Used => write!(f, "Already in use"),
            CspErrorKind::Notsup => write!(f, "Not supported"),
            CspErrorKind::Busy => write!(f, "Busy"),
            CspErrorKind::Already => write!(f, "Already"),
            CspErrorKind::Reset => write!(f, "Reset"),
            CspErrorKind::Nobufs => write!(f, "No buffers available"),
            CspErrorKind::Tx => write!(f, "Transmit error"),
            CspErrorKind::Driver => write!(f, "Driver error"),
            CspErrorKind::Again => write!(f, "Again"),
            CspErrorKind::Hmac => write!(f, "HMAC error"),
            CspErrorKind::Xtea => write!(f, "XTEA error"),
            CspErrorKind::Crc32 => write!(f, "CRC32 error"),
            CspErrorKind::Sfp => write!(f, "SFP error"),
            CspErrorKind::Unknown(code) => write!(f, "Unknown error code: {}", code),
            CspErrorKind::NoBuffersAvailable => write!(f, "No buffers available"),
            CspErrorKind::FailedToSend => write!(f, "Failed to send packet"),
        }
    }
}

pub fn result_from_i32(err_code: i32) -> Result<(), CspErrorKind> {
    match err_code {
        0 => Ok(()),
        -1 => Err(CspErrorKind::Nomem),
        -2 => Err(CspErrorKind::Inval),
        -3 => Err(CspErrorKind::Timedout),
        -4 => Err(CspErrorKind::Used),
        -5 => Err(CspErrorKind::Notsup),
        -6 => Err(CspErrorKind::Busy),
        -7 => Err(CspErrorKind::Already),
        -8 => Err(CspErrorKind::Reset),
        -9 => Err(CspErrorKind::Nobufs),
        -10 => Err(CspErrorKind::Tx),
        -11 => Err(CspErrorKind::Driver),
        -12 => Err(CspErrorKind::Again),
        -100 => Err(CspErrorKind::Hmac),
        -101 => Err(CspErrorKind::Xtea),
        -102 => Err(CspErrorKind::Crc32),
        -103 => Err(CspErrorKind::Sfp),
        _ => Err(CspErrorKind::Unknown(err_code)),
    }
}

#[derive(Debug)]
pub struct CspError {
    pub kind: CspErrorKind,
    pub message: String,
}

impl std::fmt::Display for CspError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "CSP error: {} ({})", self.message, self.kind)
    }
}

impl Error for CspError {}

macro_rules! csp_assert {
    ($err_code:expr, $msg:expr) => {
        crate::errors::result_from_i32($err_code).map_err(|kind| crate::errors::CspError {
            kind,
            message: ToString::to_string($msg),
        })?;
    };
}
use std::error::Error;

pub(crate) use csp_assert;
