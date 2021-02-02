#![feature(rustc_private)]

mod io;

#[macro_use]
extern crate crt_macros;

use std::ffi::CStr;
use std::os::raw::c_char;

#[allow(dead_code)]
extern "C" {
    pub fn aws_crt_init();
    pub fn aws_crt_clean_up();

    pub fn aws_crt_error_str(error_code: i32) ->  *const c_char;
    pub fn aws_crt_error_name(error_code: i32) -> *const c_char;
    pub fn aws_crt_error_debug_str(error_code: i32) -> *const c_char;
}

#[crt_export]
pub struct CRT {}

#[allow(dead_code)]
#[crt_export]
impl CRT {
    pub fn init() {
        unsafe {
            aws_crt_init();
        }
    }

    pub fn clean_up() {
        unsafe {
            aws_crt_clean_up();
        }
    }

    pub fn error_str(error_code: i32) -> &'static CStr {
        unsafe {
            CStr::from_ptr(aws_crt_error_str(error_code))
        }
    }

    pub fn error_name(error_code: i32) -> &'static CStr {
        unsafe {
            CStr::from_ptr(aws_crt_error_name(error_code))
        }
    }

    pub fn error_debug_str(error_code: i32) -> &'static CStr {
        unsafe {
            CStr::from_ptr(aws_crt_error_debug_str(error_code))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_clean_up() {
        CRT::init();
        CRT::clean_up();
    }

    #[test]
    fn error_lookup() {
        CRT::init();
        assert_eq!("Success.", CRT::error_str(0).to_string_lossy());
        assert_eq!("AWS_ERROR_SUCCESS", CRT::error_name(0).to_string_lossy());
        assert_eq!("aws-c-common: AWS_ERROR_SUCCESS, Success.", CRT::error_debug_str(0).to_string_lossy());
        CRT::clean_up();
    }
}
