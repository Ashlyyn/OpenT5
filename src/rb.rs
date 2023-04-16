use std::sync::RwLock;

use crate::{
    cl::Connstate,
    gfx::R_GLOB,
    sys::{KeyboardScancode, Modifiers, WindowEvent},
    *,
};

static MODIFIERS: RwLock<Modifiers> = RwLock::new(Modifiers::empty());

fn swap_buffers() {
    while let Some(ev) = sys::next_window_event() {
        match ev {
            WindowEvent::Created(handle) => {
                platform::set_window_handle(handle);
                if dvar::get_bool("r_reflectionProbeGenerate").unwrap()
                    && dvar::get_bool("r_fullscreen").unwrap()
                {
                    dvar::set_bool("r_fullscreen", false).unwrap();
                    cbuf::add_textln(0, "vid_restart");
                }
                dvar::register_bool(
                    "r_autopriority", 
                    false,
                    dvar::DvarFlags::ARCHIVE,
                     Some("Automatically set the priority of the windows process when the game is minimized")
                ).unwrap();
            }
            WindowEvent::CloseRequested => {
                cbuf::add_textln(0, "quit");
                sys::set_quit_event();
            }
            WindowEvent::Destroyed => {
                //FUN_004dfd60()
                platform::clear_window_handle();
            }
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
            }
            WindowEvent::SetFocus => {
                vid::app_activate(true, platform::get_minimized());
            }
            WindowEvent::KillFocus => {
                vid::app_activate(true, platform::get_minimized());
            }
            WindowEvent::ModifiersChanged { modifier, down } => {
                if modifier == Modifiers::CAPSLOCK
                    || modifier == Modifiers::NUMLOCK
                    || modifier == Modifiers::SCRLOCK
                {
                    *MODIFIERS.write().unwrap() ^= modifier;
                } else if down {
                    *MODIFIERS.write().unwrap() |= modifier;
                } else {
                    *MODIFIERS.write().unwrap() &= !modifier;
                }
                sys::enqueue_event(sys::Event::new(
                    Some(platform::get_msg_time() as _),
                    sys::EventType::Key(modifier.try_into().unwrap(), down),
                    None,
                ));
            }
            WindowEvent::KeyDown {
                logical_scancode, ..
            } => {
                if logical_scancode == KeyboardScancode::Enter
                    && MODIFIERS.read().unwrap().contains(Modifiers::LALT)
                {
                    if cl::get_local_client_connection_state(0)
                        == Connstate::LOADING
                    {
                        return;
                    }

                    if dvar::get_int("developer").unwrap() != 0 {
                        // FUN_005a5360()
                        dvar::set_bool(
                            "r_fullscreen",
                            dvar::get_bool("r_fullscreen").unwrap() == false,
                        )
                        .unwrap();
                        cbuf::add_textln(0, "vid_restart");
                    }
                }
                sys::enqueue_event(sys::Event::new(
                    Some(platform::get_msg_time() as _),
                    sys::EventType::Key(logical_scancode, true),
                    None,
                ));
            }
            WindowEvent::KeyUp {
                logical_scancode, ..
            } => {
                sys::enqueue_event(sys::Event::new(
                    Some(platform::get_msg_time() as _),
                    sys::EventType::Key(logical_scancode, false),
                    None,
                ));
            }
            _ => {}
        }
    }
}

#[allow(clippy::panic, clippy::print_stdout)]
pub fn render_thread() -> ! {
    loop {
        loop {
            if !sys::query_backend_event() {
                if !sys::query_rg_registered_event() {
                    swap_buffers();
                } else {
                    render::begin_registration_internal().unwrap();
                    sys::clear_rg_registered_event();
                }
            } else {
            }

            if R_GLOB.read().unwrap().remote_screen_update_nesting != 0 {
                break;
            }
        }

        assert_eq!(R_GLOB.read().unwrap().screen_update_notify, false);
        R_GLOB.write().unwrap().screen_update_notify = true;
        assert_eq!(R_GLOB.read().unwrap().is_rendering_remote_update, false);
        R_GLOB.write().unwrap().is_rendering_remote_update = true;
    }
}
