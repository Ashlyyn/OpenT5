use crate::{*, sys::WindowEvent};

fn swap_buffers() {
    while let Some(ev) = sys::next_window_event() {
        match ev {
            WindowEvent::Created(handle) => {
                platform::set_window_handle(handle);
                if dvar::get_bool("r_reflectionProbeGenerate").unwrap() && dvar::get_bool("r_fullscreen").unwrap() {
                    dvar::set_bool("r_fullscreen", false).unwrap();
                    cbuf::add_textln(0, "vid_restart");
                }
                dvar::register_bool(
                    "r_autopriority", 
                    false, 
                    dvar::DvarFlags::ARCHIVE,
                     Some("Automatically set the priority of the windows process when the game is minimized")
                ).unwrap();
            },
            WindowEvent::CloseRequested => {
                cbuf::add_textln(0, "quit");
                sys::set_quit_event();
            },
            WindowEvent::Destroyed => {
                //FUN_004dfd60()
                platform::clear_window_handle();
            },
            WindowEvent::Moved { x, y } => {
                if dvar::get_bool("r_fullscreen").unwrap() {
                    input::mouse::activate(0);
                } else {
                    dvar::set_int_internal("vid_xpos", x as _).unwrap();
                    dvar::set_int_internal("vid_ypos", y as _).unwrap();
                    dvar::clear_modified("vid_xpos").unwrap();
                    dvar::clear_modified("vid_ypos").unwrap();
                    if platform::get_platform_vars().active_app {
                        input::activate(true);
                    }
                }
            },
            WindowEvent::SetFocus => {
                vid::app_activate(true, platform::get_minimized());
            },
            WindowEvent::KillFocus => {
                vid::app_activate(true, platform::get_minimized());
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
