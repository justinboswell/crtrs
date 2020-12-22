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

    pub fn return_str(&self) -> String {
        return String::from("RETURN_STR");
    }
}
