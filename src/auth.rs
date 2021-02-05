use std::ffi::c_void;
use std::os::raw::c_char;
use crate::CByteCursor;

#[crt_export]
pub struct AwsCredentialsOptions {
    access_key_id : *const c_char,
    secret_access_key: *const c_char,
    session_token: *const c_char,
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
                    options.access_key_id,
                    options.secret_access_key,
                    options.session_token,
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

#[crt_export]
impl Drop for AwsCredentials {
    fn drop(&mut self) {
        unsafe {
            aws_crt_credentials_release(self.aws_credentials);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn static_str(s: &str) -> &'static str {
        Box::leak(s.to_string().into_boxed_str())
    }

    #[test]
    fn aws_credentials_lifetime() {
        let access_key_id: &'static CStr = unsafe {CStr::from_ptr(static_str("ACCESS_KEY").as_ptr() as *const i8)};
        let secret_access_key: &'static CStr = unsafe {CStr::from_ptr(static_str("SECRET_ACCESS_KEY").as_ptr() as *const i8)};
        let session_token: &'static CStr = unsafe {CStr::from_ptr(static_str("SESSION_TOKEN").as_ptr() as *const i8)};
        let _creds = AwsCredentials::new(&AwsCredentialsOptions {
            access_key_id: access_key_id.as_ptr(),
            secret_access_key: secret_access_key.as_ptr(),
            session_token: session_token.as_ptr(),
            expiration_timepoint_seconds: 0,
        });
    }
}
