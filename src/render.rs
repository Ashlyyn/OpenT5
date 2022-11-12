#![allow(dead_code)]

use crate::{util::*, *};
use cfg_if::cfg_if;
use pollster::block_on;
use sscanf::scanf;
use std::collections::HashSet;
cfg_if! {
    if #[cfg(debug_assertions)] {
        use no_deadlocks::{RwLock};
    } else {
        use std::sync::{RwLock};
    }
}

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder},
};

const MIN_HORIZONTAL_RESOLUTION: u16 = 640;
const MIN_VERTICAL_RESOLUTION: u16 = 480;

lazy_static! {
    pub static ref ALIVE: AtomicBool = AtomicBool::new(false);
}

fn init_render_thread() {
    if !sys::spawn_render_thread(rb::render_thread) {
        com::errorln(com::ErrorParm::FATAL, "Failed to create render thread");
    }
}

pub fn init_threads() {
    ALIVE.store(true, Ordering::SeqCst);
    com::println(&format!(
        "{}: Trying SMP acceleration...",
        std::thread::current().name().unwrap_or("main")
    ));
    init_render_thread();
    //init_worker_threads();
    com::println(&format!(
        "{}: ...succeeded",
        std::thread::current().name().unwrap_or("main")
    ));
}

pub fn begin_registration_internal() -> Result<(), ()> {
    com::println(&format!(
        "{}: render::begin_registration_internal()...",
        std::thread::current().name().unwrap_or("main")
    ));

    if init().is_err() {
        return Err(());
    }
    sys::wait_event("rg_registered", usize::MAX);
    Ok(())
}

#[allow(dead_code)]
fn init() -> Result<(), ()> {
    com::println(&format!(
        "{}: render::init()...",
        std::thread::current().name().unwrap_or("main")
    ));
    ALIVE.store(true, Ordering::SeqCst);
    env_logger::init();
    init_graphics_api()
}

#[derive(Default)]
struct WinitGlobals {
    current_monitor_handle: Option<winit::monitor::MonitorHandle>,
    best_monitor_handle: Option<winit::monitor::MonitorHandle>,
    video_modes: Vec<winit::monitor::VideoMode>,
    window: Option<winit::window::Window>,
}

lazy_static! {
    static ref WINIT_GLOBALS: Arc<RwLock<WinitGlobals>> =
        Arc::new(RwLock::new(WinitGlobals {
            current_monitor_handle: None,
            best_monitor_handle: None,
            window: None,
            video_modes: Vec::new(),
        }));
}

#[derive(Default)]
pub struct RenderGlobals {
    adapter_native_width: u16,
    adapter_native_height: u16,
    resolution_names: HashSet<String>,
    refresh_rate_names: HashSet<String>,
    target_window_index: i32,
    device: Option<sys::gpu::Device>,
    adapter: Option<sys::gpu::Adapter>,
    instance: Option<sys::gpu::Instance>,
}

lazy_static! {
    pub static ref RENDER_GLOBALS: Arc<RwLock<RenderGlobals>> =
        Arc::new(RwLock::new(Default::default()));
}

fn fatal_init_error(error: &str) -> ! {
    com::println("********** Device returned an unrecoverable error code during initialization  **********");
    com::println("********** Initialization also happens while playing if Renderer loses a device **********");
    com::println(error);
    sys::render_fatal_error();
}

/*
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
*/

fn set_custom_resolution(
    wnd_parms: &mut gfx::WindowParms,
    width: u16,
    height: u16,
) -> Result<(), ()> {
    let r_custom_mode = match dvar::get_enumeration("r_customMode") {
        Some(s) => s,
        None => return Err(()),
    };

    let (display_width, display_height) =
        match scanf!(r_custom_mode, "{}x{}", u16, u16) {
            Ok((w, h)) => (w, h),
            Err(_) => return Err(()),
        };

    wnd_parms.display_width = display_width;
    wnd_parms.display_height = display_height;

    match wnd_parms.display_width <= width && wnd_parms.display_height <= height
    {
        true => Ok(()),
        false => Err(()),
    }
}

/*
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

    WINIT_GLOBALS
        .clone()
        .try_read()
        .expect("")
        .video_modes
        .clone()
}
*/

fn closest_refresh_rate_for_mode(
    width: u16,
    height: u16,
    hz: u16,
) -> Option<u16> {
    let video_modes = WINIT_GLOBALS
        .clone()
        .try_read()
        .expect("")
        .video_modes
        .clone();
    let mode = video_modes.iter().find(|&m| {
        ((m.refresh_rate_millihertz() - (m.refresh_rate_millihertz() % 1000))
            / 1000
            == hz as _)
            && m.size().width == width as _
            && m.size().height == height as _
    });

    if let Some(..) = mode {
        return Some((mode.unwrap().refresh_rate_millihertz() / 1000) as _);
    }

    let mode = video_modes
        .iter()
        .find(|&m| (m.refresh_rate_millihertz() / 1000) == hz as _);
    if let Some(..) = mode {
        return Some((mode.unwrap().refresh_rate_millihertz() / 1000) as _);
    }

    let mode = video_modes.iter().find(|&m| {
        m.size().width == width as _ && m.size().height == height as _
    });

    if let Some(..) = mode {
        return Some((mode.unwrap().refresh_rate_millihertz() / 1000) as _);
    }

    None
}

fn set_wnd_parms(
    wnd_parms: &mut gfx::WindowParms,
    width: u16,
    height: u16,
    hz: u16,
) {
    let r_fullscreen = dvar::get_bool("r_fullscreen").unwrap_or(false);
    #[allow(clippy::collapsible_if)]
    if !r_fullscreen {
        if set_custom_resolution(wnd_parms, width as _, height as _).is_err() {
            /*
            let r_mode = dvar::get_enumeration("r_mode").unwrap_or_default();
            (wnd_parms.display_width, wnd_parms.display_height) = scanf!(r_mode, "{}x{}", u16, u16).unwrap_or((0, 0));
            */
            wnd_parms.display_width = width;
            wnd_parms.display_height = height;
        }
    }

    if !wnd_parms.fullscreen {
        let lock = RENDER_GLOBALS.clone();
        let mut writer = lock.try_write().expect("");
        writer.adapter_native_width = width;
        writer.adapter_native_height = height;

        if writer.adapter_native_width < wnd_parms.display_width {
            wnd_parms.display_width = wnd_parms
                .display_width
                .clamp(0, writer.adapter_native_width);
        }
        if writer.adapter_native_height < wnd_parms.display_height {
            wnd_parms.display_height = wnd_parms
                .display_height
                .clamp(0, writer.adapter_native_height);
        }
    }

    wnd_parms.scene_width = wnd_parms.display_width;
    wnd_parms.scene_height = wnd_parms.display_height;

    if !wnd_parms.fullscreen {
        wnd_parms.hz = hz;
    } else {
        let hz = closest_refresh_rate_for_mode(
            wnd_parms.display_width,
            wnd_parms.display_height,
            wnd_parms.hz,
        )
        .unwrap_or(0);
        wnd_parms.hz = hz;
        #[allow(unused_must_use)]
        {
            dvar::set_string("r_displayRefresh", &format!("{} Hz", hz));
        }
    }

    wnd_parms.x = dvar::get_int("vid_xpos").unwrap_or(0) as _;
    wnd_parms.y = dvar::get_int("vid_ypos").unwrap_or(0) as _;
    wnd_parms.aa_samples = dvar::get_int("r_aaSamples").unwrap_or(0) as _;
}

fn reduce_window_settings() -> Result<(), ()> {
    if dvar::get_int("r_aaSamples").unwrap_or(0) < 2 {
        if dvar::get_enumeration("r_displayRefresh")
            .unwrap()
            .is_empty()
            || vid::config().display_frequency < 60
        {
            if dvar::get_enumeration("r_mode").unwrap().is_empty()
                || vid::config().display_width < MIN_HORIZONTAL_RESOLUTION
                || vid::config().display_height < MIN_VERTICAL_RESOLUTION
            {
                Err(())
            } else {
                dvar::set_enumeration_prev("r_mode")
            }
        } else {
            dvar::set_enumeration_prev("r_displayRefresh")
        }
    } else {
        dvar::set_enumeration_prev("r_aaSamples")
    }
}

fn choose_adapter() -> Option<sys::gpu::Adapter> {
    let instance = sys::gpu::Instance::new();
    Some(block_on(sys::gpu::Adapter::new(&instance, None)))
}

/*
fn enum_display_modes() {
    let mut modes: Vec<winit::monitor::VideoMode> =
        get_current_monitor().video_modes().collect();
    modes.sort_by(|a, b| a.size().width.cmp(&b.size().width));

    let valid_modes: Vec<&winit::monitor::VideoMode> = modes
        .iter()
        .filter(|&m| {
            m.size().width > MIN_HORIZONTAL_RESOLUTION as _
                && m.size().height > MIN_VERTICAL_RESOLUTION as _
        })
        .collect();

    valid_modes.iter().for_each(|&m| {
        RENDER_GLOBALS
            .clone()
            .try_write()
            .expect("")
            .resolution_names
            .insert(format!("{}x{}", m.size().width, m.size().height));
    });

    dvar::register_enumeration(
        "r_mode",
        RENDER_GLOBALS
            .clone()
            .try_read()
            .expect("")
            .resolution_names
            .iter()
            .last()
            .unwrap()
            .clone(),
        Some(Vec::from_iter(
            RENDER_GLOBALS
                .clone()
                .try_read()
                .expect("")
                .resolution_names
                .iter()
                .map(|s| s.clone())
                .collect::<Vec<String>>(),
        )),
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::LATCHED,
        Some("Renderer resolution mode"),
    );

    modes.sort_by(|a, b| {
        a.refresh_rate_millihertz()
            .cmp(&b.refresh_rate_millihertz())
    });

    modes.iter().for_each(|m| {
        RENDER_GLOBALS
            .clone()
            .try_write()
            .expect("")
            .refresh_rate_names
            .insert(format!(
                "{} Hz",
                (m.refresh_rate_millihertz()
                    - (m.refresh_rate_millihertz() % 1000))
                    / 1000
            ));
    });

    dvar::register_enumeration(
        "r_displayRefresh",
        RENDER_GLOBALS
            .clone()
            .try_read()
            .expect("")
            .refresh_rate_names
            .iter()
            .last()
            .unwrap()
            .clone(),
        Some(Vec::from_iter(
            RENDER_GLOBALS
                .clone()
                .try_read()
                .expect("")
                .refresh_rate_names
                .iter()
                .map(|s| s.clone())
                .collect::<Vec<String>>(),
        )),
        dvar::DvarFlags::ARCHIVE
            | dvar::DvarFlags::LATCHED
            | dvar::DvarFlags::CHANGEABLE_RESET,
        Some("Refresh rate"),
    );
}
*/

fn pre_create_window() -> Result<(), ()> {
    com::println("Getting Device interface...");
    let instance = sys::gpu::Instance::new();
    let adapter = block_on(sys::gpu::Adapter::new(&instance, None));
    RENDER_GLOBALS.clone().try_write().expect("").device =
        match block_on(sys::gpu::Device::new(&adapter)) {
            Some(d) => Some(d),
            None => {
                com::println("Device failed to initialize.");
                return Err(());
            }
        };

    RENDER_GLOBALS.clone().try_write().expect("").adapter = choose_adapter();
    //enum_display_modes();
    Ok(())
}

lazy_static! {
    pub static ref AWAITING_WINDOW_INIT: Arc<RwLock<SmpEvent<()>>> =
        Arc::new(RwLock::new(SmpEvent::new((), false, false)));
    pub static ref WINDOW_INITIALIZING: Arc<RwLock<SmpEvent<()>>> =
        Arc::new(RwLock::new(SmpEvent::new((), false, false)));
    pub static ref WINDOW_INITIALIZED: Arc<RwLock<SmpEvent<bool>>> =
        Arc::new(RwLock::new(SmpEvent::new(false, false, false)));
}

pub fn create_window_2(wnd_parms: &mut gfx::WindowParms) -> Result<(), ()> {
    {
        let lock = AWAITING_WINDOW_INIT.clone();
        let mut writer = lock.write().unwrap();
        writer.send_cleared(()).unwrap();
    }
    {
        let lock = WINDOW_INITIALIZED.clone();
        let mut writer = lock.write().unwrap();
        writer.send(false).unwrap();
    }

    if wnd_parms.fullscreen {
        com::println(&format!(
            "Attempting {} x {} fullscreen with 32 bpp at {} hz",
            wnd_parms.display_width, wnd_parms.display_height, wnd_parms.hz
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

    // ========================================================================
    // The following code is done in the original engine's WM_CREATE handler,
    // but winit has no equivalent message for WM_CREATE. Do them here after
    // the window has been created instead

    //platform::set_window_handle(
    //    platform::WindowHandle::new(window.raw_window_handle()));

    // ========================================================================

    let mut modifiers = winit::event::ModifiersState::empty();
    let fullscreen = wnd_parms.fullscreen;
    let width = wnd_parms.scene_width;
    let height = wnd_parms.scene_height;
    let x = wnd_parms.x;
    let y = wnd_parms.y;
    let hz = wnd_parms.hz;

    let event_loop = EventLoop::new();
    let window = match WindowBuilder::new()
        .with_title(window_name)
        .with_position(PhysicalPosition::<i32>::new(x as _, y as _))
        .with_inner_size(PhysicalSize::new(width, height))
        .with_resizable(true)
        .with_visible(false)
        .with_decorations(!fullscreen)
        .with_window_icon(com::get_icon_rgba())
        .build(&event_loop)
    {
        Ok(w) => w,
        Err(e) => {
            com::println("couldn't create a window.");
            println!("{}", e);
            {
                let lock = WINDOW_INITIALIZING.clone();
                let mut writer = lock.write().unwrap();
                writer.send_cleared(()).unwrap();
            }
            {
                let lock = WINDOW_INITIALIZED.clone();
                let mut writer = lock.write().unwrap();
                writer.send(false).unwrap();
            }
            return Err(());
        }
    };

    window.set_visible(true);

    if fullscreen == false {
        window.focus_window();
    }

    com::println("Game window successfully created.");

    event_loop.run(move |event, _, control_flow| match event {
            Event::NewEvents(StartCause::Init) => {
                let window_fullscreen = if fullscreen {
                    let mode = window.current_monitor().unwrap().video_modes()
                        .find(|m| {
                            m.size().width == width as _
                                && m.size().height == height as _
                                && (m.refresh_rate_millihertz()
                                    - (m.refresh_rate_millihertz() % 1000))
                                    == hz as _
                        })
                        .unwrap();
                    Some(Fullscreen::Exclusive(mode))
                } else {
                    None
                };
                window.set_fullscreen(window_fullscreen);
                #[allow(unused_must_use)]
                {
                if dvar::get_bool("r_reflectionProbeGenerate").unwrap_or(false) 
                    && dvar::get_bool("r_fullscreen").unwrap_or(false) {
                        dvar::set_bool_internal("r_fullscreen", false);
                        cbuf::add_textln(0, "vid_restart");
                    }
                dvar::register_bool("r_autopriority",
                    false,
                    dvar::DvarFlags::ARCHIVE,
                    Some("Automatically set the priority of the windows process when the game is minimized"),
                );
                }

                /*
                let width = window.current_monitor().unwrap().size().width;
                let height = window.current_monitor().unwrap().size().height;
                let hz = window.current_monitor().unwrap().refresh_rate_millihertz().unwrap() / 1000;
                set_wnd_parms(wnd_parms, width as _, height as _, hz as _);
                */
                {
                    let lock = WINDOW_INITIALIZING.clone();
                    let mut writer = lock.write().unwrap();
                    writer.send_cleared(()).unwrap();
                }
                {
                    let lock = WINDOW_INITIALIZED.clone();
                    let mut writer = lock.write().unwrap();
                    writer.send(true).unwrap();
                }
            },
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
                    #[allow(unused_must_use)]
                    {
                    dvar::set_int("vid_xpos", pos.x);
                    dvar::set_int("vid_ypos", pos.y);
                    dvar::clear_modified("vid_xpos");
                    dvar::clear_modified("vid_ypos");
                    }
                    if platform::get_active_app() {
                        input::activate(true);
                    } else {
                        input::mouse::activate(1);
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
                        dvar::exists("r_fullscreen") &&
                        dvar::get_int("developer").unwrap() != 0 {
                            // FUN_005a5360()
                            #[allow(unused_must_use)]
                            {
                            dvar::set_bool_internal(
                                "r_fullscreen", 
                                !dvar::get_bool("r_fullscreen")
                                .unwrap_or_default());
                            }
                            cbuf::add_textln(0, "vid_restart");
                        }
                        // FUN_0053f880()
                    }
                },
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                // TODO - R_ResizeWindow() reimpl
            },
            _ => {}
        });
}

fn init_hardware(_wnd_parms: &mut gfx::WindowParms) -> Result<(), ()> {
    com::println("TODO: render::init_hardware");
    Ok(())
}

pub fn create_window(wnd_parms: &mut gfx::WindowParms) -> Result<(), ()> {
    com::println(&format!(
        "{}: render::create_window()...",
        std::thread::current().name().unwrap_or("main")
    ));
    {
        let lock = AWAITING_WINDOW_INIT.clone();
        let mut writer = lock.write().unwrap();
        writer.send(()).unwrap();
    }
    com::println(&format!(
        "{}: written AWAITING_WINDOW_INIT.",
        std::thread::current().name().unwrap_or("main")
    ));
    {
        let lock = WND_PARMS.clone();
        let mut writer = lock.write().expect("");
        *writer = *wnd_parms;
    }
    com::println(&format!(
        "{}: written WND_PARMS.",
        std::thread::current().name().unwrap_or("main")
    ));

    com::println(&format!(
        "{}: waiting for init...",
        std::thread::current().name().unwrap_or("main")
    ));

    let res = {
        let mut window_initialized = WINDOW_INITIALIZED.write().unwrap();
        window_initialized.acknowledge().unwrap_or(false)
    };
    com::println(&format!(
        "{}: init complete, res={}...",
        std::thread::current().name().unwrap_or("main"),
        res
    ));

    match res {
        true => Ok(()),
        false => Err(()),
    }
}

fn init_systems() -> Result<(), ()> {
    Ok(())
}

lazy_static! {
    pub static ref WND_PARMS: Arc<RwLock<gfx::WindowParms>> =
        Arc::new(RwLock::new(Default::default()));
}

fn init_graphics_api() -> Result<(), ()> {
    com::println(&format!(
        "{}: render::init_graphics_api()...",
        std::thread::current().name().unwrap_or("main")
    ));
    if RENDER_GLOBALS
        .clone()
        .try_read()
        .expect("")
        .device
        .is_none()
    {
        if pre_create_window().is_err() {
            return Err(());
        }

        loop {
            let mut wnd_parms: gfx::WindowParms = gfx::WindowParms::new();
            println!(
                "{}, {}, {}",
                wnd_parms.display_width, wnd_parms.display_height, wnd_parms.hz
            );
            set_wnd_parms(&mut wnd_parms, 800, 600, 60);
            println!(
                "{}, {}, {}",
                wnd_parms.display_width, wnd_parms.display_height, wnd_parms.hz
            );
            if create_window(&mut wnd_parms).is_err() {
                break;
            }
            if reduce_window_settings().is_err() {
                fatal_init_error("Couldn't initialize renderer")
            }
        }

        Ok(())
    } else {
        init_systems()
    }
}
