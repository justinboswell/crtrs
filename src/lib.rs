#![feature(rustc_private)]
#[macro_use]
extern crate crt_macros;

#[crt_export]
pub struct TestStruct {
    member_int: i32,
}

#[crt_export]
impl TestStruct {
    pub fn do_thing(&self) {
        println!("DO_THING");
    }

    pub fn return_str(&self) -> *const u8 {
        return "RETURN_STR".as_ptr();
    }
}

#[allow(dead_code)]
extern "C" {
    pub fn aws_crt_init();
    pub fn aws_crt_clean_up();
}

pub struct CRT {

}

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_clean_up() {
        CRT::init();
        CRT::clean_up();
    }
}
