use crate::*;

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    monitor::VideoMode,
    window::{Fullscreen, WindowBuilder},
};

#[allow(dead_code)]
fn init() {
    env_logger::init();
}

#[allow(deprecated)]
pub fn create_window(window_parms: &mut gfx::WindowParms) {
    if window_parms.fullscreen {
        com::println(format!(
            "Attempting {} x {} fullscreen with 32 bpp at {} hz",
            window_parms.display_width,
            window_parms.display_height,
            window_parms.hz
        ));
    } else {
        com::println(format!(
            "Attempting {} x {} window at ({}, {})",
            window_parms.display_width,
            window_parms.display_height,
            window_parms.x,
            window_parms.y
        ));
    }

    let window_name = com::get_official_build_name_r();
    let position = (window_parms.x, window_parms.y);
    let width = window_parms.display_width;
    let height = window_parms.display_height;

    let event_loop = EventLoop::new();
    let window = match WindowBuilder::new()
        .with_title(window_name)
        .with_position(PhysicalPosition::<i32>::new(
            position.0 as i32,
            position.1 as i32,
        ))
        .with_inner_size(PhysicalSize::new(width, height))
        .with_resizable(true)
        .with_visible(false)
        .with_decorations(!window_parms.fullscreen)
        .with_window_icon(com::get_icon_rgba())
        .build(&event_loop)
    {
        Ok(w) => w,
        Err(e) => {
            com::println("couldn't create a window.".to_string());
            println!("{}", e);
            return;
        }
    };

    let fullscreen = if window_parms.fullscreen {
        let monitor = window.primary_monitor().unwrap();
        let mut monitor_modes = monitor.video_modes();
        let mut mode: VideoMode = monitor_modes.next().unwrap();
        for m in monitor_modes {
            if m.refresh_rate() > mode.refresh_rate() {
                mode = m;
            }
        }
        Some(Fullscreen::Exclusive(mode))
    } else {
        None
    };

    window.set_fullscreen(fullscreen);
    window.set_visible(true);

    if window_parms.fullscreen == false {
        window.focus_window();
    }

    com::println("Game window successfully created.".to_string());

    // ===========================================================================
    // The following code is done in the original engine's WM_CREATE handler,
    // but winit has no equivalent message for WM_CREATE. Do them here after
    // the window has been created instead
    //platform::set_window_handle(platform::WindowHandle::new(window.raw_window_handle()));
    match dvar::find("r_reflectionProbeGenerate".to_string()) {
        None => {}
        Some(d) => match d.value().as_bool() {
            None => {}
            Some(_) => {
                dvar::set_bool("r_fullscreen".to_string(), false);
                cbuf::add_textln(0, "vid_restart".to_string());
            }
        },
    };

    dvar::register_bool(
        "r_autopriority".to_string(),
        false,
        dvar::DvarFlags::UNKNOWN_00000001_A,
        None,
    );
    // TODO - MSH_MOUSEWHEEL
    // ===========================================================================

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::Destroyed => {
                    //FUN_004dfd60()
                    platform::clear_window_handle();
                },
                WindowEvent::Moved(pos) => {
                    dvar::set_int("vid_xpos".to_string(), pos.x);
                    dvar::set_int("vid_ypos".to_string(), pos.y);
                    dvar::clear_modified("vid_xpos".to_string());
                    dvar::clear_modified("vid_ypos".to_string());
                    if platform::get_active_app() {
                        input::activate(1);
                    } else {
                        input::mouse::activate(0);
                    }
                },
                WindowEvent::Focused(b) => {
                    vid::app_activate(*b, platform::get_minimized());
                },
                WindowEvent::CloseRequested => {
                    cbuf::add_textln(0, "quit".to_string());
                    *control_flow = ControlFlow::Exit;
                },
                #[allow(unused_variables)]
                WindowEvent::MouseWheel {
                    device_id,
                    delta,
                    phase,
                    modifiers
                } => {
                    let lines = match delta {
                        MouseScrollDelta::LineDelta(f, _) => *f,
                        MouseScrollDelta::PixelDelta(_) =>
                            panic!("render::create_window: unable to handle PixelDelta variant of MouseScrollDelta for MouseWheel event")
                    };
                    if lines < 0.0 {
                        sys::enqueue_event(
                            sys::Event::new(Some(platform::get_msg_time()),
                            sys::EventType::Mouse(
                                input::mouse::Scancode::MWheelDown,
                                true),
                            None));
                        sys::enqueue_event(
                            sys::Event::new(Some(platform::get_msg_time()),
                            sys::EventType::Mouse(
                                input::mouse::Scancode::MWheelDown,
                                false),
                            None));
                    } else {
                        sys::enqueue_event(
                            sys::Event::new(Some(platform::get_msg_time()),
                            sys::EventType::Mouse(
                                input::mouse::Scancode::MWheelUp,
                                true),
                            None));
                        sys::enqueue_event(
                            sys::Event::new(Some(platform::get_msg_time()),
                            sys::EventType::Mouse(
                                input::mouse::Scancode::MWheelUp,
                                false),
                            None));
                    }
                },
                #[allow(unused_variables)]
                WindowEvent::KeyboardInput {
                    device_id,
                    input,
                    is_synthetic
                } => {
                    let scancode: input::keyboard::KeyScancode =
                        num::FromPrimitive::from_u8(input.scancode as u8)
                        .unwrap();
                    let alt = input.modifiers.alt();

                    #[allow(clippy::collapsible_if)]
                    if !alt {
                        sys::enqueue_event(
                            sys::Event::new(Some(platform::get_msg_time()),
                            sys::EventType::Key(scancode, false),
                            None));
                        // toggle fullscreen on Alt+Enter
                    } else if scancode == input::keyboard::KeyScancode::Enter {
                        if // (_DAT_02910164 != 8) &&
                        dvar::find("r_fullscreen".to_string()).is_some() &&
                        dvar::get_int("developer".to_string()).unwrap() != 0 {
                            // FUN_005a5360()
                            dvar::set_bool(
                                "r_fullscreen".to_string(), 
                                !dvar::find("r_fullscreen".to_string())
                                .unwrap()
                                .value()
                                .as_bool()
                                .unwrap());
                            cbuf::add_textln(0, "vid_restart".to_string());
                        }
                        // FUN_0053f880()
                    }
                },
                _ => {}
            },
            _ => {}
    })
}
