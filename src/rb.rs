use crate::*;

#[allow(clippy::panic, clippy::print_stdout)]
pub fn render_thread() -> ! {
    com::println!(
        0.into(),
        "{}: rb::render_thread()...",
        std::thread::current().name().unwrap_or("main"),
    );
    loop {
        println!("loop1");
        loop {
            println!("loop2");
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
