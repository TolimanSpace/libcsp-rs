#[repr(i32)]
pub enum CspError {
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
}

fn result_from_i32(err_code: i32) -> Result<(), CspError> {
    match err_code {
        0 => Ok(()),
        -1 => Err(CspError::Nomem),
        -2 => Err(CspError::Inval),
        -3 => Err(CspError::Timedout),
        -4 => Err(CspError::Used),
        -5 => Err(CspError::Notsup),
        -6 => Err(CspError::Busy),
        -7 => Err(CspError::Already),
        -8 => Err(CspError::Reset),
        -9 => Err(CspError::Nobufs),
        -10 => Err(CspError::Tx),
        -11 => Err(CspError::Driver),
        -12 => Err(CspError::Again),
        -100 => Err(CspError::Hmac),
        -101 => Err(CspError::Xtea),
        -102 => Err(CspError::Crc32),
        -103 => Err(CspError::Sfp),
        _ => Err(CspError::Unknown(err_code)),
    }
}
