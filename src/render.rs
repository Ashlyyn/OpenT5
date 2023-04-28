#![allow(dead_code)]

use crate::gfx::{WindowTarget, R_GLOB};
use crate::platform::os::target::MonitorHandle;
use crate::sys::gpu::Device;
use crate::sys::show_window;
use crate::util::{EasierAtomic, SignalState};
use crate::{platform::WindowHandle, *};
use pollster::block_on;
use raw_window_handle::RawWindowHandle;
use sscanf::scanf;
extern crate alloc;
use alloc::collections::VecDeque;
use alloc::ffi::CString;
use core::ptr::addr_of_mut;
use core::sync::atomic::AtomicUsize;
use std::collections::HashSet;
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
        use std::mem::size_of_val;
        use alloc::collections::BTreeSet;
        use raw_window_handle::Win32WindowHandle;
    } else if #[cfg(unix)] {
        use core::slice;
        use x11::xlib::XStoreName;
        use x11::xlib::{XOpenDisplay};
        use raw_window_handle::XlibWindowHandle;
        use raw_window_handle::XlibDisplayHandle;
        use x11::xlib::{XDefaultScreen, XCreateSimpleWindow, XDefaultVisual, XScreenCount, XRootWindow, XScreenOfDisplay, XWhitePixel, XWidthOfScreen, XHeightOfScreen, XDestroyWindow, XDefaultDepth, XSetInputFocus, RevertToParent, CurrentTime, XVisualIDFromVisual};
        use x11::xrandr::{XRRGetMonitors, XRRFreeMonitors, XRRConfigCurrentRate, XRRGetScreenInfo, XRRFreeScreenConfigInfo, XRRConfigSizes, XRRConfigRates};
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
        if sys::query_rg_registered_event() == SignalState::Cleared {
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

static G_MAIN_THREAD_BLOCKED: AtomicUsize = AtomicUsize::new(0);

pub fn end_remote_screen_update() {
    end_remote_screen_update_with(|| {})
}

pub fn end_remote_screen_update_with(f: impl Fn()) {
    if dvar::get_bool("useFastFile").unwrap() == false
        || !sys::is_main_thread()
        || dvar::get_bool("sys_smp_allowed").unwrap() == false
    {
        return;
    }

    assert!(R_GLOB.read().unwrap().remote_screen_update_nesting >= 0);

    if R_GLOB.read().unwrap().started_render_thread == false {
        assert_eq!(R_GLOB.read().unwrap().remote_screen_update_nesting, 0);
        return;
    }

    if cl::local_client_is_in_game(0)
        && R_GLOB.read().unwrap().remote_screen_update_in_game == 0
    {
        assert_eq!(R_GLOB.read().unwrap().remote_screen_update_nesting, 0);
        return;
    }

    assert!(R_GLOB.read().unwrap().remote_screen_update_nesting > 0);

    if R_GLOB.read().unwrap().remote_screen_update_nesting != 1 {
        R_GLOB.write().unwrap().remote_screen_update_nesting -= 1;
        return;
    }

    while R_GLOB.read().unwrap().screen_update_notify == false {
        net::sleep(Duration::from_millis(1));
        f();
        sys::wait_renderer();
    }
    R_GLOB.write().unwrap().screen_update_notify = false;
    assert!(R_GLOB.read().unwrap().remote_screen_update_nesting > 0);
    R_GLOB.write().unwrap().remote_screen_update_nesting -= 1;
    while R_GLOB.read().unwrap().screen_update_notify == false {
        G_MAIN_THREAD_BLOCKED.increment_wrapping();
        net::sleep(Duration::from_millis(1));
        f();
        sys::wait_renderer();
        G_MAIN_THREAD_BLOCKED.decrement_wrapping();
    }
    R_GLOB.write().unwrap().screen_update_notify = false;
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

pub struct RenderGlobals {
    adapter_native_width: u32,
    adapter_native_height: u32,
    adapter_fullscreen_width: u32,
    adapter_fullscreen_height: u32,
    video_modes: Vec<VideoMode>,
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
            video_modes: Vec::new(),
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

#[allow(clippy::needless_pass_by_value)]
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

#[allow(clippy::missing_trait_methods)]
impl PartialEq for VideoMode {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.bit_depth == other.bit_depth
            && self.refresh == other.refresh
    }
}

#[allow(clippy::missing_trait_methods)]
impl Eq for VideoMode {}

#[allow(clippy::missing_trait_methods)]
impl Ord for VideoMode {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
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

#[allow(clippy::missing_trait_methods)]
impl PartialOrd for VideoMode {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(
    clippy::std_instead_of_core,
    clippy::indexing_slicing,
    clippy::manual_let_else
)]
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
    let video_modes = RENDER_GLOBALS.read().unwrap().video_modes.clone();
    if video_modes.is_empty() {
        return None;
    }
    let mode = video_modes.iter().find(|&m| {
        (m.refresh - hz).abs() < f32::EPSILON
            && m.width == width
            && m.height == height
    });

    if let Some(m) = mode {
        return Some(m.refresh / 1000.0);
    }

    let mode = video_modes
        .iter()
        .find(|&m| (m.refresh - hz).abs() < f32::EPSILON);
    if let Some(m) = mode {
        return Some(m.refresh);
    }

    let mode = video_modes
        .iter()
        .find(|&m| m.width == width && m.height == height);

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
    wnd_parms.monitor_handle = Some(primary_monitor().unwrap_or_else(|| {
        current_monitor(None).unwrap_or_else(|| {
            primary_monitor()
                .unwrap_or_else(|| *available_monitors().get(0).unwrap())
        })
    }));
}

#[allow(
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unnecessary_wraps,
    clippy::cast_precision_loss
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

// All uses of unsafe in the following cfg_if! block are just for FFI,
// and all of those functions should be safe. No reason to comment them
// individually.
cfg_if! {
    if #[cfg(windows)] {
        #[allow(clippy::undocumented_unsafe_blocks)]
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

        #[allow(clippy::undocumented_unsafe_blocks)]
        fn primary_monitor() -> Option<MonitorHandle> {
            const ORIGIN: POINT = POINT { x: 0, y: 0 };
            let hmonitor = unsafe { MonitorFromPoint(ORIGIN, MONITOR_DEFAULTTOPRIMARY) };
            Some(MonitorHandle::Win32(hmonitor.0))
        }

        #[allow(clippy::undocumented_unsafe_blocks)]
        fn current_monitor(handle: Option<WindowHandle>) -> Option<MonitorHandle> {
            if let Some(handle) = handle {
                let hmonitor = unsafe { MonitorFromWindow(HWND(handle.get_win32().unwrap().hwnd as _), MONITOR_DEFAULTTONEAREST) };
                Some(MonitorHandle::Win32(hmonitor.0))
            } else {
                None
            }
        }

        #[repr(C)]
        struct MonitorEnumData {
            monitor: i32,
            handle: HMONITOR,
        }

        #[allow(clippy::undocumented_unsafe_blocks)]
        unsafe extern "system" fn monitor_enum_callback(hmonitor: HMONITOR, _hdc: HDC, _rect: *mut RECT, data: LPARAM) -> BOOL {
            // SAFETY: This is safe because the only time we ever call this
            // function, we wrap a *mut MonitorEnumData in `data`
            let data = data.0 as *mut MonitorEnumData;
            (*data).monitor -= 1;
            if (*data).monitor == 0 {
                (*data).handle = hmonitor;
            }
            BOOL(((*data).monitor != 0) as _)
        }

        #[allow(clippy::undocumented_unsafe_blocks)]
        fn choose_monitor() -> MonitorHandle {
            let fullscreen = dvar::get_bool("r_fullscreen").unwrap();
            if fullscreen {
                let monitor = dvar::get_int("r_monitor").unwrap();
                let mut data = MonitorEnumData { monitor, handle: HMONITOR(0) };
                unsafe { EnumDisplayMonitors(None, None, Some(monitor_enum_callback), LPARAM(addr_of_mut!(data) as _)) };
                if data.handle != HMONITOR(0) {
                    return MonitorHandle::Win32(data.handle.0);
                }
            }

            let xpos = dvar::get_int("vid_xpos").unwrap();
            let ypos = dvar::get_int("vid_ypos").unwrap();
            let hmonitor = unsafe { MonitorFromPoint(POINT { x: xpos, y: ypos }, MONITOR_DEFAULTTOPRIMARY) };
            MonitorHandle::Win32(hmonitor.0)
        }

        #[allow(clippy::undocumented_unsafe_blocks)]
        fn get_monitor_dimensions(monitor_handle: MonitorHandle) -> Option<(u32, u32)> {
            let mut mi = MONITORINFOEXW::default();
            mi.monitorInfo.cbSize = size_of_val(&mi) as _;
            unsafe { GetMonitorInfoW(monitor_handle.get_win32().unwrap(), addr_of_mut!(mi.monitorInfo)) };

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

        #[allow(clippy::undocumented_unsafe_blocks)]
        fn monitor_info(monitor_handle: MonitorHandle) -> Option<MonitorInfo> {
            let mut mi = MONITORINFOEXW::default();
            mi.monitorInfo.cbSize = size_of_val(&mi) as _;
            unsafe { GetMonitorInfoW(monitor_handle.get_win32().unwrap(), addr_of_mut!(mi.monitorInfo)) };
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

        #[allow(clippy::undocumented_unsafe_blocks)]
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
        #[allow(clippy::undocumented_unsafe_blocks)]
        fn available_monitors() -> VecDeque<MonitorHandle> {
            let display = unsafe { XOpenDisplay(core::ptr::null_mut()) };
            let num_screens = unsafe { XScreenCount(display) };
            let mut monitors = VecDeque::new();
            for i in 0..num_screens {
                let mut handle = XlibDisplayHandle::empty();
                handle.display = display.cast();
                handle.screen = i as _;
                monitors.push_back(MonitorHandle::Xlib(handle));
            }
            monitors
        }

        #[allow(
            clippy::undocumented_unsafe_blocks,
            clippy::cast_possible_wrap,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
        )]
        fn primary_monitor() -> Option<MonitorHandle> {
            let display = unsafe { XOpenDisplay(core::ptr::null_mut()) };
            let screen = unsafe { XDefaultScreen(display) };
            let root_window = unsafe { XRootWindow(display, screen) };
            let white_pixel = unsafe { XWhitePixel(display, screen) };
            let window = unsafe { XCreateSimpleWindow(display, root_window, 0, 0, 1, 1, 1, white_pixel, white_pixel) };
            let mut nmonitors = 0;
            let monitors_ptr = unsafe { XRRGetMonitors(display, window, x11::xlib::True, addr_of_mut!(nmonitors)) };
            let monitors = unsafe { slice::from_raw_parts(monitors_ptr, nmonitors as _) };
            let primary_monitor = monitors.iter().enumerate().find(|(_, m)| m.primary != 0).map(|(i, _)| {
                let mut handle = XlibDisplayHandle::empty();
                handle.display = display.cast();
                handle.screen = i as _;
                MonitorHandle::Xlib(handle)
            });

            unsafe { XRRFreeMonitors(monitors_ptr); }
            unsafe { XDestroyWindow(display, window); }
            primary_monitor
        }

        #[allow(clippy::undocumented_unsafe_blocks, clippy::unnecessary_wraps)]
        fn current_monitor(_: Option<WindowHandle>) -> Option<MonitorHandle> {
            let display = unsafe { XOpenDisplay(core::ptr::null_mut()) };
            let screen = unsafe { XDefaultScreen(display) };
            let mut handle = XlibDisplayHandle::empty();
            handle.display = display.cast();
            handle.screen = screen as _;
            Some(MonitorHandle::Xlib(handle))
        }

        #[allow(clippy::undocumented_unsafe_blocks)]
        fn choose_monitor() -> MonitorHandle {
            let monitor = dvar::get_int("r_monitor").unwrap();
            let display = unsafe { XOpenDisplay(core::ptr::null_mut()) };

            let mut handle = XlibDisplayHandle::empty();
            handle.display = display.cast();

            let screen = unsafe { XScreenOfDisplay(display, monitor) };
            handle.screen = if screen.is_null() {
                unsafe { XDefaultScreen(display) as _ }
            } else {
                monitor as _
            };

            MonitorHandle::Xlib(handle)
        }

        #[allow(clippy::undocumented_unsafe_blocks, clippy::cast_sign_loss)]
        fn get_monitor_dimensions(monitor: MonitorHandle) -> Option<(u32, u32)> {
            let display = unsafe { XOpenDisplay(core::ptr::null_mut()) };
            let screen_num = monitor.get_xlib().unwrap().screen;
            let screen = unsafe { XScreenOfDisplay(display, screen_num) };
            if screen.is_null() {
                return None;
            }

            let width = unsafe { XWidthOfScreen(screen) };
            let height = unsafe { XHeightOfScreen(screen) };
            if width <= 0 || height <= 0 {
                return None;
            }

            Some((width as _, height as _))
        }

        #[allow(
            clippy::undocumented_unsafe_blocks,
            clippy::cast_sign_loss,
            clippy::cast_possible_wrap,
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
        )]
        fn monitor_info(monitor: MonitorHandle) -> Option<MonitorInfo> {
            let display = monitor.get_xlib().unwrap().display.cast::<x11::xlib::Display>();
            let screen_num = monitor.get_xlib().unwrap().screen;
            let screen = unsafe { XScreenOfDisplay(display, screen_num) };
            if screen.is_null() {
                return None;
            }

            let root_window = unsafe { XRootWindow(display, screen_num) };
            let white_pixel = unsafe { XWhitePixel(display, screen_num) };
            let window = unsafe { XCreateSimpleWindow(display, root_window, 0, 0, 1, 1, 1, white_pixel, white_pixel) };
            let screen_info = unsafe { XRRGetScreenInfo(display, window) };
            if screen_info.is_null() {
                return None;
            }

            let width = unsafe { XWidthOfScreen(screen) };
            let height = unsafe { XHeightOfScreen(screen) };
            if width <= 0 || height <= 0 {
                return None;
            }

            let refresh = unsafe { XRRConfigCurrentRate(screen_info) };
            if refresh <= 0 {
                return None;
            }

            let mut nsizes = 0;
            let sizes_ptr = unsafe { XRRConfigSizes(screen_info, addr_of_mut!(nsizes)) };
            if sizes_ptr.is_null() {
                return None;
            }

            let sizes = unsafe { core::slice::from_raw_parts(sizes_ptr, nsizes as _) };

            let mut video_modes = Vec::new();
            for (i, m) in sizes.iter().enumerate() {
                let mut nrates = 0;
                let rates_ptr = unsafe { XRRConfigRates(screen_info, i as _, addr_of_mut!(nrates)) };
                if rates_ptr.is_null() {
                    return None;
                }

                let rates = unsafe { slice::from_raw_parts(rates_ptr, nrates as _) };

                for rate in rates {
                    video_modes.push(VideoMode {
                        width: m.width as _,
                        height: m.height as _,
                        bit_depth: unsafe { XDefaultDepth(display, screen_num) as _ },
                        refresh: f32::from(*rate),
                    });
                }
            }

            unsafe { XRRFreeScreenConfigInfo(screen_info); }
            unsafe { XDestroyWindow(display, window); }

            Some(MonitorInfo {
                name: String::new(), // TODO - get name
                width: width as _,
                height: height as _,
                refresh: f32::from(refresh),
                video_modes,
            })
        }

        #[allow(clippy::undocumented_unsafe_blocks)]
        pub fn create_window_2(wnd_parms: &mut gfx::WindowParms) -> Result<(), ()> {
            assert!(wnd_parms.window_handle.is_none());

            let display = unsafe { XOpenDisplay(core::ptr::null()) };
            let screen = unsafe { XDefaultScreen(display) };
            let root_window = unsafe { XRootWindow(display, screen) };
            let white_pixel = unsafe { XWhitePixel(display, screen) };
            let window = unsafe { XCreateSimpleWindow(
                display,
                root_window,
                i32::from(wnd_parms.x),
                i32::from(wnd_parms.y),
                wnd_parms.display_width,
                wnd_parms.display_height,
                0,
                white_pixel,
                white_pixel,
            ) };

            if window == 0 {
                com::println!(8.into(), "Couldn't create a window.");
                wnd_parms.window_handle = None;
                Err(())
            } else {
                let window_name = CString::new(com::get_official_build_name_r()).unwrap();
                unsafe { XStoreName(display, window, window_name.as_ptr()); }

                let mut handle = XlibWindowHandle::empty();
                handle.window = window as _;

                let visual = unsafe { XDefaultVisual(display, screen) };
                handle.visual_id = unsafe { XVisualIDFromVisual(visual) };
                wnd_parms.window_handle = Some(WindowHandle(RawWindowHandle::Xlib(handle)));

                if wnd_parms.fullscreen == false {
                    unsafe { XSetInputFocus(display, window, RevertToParent, CurrentTime); }
                }
                com::println!(8.into(), "Game window successfully created.");
                Ok(())
            }
        }
    }
}

struct MonitorInfo {
    name: String,
    width: u32,
    height: u32,
    refresh: f32,
    video_modes: Vec<VideoMode>,
}

fn enum_display_modes() {
    let info = monitor_info(
        primary_monitor().unwrap_or(*available_monitors().get(0).unwrap()),
    )
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
    if modes.is_empty() {
        fatal_init_error(format!(
            "No valid resolutions of {} x {} or above found",
            MIN_HORIZONTAL_RESOLUTION, MIN_VERTICAL_RESOLUTION
        ));
    }

    dvar::register_enumeration(
        "r_mode",
        modes.get(0).unwrap().clone(),
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
        refreshes.get(0).unwrap().clone(),
        Some(refreshes),
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::LATCHED,
        "Refresh rate".into(),
    )
    .unwrap();

    RENDER_GLOBALS.write().unwrap().video_modes =
        valid_modes.iter().copied().cloned().collect();
}

#[allow(clippy::unnecessary_wraps)]
fn pre_create_window() -> Result<(), ()> {
    com::println!(8.into(), "Getting Device interface...");
    let instance = sys::gpu::Instance::new();
    RENDER_GLOBALS.write().unwrap().instance = Some(instance);

    let adapter = choose_adapter();
    enum_display_modes();
    RENDER_GLOBALS.write().unwrap().adapter = adapter;

    Ok(())
}

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
        assert!(rg.device.is_some());
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
