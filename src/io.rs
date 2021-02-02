use std::ffi::c_void;

#[allow(dead_code)]
struct EventLoopGroupOptions {
    num_threads: u16
}

impl Default for EventLoopGroupOptions {
    fn default() -> Self {
        EventLoopGroupOptions {
            num_threads: 0,
        }
    }
}

struct EventLoopGroup {
    c_elg : *const c_void,
}

#[allow(dead_code)]
impl EventLoopGroup {
    fn new(options: EventLoopGroupOptions) -> EventLoopGroup {
        EventLoopGroup {
            c_elg: unsafe {aws_crt_event_loop_group_new(options.num_threads)}
        }
    }
}

impl Drop for EventLoopGroup {
    fn drop(&mut self) {
        unsafe {
            aws_crt_event_loop_group_release(self.c_elg);
        }
    }
}

#[allow(dead_code)]
extern "C" {
    fn aws_crt_event_loop_group_new(num_threads: u16) -> *const c_void;
    fn aws_crt_event_loop_group_release(c_elg: *const c_void);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_loop_group_lifetime() {
        let _elg = EventLoopGroup::new(EventLoopGroupOptions::default());
    }
}