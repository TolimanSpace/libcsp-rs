use std::ffi::CString;

pub fn to_owned_c_str_ptr(str: &str) -> *mut i8 {
    let c_str = CString::new(str).unwrap();
    let c_str = c_str.into_raw();
    c_str
}
