use crate::{*, sys::WindowEvent};

fn swap_buffers() {
    while let Some(ev) = sys::next_window_event() {
        match ev {
            WindowEvent::CloseRequested => {
                sys::set_quit_event();
            },
            _ => { },
        }
    }
}

#[allow(clippy::panic, clippy::print_stdout)]
pub fn render_thread() -> ! {
    loop {
        //com::dprintln!(8.into(), "loop1");
        loop {
            //com::dprintln!(8.into(), "loop2");
            if !sys::query_backend_event() {
                if !sys::query_rg_registered_event() {
                    swap_buffers();
                } else {
                    render::begin_registration_internal().unwrap();
                    sys::clear_rg_registered_event();
                }
            } else {
                panic!("");
            }
        }
    }
}
