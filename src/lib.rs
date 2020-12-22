#[macro_use]
extern crate crt_macros;

#[crt_export]
pub struct TestStruct;

#[crt_export]
impl TestStruct {
    fn do_thing(&self) {
        println!("DO_THING");
    }

    fn return_str(&self) -> String {
        return String::from("RETURN_STR");
    }
}
