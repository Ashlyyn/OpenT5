use crate::{
    render::{r_glob, r_glob_mut},
    sys::handle_main_window_event,
    util::SignalState,
    *,
};

#[allow(clippy::cast_possible_wrap)]
fn swap_buffers() {
    while let Some(ev) = sys::next_main_window_event() {
        handle_main_window_event(ev);
    }
}

#[allow(clippy::panic, clippy::print_stdout)]
pub fn render_thread() -> ! {
    loop {
        loop {
            if sys::query_backend_event() == SignalState::Cleared {
                if sys::query_rg_registered_event() == SignalState::Cleared {
                    swap_buffers();
                } else {
                    render::begin_registration_internal().unwrap();
                    sys::clear_rg_registered_event();
                }
            } else {
            }

            if r_glob().remote_screen_update_nesting != 0 {
                break;
            }
        }

        assert_eq!(r_glob().screen_update_notify, false);
        r_glob_mut().screen_update_notify = true;
        assert_eq!(r_glob().is_rendering_remote_update, false);
        r_glob_mut().is_rendering_remote_update = true;
    }
}
