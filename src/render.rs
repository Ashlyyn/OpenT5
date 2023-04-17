#![allow(dead_code)]

use crate::gfx::{WindowTarget, R_GLOB};
use crate::platform::os::target::{MonitorHandle, show_window};
use crate::sys::gpu::Device;
use crate::{platform::WindowHandle, *};
use pollster::block_on;
//use raw_window_handle::{RawWindowHandle};
use sscanf::scanf;
extern crate alloc;
use std::collections::{HashSet};
use std::collections::VecDeque;
use std::sync::RwLock;

pub const MIN_HORIZONTAL_RESOLUTION: u32 = 640;
pub const MIN_VERTICAL_RESOLUTION: u32 = 480;

cfg_if! {
    if #[cfg(windows)] {
        use windows::Win32::Foundation::{RECT, HWND, LPARAM, POINT, BOOL};
        use windows::Win32::Graphics::Gdi::{DEVMODEW, EnumDisplayMonitors, MONITOR_DEFAULTTOPRIMARY, MONITOR_DEFAULTTONEAREST, MonitorFromPoint, MonitorFromWindow, GetMonitorInfoW, MONITORINFOEXW, EnumDisplaySettingsExW, ENUM_CURRENT_SETTINGS, ENUM_DISPLAY_SETTINGS_FLAGS, ENUM_DISPLAY_SETTINGS_MODE, DEVMODE_FIELD_FLAGS, DM_BITSPERPEL, DM_PELSWIDTH, DM_PELSHEIGHT, DM_DISPLAYFREQUENCY, HMONITOR, HDC};
        use windows::Win32::System::LibraryLoader::GetModuleHandleA;
        use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
        use windows::Win32::UI::WindowsAndMessaging::{WS_EX_LEFT, WS_SYSMENU, WS_CAPTION, WS_VISIBLE, WS_EX_TOPMOST, WS_POPUP, AdjustWindowRectEx, CreateWindowExA, SetWindowPos, HWND_NOTOPMOST, SWP_NOSIZE, SWP_NOMOVE, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
        use windows::core::{PCSTR, PCWSTR};
        use windows::s;
        use crate::platform::os::target::monitor_enum_proc;
        use core::ptr::addr_of_mut;
        use std::mem::size_of_val;
        use alloc::collections::BTreeSet;
        use std::ffi::CString;
        use raw_window_handle::Win32WindowHandle;
        use raw_window_handle::WindowsDisplayHandle;
    } else if #[cfg(unix)] {
        use x11::xlib::{XOpenDisplay};
        use x11rb::connection::Connection;
        use x11rb::protocol::xproto::{
            ConnectionExt, CreateWindowAux, InputFocus, WindowClass,
        };
        use x11rb::COPY_DEPTH_FROM_PARENT;
        use raw_window_handle::XlibWindowHandle;
        use std::time::{SystemTime, UNIX_EPOCH};
        use raw_window_handle::XlibDisplayHandle;
    }
}

fn init_render_thread() {
    if !sys::spawn_render_thread(rb::render_thread) {
        com::errorln(com::ErrorParm::FATAL, "Failed to create render thread");
    }
}

pub fn init_threads() {
    com::println!(
        8.into(),
        "{}: Trying SMP acceleration...",
        std::thread::current().name().unwrap_or("main"),
    );
    init_render_thread();
    //init_worker_threads();
    com::println!(
        8.into(),
        "{}: ...succeeded",
        std::thread::current().name().unwrap_or("main"),
    );
}

pub fn begin_registration(_vid_config: &mut vid::Config) {
    sys::set_rg_registered_event();
    loop {
        if sys::query_rg_registered_event() == false {
            break;
        }
    }
}

pub fn begin_registration_internal() -> Result<(), ()> {
    if init().is_err() {
        return Err(());
    }
    sys::wait_rg_registered_event();
    Ok(())
}

pub fn begin_remote_screen_update() {
    if dvar::get_bool("useFastFile").unwrap()
        && sys::is_main_thread()
        && dvar::get_bool("sys_smp_allowed").unwrap()
    {
        assert!(R_GLOB.read().unwrap().remote_screen_update_nesting >= 0);
        if R_GLOB.read().unwrap().started_render_thread
            && (cl::local_client_is_in_game(0) == false
                || R_GLOB.read().unwrap().remote_screen_update_in_game != 0)
        {
            assert_ne!(R_GLOB.read().unwrap().screen_update_notify, false);
            R_GLOB.write().unwrap().remote_screen_update_nesting += 1;
            sys::notify_renderer();
        } else {
            R_GLOB.write().unwrap().remote_screen_update_nesting = 1;
        }
    }
}

fn register() {
    register_dvars();
}

fn reflection_probe_register_dvars() {
    dvar::register_bool(
        "r_reflectionProbeGenerate",
        false,
        dvar::DvarFlags::empty(),
        "Generate cube maps for reflection probes.".into(),
    )
    .unwrap();
}

const ASPECT_RATIO_AUTO: &str = "auto";
const ASPECT_RATIO_STANDARD: &str = "standard";
const ASPECT_RATIO_16_10: &str = "wide 16:10";
const ASPECT_RATIO_16_9: &str = "wide 16:9";

fn register_dvars() {
    dvar::register_bool(
        "r_fullscreen",
        false,
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::LATCHED,
        Some("Display game full screen"),
    )
    .unwrap();
    dvar::register_enumeration(
        "r_aspectRatio", 
        "auto".into(), 
        Some(vec![
            ASPECT_RATIO_AUTO.into(),
            ASPECT_RATIO_STANDARD.into(),
            ASPECT_RATIO_16_10.into(),
            ASPECT_RATIO_16_9.into()
            ]),
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::LATCHED,
        Some("Screen aspect ratio.  Most widescreen monitors are 16:10 instead of 16:9.")
    ).unwrap();
    dvar::register_int(
        "r_aaSamples",
        1,
        Some(1),
        Some(16),
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::LATCHED,
        Some("Anti-aliasing sample count; 1 disables anti-aliasing"),
    )
    .unwrap();
    dvar::register_bool(
        "r_vsync",
        true,
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::LATCHED,
        Some("Enable v-sync before drawing the next frame to avoid \'tearing\' artifacts.")
    ).unwrap();
    dvar::register_string(
        "r_customMode",
        "",
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::LATCHED,
        Some("Special resolution mode for the remote debugger"),
    )
    .unwrap();
    dvar::register_int(
        "vid_xpos",
        3,
        Some(-4096),
        4096.into(),
        dvar::DvarFlags::ARCHIVE,
        "Game window horizontal position".into(),
    )
    .unwrap();
    dvar::register_int(
        "vid_ypos",
        3,
        Some(-4096),
        4096.into(),
        dvar::DvarFlags::ARCHIVE,
        "game window vertical position".into(),
    )
    .unwrap();

    reflection_probe_register_dvars();
}

#[allow(clippy::unnecessary_wraps)]
fn init() -> Result<(), ()> {
    com::println!(8.into(), "----- render::init -----");

    register();

    init_graphics_api().unwrap();

    Ok(())
}

#[derive(Default)]
struct WindowGlobals {
    //current_monitor_handle: Option<winit::monitor::MonitorHandle>,
    //best_monitor_handle: Option<winit::monitor::MonitorHandle>,
    video_modes: Vec<VideoMode>,
    window_handle: Option<WindowHandle>,
}

lazy_static! {
    static ref WINDOW_GLOBALS: Arc<RwLock<WindowGlobals>> =
        Arc::new(RwLock::new(WindowGlobals {
            //current_monitor_handle: None,
           // best_monitor_handle: None,
            window_handle: None,
            video_modes: Vec::new(),
        }));
}

pub fn main_window_handle() -> Option<WindowHandle> {
    WINDOW_GLOBALS.read().unwrap().window_handle
}

pub struct RenderGlobals {
    adapter_native_width: u32,
    adapter_native_height: u32,
    adapter_fullscreen_width: u32,
    adapter_fullscreen_height: u32,
    resolution_names: HashSet<String>,
    refresh_rate_names: HashSet<String>,
    target_window_index: i32,
    device: Option<sys::gpu::Device>,
    adapter: Option<sys::gpu::Adapter>,
    instance: Option<sys::gpu::Instance>,
    windows: Vec<WindowTarget>,
}

impl RenderGlobals {
    pub fn new() -> Self {
        Self {
            adapter_native_width: MIN_HORIZONTAL_RESOLUTION,
            adapter_native_height: MIN_VERTICAL_RESOLUTION,
            adapter_fullscreen_width: MIN_HORIZONTAL_RESOLUTION,
            adapter_fullscreen_height: MIN_VERTICAL_RESOLUTION,
            resolution_names: HashSet::new(),
            refresh_rate_names: HashSet::new(),
            target_window_index: 0,
            device: None,
            adapter: None,
            instance: None,
            windows: Vec::new(),
        }
    }
}

impl Default for RenderGlobals {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static! {
    pub static ref RENDER_GLOBALS: RwLock<RenderGlobals> =
        RwLock::new(RenderGlobals::default());
}

fn fatal_init_error(error: impl ToString) -> ! {
    com::println!(8.into(), "********** Device returned an unrecoverable error code during initialization  **********");
    com::println!(8.into(), "********** Initialization also happens while playing if Renderer loses a device **********");
    com::println!(8.into(), "{}", error.to_string());
    sys::render_fatal_error();
}

#[derive(Clone)]
pub struct VideoMode {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u32,
    pub refresh: f32,
}

impl PartialEq for VideoMode {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.bit_depth == other.bit_depth
            && self.refresh == other.refresh
    }
}

impl Eq for VideoMode {}

impl Ord for VideoMode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.width
            .cmp(&other.width)
            .then(self.height.cmp(&other.height))
            .then(
                self.refresh
                    .total_cmp(&other.refresh)
                    .then(self.bit_depth.cmp(&other.bit_depth)),
            )
            .reverse()
    }
}

impl PartialOrd for VideoMode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn set_custom_resolution(
    wnd_parms: &mut gfx::WindowParms,
) -> Result<(), ()> {
    let custom_mode = dvar::get_string("r_customMode").unwrap();
    (wnd_parms.display_width, wnd_parms.display_height) =
        match scanf!(custom_mode, "{}x{}", u32, u32) {
            Err(_) => return Err(()),
            Ok((w, h)) => (w, h),
        };
    if let Some((width, height)) =
        get_monitor_dimensions(wnd_parms.monitor_handle.unwrap())
    {
        if width < wnd_parms.display_width as _
            || height < wnd_parms.display_height as _
        {
            Err(())
        } else {
            Ok(())
        }
    } else {
        Err(())
    }
}

#[allow(clippy::cast_possible_truncation)]
fn closest_refresh_rate_for_mode(
    width: u32,
    height: u32,
    hz: f32,
) -> Option<f32> {
    let video_modes =
        WINDOW_GLOBALS.clone().read().unwrap().video_modes.clone();
    if video_modes.is_empty() {
        return None;
    }
    let mode = video_modes.iter().find(|&m| {
        m.refresh == hz
            && m.width == u32::from(width)
            && m.height == u32::from(height)
    });

    if let Some(m) = mode {
        return Some(m.refresh / 1000.0);
    }

    let mode = video_modes.iter().find(|&m| m.refresh == hz);
    if let Some(m) = mode {
        return Some(m.refresh);
    }

    let mode = video_modes.iter().find(|&m| {
        m.width == u32::from(width) && m.height == u32::from(height)
    });

    if let Some(m) = mode {
        return Some(m.refresh / 1000.0);
    }

    None
}

#[allow(
    clippy::cast_sign_loss,
    clippy::std_instead_of_core,
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
fn set_wnd_parms(wnd_parms: &mut gfx::WindowParms) {
    let r_fullscreen = dvar::get_bool("r_fullscreen").unwrap();
    wnd_parms.fullscreen = r_fullscreen;

    if r_fullscreen && set_custom_resolution(wnd_parms).is_err() {
        let r_mode = dvar::get_enumeration("r_mode").unwrap();
        (wnd_parms.display_width, wnd_parms.display_height) =
            scanf!(r_mode, "{}x{}", u32, u32).unwrap();
    }

    let r_mode = dvar::get_enumeration("r_mode").unwrap();
    (wnd_parms.display_width, wnd_parms.display_height) =
        scanf!(r_mode, "{}x{}", u32, u32).unwrap();

    if !wnd_parms.fullscreen {
        let render_globals = RENDER_GLOBALS.read().unwrap();

        if render_globals.adapter_native_width < wnd_parms.display_width {
            wnd_parms.display_width = wnd_parms
                .display_width
                .clamp(0, render_globals.adapter_native_width);
        }
        if render_globals.adapter_native_height < wnd_parms.display_height {
            wnd_parms.display_height = wnd_parms
                .display_height
                .clamp(0, render_globals.adapter_native_height);
        }
    }

    wnd_parms.scene_width = wnd_parms.display_width;
    wnd_parms.scene_height = wnd_parms.display_height;

    if !wnd_parms.fullscreen {
        wnd_parms.hz = 60.0;
    } else {
        let hz = closest_refresh_rate_for_mode(
            wnd_parms.display_width,
            wnd_parms.display_height,
            wnd_parms.hz,
        )
        .unwrap();
        wnd_parms.hz = hz;
        dvar::set_string_internal("r_displayRefresh", &format!("{} Hz", hz))
            .unwrap();
    }

    wnd_parms.x = dvar::get_int("vid_xpos").unwrap().clamp(0, i32::MAX) as _;
    wnd_parms.y = dvar::get_int("vid_ypos").unwrap().clamp(0, i32::MAX) as _;
    wnd_parms.window_handle = None;
    wnd_parms.aa_samples =
        dvar::get_int("r_aaSamples").unwrap().clamp(0, i32::MAX) as _;
    wnd_parms.monitor_handle = primary_monitor();
}

#[allow(
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unnecessary_wraps,
    clippy::significant_drop_tightening
)]
fn store_window_settings(wnd_parms: &mut gfx::WindowParms) -> Result<(), ()> {
    let mut vid_config = vid::CONFIG.write().unwrap();

    vid_config.scene_width = wnd_parms.scene_width;
    vid_config.scene_height = wnd_parms.scene_height;
    vid_config.display_width = wnd_parms.display_width;
    vid_config.display_height = wnd_parms.display_height;
    vid_config.display_frequency = wnd_parms.hz;
    vid_config.is_fullscreen = wnd_parms.fullscreen;

    vid_config.aspect_ratio_window =
        match dvar::get_enumeration("r_aspectRatio").unwrap().as_str() {
            ASPECT_RATIO_AUTO => {
                let render_globals = RENDER_GLOBALS.write().unwrap();

                let (display_width, display_height) =
                    if vid_config.is_fullscreen {
                        (
                            render_globals.adapter_native_width,
                            render_globals.adapter_native_height,
                        )
                    } else {
                        (vid_config.display_width, vid_config.display_height)
                    };

                if (display_width as f32 / display_height as f32 - 16.0 / 10.0)
                    .abs()
                    < f32::EPSILON
                {
                    16.0 / 10.0
                } else if display_width as f32 / display_height as f32
                    > 16.0 / 10.0
                {
                    16.0 / 9.0
                } else {
                    4.0 / 3.0
                }
            }
            ASPECT_RATIO_STANDARD => 4.0 / 3.0,
            ASPECT_RATIO_16_10 => 16.0 / 10.0,
            ASPECT_RATIO_16_9 => 16.0 / 9.0,
            _ => panic!(
                "unhandled case, aspectRatio = {}",
                dvar::get_enumeration("r_aspectRatio").unwrap()
            ),
        };

    dvar::set_bool_internal(
        "wideScreen",
        (vid_config.aspect_ratio_window - 4.0 / 3.0).abs() > f32::EPSILON,
    )
    .unwrap();
    vid_config.aspect_ratio_scene_pixel = (vid_config.scene_height as f32
        * vid_config.aspect_ratio_window)
        / vid_config.scene_width as f32;

    let render_globals = RENDER_GLOBALS.write().unwrap();

    vid_config.aspect_ratio_display_pixel = if !vid_config.is_fullscreen {
        1.0
    } else {
        (render_globals.adapter_fullscreen_height as f32
            * vid_config.aspect_ratio_window)
            / render_globals.adapter_fullscreen_width as f32
    };

    vid_config.is_tool_mode = dvar::get_bool("r_reflectionProbeGenerate")
        .map_or(false, |enabled| enabled);

    Ok(())
}

#[allow(clippy::collapsible_else_if)]
fn reduce_window_settings() -> Result<(), ()> {
    if dvar::get_int("r_aaSamples").unwrap() > 1 {
        dvar::set_int("r_aaSamples", dvar::get_int("r_aaSamples").unwrap() - 1)
    } else {
        if dvar::get_enumeration("r_displayRefresh")
            .unwrap()
            .is_empty()
            || vid::config().display_frequency < 60.0
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
    }
}

#[allow(clippy::unnecessary_wraps)]
fn choose_adapter() -> Option<sys::gpu::Adapter> {
    let rg = RENDER_GLOBALS.write().unwrap();
    let adapter =
        block_on(sys::gpu::Adapter::new(rg.instance.as_ref().unwrap(), None));
    Some(adapter)
}

cfg_if! {
    if #[cfg(windows)] {
        fn available_monitors() -> VecDeque<MonitorHandle> {
            let mut monitors: VecDeque<MonitorHandle> = VecDeque::new();
            unsafe {
                EnumDisplayMonitors(
                    None,
                    None,
                    Some(monitor_enum_proc),
                    LPARAM(&mut monitors as *mut _ as _),
                );
            }
            monitors
        }

        fn primary_monitor() -> Option<MonitorHandle> {
            const ORIGIN: POINT = POINT { x: 0, y: 0 };
            let hmonitor = unsafe { MonitorFromPoint(ORIGIN, MONITOR_DEFAULTTOPRIMARY) };
            Some(MonitorHandle::Win32(WindowsDisplayHandle::empty(), hmonitor.0))
        }

        fn current_monitor(handle: WindowHandle) -> Option<MonitorHandle> {
            let hmonitor = unsafe { MonitorFromWindow(HWND(handle.get_win32().unwrap().hwnd as _), MONITOR_DEFAULTTONEAREST) };
            Some(MonitorHandle::Win32(WindowsDisplayHandle::empty(), hmonitor.0))
        }

        #[repr(C)]
        struct MonitorEnumData {
            monitor: i32,
            handle: HMONITOR,
        }

        unsafe extern "system" fn monitor_enum_callback(hmonitor: HMONITOR, _hdc: HDC, _rect: *mut RECT, data: LPARAM) -> BOOL {
            let data = data.0 as *mut MonitorEnumData;
            (*data).monitor -= 1;
            if (*data).monitor == 0 {
                (*data).handle = hmonitor;
            }
            BOOL(((*data).monitor != 0) as _)
        }

        fn choose_monitor() -> MonitorHandle {
            let fullscreen = dvar::get_bool("r_fullscreen").unwrap();
            if fullscreen {
                let monitor = dvar::get_int("r_monitor").unwrap();
                let mut data = MonitorEnumData { monitor, handle: HMONITOR(0) };
                unsafe { EnumDisplayMonitors(None, None, Some(monitor_enum_callback), LPARAM(addr_of_mut!(data) as _)) };
                if data.handle != HMONITOR(0) {
                    return MonitorHandle::Win32(WindowsDisplayHandle::empty(), data.handle.0);
                }
            }

            let xpos = dvar::get_int("vid_xpos").unwrap();
            let ypos = dvar::get_int("vid_ypos").unwrap();
            let hmonitor = unsafe { MonitorFromPoint(POINT { x: xpos, y: ypos }, MONITOR_DEFAULTTOPRIMARY) };
            MonitorHandle::Win32(WindowsDisplayHandle::empty(), hmonitor.0)
        }

        fn get_monitor_dimensions(monitor_handle: MonitorHandle) -> Option<(u32, u32)> {
            let mut mi = MONITORINFOEXW::default();
            mi.monitorInfo.cbSize = size_of_val(&mi) as _;
            unsafe { GetMonitorInfoW(monitor_handle.get_win32().unwrap().1, addr_of_mut!(mi.monitorInfo)) };

            let mi_width = (mi.monitorInfo.rcMonitor.right - mi.monitorInfo.rcMonitor.left) as u32;
            let mi_height = (mi.monitorInfo.rcMonitor.bottom - mi.monitorInfo.rcMonitor.top) as u32;
            let width = if mi_width > 0 {
                mi_width
            } else {
                let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
                if width == 0 {
                    return None;
                } else {
                    width as _
                }
            };
            let height = if mi_width > 0 {
                mi_height
            } else {
                let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
                if height == 0 {
                    return None;
                } else {
                    height as _
                }
            };

            Some((width, height))
        }

        fn monitor_info(monitor_handle: MonitorHandle) -> Option<MonitorInfo> {
            let mut mi = MONITORINFOEXW::default();
            mi.monitorInfo.cbSize = size_of_val(&mi) as _;
            unsafe { GetMonitorInfoW(monitor_handle.get_win32().unwrap().1, addr_of_mut!(mi.monitorInfo)) };
            let name = char::decode_utf16(mi.szDevice).flatten().collect::<String>();

            let mut mode = DEVMODEW::default();
            mode.dmSize = size_of_val(&mode) as _;
            if unsafe { EnumDisplaySettingsExW(PCWSTR(mi.szDevice.as_ptr()), ENUM_CURRENT_SETTINGS, addr_of_mut!(mode), ENUM_DISPLAY_SETTINGS_FLAGS(0)) }.ok().is_err() {
                return None;
            };
            let refresh = mode.dmDisplayFrequency as f32;

            let mut modes = BTreeSet::new();
            let mut i = 0;
            loop {
                if unsafe { EnumDisplaySettingsExW(
                    PCWSTR(mi.szDevice.as_ptr()),
                    ENUM_DISPLAY_SETTINGS_MODE(i),
                    addr_of_mut!(mode),
                    ENUM_DISPLAY_SETTINGS_FLAGS(0))
                }.as_bool() == false {
                    break;
                }
                i += 1;

                const REQUIRED_FIELDS: DEVMODE_FIELD_FLAGS = DEVMODE_FIELD_FLAGS(DM_BITSPERPEL.0 | DM_PELSWIDTH.0 | DM_PELSHEIGHT.0 | DM_DISPLAYFREQUENCY.0);
                assert!(mode.dmFields.contains(REQUIRED_FIELDS));
                modes.insert(VideoMode {
                    width: mode.dmPelsWidth,
                    height: mode.dmPelsHeight,
                    bit_depth: mode.dmBitsPerPel as _,
                    refresh: mode.dmDisplayFrequency as _,
                });
            }

            let (width, height) = get_monitor_dimensions(monitor_handle)?;
            Some(MonitorInfo { name, width, height, refresh: refresh as _, video_modes: modes.into_iter().collect() })

        }

        pub fn create_window_2(wnd_parms: &mut gfx::WindowParms) -> Result<(), ()> {
            assert!(wnd_parms.window_handle.is_none());

            let (dw_ex_style, dw_style) = if wnd_parms.fullscreen == false {
                com::println!(8.into(), "Attempting {} x {} window at ({}, {})", wnd_parms.display_width, wnd_parms.display_height, wnd_parms.x, wnd_parms.y);
                (WS_EX_LEFT, WS_SYSMENU | WS_CAPTION | WS_VISIBLE)
            } else {
                com::println!(8.into(), "Attempting {} x {} fullscreen with 32 bpp at {} hz", wnd_parms.display_width, wnd_parms.display_height, wnd_parms.hz);
                (WS_EX_TOPMOST, WS_POPUP)
            };

            let mut rect = RECT { left: 0, right: wnd_parms.display_width as _, top: 0, bottom: wnd_parms.display_height as _ };
            unsafe { AdjustWindowRectEx(addr_of_mut!(rect), dw_style, false, dw_ex_style) };
            let hinstance = unsafe { GetModuleHandleA(None) }.unwrap_or_default();
            let height = rect.bottom - rect.top;
            let width = rect.right - rect.left;
            let window_name = CString::new(com::get_official_build_name_r()).unwrap();
            let hwnd = unsafe { CreateWindowExA(
                dw_ex_style,
                s!("CoDBlackOps"),
                PCSTR(window_name.as_ptr() as _),
                dw_style,
                wnd_parms.x as _,
                wnd_parms.y as _,
                width,
                height,
                None,
                None,
                hinstance,
                None)
            };

            if hwnd.0 == 0 {
                com::println!(8.into(), "Couldn't create a window.");
                wnd_parms.window_handle = None;
                Err(())
            } else {
                let mut handle = Win32WindowHandle::empty();
                handle.hinstance = hinstance.0 as _;
                handle.hwnd = hwnd.0 as _;
                wnd_parms.window_handle = Some(WindowHandle(RawWindowHandle::Win32(handle)));

                if wnd_parms.fullscreen == false {
                    unsafe { SetWindowPos(hwnd, HWND_NOTOPMOST, 0, 0, 0, 0, SWP_NOSIZE | SWP_NOMOVE) };
                    unsafe { SetFocus(hwnd) };
                }
                com::println!(8.into(), "Game window successfully created.");
                Ok(())
            }
        }
    } else if #[cfg(unix)] {
        fn available_monitors() -> VecDeque<MonitorHandle> {
            // I can't figure out how to get the display pointer from x11rb,
            // so we're pulling in the x11 crate for now.
            // Fix this as soon as possible.
            let display = unsafe { XOpenDisplay(core::ptr::null_mut()) };
            let (conn, _) = x11rb::connect(None).unwrap();
            conn.setup().roots.iter().map(|screen| {
                let mut handle = XlibDisplayHandle::empty();
                handle.display = display as _;
                handle.screen = screen.root as _;
                MonitorHandle::Xlib(handle)
            }).collect()
        }

        fn primary_monitor() -> Option<MonitorHandle> {
            None
        }

        fn current_monitor(_: WindowHandle) -> Option<MonitorHandle> {
            let display = unsafe { XOpenDisplay(core::ptr::null_mut()) };
            let (_, screen_num) = x11rb::connect(None).unwrap();
            let mut handle = XlibDisplayHandle::empty();
            handle.display = display as _;
            handle.screen = screen_num as _;
            Some(MonitorHandle::Xlib(handle))
        }

        fn choose_monitor() -> MonitorHandle {
            let fullscreen = dvar::get_bool("r_fullscreen").unwrap();
            if fullscreen {
                let monitor = dvar::get_int("r_monitor").unwrap();
                //let mut data = MonitorEnumData { monitor, handle: HMONITOR(0) };
                //unsafe { EnumDisplayMonitors(None, None, Some(monitor_enum_callback), LPARAM(addr_of_mut!(data) as _)) };
                //if data.handle != HMONITOR(0) {
                //    return MonitorHandle::Win32(data.handle);
                //}
                let mut handle = XlibDisplayHandle::empty();
                //handle.display = display as _;
                //handle.screen = screen.root as _;
                return MonitorHandle::Xlib(handle)
            }

            let xpos = dvar::get_int("vid_xpos").unwrap();
            let ypos = dvar::get_int("vid_ypos").unwrap();
            //let hmonitor = unsafe { MonitorFromPoint(POINT { x: xpos, y: ypos }, MONITOR_DEFAULTTOPRIMARY) };
            //MonitorHandle::Win32(hmonitor)
            let mut handle = XlibDisplayHandle::empty();
            //handle.display = display as _;
            //handle.screen = screen.root as _;
            MonitorHandle::Xlib(handle)
        }

        fn get_monitor_dimensions(monitor_handle: MonitorHandle) -> Option<(u32, u32)> {
            None
        }

        fn monitor_info(monitor_handle: MonitorHandle) -> Option<MonitorInfo> {
            None
        }

        pub fn create_window_2(wnd_parms: &mut gfx::WindowParms) -> Result<(), ()> {
            assert!(wnd_parms.window_handle.is_none());

            let (conn, screen_num) = x11rb::connect(None).unwrap();
            let screen = &conn.setup().roots[screen_num];
            let win_id = conn.generate_id().unwrap();
            let r = if let Err(e) = conn.create_window(
                COPY_DEPTH_FROM_PARENT,
                win_id,
                screen.root,
                wnd_parms.x as _,
                wnd_parms.y as _,
                wnd_parms.display_width as _,
                wnd_parms.display_height as _,
                0,
                WindowClass::INPUT_OUTPUT,
                0,
                &CreateWindowAux::new().background_pixel(screen.white_pixel),
            ) {
                com::println!(8.into(), "Couldn't create a window.");
                wnd_parms.window_handle = None;
                Err(())
            } else {
                let mut handle = XlibWindowHandle::empty();
                handle.window = win_id as _;
                handle.visual_id = 0;
                //wnd_parms.window_handle = Some(WindowHandle(RawWindowHandle::Xlib(handle)));

                if wnd_parms.fullscreen == false {
                    conn.set_input_focus(InputFocus::PARENT, win_id, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u32).unwrap();
                }
                com::println!(8.into(), "Game window successfully created.");
                Ok(())
            };

            r
        }
    }
}

struct MonitorInfo {
    name: String,
    width: u32,
    height: u32,
    refresh: u32,
    video_modes: Vec<VideoMode>,
}

fn enum_display_modes() {
    let info =
        monitor_info(primary_monitor().unwrap_or(available_monitors()[0]))
            .unwrap();

    let valid_modes = info
        .video_modes
        .iter()
        .filter(|m| {
            m.width >= MIN_HORIZONTAL_RESOLUTION as _
                && m.height >= MIN_VERTICAL_RESOLUTION as _
        })
        .collect::<Vec<_>>();

    let modes = valid_modes
        .iter()
        .map(|m| format!("{}x{}", m.width, m.height))
        .collect::<Vec<_>>();
    if modes.len() == 0 {
        fatal_init_error(format!(
            "No valid resolutions of {} x {} or above found",
            MIN_HORIZONTAL_RESOLUTION, MIN_VERTICAL_RESOLUTION
        ));
    }

    dvar::register_enumeration(
        "r_mode",
        modes.iter().nth(0).unwrap().clone(),
        Some(modes),
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::LATCHED,
        "Renderer resolution mode".into(),
    )
    .unwrap();

    let refreshes = info
        .video_modes
        .iter()
        .map(|m| format!("{} Hz", m.refresh))
        .collect::<Vec<_>>();
    dvar::register_enumeration(
        "r_displayRefresh",
        refreshes.iter().nth(0).unwrap().clone(),
        Some(refreshes),
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::LATCHED,
        "Refresh rate".into(),
    )
    .unwrap();
}

#[allow(clippy::significant_drop_tightening)]
fn pre_create_window() -> Result<(), ()> {
    com::println!(8.into(), "Getting Device interface...");
    let instance = sys::gpu::Instance::new();
    RENDER_GLOBALS.write().unwrap().instance = Some(instance);

    let adapter = choose_adapter();
    enum_display_modes();
    RENDER_GLOBALS.write().unwrap().adapter = adapter;

    Ok(())
}

/*
#[allow(
    clippy::as_conversions,
    clippy::items_after_statements,
    clippy::pattern_type_mismatch,
    clippy::if_then_some_else_none,
    clippy::semicolon_outside_block,
    clippy::indexing_slicing,
    clippy::std_instead_of_core,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::integer_division,
    clippy::too_many_lines,
    clippy::expect_used,
    clippy::significant_drop_tightening,
    clippy::panic_in_result_fn
)]
pub fn create_window_2(wnd_parms: &mut gfx::WindowParms) -> Result<(), ()> {
    WINDOW_AWAITING_INIT.lock().unwrap().send_cleared(());
    WINDOW_INITIALIZING.lock().unwrap().send(());

    if wnd_parms.fullscreen {
        com::println!(
            8.into(),
            "Attempting {} x {} fullscreen with 32 bpp at {} hz",
            wnd_parms.display_width,
            wnd_parms.display_height,
            wnd_parms.hz,
        );
    } else {
        com::println!(
            8.into(),
            "Attempting {} x {} window at ({}, {})",
            wnd_parms.display_width,
            wnd_parms.display_height,
            wnd_parms.x,
            wnd_parms.y,
        );
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

    let event_loop = EventLoop::new();
    let main_window = match WindowBuilder::new()
        .with_title(window_name)
        .with_position(PhysicalPosition::<i32>::new(i32::from(x), i32::from(y)))
        .with_inner_size(PhysicalSize::new(width, height))
        .with_resizable(true)
        .with_visible(false)
        .with_decorations(!fullscreen)
        .with_window_icon(com::get_icon_rgba())
        .build(&event_loop)
    {
        Ok(w) => w,
        Err(e) => {
            com::println!(8.into(), "Couldn't create a window.");
            com::dprintln!(8.into(), "{}", e);
            WINDOW_INITIALIZING.lock().unwrap().clone().send_cleared(());
            WINDOW_INITIALIZED.lock().unwrap().clone().send(false);
            return Err(());
        }
    };

    main_window.set_visible(true);

    if fullscreen == false {
        main_window.focus_window();
    }

    {
        let lock = WINIT_GLOBALS.clone();
        let mut wg = lock.write().unwrap();
        wg.window_handle = Some(main_window.window_handle());
    }

    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            use winit::dpi::PhysicalSize;
            main_window.set_inner_size(PhysicalSize::new(MIN_HORIZONTAL_RESOLUTION, MIN_VERTICAL_RESOLUTION));

            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("open_t5")?;
                    let canvas = web_sys::Element::from(main_window.canvas());
                    dst.append_child(&canvas).ok()?;
                 Some(())
                })
                .expect("Couldn't append canvas to document body."
            );
        }
    }

    com::println!(8.into(), "Game window successfully created.");

    // ========================================================================
    cfg_if! {
        if #[cfg(not(target_arch = "wasm32"))] {
            // This part is supposed to be done in sys::create_console, but
            // you can't bind windows to an event loop after calling
            // event_loop::run, so instead we create them here, set them to
            // invisible, and then set them to visible in sys::create_console
            // instead of creating them there.
            //
            // I'm not entirely sure how we're going to implement the console
            // for other platforms, so this logic might end up being handled
            // with, e.g., GTK, instead, but for now we're just going to keep
            // things simple. If we have to move things around later, we can.

            let console_title = com::get_build_display_name();
            let monitor = main_window
                .current_monitor()
                .or_else(|| main_window.available_monitors().nth(0))
                .unwrap();
            let horzres = (monitor.size().width - 450) / 2;
            assert_ne!(horzres, 0, "Horizontal resolution should never be zero. It's a logic error, and it'll cause invalid calcultations.");
            let vertres = (monitor.size().height - 600) / 2;
            assert_ne!(vertres, 0, "Vertical resolution should never be zero. It's a logic error, and it'll cause invalid calcultations.");
            let console_width = conbuf::s_wcd_window_width();
            assert_ne!(console_width, 0, "Console window width should not be zero. It's a logic error, and it causes a runtime panic with X.");
            let console_height = conbuf::s_wcd_window_height();
            assert_ne!(console_height, 0, "Console window height should not be zero. It's a logic error, and it causes a runtime panic with X.");
            let console_window = winit::window::WindowBuilder::new()
                .with_title(console_title)
                .with_position(Position::Physical(PhysicalPosition::new(
                    horzres.clamp(0, u32::MAX) as _,
                    vertres.clamp(0, u32::MAX) as _,
                )))
                .with_inner_size(PhysicalSize::new(console_width, console_height))
                .with_visible(false)
                .build(&event_loop)
                .unwrap();

            conbuf::s_wcd_set_window(console_window);

            const CODLOGO_POS_X: i32 = 5;
            const CODLOGO_POS_Y: i32 = 5;
            const INPUT_LINE_POS_X: i32 = 6;
            const INPUT_LINE_POS_Y: i32 = 400;
            const INPUT_LINE_SIZE_W: i32 = 608;
            const INPUT_LINE_SIZE_H: i32 = 20;
            const BUFFER_POS_X: i32 = 6;
            const BUFFER_POS_Y: i32 = 70;
            const BUFFER_SIZE_W: i32 = 606;
            const BUFFER_SIZE_H: i32 = 324;

            let parent = Some(conbuf::s_wcd_window_handle());
            // SAFETY:
            // Assuming the state of the program is otherwise valid,
            // the parent window being passed will be valid.
            let cod_logo_window = unsafe {
                winit::window::WindowBuilder::new()
                    .with_parent_window(parent)
                    .with_position(PhysicalPosition::new(CODLOGO_POS_X, CODLOGO_POS_Y))
                    .with_decorations(false)
                    .with_visible(false)
                    .build(&event_loop)
                    .unwrap()
            };

            // SAFETY:
            // Assuming the state of the program is otherwise valid,
            // the parent window being passed will be valid.
            let input_line_window = unsafe {
                winit::window::WindowBuilder::new()
                    .with_parent_window(parent)
                    .with_position(PhysicalPosition::new(
                        INPUT_LINE_POS_X,
                        INPUT_LINE_POS_Y,
                    ))
                    .with_inner_size(PhysicalSize::new(
                        INPUT_LINE_SIZE_H,
                        INPUT_LINE_SIZE_W,
                    ))
                    .with_visible(false)
                    .build(&event_loop)
                    .unwrap()
            };

            // SAFETY:
            // Assuming the state of the program is otherwise valid,
            // the parent window being passed will be valid.
            let buffer_window = unsafe {
                winit::window::WindowBuilder::new()
                    .with_parent_window(parent)
                    .with_position(PhysicalPosition::new(BUFFER_POS_X, BUFFER_POS_Y))
                    .with_inner_size(PhysicalSize::new(BUFFER_SIZE_H, BUFFER_SIZE_W))
                    .with_visible(false)
                    .build(&event_loop)
                    .unwrap()
            };

            conbuf::s_wcd_set_cod_logo_window(cod_logo_window);
            conbuf::s_wcd_set_input_line_window(input_line_window);
            conbuf::s_wcd_set_buffer_window(buffer_window);
        }
    }
    // ========================================================================

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == main_window.id() => match event {
            WindowEvent::MouseWheel {
                delta,
                ..
            } => {
                #[allow(clippy::panic)]
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
                            true
                        ),
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
                WindowEvent::KeyboardInput {
                input,
                ..
            } => {
                #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
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
                        dvar::get_int("developer").unwrap() != 0
                    {
                        // FUN_005a5360()
                        dvar::set_bool_internal(
                            "r_fullscreen",
                            !dvar::get_bool("r_fullscreen")
                                .unwrap()).unwrap();
                            cbuf::add_textln(0, "vid_restart");
                    }
                        // FUN_0053f880()
                } else {

                }
            },
            WindowEvent::Resized(size) => {
                dvar::make_latched_value_current("r_aspectRatio").unwrap();
                dvar::make_latched_value_current("r_aaSamples").unwrap();
                dvar::make_latched_value_current("r_vsync").unwrap();
                dvar::make_latched_value_current("r_fullscreen").unwrap();
                dvar::make_latched_value_current("r_displayRefresh").unwrap();
                let mut wnd_parms = gfx::WindowParms::new();
                let width = size.width;
                let height = size.height;
                let old_mode = dvar::get_enumeration("r_mode").unwrap();
                let new_mode = format!("{}x{}", width, height);
                dvar::add_to_enumeration_domain("r_mode", &new_mode).unwrap();
                dvar::set_enumeration_internal("r_mode", &new_mode).unwrap();
                dvar::remove_from_enumeration_domain("r_mode", &old_mode).unwrap();
                set_wnd_parms(&mut wnd_parms);
                store_window_settings(&mut wnd_parms).unwrap();
                set_wnd_parms(&mut wnd_parms);
                {
                    let mut render_globals = RENDER_GLOBALS.write().unwrap();
                    render_globals.window.width = wnd_parms.display_width;
                    render_globals.window.height = wnd_parms.display_height;
                }
                if !wnd_parms.fullscreen {
                    com::println!(8.into(), "Resizing {} x {} window at ({}, {})", wnd_parms.display_width, wnd_parms.display_height, wnd_parms.x, wnd_parms.y);
                } else {
                    com::println!(8.into(), "Resizing {} x {} fullscreen at ({}, {})", wnd_parms.display_width, wnd_parms.display_height, wnd_parms.x, wnd_parms.y);
                }
            },
            _ => {}
        },
        _ => {}
    });
}
*/

static HARDWARE_INITED: AtomicBool = AtomicBool::new(false);

#[allow(clippy::unnecessary_wraps)]
fn init_hardware(wnd_parms: &mut gfx::WindowParms) -> Result<(), ()> {
    store_window_settings(wnd_parms).unwrap();
    if HARDWARE_INITED.load(Ordering::Relaxed) == false {
        finish_attaching_to_window(wnd_parms);
    }

    Ok(())
}

#[allow(clippy::semicolon_outside_block)]
pub fn create_window(wnd_parms: &mut gfx::WindowParms) -> Result<(), ()> {
    create_window_2(wnd_parms)?;
    init_hardware(wnd_parms)?;
    RENDER_GLOBALS.write().unwrap().target_window_index = 0;
    show_window(wnd_parms.window_handle.unwrap());
    Ok(())
}

// TODO - implement
#[allow(clippy::unnecessary_wraps)]
const fn init_systems() -> Result<(), ()> {
    Ok(())
}

fn init_graphics_api() -> Result<(), ()> {
    if RENDER_GLOBALS.read().unwrap().device.is_none() {
        if pre_create_window().is_err() {
            return Err(());
        }

        let mut wnd_parms: gfx::WindowParms = gfx::WindowParms::new();
        loop {
            set_wnd_parms(&mut wnd_parms);
            if create_window(&mut wnd_parms).is_ok() {
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

fn finish_attaching_to_window(wnd_parms: &gfx::WindowParms) {
    let mut rg = RENDER_GLOBALS.write().unwrap();
    assert_eq!(rg.windows.len(), 0);
    // swapchain
    let window = WindowTarget {
        handle: wnd_parms.window_handle,
        width: wnd_parms.display_width,
        height: wnd_parms.display_height,
    };
    rg.windows.push(window);
    HARDWARE_INITED.store(true, Ordering::Relaxed);
}

fn create_device_internal(wnd_parms: &gfx::WindowParms) -> Result<(), ()> {
    com::println!(8.into(), "Creating Render device...");

    let mut rg = RENDER_GLOBALS.write().unwrap();
    rg.device = block_on(Device::new(rg.adapter.as_ref().unwrap()));

    (rg.adapter_native_width, rg.adapter_native_height) =
        get_monitor_dimensions(wnd_parms.monitor_handle.unwrap()).unwrap();
    rg.device = block_on(sys::gpu::Device::new(rg.adapter.as_ref().unwrap()));
    if rg.device.is_none() {
        return Err(());
    }

    Ok(())
}

fn create_device(wnd_parms: &gfx::WindowParms) -> Result<(), ()> {
    {
        let rg = RENDER_GLOBALS.read().unwrap();
        assert_ne!(rg.windows.len(), 0);
        assert_ne!(wnd_parms.window_handle, None);
        assert!(!rg.device.is_none());
    }

    // depth stencil

    if create_device_internal(wnd_parms).is_err() {
        com::print_errorln!(8.into(), "Couldn't create a Renderer device");
        return Err(());
    }

    let rg = RENDER_GLOBALS.read().unwrap();
    assert!(rg.device.is_some());
    Ok(())
}
