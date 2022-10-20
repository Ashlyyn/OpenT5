use crate::*;

pub fn render_thread() -> ! {
    println!("rb::render_thread()...");
    loop {
        println!("loop1");
        loop {
            println!("loop2");
            if !sys::wait_event("backendEvent1", 0) {
                if sys::query_event("rgRegisteredEvent") {
                    panic!("");
                } else {
                    render::begin_registration_internal();
                }
            } else {
                panic!("");
            }
        }
    }
}