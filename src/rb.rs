use crate::*;

#[allow(clippy::panic, clippy::print_stdout)]
pub fn render_thread() -> ! {
    loop {
        com::dprintln!(8.into(), "loop1");
        loop {
            com::dprintln!(8.into(), "loop2");
            if !sys::wait_event("backendEvent1", 0) {
                if sys::query_event("rgRegisteredEvent") {
                    panic!("");
                } else {
                    render::begin_registration_internal().unwrap();
                }
            } else {
                panic!("");
            }
        }
    }
}
