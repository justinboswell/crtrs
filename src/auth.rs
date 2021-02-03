use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use crate::CByteCursor;

#[crt_export]
pub struct AwsCredentialsOptions {
    access_key_id : &'static CStr,
    secret_access_key: &'static CStr,
    session_token: &'static CStr,
    expiration_timepoint_seconds: u64,
}

#[crt_export]
pub struct AwsCredentials {
    aws_credentials: *const c_void,
}

#[crt_export]
impl AwsCredentials {
    fn new(options: &AwsCredentialsOptions) -> AwsCredentials {
        AwsCredentials {
            aws_credentials: unsafe {
                aws_crt_credentials_new(
                    options.access_key_id.as_ptr(),
                    options.secret_access_key.as_ptr(),
                    options.session_token.as_ptr(),
                    options.expiration_timepoint_seconds
                )
            }
        }
    }

    fn get_access_key_id(&self) -> CByteCursor {
        unsafe {
            aws_crt_credentials_get_access_key_id(self.aws_credentials)
        }
    }

    fn get_secret_access_key(&self) -> CByteCursor {
        unsafe {
            aws_crt_credentials_get_secret_access_key(self.aws_credentials)
        }
    }

    fn get_session_token(&self) -> CByteCursor {
        unsafe {
            aws_crt_credentials_get_session_token(self.aws_credentials)
        }
    }

    fn get_expiration_timepoint_seconds(&self) -> CByteCursor {
        unsafe {
            aws_crt_credentials_get_expiration_timepoint_seconds(self.aws_credentials)
        }
    }
}

#[allow(dead_code)]
extern "C" {
    pub fn aws_crt_credentials_new(
        access_key_id: *const c_char,
        secret_access_key: *const c_char,
        session_token: *const c_char,
        expiration_timepoint_seconds: u64) -> *const c_void;
    pub fn aws_crt_credentials_release(creds: *const c_void);

    pub fn aws_crt_credentials_get_access_key_id(creds: *const c_void) -> CByteCursor;
    pub fn aws_crt_credentials_get_secret_access_key(creds: *const c_void) -> CByteCursor;
    pub fn aws_crt_credentials_get_session_token(creds: *const c_void) -> CByteCursor;
    pub fn aws_crt_credentials_get_expiration_timepoint_seconds(creds: *const c_void) -> CByteCursor;
}
