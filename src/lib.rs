use crt_macros::crt_export;

pub trait TestTrait {
    fn do_thing(&self);
    fn return_str(&self) -> String;
}

pub struct TestStruct {

}

impl TestStruct {
    fn do_thing(&self) {
        println!("DO_THING");
    }

    fn return_str(&self) -> String {
        return String::new("RETURN_STR");
    }
}
