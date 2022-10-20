#![allow(dead_code)]

use cfg_if::cfg_if;
use pollster::block_on;
use sscanf::scanf;
use std::{collections::HashSet};
cfg_if! {
    if #[cfg(debug)] {
        use no_deadlocks::{RwLock, Condvar, Mutex};
    } else {
        use std::sync::{RwLock, Condvar, Mutex};
    }
}

use crate::*;

use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Fullscreen, WindowBuilder}, platform::run_return::EventLoopExtRunReturn,
};

const MIN_HORIZONTAL_RESOLUTION: u16 = 640;
const MIN_VERTICAL_RESOLUTION: u16 = 480;

lazy_static! {
    pub static ref ALIVE: AtomicBool = AtomicBool::new(false);
}

fn init_render_thread() {
    if !sys::spawn_render_thread(&rb::render_thread) {
        com::errorln(com::ErrorParm::FATAL, "Failed to create render thread");
    }
}

pub fn init_threads() {
    ALIVE.store(true, Ordering::SeqCst);
    com::println("Trying SMP acceleration...");
    init_render_thread();
    //init_worker_threads();
    com::println("...succeeded.");
}

pub fn begin_registration_internal() {
    println!("render::begin_registration_internal()...");
    init();
    sys::wait_event("rg_registered", usize::MAX);
}

#[allow(dead_code)]
fn init() {
    println!("render::init()...");
    ALIVE.store(true, Ordering::SeqCst);
    env_logger::init();
    init_graphics_api();
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

impl Default for RenderGlobals {
    fn default() -> Self {
        Self {
            device: None,
            adapter: None,
            instance: None,
            adapter_native_width: 0,
            adapter_native_height: 0,
            target_window_index: 0,
            resolution_names: HashSet::new(),
            refresh_rate_names: HashSet::new(),
        }
    }
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
) -> bool {
    let r_custom_mode = match dvar::get_enumeration("r_customMode") {
        Some(s) => s,
        None => return false,
    };

    let (display_width, display_height) =
        match scanf!(r_custom_mode, "{}x{}", u16, u16) {
            Ok((w, h)) => (w, h),
            Err(_) => return false,
        };

    wnd_parms.display_width = display_width;
    wnd_parms.display_height = display_height;

    wnd_parms.display_width <= width && wnd_parms.display_height <= height
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
    if !r_fullscreen {
        if !set_custom_resolution(wnd_parms, width as _, height as _) {
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
        dvar::set_string("r_displayRefresh", &format!("{} Hz", hz));
    }

    wnd_parms.x = dvar::get_int("vid_xpos").unwrap_or(0) as _;
    wnd_parms.y = dvar::get_int("vid_ypos").unwrap_or(0) as _;
    wnd_parms.aa_samples = dvar::get_int("r_aaSamples").unwrap_or(0) as _;
}

fn reduce_window_settings() -> bool {
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
                false
            } else {
                dvar::set_enumeration_prev("r_mode");
                true
            }
        } else {
            dvar::set_enumeration_prev("r_displayRefresh");
            true
        }
    } else {
        dvar::set_enumeration_prev("r_aaSamples");
        true
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
        dvar::DvarFlags::UNKNOWN_00000001_A | dvar::DvarFlags::LATCHED,
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
        dvar::DvarFlags::UNKNOWN_00000001_A
            | dvar::DvarFlags::LATCHED
            | dvar::DvarFlags::CHANGEABLE_RESET,
        Some("Refresh rate"),
    );
}
*/

fn pre_create_window() -> bool {
    com::println("Getting Device interface...");
    let instance = sys::gpu::Instance::new();
    let adapter = block_on(sys::gpu::Adapter::new(&instance, None));
    RENDER_GLOBALS.clone().try_write().expect("").device =
        match block_on(sys::gpu::Device::new(&adapter)) {
            Some(d) => Some(d),
            None => {
                com::println("Device failed to initialize.");
                return false;
            }
        };

    RENDER_GLOBALS.clone().try_write().expect("").adapter = choose_adapter();
    //enum_display_modes();
    true
}

lazy_static! {
    pub static ref AWAITING_WINDOW_INIT: RwLock<Arc<(Mutex<bool>, Condvar)>> = RwLock::new(Arc::new((Mutex::new(false), Condvar::new())));
    pub static ref WINDOW_INITIALIZING: RwLock<Arc<(Mutex<bool>, Condvar)>> = RwLock::new(Arc::new((Mutex::new(false), Condvar::new())));
    pub static ref WINDOW_INITIALIZED: RwLock<Arc<(Mutex<(bool, bool)>, Condvar)>> = RwLock::new(Arc::new((Mutex::new((false, false)), Condvar::new())));
}

pub fn create_window_2(wnd_parms: &mut gfx::WindowParms) -> bool {
    {
        *AWAITING_WINDOW_INIT.write().expect("").0.lock().unwrap() = false;
        AWAITING_WINDOW_INIT.write().expect("").1.notify_one();

        *WINDOW_INITIALIZING.write().expect("").0.lock().unwrap() = true;
        WINDOW_INITIALIZING.write().expect("").1.notify_one();
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

        let mut event_loop = EventLoop::new();
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
                    *WINDOW_INITIALIZING.try_write().expect("").0.lock().unwrap() = false;
                WINDOW_INITIALIZING.try_write().expect("").1.notify_one();

                *WINDOW_INITIALIZED.try_write().expect("").0.lock().unwrap() = (true, false);
                WINDOW_INITIALIZED.try_write().expect("").1.notify_one();
                }
                return false;
            }
        };

        window.set_visible(true);

        if fullscreen == false {
            window.focus_window();
        }

        com::println("Game window successfully created.");

        event_loop.run_return(|event, _, control_flow| match event {
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
                if dvar::get_bool("r_reflectionProbeGenerate").unwrap_or(false) 
                    && dvar::get_bool("r_fullscreen").unwrap_or(false) {
                        dvar::set_bool_internal("r_fullscreen", false);
                        cbuf::add_textln(0, "vid_restart");
                    }
                dvar::register_bool("r_autopriority",
                    false,
                    dvar::DvarFlags::UNKNOWN_00000001_A,
                    Some("Automatically set the priority of the windows process when the game is minimized"),
                );

                /*
                let width = window.current_monitor().unwrap().size().width;
                let height = window.current_monitor().unwrap().size().height;
                let hz = window.current_monitor().unwrap().refresh_rate_millihertz().unwrap() / 1000;
                set_wnd_parms(wnd_parms, width as _, height as _, hz as _);
                */
                {
                    *WINDOW_INITIALIZING.try_write().expect("").0.lock().unwrap() = false;
                WINDOW_INITIALIZING.try_write().expect("").1.notify_one();

                *WINDOW_INITIALIZED.try_write().expect("").0.lock().unwrap() = (true, true);
                WINDOW_INITIALIZED.try_write().expect("").1.notify_one();
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
                    dvar::set_int("vid_xpos", pos.x);
                    dvar::set_int("vid_ypos", pos.y);
                    dvar::clear_modified("vid_xpos");
                    dvar::clear_modified("vid_ypos");
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
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                // TODO - R_ResizeWindow() reimpl
            },
            _ => {}
        });
    true
}

fn init_hardware(wnd_parms: &mut gfx::WindowParms) -> bool {
    com::println("TODO: render::init_hardware");
    true
}

pub fn create_window(wnd_parms: &mut gfx::WindowParms) -> bool {
    println!("render::create_window()...");
    {
        *AWAITING_WINDOW_INIT.write().expect("aaaa").0.lock().unwrap() = true;
        AWAITING_WINDOW_INIT.write().expect("uuuu").1.notify_one();
    }
    println!("written AWAITING_WINDOW_INIT.");

    {
        let lock = WND_PARMS.clone();
        let mut writer = lock.write().expect("");
        *writer = *wnd_parms;
    }
    println!("written WND_PARMS.");

    println!("waiting for init...");
    loop {
        let reader = WINDOW_INITIALIZED.read().expect("");
        let guard = reader.0.lock().unwrap();
        reader.1.wait(guard).unwrap();
        if reader.0.lock().unwrap().0 == true {
            let res = reader.0.lock().unwrap();
            println!("init complete, res={}...", res.1);
            return res.1
        }
    }
}

fn init_systems() {}

lazy_static! {
    pub static ref WND_PARMS: Arc<RwLock<gfx::WindowParms>> = Arc::new(RwLock::new(Default::default()));
}

fn init_graphics_api() {
    println!("render::init_graphics_api()...");
    if RENDER_GLOBALS
        .clone()
        .try_read()
        .expect("")
        .device
        .is_none()
    {
        pre_create_window();
        loop {
            let mut wnd_parms: gfx::WindowParms = gfx::WindowParms::new();
            println!("{}, {}, {}", wnd_parms.display_width, wnd_parms.display_height, wnd_parms.hz);
            set_wnd_parms(&mut wnd_parms, 800, 600, 60);
            println!("{}, {}, {}", wnd_parms.display_width, wnd_parms.display_height, wnd_parms.hz);
            if create_window(&mut wnd_parms) {
                break;
            }
            if !reduce_window_settings() {
                fatal_init_error("Couldn't initialize renderer")
            }
        }
    } else {
        init_systems();
    }
}
