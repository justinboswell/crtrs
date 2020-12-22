# crtrs

## Example:
`$ cargo build` will result in the following code being generated from src/lib.rs (dumped via `cargo expand`):

```rust
#![feature(prelude_import)]
#![feature(rustc_private)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
#[macro_use]
extern crate crt_macros;
extern crate libc;
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn TestStruct_new() -> *mut TestStruct {
    unsafe { std::mem::zeroed() }
}
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn TestStruct_destroy(s: *mut TestStruct) {
    std::mem::drop(s);
}
#[repr(C)]
pub struct TestStruct {
    member_int: i32,
}
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn TestStruct_do_thing() {}
#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn TestStruct_return_str() -> String {}
impl TestStruct {
    pub fn do_thing(&self) {
        {
            ::std::io::_print(::core::fmt::Arguments::new_v1(
                &["DO_THING\n"],
                &match () {
                    () => [],
                },
            ));
        };
    }
    pub fn return_str(&self) -> String {
        return String::from("RETURN_STR");
    }
}
```
