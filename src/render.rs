#![allow(dead_code)]

use std::sync::RwLock;
use sscanf::scanf;

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

#[derive(Clone, Default)]
struct WinitGlobals {
    current_monitor_handle: Option<winit::monitor::MonitorHandle>,
    best_monitor_handle: Option<winit::monitor::MonitorHandle>,
    video_modes: Vec<winit::monitor::VideoMode>,
}

lazy_static! {
    static ref WINIT_GLOBALS: Arc<RwLock<WinitGlobals>> =
        Arc::new(RwLock::new(WinitGlobals {
            current_monitor_handle: None,
            best_monitor_handle: None,
            video_modes: Vec::new(),
        }));
}

#[derive(Copy, Clone, Default)]
struct RenderGlobals {
    adapter_native_width: u16,
    adapter_native_height: u16,
}

lazy_static! {
    static ref RENDER_GLOBALS: Arc<RwLock<RenderGlobals>> =
        Arc::new(RwLock::new(Default::default()));
}

fn get_available_monitors() -> Vec<winit::monitor::MonitorHandle> {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    window.available_monitors().collect()
}

fn get_current_monitor() -> winit::monitor::MonitorHandle {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    window.current_monitor().unwrap()
}

fn get_best_monitor() -> winit::monitor::MonitorHandle {
    let handles = get_available_monitors();
    let handle = handles
        .iter()
        .max_by_key(|&h| h.refresh_rate_millihertz())
        .unwrap()
        .clone();

    if WINIT_GLOBALS
        .clone()
        .try_read()
        .expect("")
        .best_monitor_handle
        .is_none()
    {
        WINIT_GLOBALS
            .clone()
            .try_write()
            .expect("")
            .best_monitor_handle = Some(handle.clone());
    }

    handle
}

fn get_monitor_dimensions() -> (u16, u16) {
    let size = get_best_monitor().size();
    (size.width as _, size.height as _)
}

fn set_custom_resolution(wnd_parms: &mut gfx::WindowParms) -> bool {
    let r_custom_mode = match dvar::get_string("r_customMode") {
        Some(s) => s,
        None => return false,
    };

    let (display_width, display_height) = match scanf!(r_custom_mode, "{}x{}", u16, u16) {
        Ok((w, h)) => (w, h),
        Err(_) => return false
    };

    wnd_parms.display_width = display_width;
    wnd_parms.display_height = display_height;

    let (width, height) = get_monitor_dimensions();
    wnd_parms.display_width <= width && wnd_parms.display_height <= height
}

fn get_video_modes() -> Vec<winit::monitor::VideoMode> {
    if WINIT_GLOBALS
        .clone()
        .try_read()
        .expect("")
        .video_modes
        .is_empty()
    {
        let current_monitor_handle = match WINIT_GLOBALS
            .clone()
            .try_read()
            .expect("")
            .current_monitor_handle
            .clone()
        {
            Some(h) => h,
            None => return Vec::new(),
        };

        return current_monitor_handle.video_modes().collect();
    }

    WINIT_GLOBALS.clone().try_read().expect("").video_modes.clone()
}

fn closest_refresh_rate_for_mode(width: u16, height: u16, hz: u16) -> u16 {
    let video_modes = WINIT_GLOBALS.clone().try_read().expect("").video_modes.clone();
    let mode = video_modes.iter().find(|&m| {
        ((m.refresh_rate_millihertz() - (m.refresh_rate_millihertz() % 1000))
            * 1000
            == hz as _)
            && m.size().width == width as _
            && m.size().height == height as _
    });

    if mode.is_some() {
        return (mode.unwrap().refresh_rate_millihertz() / 1000) as _;
    }

    let mode = video_modes
        .iter()
        .find(|&m| (m.refresh_rate_millihertz() / 1000) == hz as _);
    if mode.is_some() {
        return (mode.unwrap().refresh_rate_millihertz() / 1000) as _;
    }

    let mode = video_modes.iter().find(|&m| {
        m.size().width == width as _ && m.size().height == height as _
    });

    if mode.is_some() {
        return (mode.unwrap().refresh_rate_millihertz() / 1000) as _;
    }

    video_modes
        .iter()
        .max_by(|&a, &b| {
            a.refresh_rate_millihertz()
                .cmp(&b.refresh_rate_millihertz())
        })
        .unwrap()
        .refresh_rate_millihertz() as _
}

fn set_wnd_parms(wnd_parms: &mut gfx::WindowParms) {
    let r_fullscreen = dvar::get_bool("r_fullscreen").unwrap_or(false);
    if !r_fullscreen {
        if !set_custom_resolution(wnd_parms) {
            dvar::set_string("r_mode", &format!("{}x{}", wnd_parms.display_width, wnd_parms.display_height));
        }
    }

    if !wnd_parms.fullscreen {
        let lock = RENDER_GLOBALS.clone();
        let writer = lock.try_write().expect("");
        if writer.adapter_native_width < wnd_parms.display_width {
            wnd_parms.display_width = wnd_parms.display_width.clamp(0, writer.adapter_native_width);
        }
        if writer.adapter_native_height < wnd_parms.display_height {
            wnd_parms.display_height = wnd_parms.display_height.clamp(0, writer.adapter_native_height);
        }
    }

    wnd_parms.scene_width = wnd_parms.display_width;
    wnd_parms.scene_height = wnd_parms.display_height;

    if !wnd_parms.fullscreen {
        wnd_parms.hz = (get_current_monitor().refresh_rate_millihertz().unwrap() / 1000) as _;
    } else {
        let hz = closest_refresh_rate_for_mode(wnd_parms.display_width, wnd_parms.display_height, wnd_parms.hz);
        wnd_parms.hz = hz;
        dvar::set_string("r_displayRefresh", &format!("{} Hz", hz));
    }

    wnd_parms.x = dvar::get_int("vid_xpos").unwrap_or(0) as _;
    wnd_parms.y = dvar::get_int("vid_ypos").unwrap_or(0) as _;
    wnd_parms.aa_samples = dvar::get_int("r_aaSamples").unwrap_or(0) as _;
}

/*
fn reduce_window_settings(wnd_parms: &mut gfx::WindowParms) -> bool {
    if dvar::get_int("r_aaSamples").unwrap_or(0) < 2 {

    }
}
*/

pub fn create_window(wnd_parms: &mut gfx::WindowParms) -> bool {
    if wnd_parms.fullscreen {
        com::println(&format!(
            "Attempting {} x {} fullscreen with 32 bpp at {} hz",
            wnd_parms.display_width,
            wnd_parms.display_height,
            wnd_parms.hz
        ));
    } else {
        com::println(&format!(
            "Attempting {} x {} window at ({}, {})",
            wnd_parms.display_width,
            wnd_parms.display_height,
            wnd_parms.x,
            wnd_parms.y
        ));
    }

    let window_name = com::get_official_build_name_r();
    let position = (wnd_parms.x, wnd_parms.y);
    let width = wnd_parms.display_width;
    let height = wnd_parms.display_height;

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
        .with_decorations(!wnd_parms.fullscreen)
        .with_window_icon(com::get_icon_rgba())
        .build(&event_loop)
    {
        Ok(w) => w,
        Err(e) => {
            com::println("couldn't create a window.");
            println!("{}", e);
            return false;
        }
    };

    let fullscreen = if wnd_parms.fullscreen {
        let monitor = window.primary_monitor().unwrap();
        let mut monitor_modes = monitor.video_modes();
        let mut mode: VideoMode = monitor_modes.next().unwrap();
        for m in monitor_modes {
            if m.refresh_rate_millihertz() > mode.refresh_rate_millihertz() {
                mode = m;
            }
        }
        Some(Fullscreen::Exclusive(mode))
    } else {
        None
    };

    window.set_fullscreen(fullscreen);
    window.set_visible(true);

    if wnd_parms.fullscreen == false {
        window.focus_window();
    }

    com::println("Game window successfully created.");

    // ========================================================================
    // The following code is done in the original engine's WM_CREATE handler,
    // but winit has no equivalent message for WM_CREATE. Do them here after
    // the window has been created instead

    //platform::set_window_handle(
    //    platform::WindowHandle::new(window.raw_window_handle()));
    match dvar::find("r_reflectionProbeGenerate") {
        None => {}
        Some(d) => match d.current.as_bool() {
            None => {}
            Some(_) => {
                dvar::set_bool_internal("r_fullscreen", false);
                cbuf::add_textln(0, "vid_restart");
            }
        },
    };

    dvar::register_bool(
        "r_autopriority",
        false,
        dvar::DvarFlags::UNKNOWN_00000001_A,
        None,
    );
    // ========================================================================

    let mut modifiers = winit::event::ModifiersState::empty();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::Destroyed => {
                //FUN_004dfd60()
                platform::clear_window_handle();
            },
            WindowEvent::ModifiersChanged(m) => {
                modifiers = *m;
            }
            WindowEvent::Moved(pos) => {
                dvar::set_int("vid_xpos", pos.x);
                dvar::set_int("vid_ypos", pos.y);
                dvar::clear_modified("vid_xpos");
                dvar::clear_modified("vid_ypos");
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
                cbuf::add_textln(0, "quit");
                *control_flow = ControlFlow::Exit;
            },
            #[allow(unused_variables, deprecated)]
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
                let alt = modifiers.alt();

                #[allow(clippy::collapsible_if)]
                if !alt {
                    sys::enqueue_event(
                        sys::Event::new(Some(platform::get_msg_time()),
                        sys::EventType::Key(scancode, false),
                        None));
                        // toggle fullscreen on Alt+Enter
                } else if scancode == input::keyboard::KeyScancode::Enter {
                    if // (_DAT_02910164 != 8) &&
                    dvar::find("r_fullscreen").is_some() &&
                    dvar::get_int("developer").unwrap() != 0 {
                        // FUN_005a5360()
                        dvar::set_bool_internal(
                            "r_fullscreen", 
                            !dvar::find("r_fullscreen")
                            .unwrap()
                            .current
                            .as_bool()
                            .unwrap());
                        cbuf::add_textln(0, "vid_restart");
                    }
                    // FUN_0053f880()
                }
            },
            _ => {}
        },
        _ => {}
    });

    true
}

#[allow(dead_code)]
struct RenderState {
    instance: sys::gpu::Instance,
    device: sys::gpu::Device,
    vendor_id: usize,
    adapter_native_is_valid: bool,
    adapter_native_width: u16,
    adapter_native_height: u16,
    fullscreen_width: u16,
    fullscreen_height: u16,
    resize_window: bool,
}
