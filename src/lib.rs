#[macro_use]
extern crate crt_macros;

#[crt_export]
struct TestStruct {
    member_int: i32,
}

#[crt_export]
impl TestStruct {
    fn do_thing(&self) {
        println!("DO_THING");
    }

    fn return_str(&self) -> String {
        return String::from("RETURN_STR");
    }
}
