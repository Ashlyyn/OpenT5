#![allow(dead_code)]

use crate::{
    gfx::WindowTarget,
    platform::{display_server::target::MonitorHandle, WindowHandle},
    sys::show_window,
    util::{EasierAtomic, SignalState},
    *,
};

cfg_if! {
    if #[cfg(wgpu)] {
        use pollster::block_on;
        use platform::render::wgpu::Device;
    } else if #[cfg(d3d9)] {
        use crate::platform::render::d3d9::{
            Adapter, DxCapsCheckBits, DxCapsCheckInteger, DxCapsResponse,
            ShadowmapBuildTechType, ShadowmapSamplerState, D3DFMT_NULL,
            D3DPTFILTERCAPS_MAGFANISOTROPIC, D3DPTFILTERCAPS_MINFANISOTROPIC,
            D3D_VENDOR_ID_NVIDIA, D3DERR_INVALID_CALL,
        };
        use core::{ffi::CStr, ptr::addr_of};
        use cstr::cstr;
        use nvapi_sys::{nvapi::NvAPI_Initialize, status::NVAPI_OK};
        use windows::Win32::{
            Graphics::Direct3D9::{
                Direct3DCreate9, D3DADAPTER_IDENTIFIER9,
                D3DCAPS2_FULLSCREENGAMMA, D3DCAPS9, D3DDEVTYPE_HAL,
                D3DFMT_A8R8G8B8, D3DFMT_D24S8, D3DFMT_D24X8,
                D3DFMT_R32F, D3DFMT_R5G6B5, D3DFMT_X8R8G8B8,
                D3DMULTISAMPLE_NONMASKABLE, D3DPRASTERCAPS_SLOPESCALEDEPTHBIAS,
                D3DPTEXTURECAPS_MIPCUBEMAP, D3DRTYPE_SURFACE, D3DRTYPE_TEXTURE,
                D3DUSAGE_DEPTHSTENCIL, D3DUSAGE_RENDERTARGET, D3D_SDK_VERSION,
                D3DDEVTYPE, D3DDEVTYPE_REF, D3DPRESENT_PARAMETERS, D3DDISPLAYMODE,
                D3DFORMAT, D3DFMT_D24FS8, D3DMULTISAMPLE_TYPE, D3DMULTISAMPLE_NONE,
                D3DCREATE_HARDWARE_VERTEXPROCESSING, D3DCREATE_MULTITHREADED,
                D3DPRESENT_INTERVAL_IMMEDIATE, D3DPRESENT_INTERVAL_ONE,
                D3DSWAPEFFECT_DISCARD,
            },
            System::Threading::Sleep,
            UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN},
        };
    } else if #[cfg(vulkan)] {
        use ash::extensions::khr;
    }
}

use sscanf::scanf;
extern crate alloc;
use alloc::collections::VecDeque;
use core::sync::atomic::AtomicUsize;
use std::{
    collections::HashSet,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

pub const MIN_HORIZONTAL_RESOLUTION: u32 = 640;
pub const MIN_VERTICAL_RESOLUTION: u32 = 480;

cfg_if! {
    if #[cfg(windows)] {
        use windows::Win32::Foundation::{RECT, HWND, LPARAM, POINT, BOOL};
        use windows::Win32::Graphics::Gdi::{
            DEVMODEW, EnumDisplayMonitors, MONITOR_DEFAULTTOPRIMARY,
            MONITOR_DEFAULTTONEAREST, MonitorFromPoint, MonitorFromWindow,
            GetMonitorInfoW, MONITORINFOEXW, EnumDisplaySettingsExW,
            ENUM_CURRENT_SETTINGS, ENUM_DISPLAY_SETTINGS_FLAGS,
            ENUM_DISPLAY_SETTINGS_MODE, DEVMODE_FIELD_FLAGS, DM_BITSPERPEL,
            DM_PELSWIDTH, DM_PELSHEIGHT, DM_DISPLAYFREQUENCY, HMONITOR, HDC
        };
        use windows::Win32::System::LibraryLoader::GetModuleHandleA;
        use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
        use windows::Win32::UI::WindowsAndMessaging::{
            WS_EX_LEFT, WS_SYSMENU, WS_CAPTION, WS_VISIBLE, WS_EX_TOPMOST,
            WS_POPUP, AdjustWindowRectEx, CreateWindowExA, SetWindowPos,
            HWND_NOTOPMOST, SWP_NOSIZE, SWP_NOMOVE,
        };
        use windows::core::{PCSTR, PCWSTR};
        use windows::s;
        use crate::platform::os::target::monitor_enum_proc;
        use core::mem::size_of_val;
        use alloc::collections::BTreeSet;
        use raw_window_handle::Win32WindowHandle;
        use raw_window_handle::RawWindowHandle;
        use alloc::ffi::CString;
        use core::ptr::addr_of_mut;
        use crate::platform::display_server::target::WindowHandleExt;
    } else if #[cfg(xlib)] {
        use raw_window_handle::{
            RawWindowHandle, XlibWindowHandle, XlibDisplayHandle
        };
        use x11::xlib::{
            XStoreName, XOpenDisplay, XSetWMProtocols, XCloseDisplay,
            XDefaultScreen, XCreateSimpleWindow, XDefaultVisual, XScreenCount,
            XRootWindow, XScreenOfDisplay, XWhitePixel, XWidthOfScreen,
            XHeightOfScreen, XDestroyWindow, XDefaultDepth, XSetInputFocus,
            RevertToParent, CurrentTime, XVisualIDFromVisual,
        };
        use x11::xrandr::{
            XRRGetMonitors, XRRFreeMonitors, XRRConfigCurrentRate,
            XRRGetScreenInfo, XRRFreeScreenConfigInfo, XRRConfigSizes,
            XRRConfigRates,
        };
        use platform::display_server::target::WM_DELETE_WINDOW;
        use alloc::ffi::CString;
        use core::ptr::addr_of_mut;
    } else if #[cfg(appkit)] {
        use objc2::rc::autoreleasepool;
        use std::ptr::addr_of;

        use crate::platform::display_server::appkit::{
            AppDelegate, WindowDelegate,
        };
        use icrate::{
            ns_string,
            AppKit::{
                NSApp, NSApplication, NSApplicationActivationPolicyRegular,
                NSBackingStoreBuffered, NSClosableWindowMask, NSMenu,
                NSMenuItem, NSResizableWindowMask, NSTitledWindowMask,
                NSWindow, NSWindowController,
            },
            Foundation::{CGPoint, CGSize, NSProcessInfo, NSRect, NSString},
        };
        use objc2::{rc::Id, runtime::ProtocolObject, sel, ClassType};
        use raw_window_handle::AppKitWindowHandle;
    }
}

fn init_render_thread() {
    if !sys::spawn_render_thread(rb::render_thread) {
        com::errorln!(com::ErrorParm::FATAL, "Failed to create render thread");
    }
}

pub fn init_threads() {
    com::println!(
        8.into(),
        "{}: Trying SMP acceleration...",
        std::thread::current().name().unwrap_or("main"),
    );
    init_render_thread();
    // init_worker_threads();
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
        assert!(r_glob().remote_screen_update_nesting >= 0);
        if r_glob().started_render_thread
            && (cl::local_client_is_in_game(0) == false
                || r_glob().remote_screen_update_in_game != 0)
        {
            assert_ne!(r_glob().screen_update_notify, false);
            r_glob_mut().remote_screen_update_nesting += 1;
            sys::notify_renderer();
        } else {
            r_glob_mut().remote_screen_update_nesting = 1;
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

    assert!(r_glob().remote_screen_update_nesting >= 0);

    if r_glob().started_render_thread == false {
        assert_eq!(r_glob().remote_screen_update_nesting, 0);
        return;
    }

    if cl::local_client_is_in_game(0)
        && r_glob().remote_screen_update_in_game == 0
    {
        assert_eq!(r_glob().remote_screen_update_nesting, 0);
        return;
    }

    assert!(r_glob().remote_screen_update_nesting > 0);

    if r_glob().remote_screen_update_nesting != 1 {
        r_glob_mut().remote_screen_update_nesting -= 1;
        return;
    }

    while r_glob().screen_update_notify == false {
        net::sleep(Duration::from_millis(1));
        f();
        sys::wait_renderer();
    }
    r_glob_mut().screen_update_notify = false;
    assert!(r_glob().remote_screen_update_nesting > 0);
    r_glob_mut().remote_screen_update_nesting -= 1;
    while r_glob().screen_update_notify == false {
        G_MAIN_THREAD_BLOCKED.increment_wrapping();
        net::sleep(Duration::from_millis(1));
        f();
        sys::wait_renderer();
        G_MAIN_THREAD_BLOCKED.decrement_wrapping();
    }
    r_glob_mut().screen_update_notify = false;
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

    dvar::register_bool(
        "sm_enable",
        true,
        dvar::DvarFlags::empty(),
        Some("Enable shadow mapping"),
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
            ASPECT_RATIO_16_9.into(),
        ]),
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::LATCHED,
        Some(
            "Screen aspect ratio.  Most widescreen monitors are 16:10 instead \
             of 16:9.",
        ),
    )
    .unwrap();
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
        Some(
            "Enable v-sync before drawing the next frame to avoid \'tearing\' \
             artifacts.",
        ),
    )
    .unwrap();
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

struct MonitorInfo {
    name: String,
    width: u32,
    height: u32,
    refresh: f32,
    video_modes: Vec<VideoMode>,
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
    #[cfg(any(feature = "windows_use_wgpu", feature = "macos_use_wgpu", feature = "linux_use_wgpu"))]
    device: Option<platform::render::wgpu::Device>,
    #[cfg(any(feature = "windows_use_wgpu", feature = "macos_use_wgpu", feature = "linux_use_wgpu"))]
    adapter: Option<platform::render::wgpu::Adapter>,
    #[cfg(any(feature = "windows_use_wgpu", feature = "macos_use_wgpu", feature = "linux_use_wgpu"))]
    instance: Option<platform::render::wgpu::Instance>,
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
            #[cfg(any(feature = "windows_use_wgpu", feature = "macos_use_wgpu", feature = "linux_use_wgpu"))]
            device: None,
            #[cfg(any(feature = "windows_use_wgpu", feature = "macos_use_wgpu", feature = "linux_use_wgpu"))]
            adapter: None,
            #[cfg(any(feature = "windows_use_wgpu", feature = "macos_use_wgpu", feature = "linux_use_wgpu"))]
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
#[doc(hidden)]
fn _fatal_init_error_internal(error: impl ToString) -> ! {
    com::println!(
        8.into(),
        "********** Device returned an unrecoverable error code during \
         initialization  **********"
    );
    com::println!(
        8.into(),
        "********** Initialization also happens while playing if Renderer \
         loses a device **********"
    );
    com::println!(8.into(), "{}", error.to_string());
    sys::render_fatal_error();
}

#[allow(unused_macros)]
macro_rules! fatal_init_error {
    ($($arg:tt)*) => {{
        $crate::render::_fatal_init_error_internal(core::format_args!($($arg)*));
    }};
}

#[allow(unused_macros)]
macro_rules! fatal_init_errorln {
    ($channel:expr) => {
        $crate::render::fatal_init_error!("\n")
    };
    ($channel:expr, $($arg:tt)*) => {{
        $crate::render::fatal_init_error!("{}\n", core::format_args!($($arg)*));
    }};
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
    if let Some((width, height)) = get_monitor_dimensions() {
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

#[cfg(wgpu)]
#[allow(clippy::unnecessary_wraps)]
fn choose_adapter() -> Option<platform::render::wgpu::Adapter> {
    let rg = RENDER_GLOBALS.write().unwrap();
    let adapter = block_on(platform::render::wgpu::Adapter::new(
        rg.instance.as_ref().unwrap(),
        None,
    ));
    Some(adapter)
}

#[cfg(d3d9)]
fn choose_adapter() -> Option<Adapter> {
    let hmonitor = choose_monitor().get_win32().unwrap();
    let dx = platform::render::d3d9::dx();

    let adapter_count = unsafe { dx.d3d9.as_ref().unwrap().GetAdapterCount() };

    let mut id = D3DADAPTER_IDENTIFIER9::default();
    for adapter in 0..adapter_count {
        if hmonitor.0 != 0 {
            let hmonitor_2 =
                unsafe { dx.d3d9.as_ref().unwrap().GetAdapterMonitor(adapter) };
            if hmonitor == hmonitor_2 {
                return Some(Adapter::from_d3d9(adapter));
            }
        } else if unsafe {
            dx.d3d9.as_ref().unwrap().GetAdapterIdentifier(
                adapter,
                0,
                addr_of_mut!(id),
            )
        }
        .is_ok()
        {
            if CStr::from_bytes_until_nul(&id.Description).unwrap()
                == cstr!("NVIDIA NVPerfHUD")
            {
                return Some(Adapter::from_d3d9(adapter));
            }
        }
    }

    Some(Adapter::from_d3d9(adapter_count))
}

#[cfg(windows)]
#[allow(clippy::undocumented_unsafe_blocks)]
fn available_monitors() -> VecDeque<MonitorHandle> {
    let mut monitors: VecDeque<MonitorHandle> = VecDeque::new();
    unsafe {
        EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(addr_of_mut!(monitors) as _),
        );
    }
    monitors
}

#[cfg(windows)]
#[allow(clippy::undocumented_unsafe_blocks, clippy::unnecessary_wraps)]
fn primary_monitor() -> Option<MonitorHandle> {
    const ORIGIN: POINT = POINT { x: 0, y: 0 };
    let hmonitor =
        unsafe { MonitorFromPoint(ORIGIN, MONITOR_DEFAULTTOPRIMARY) };
    Some(MonitorHandle::Win32(hmonitor.0))
}

#[cfg(windows)]
#[allow(clippy::undocumented_unsafe_blocks, clippy::unnecessary_wraps)]
fn current_monitor(handle: Option<WindowHandle>) -> Option<MonitorHandle> {
    handle.map(|handle| {
        let hmonitor = unsafe {
            MonitorFromWindow(
                HWND(handle.get_win32().unwrap().hwnd as _),
                MONITOR_DEFAULTTONEAREST,
            )
        };
        MonitorHandle::Win32(hmonitor.0)
    })
}

#[cfg(windows)]
#[repr(C)]
struct MonitorEnumData {
    monitor: i32,
    handle: HMONITOR,
}

#[cfg(windows)]
#[allow(clippy::undocumented_unsafe_blocks)]
unsafe extern "system" fn monitor_enum_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    data: LPARAM,
) -> BOOL {
    // This is safe because the only time we ever call this
    // function, we wrap a *mut MonitorEnumData in `data`
    let data = data.0 as *mut MonitorEnumData;
    (*data).monitor -= 1;
    if (*data).monitor == 0 {
        (*data).handle = hmonitor;
    }
    BOOL(i32::from((*data).monitor != 0))
}

#[cfg(windows)]
#[allow(clippy::undocumented_unsafe_blocks)]
fn choose_monitor() -> MonitorHandle {
    let fullscreen = dvar::get_bool("r_fullscreen").unwrap();
    if fullscreen {
        let monitor = dvar::get_int("r_monitor").unwrap();
        let mut data = MonitorEnumData {
            monitor,
            handle: HMONITOR(0),
        };
        unsafe {
            EnumDisplayMonitors(
                None,
                None,
                Some(monitor_enum_callback),
                LPARAM(addr_of_mut!(data) as _),
            );
        }
        if data.handle != HMONITOR(0) {
            return MonitorHandle::Win32(data.handle.0);
        }
    }

    let xpos = dvar::get_int("vid_xpos").unwrap();
    let ypos = dvar::get_int("vid_ypos").unwrap();
    let hmonitor = unsafe {
        MonitorFromPoint(POINT { x: xpos, y: ypos }, MONITOR_DEFAULTTOPRIMARY)
    };
    MonitorHandle::Win32(hmonitor.0)
}

#[cfg(d3d9)]
#[allow(
    clippy::undocumented_unsafe_blocks,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
fn get_monitor_dimensions() -> Option<(u32, u32)> {
    let hmonitor = {
        let dx = platform::render::d3d9::dx();
        unsafe {
            dx.d3d9
                .as_ref()
                .unwrap()
                .GetAdapterMonitor(dx.adapter.as_d3d9())
        }
    };

    let mut mi = MONITORINFOEXW::default();
    mi.monitorInfo.cbSize = size_of_val(&mi) as _;
    unsafe {
        GetMonitorInfoW(hmonitor, addr_of_mut!(mi.monitorInfo));
    }

    let mi_width =
        (mi.monitorInfo.rcMonitor.right - mi.monitorInfo.rcMonitor.left) as u32;
    let mi_height =
        (mi.monitorInfo.rcMonitor.bottom - mi.monitorInfo.rcMonitor.top) as u32;
    let width = if mi_width > 0 {
        mi_width
    } else {
        let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        if width == 0 {
            return None;
        }
        width as _
    };
    let height = if mi_width > 0 {
        mi_height
    } else {
        let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
        if height == 0 {
            return None;
        }
        height as _
    };

    Some((width, height))
}

#[cfg(vulkan)]
#[allow(
    clippy::undocumented_unsafe_blocks,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
fn get_monitor_dimensions() -> Option<(u32, u32)> {
    let vk = platform::render::vulkan::vk_mut();
    let display_ext = khr::Display::new(vk.entry.as_ref()?, vk.instance.as_ref()?);
    let displays = unsafe { display_ext.get_physical_device_display_properties(*vk.physical_device.as_ref()?) }.ok()?;
    let props = displays.get(0)?;
    Some((props.physical_resolution.width, props.physical_resolution.height))
}

#[cfg(not(any(d3d9, vulkan, appkit)))]
fn get_monitor_dimensions() -> Option<(u32, u32)> {
    todo!()
}

#[cfg(windows)]
#[allow(
    clippy::undocumented_unsafe_blocks,
    clippy::cast_precision_loss,
    clippy::items_after_statements,
    clippy::cast_possible_truncation
)]
fn monitor_info(monitor_handle: MonitorHandle) -> Option<MonitorInfo> {
    let mut mi = MONITORINFOEXW::default();
    mi.monitorInfo.cbSize = size_of_val(&mi) as _;
    unsafe {
        GetMonitorInfoW(
            monitor_handle.get_win32().unwrap(),
            addr_of_mut!(mi.monitorInfo),
        );
    }
    let name = char::decode_utf16(mi.szDevice)
        .flatten()
        .collect::<String>();

    let mut mode = DEVMODEW::default();
    mode.dmSize = size_of_val(&mode) as _;
    if unsafe {
        EnumDisplaySettingsExW(
            PCWSTR(mi.szDevice.as_ptr()),
            ENUM_CURRENT_SETTINGS,
            addr_of_mut!(mode),
            ENUM_DISPLAY_SETTINGS_FLAGS(0),
        )
    }
    .ok()
    .is_err()
    {
        return None;
    };
    let refresh = mode.dmDisplayFrequency as f32;

    let mut modes = BTreeSet::new();
    let mut i = 0;
    loop {
        if unsafe {
            EnumDisplaySettingsExW(
                PCWSTR(mi.szDevice.as_ptr()),
                ENUM_DISPLAY_SETTINGS_MODE(i),
                addr_of_mut!(mode),
                ENUM_DISPLAY_SETTINGS_FLAGS(0),
            )
        }
        .as_bool()
            == false
        {
            break;
        }
        i += 1;

        const REQUIRED_FIELDS: DEVMODE_FIELD_FLAGS = DEVMODE_FIELD_FLAGS(
            DM_BITSPERPEL.0
                | DM_PELSWIDTH.0
                | DM_PELSHEIGHT.0
                | DM_DISPLAYFREQUENCY.0,
        );
        assert!(mode.dmFields.contains(REQUIRED_FIELDS));
        modes.insert(VideoMode {
            width: mode.dmPelsWidth,
            height: mode.dmPelsHeight,
            bit_depth: mode.dmBitsPerPel as _,
            refresh: mode.dmDisplayFrequency as _,
        });
    }

    let (width, height) = get_monitor_dimensions()?;
    Some(MonitorInfo {
        name,
        width,
        height,
        refresh: refresh as _,
        video_modes: modes.into_iter().collect(),
    })
}

#[cfg(windows)]
#[allow(
    clippy::undocumented_unsafe_blocks,
    clippy::cast_lossless,
    clippy::cast_possible_wrap
)]
pub fn create_window_2(wnd_parms: &mut gfx::WindowParms) -> Result<(), ()> {
    assert!(wnd_parms.window_handle.is_none());

    let (dw_ex_style, dw_style) = if wnd_parms.fullscreen == false {
        com::println!(
            8.into(),
            "Attempting {} x {} window at ({}, {})",
            wnd_parms.display_width,
            wnd_parms.display_height,
            wnd_parms.x,
            wnd_parms.y
        );
        (WS_EX_LEFT, WS_SYSMENU | WS_CAPTION | WS_VISIBLE)
    } else {
        com::println!(
            8.into(),
            "Attempting {} x {} fullscreen with 32 bpp at {} hz",
            wnd_parms.display_width,
            wnd_parms.display_height,
            wnd_parms.hz
        );
        (WS_EX_TOPMOST, WS_POPUP)
    };

    let mut rect = RECT {
        left: 0,
        right: wnd_parms.display_width as _,
        top: 0,
        bottom: wnd_parms.display_height as _,
    };
    unsafe {
        AdjustWindowRectEx(addr_of_mut!(rect), dw_style, false, dw_ex_style);
    }
    let hinstance = unsafe { GetModuleHandleA(None) }.unwrap_or_default();
    let height = rect.bottom - rect.top;
    let width = rect.right - rect.left;
    let window_name = CString::new(com::get_official_build_name_r()).unwrap();
    let hwnd = unsafe {
        CreateWindowExA(
            dw_ex_style,
            s!("CoDBlackOps"),
            PCSTR(window_name.as_ptr().cast()),
            dw_style,
            wnd_parms.x as _,
            wnd_parms.y as _,
            width,
            height,
            None,
            None,
            hinstance,
            None,
        )
    };

    if hwnd.0 == 0 {
        com::println!(8.into(), "Couldn't create a window.");
        wnd_parms.window_handle = None;
        Err(())
    } else {
        let mut handle = Win32WindowHandle::empty();
        handle.hinstance = hinstance.0 as _;
        handle.hwnd = hwnd.0 as _;
        wnd_parms.window_handle =
            Some(WindowHandle(RawWindowHandle::Win32(handle)));

        if wnd_parms.fullscreen == false {
            unsafe {
                SetWindowPos(
                    hwnd,
                    HWND_NOTOPMOST,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOMOVE,
                );
            }
            unsafe {
                SetFocus(hwnd);
            }
        }
        com::println!(8.into(), "Game window successfully created.");
        Ok(())
    }
}

#[cfg(wayland)]
fn available_monitors() -> VecDeque<MonitorHandle> {
    todo!()
}

#[cfg(wayland)]
fn primary_monitor() -> Option<MonitorHandle> {
    todo!()
}

#[cfg(wayland)]
fn current_monitor(_: Option<WindowHandle>) -> Option<MonitorHandle> {
    todo!()
}

#[cfg(wayland)]
fn choose_monitor() -> MonitorHandle {
    todo!()
}

#[cfg(wayland)]
fn monitor_info(_monitor: MonitorHandle) -> Option<MonitorInfo> {
    todo!()
}

#[cfg(wayland)]
pub fn create_window_2(_wnd_parms: &mut gfx::WindowParms) -> Result<(), ()> {
    todo!()
}

#[cfg(appkit)]
fn available_monitors() -> VecDeque<MonitorHandle> {
    todo!()
}

#[cfg(appkit)]
fn primary_monitor() -> Option<MonitorHandle> {
    todo!()
}

#[cfg(appkit)]
fn current_monitor(_: Option<WindowHandle>) -> Option<MonitorHandle> {
    todo!()
}

#[cfg(appkit)]
fn choose_monitor() -> MonitorHandle {
    todo!()
}

#[cfg(appkit)]
fn monitor_info(_monitor: MonitorHandle) -> Option<MonitorInfo> {
    todo!()
}

#[cfg(appkit)]
pub fn create_window_2(wnd_parms: &mut gfx::WindowParms) -> Result<(), ()> {
    assert!(wnd_parms.window_handle.is_none());

    autoreleasepool(|pool| {
        let _ = unsafe { NSApplication::sharedApplication() };

        let Some(ns_app) = (unsafe { NSApp }) else {
            // this situation should never occur unless Cocoa is buggy/corrupted
            panic!(
                "NSApplication::sharedApplication failed to initialize NSApp"
            );
        };

        unsafe {
            ns_app.setActivationPolicy(NSApplicationActivationPolicyRegular)
        };

        let dg = AppDelegate::new();
        unsafe { ns_app.setDelegate(Some(&ProtocolObject::from_id(dg))) };
        unsafe { ns_app.finishLaunching() };

        let menu_bar = Id::autorelease(unsafe { NSMenu::new() }, pool);
        let app_menu_item = Id::autorelease(unsafe { NSMenuItem::new() }, pool);
        unsafe { menu_bar.addItem(&app_menu_item) };
        unsafe { ns_app.setMainMenu(Some(&menu_bar)) };

        let app_menu = Id::autorelease(unsafe { NSMenu::new() }, pool);
        let proc_info = Id::autorelease(NSProcessInfo::processInfo(), pool);
        let app_name = Id::autorelease(proc_info.processName(), pool);
        let quit_title = Id::autorelease(
            ns_string!("Quit ").stringByAppendingString(&app_name),
            pool,
        );

        let terminate = sel!(terminate:);
        let key_equivalent = ns_string!("q");
        let quit_menu_item = Id::autorelease(
            unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    NSMenuItem::alloc(),
                    &quit_title,
                    Some(terminate),
                    key_equivalent,
                )
            },
            pool,
        );
        unsafe { app_menu.addItem(&quit_menu_item) };
        unsafe { app_menu_item.setSubmenu(Some(&app_menu)) };

        let rect = NSRect::new(
            CGPoint::new(wnd_parms.x as _, wnd_parms.y as _),
            CGSize::new(
                wnd_parms.display_width as _,
                wnd_parms.display_height as _,
            ),
        );
        let window_style =
            NSTitledWindowMask | NSClosableWindowMask | NSResizableWindowMask;

        let Ok(window) = (unsafe {
            objc2::exception::catch(|| {
                Id::autorelease(
                    NSWindow::initWithContentRect_styleMask_backing_defer(
                        NSWindow::alloc(),
                        rect,
                        window_style,
                        NSBackingStoreBuffered,
                        false,
                    ),
                    pool,
                )
            })
        }) else {
            com::println!(8.into(), "Couldn't create a window.");
            wnd_parms.window_handle = None;
            return Err(());
        };

        let _window_controller = Id::autorelease(
            unsafe {
                NSWindowController::initWithWindow(
                    NSWindowController::alloc(),
                    Some(&window),
                )
            },
            pool,
        );

        unsafe { window.setReleasedWhenClosed(false) };

        let wdg = WindowDelegate::new(unsafe { window.windowNumber() });
        unsafe { window.setDelegate(Some(&ProtocolObject::from_id(wdg))) };

        let title = NSString::from_str(com::get_official_build_name_r());
        unsafe { window.setTitle(&title) };

        let mut handle = AppKitWindowHandle::empty();
        handle.ns_window = addr_of!(*window) as _;
        // TODO - Get (create?) view

        if wnd_parms.fullscreen == false {
            unsafe { window.makeKeyAndOrderFront(Some(&window)) };
        }
        unsafe { window.setAcceptsMouseMovedEvents(true) };

        com::println!(8.into(), "Game window successfully created.");
        Ok(())
    })
}

#[cfg(xlib)]
#[allow(clippy::undocumented_unsafe_blocks)]
fn available_monitors() -> VecDeque<MonitorHandle> {
    let display =
        unsafe { XOpenDisplay(platform::display_server::xlib::display_name()) };
    let num_screens = unsafe { XScreenCount(display) };
    unsafe { XCloseDisplay(display) };
    let mut monitors = VecDeque::new();
    for i in 0..num_screens {
        let mut handle = XlibDisplayHandle::empty();
        handle.display = display.cast();
        handle.screen = i as _;
        monitors.push_back(MonitorHandle::Xlib(handle));
    }
    monitors
}

#[cfg(xlib)]
#[allow(
    clippy::undocumented_unsafe_blocks,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn primary_monitor() -> Option<MonitorHandle> {
    let display =
        unsafe { XOpenDisplay(platform::display_server::xlib::display_name()) };
    let screen = unsafe { XDefaultScreen(display) };
    let root_window = unsafe { XRootWindow(display, screen) };
    let white_pixel = unsafe { XWhitePixel(display, screen) };
    let window = unsafe {
        XCreateSimpleWindow(
            display,
            root_window,
            0,
            0,
            1,
            1,
            1,
            white_pixel,
            white_pixel,
        )
    };
    let mut nmonitors = 0;
    let monitors_ptr = unsafe {
        XRRGetMonitors(
            display,
            window,
            x11::xlib::True,
            addr_of_mut!(nmonitors),
        )
    };
    // SAFETY: assuming the Xlib implementation is conforming,
    // [monitors_ptr, monitors_ptr + nmonitors) should always
    // be valid.
    let monitors =
        unsafe { core::slice::from_raw_parts(monitors_ptr, nmonitors as _) };
    let primary_monitor = monitors
        .iter()
        .enumerate()
        .find(|(_, m)| m.primary != 0)
        .map(|(i, _)| {
            let mut handle = XlibDisplayHandle::empty();
            handle.display = display.cast();
            handle.screen = i as _;
            MonitorHandle::Xlib(handle)
        });

    unsafe {
        XRRFreeMonitors(monitors_ptr);
    }
    unsafe {
        XDestroyWindow(display, window);
    }
    primary_monitor
}

#[cfg(xlib)]
#[allow(clippy::undocumented_unsafe_blocks, clippy::unnecessary_wraps)]
fn current_monitor(_: Option<WindowHandle>) -> Option<MonitorHandle> {
    let display =
        unsafe { XOpenDisplay(platform::display_server::xlib::display_name()) };
    let screen = unsafe { XDefaultScreen(display) };
    let mut handle = XlibDisplayHandle::empty();
    handle.display = display.cast();
    handle.screen = screen as _;
    Some(MonitorHandle::Xlib(handle))
}

#[cfg(xlib)]
#[allow(clippy::undocumented_unsafe_blocks)]
fn choose_monitor() -> MonitorHandle {
    let monitor = dvar::get_int("r_monitor").unwrap();
    let display =
        unsafe { XOpenDisplay(platform::display_server::xlib::display_name()) };

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

#[cfg(xlib)]
#[allow(clippy::undocumented_unsafe_blocks, clippy::cast_sign_loss)]
fn get_monitor_dimensions() -> Option<(u32, u32)> {
    todo!()
    // let display = monitor.get_xlib().unwrap().display as *mut _XDisplay;
    // let screen_num = monitor.get_xlib().unwrap().screen;
    // let screen = unsafe { XScreenOfDisplay(display, screen_num) };
    // if screen.is_null() {
    //     return None;
    // }

    // let width = unsafe { XWidthOfScreen(screen) };
    // let height = unsafe { XHeightOfScreen(screen) };
    // if width <= 0 || height <= 0 {
    //     return None;
    // }

    // Some((width as _, height as _))
}

#[cfg(xlib)]
#[allow(
    clippy::undocumented_unsafe_blocks,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
fn monitor_info(monitor: MonitorHandle) -> Option<MonitorInfo> {
    let display = monitor
        .get_xlib()
        .unwrap()
        .display
        .cast::<x11::xlib::Display>();
    let screen_num = monitor.get_xlib().unwrap().screen;
    let screen = unsafe { XScreenOfDisplay(display, screen_num) };
    if screen.is_null() {
        return None;
    }

    let root_window = unsafe { XRootWindow(display, screen_num) };
    let white_pixel = unsafe { XWhitePixel(display, screen_num) };
    let window = unsafe {
        XCreateSimpleWindow(
            display,
            root_window,
            0,
            0,
            1,
            1,
            1,
            white_pixel,
            white_pixel,
        )
    };
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
    let sizes_ptr =
        unsafe { XRRConfigSizes(screen_info, addr_of_mut!(nsizes)) };
    if sizes_ptr.is_null() {
        return None;
    }

    // SAFETY: assuming the Xlib implementation is conforming,
    // [sizes_ptr, sizes_ptr + nsizes) should always be valid.
    let sizes = unsafe { core::slice::from_raw_parts(sizes_ptr, nsizes as _) };

    let mut video_modes = Vec::new();
    for (i, m) in sizes.iter().enumerate() {
        let mut nrates = 0;
        let rates_ptr = unsafe {
            XRRConfigRates(screen_info, i as _, addr_of_mut!(nrates))
        };
        if rates_ptr.is_null() {
            return None;
        }

        // SAFETY: assuming the Xlib implementation is conforming,
        // [rates_ptr, rates_ptr + nrates) should always be valid.
        let rates =
            unsafe { core::slice::from_raw_parts(rates_ptr, nrates as _) };

        for rate in rates {
            video_modes.push(VideoMode {
                width: m.width as _,
                height: m.height as _,
                bit_depth: unsafe { XDefaultDepth(display, screen_num) as _ },
                refresh: f32::from(*rate),
            });
        }
    }

    unsafe {
        XRRFreeScreenConfigInfo(screen_info);
    }
    unsafe {
        XDestroyWindow(display, window);
    }

    Some(MonitorInfo {
        name: String::new(), // TODO - get name
        width: width as _,
        height: height as _,
        refresh: f32::from(refresh),
        video_modes,
    })
}

#[cfg(xlib)]
#[allow(clippy::undocumented_unsafe_blocks)]
pub fn create_window_2(wnd_parms: &mut gfx::WindowParms) -> Result<(), ()> {
    assert!(wnd_parms.window_handle.is_none());

    let display =
        unsafe { XOpenDisplay(platform::display_server::xlib::display_name()) };
    let screen = unsafe { XDefaultScreen(display) };
    let root_window = unsafe { XRootWindow(display, screen) };
    let white_pixel = unsafe { XWhitePixel(display, screen) };
    let window = unsafe {
        XCreateSimpleWindow(
            display,
            root_window,
            i32::from(wnd_parms.x),
            i32::from(wnd_parms.y),
            wnd_parms.display_width,
            wnd_parms.display_height,
            0,
            white_pixel,
            white_pixel,
        )
    };

    if window == 0 {
        com::println!(8.into(), "Couldn't create a window.");
        wnd_parms.window_handle = None;
        Err(())
    } else {
        let window_name =
            CString::new(com::get_official_build_name_r()).unwrap();
        unsafe {
            XStoreName(display, window, window_name.as_ptr());
        }

        let mut handle = XlibWindowHandle::empty();
        handle.window = window as _;

        let visual = unsafe { XDefaultVisual(display, screen) };
        handle.visual_id = unsafe { XVisualIDFromVisual(visual) };
        wnd_parms.window_handle =
            Some(WindowHandle(RawWindowHandle::Xlib(handle)));

        if wnd_parms.fullscreen == false {
            unsafe {
                XSetInputFocus(display, window, RevertToParent, CurrentTime);
            }
        }

        let mut wm_delete_window = WM_DELETE_WINDOW.load_relaxed();
        unsafe {
            XSetWMProtocols(display, window, addr_of_mut!(wm_delete_window), 1);
        }

        com::println!(8.into(), "Game window successfully created.");
        Ok(())
    }
}

#[cfg(wgpu)]
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
        fatal_init_error!(
            "No valid resolutions of {} x {} or above found",
            MIN_HORIZONTAL_RESOLUTION,
            MIN_VERTICAL_RESOLUTION
        );
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

#[cfg(wgpu)]
#[allow(clippy::unnecessary_wraps)]
fn pre_create_window() -> Result<(), ()> {
    com::println!(8.into(), "Getting Device interface...");
    let instance = platform::render::wgpu::Instance::new();
    RENDER_GLOBALS.write().unwrap().instance = Some(instance);

    let adapter = choose_adapter();
    enum_display_modes();
    RENDER_GLOBALS.write().unwrap().adapter = adapter;

    Ok(())
}

#[cfg(d3d9)]
fn get_direct3d_caps(adapter: Adapter) -> D3DCAPS9 {
    let dx = platform::render::d3d9::dx();
    assert!(dx.d3d9.is_some());

    let mut caps = D3DCAPS9::default();
    let mut i = 0;
    let err = loop {
        let err = if let Err(e) = unsafe {
            dx.d3d9.as_ref().unwrap().GetDeviceCaps(
                adapter.as_d3d9(),
                D3DDEVTYPE_HAL,
                addr_of_mut!(caps),
            )
        } {
            e
        } else {
            return caps;
        };

        unsafe { Sleep(100) };
        if i == 20 {
            break err;
        }
        i += 1;
    };

    fatal_init_error!("GetDeviceCaps failed: {}", err);
}

#[cfg(d3d9)]
fn respond_to_missing_caps(response: DxCapsResponse, message: &'static str) {
    if response == DxCapsResponse::Warn {
        com::warnln!(8.into(), "Video card or driver {}.", message);
    } else {
        com::println!(8.into(), "Video card or driver {}.", message);
    }

    match response {
        DxCapsResponse::Quit => com::errorln!(
            com::ErrorParm::FATAL,
            "Video card or driver {}.",
            message
        ),
        DxCapsResponse::ForbidSm3 => com::errorln!(
            com::ErrorParm::FATAL,
            "Shader model 3.0 not available."
        ),
        _ => {}
    }
}

#[cfg(d3d9)]
const S_CAPS_CHECK_BITS: [DxCapsCheckBits; 32] = [
    DxCapsCheckBits {
        offset: 12,
        set_bits: 0x00000000,
        clear_bits: 0x20000000,
        response: DxCapsResponse::Quit,
        message: "doesn't support dynamic textures",
    },
    DxCapsCheckBits {
        offset: 12,
        set_bits: 0x00000000,
        clear_bits: 0x00020000,
        response: DxCapsResponse::Warn,
        message: "doesn't support fullscreen gamma",
    },
    DxCapsCheckBits {
        offset: 16,
        set_bits: 0x00000000,
        clear_bits: 0x00000020,
        response: DxCapsResponse::Quit,
        message: "doesn't support alpha blending",
    },
    DxCapsCheckBits {
        offset: 16,
        set_bits: 0x00000000,
        clear_bits: 0x00000100,
        response: DxCapsResponse::Warn,
        message: "doesn't accelerate dynamic textures",
    },
    DxCapsCheckBits {
        offset: 20,
        set_bits: 0x00000000,
        clear_bits: 0x80000000,
        response: DxCapsResponse::Warn,
        message: "doesn't support immediate frame buffer swapping",
    },
    DxCapsCheckBits {
        offset: 20,
        set_bits: 0x00000000,
        clear_bits: 0x00000001,
        response: DxCapsResponse::Warn,
        message: "doesn't support vertical sync",
    },
    DxCapsCheckBits {
        offset: 28,
        set_bits: 0x00000000,
        clear_bits: 0x00008000,
        response: DxCapsResponse::Quit,
        message: "is not at least DirectX 7 compliant",
    },
    DxCapsCheckBits {
        offset: 28,
        set_bits: 0x00000000,
        clear_bits: 0x00010400,
        response: DxCapsResponse::Warn,
        message: "doesn't accelerate transform and lighting",
    },
    DxCapsCheckBits {
        offset: 28,
        set_bits: 0x00000000,
        clear_bits: 0x00080000,
        response: DxCapsResponse::Warn,
        message: "doesn't accelerate rasterization",
    },
    DxCapsCheckBits {
        offset: 32,
        set_bits: 0x00000000,
        clear_bits: 0x00000002,
        response: DxCapsResponse::Quit,
        message: "can't disable depth buffer writes",
    },
    DxCapsCheckBits {
        offset: 32,
        set_bits: 0x00000000,
        clear_bits: 0x00000080,
        response: DxCapsResponse::Quit,
        message: "can't disable individual color channel writes",
    },
    DxCapsCheckBits {
        offset: 32,
        set_bits: 0x00000000,
        clear_bits: 0x00000800,
        response: DxCapsResponse::Quit,
        message: "doesn't support frame buffer blending ops besides add",
    },
    DxCapsCheckBits {
        offset: 32,
        set_bits: 0x00000000,
        clear_bits: 0x00020000,
        response: DxCapsResponse::Quit,
        message: "doesn't support separate alpha blend, glow will be disabled",
    },
    DxCapsCheckBits {
        offset: 32,
        set_bits: 0x00000000,
        clear_bits: 0x00000070,
        response: DxCapsResponse::Quit,
        message: "doesn't support all face culling modes",
    },
    DxCapsCheckBits {
        offset: 36,
        set_bits: 0x00000000,
        clear_bits: 0x02000000,
        response: DxCapsResponse::Info,
        message: "doesn't support high-quality polygon offset",
    },
    DxCapsCheckBits {
        offset: 40,
        set_bits: 0x00000000,
        clear_bits: 0x0000008D,
        response: DxCapsResponse::Quit,
        message: "doesn't support the required depth comparison modes",
    },
    DxCapsCheckBits {
        offset: 44,
        set_bits: 0x00000000,
        clear_bits: 0x000003FF,
        response: DxCapsResponse::Quit,
        message: "doesn't support the required frame buffer source blend modes",
    },
    DxCapsCheckBits {
        offset: 48,
        set_bits: 0x00000000,
        clear_bits: 0x000000D2,
        response: DxCapsResponse::Quit,
        message: "doesn't support the required frame buffer destination blend \
                  modes",
    },
    DxCapsCheckBits {
        offset: 60,
        set_bits: 0x00000000,
        clear_bits: 0x00000004,
        response: DxCapsResponse::Quit,
        message: "doesn't support alpha in texture",
    },
    DxCapsCheckBits {
        offset: 60,
        set_bits: 0x00000000,
        clear_bits: 0x00000800,
        response: DxCapsResponse::Quit,
        message: "doesn't support cubemap textures",
    },
    DxCapsCheckBits {
        offset: 60,
        set_bits: 0x00000000,
        clear_bits: 0x00004000,
        response: DxCapsResponse::Quit,
        message: "doesn't support mipmapped textures",
    },
    DxCapsCheckBits {
        offset: 60,
        set_bits: 0x00000002,
        clear_bits: 0x00000100,
        response: DxCapsResponse::Quit,
        message: "doesn't support restricted use of non-power-of-2 textures",
    },
    DxCapsCheckBits {
        offset: 60,
        set_bits: 0x00000000,
        clear_bits: 0x00000001,
        response: DxCapsResponse::Warn,
        message: "doesn't support perspective correct texturing",
    },
    DxCapsCheckBits {
        offset: 60,
        set_bits: 0x00000020,
        clear_bits: 0x00000000,
        response: DxCapsResponse::Quit,
        message: "doesn't support non-square textures",
    },
    DxCapsCheckBits {
        offset: 60,
        set_bits: 0x00000000,
        clear_bits: 0x03030300,
        response: DxCapsResponse::Quit,
        message: "doesn't support the required texture filtering modes",
    },
    DxCapsCheckBits {
        offset: 64,
        set_bits: 0x00000000,
        clear_bits: 0x03000300,
        response: DxCapsResponse::Quit,
        message: "doesn't support the required cubemap texture filtering modes",
    },
    DxCapsCheckBits {
        offset: 72,
        set_bits: 0x00000000,
        clear_bits: 0x00000004,
        response: DxCapsResponse::Quit,
        message: "doesn't support texture clamping",
    },
    DxCapsCheckBits {
        offset: 72,
        set_bits: 0x00000000,
        clear_bits: 0x00000001,
        response: DxCapsResponse::Quit,
        message: "doesn't support texture wrapping",
    },
    DxCapsCheckBits {
        offset: 136,
        set_bits: 0x00000000,
        clear_bits: 0x000001FF,
        response: DxCapsResponse::Info,
        message: "doesn't support the required stencil operations",
    },
    DxCapsCheckBits {
        offset: 212,
        set_bits: 0x00000000,
        clear_bits: 0x00000001,
        response: DxCapsResponse::Quit,
        message: "doesn't support vertex stream offsets",
    },
    DxCapsCheckBits {
        offset: 244,
        set_bits: 0x00000000,
        clear_bits: 0x00000200,
        response: DxCapsResponse::Warn,
        message: "doesn't support linear filtering when copying and shrinking \
                  the frame buffer",
    },
    DxCapsCheckBits {
        offset: 236,
        set_bits: 0x00000000,
        clear_bits: 0x00000001,
        response: DxCapsResponse::Quit,
        message: "doesn't support UBYTE4N vertex data",
    },
];

#[cfg(d3d9)]
const S_CAPS_CHECK_INT: [DxCapsCheckInteger; 10] = [
    DxCapsCheckInteger {
        offset: 88,
        min: 0x00000800,
        max: 0xFFFFFFFF,
        response: DxCapsResponse::Quit,
        message: "doesn't support large enough 2D textures",
    },
    DxCapsCheckInteger {
        offset: 92,
        min: 0x00000800,
        max: 0xFFFFFFFF,
        response: DxCapsResponse::Quit,
        message: "doesn't support large enough 2D textures",
    },
    DxCapsCheckInteger {
        offset: 96,
        min: 0x00000100,
        max: 0xFFFFFFFF,
        response: DxCapsResponse::Quit,
        message: "doesn't support large enough 3D textures",
    },
    DxCapsCheckInteger {
        offset: 148,
        min: 0x00000100,
        max: 0xFFFFFFFF,
        response: DxCapsResponse::Quit,
        message: "doesn't support enough texture coordinates for the DirectX \
                  9 code path",
    },
    DxCapsCheckInteger {
        offset: 152,
        min: 0x00000008,
        max: 0xFFFFFFFF,
        response: DxCapsResponse::Quit,
        message: "doesn't support enough textures for the DirectX 9 code path",
    },
    DxCapsCheckInteger {
        offset: 188,
        min: 0x00000001,
        max: 0xFFFFFFFF,
        response: DxCapsResponse::Quit,
        message: "is not a DirectX 9 driver",
    },
    DxCapsCheckInteger {
        offset: 196,
        min: 0xFFFE0200,
        max: 0xFFFFFFFF,
        response: DxCapsResponse::Quit,
        message: "doesn't support vertex shader 2.0 or better",
    },
    DxCapsCheckInteger {
        offset: 204,
        min: 0xFFFE0200,
        max: 0xFFFFFFFF,
        response: DxCapsResponse::Quit,
        message: "doesn't support pixel shader 2.0 or better",
    },
    DxCapsCheckInteger {
        offset: 196,
        min: 0xFFFE0300,
        max: 0xFFFFFFFF,
        response: DxCapsResponse::ForbidSm3,
        message: "doesn't support vertex shader 3.0 or better",
    },
    DxCapsCheckInteger {
        offset: 204,
        min: 0xFFFE0300,
        max: 0xFFFFFFFF,
        response: DxCapsResponse::Quit,
        message: "doesn't support pixel shader 3.0 or better",
    },
];

#[cfg(d3d9)]
fn check_dx_caps(caps: &D3DCAPS9) {
    for bit in S_CAPS_CHECK_BITS {
        let p = unsafe {
            *addr_of!(*caps)
                .cast::<u8>()
                .offset(bit.offset)
                .cast::<u32>()
        };
        if ((bit.clear_bits == 0) || ((!p & bit.clear_bits) != 0))
            && (bit.set_bits == 0 || ((p & bit.set_bits) != 0))
        {
            respond_to_missing_caps(bit.response, bit.message);
        }
    }

    for int in S_CAPS_CHECK_INT {
        let p = unsafe {
            *addr_of!(*caps)
                .cast::<u8>()
                .offset(int.offset)
                .cast::<u32>()
        };
        if p < int.min || (int.max <= p && p != int.max) {
            respond_to_missing_caps(int.response, int.message)
        }
    }
}

#[cfg(d3d9)]
fn pick_renderer(caps: &D3DCAPS9) {
    com::println!(
        8.into(),
        "Pixel shader version is {}.{}",
        (caps.PixelShaderVersion & 0xFFFF) >> 8,
        caps.PixelShaderVersion & 0xFF
    );
    com::println!(
        8.into(),
        "Vertex shader version is {}.{}",
        (caps.VertexShaderVersion & 0xFFFF) >> 8,
        caps.VertexShaderVersion & 0xFF
    );
    check_dx_caps(caps);
}

#[cfg(d3d9)]
fn check_transparency_msaa(adapter: Adapter) -> bool {
    if dvar::get_int("r_aaSamples").unwrap() == 1 {
        false
    } else {
        let dx = platform::render::d3d9::dx();
        // TODO - quality levels
        unsafe {
            dx.d3d9.as_ref().unwrap().CheckDeviceMultiSampleType(
                adapter.as_d3d9(),
                D3DDEVTYPE_HAL,
                D3DFMT_X8R8G8B8,
                false,
                D3DMULTISAMPLE_NONMASKABLE,
                core::ptr::null_mut(),
            )
        }
        .is_ok()
    }
}

#[cfg(d3d9)]
fn set_shadowmap_formats_dx(adapter: Adapter) {
    let format_1 =
        if platform::render::d3d9::nv_use_shadow_null_color_render_target() {
            D3DFMT_NULL
        } else {
            D3DFMT_D24S8
        };

    let formats = [
        (format_1, D3DFMT_R5G6B5),
        (D3DFMT_D24S8, D3DFMT_R5G6B5),
        (D3DFMT_D24S8, D3DFMT_X8R8G8B8),
        (D3DFMT_D24S8, D3DFMT_A8R8G8B8),
    ];

    let mut depth_stencil_format = formats[0].0;
    let mut render_target_format = formats[0].1;

    let mut valid = false;
    for i in 0..3 {
        depth_stencil_format = formats[i].0;
        render_target_format = formats[i].1;

        let dx = platform::render::d3d9::dx();
        if !platform::render::d3d9::nv_use_shadow_null_color_render_target()
            || render_target_format != D3DFMT_NULL
            || unsafe {
                dx.d3d9.as_ref().unwrap().CheckDeviceFormat(
                    adapter.as_d3d9(),
                    D3DDEVTYPE_HAL,
                    D3DFMT_X8R8G8B8,
                    D3DUSAGE_RENDERTARGET as _,
                    D3DRTYPE_SURFACE,
                    D3DFMT_NULL,
                )
            }
            .is_ok()
            || unsafe {
                dx.d3d9.as_ref().unwrap().CheckDepthStencilMatch(
                    adapter.as_d3d9(),
                    D3DDEVTYPE_HAL,
                    D3DFMT_X8R8G8B8,
                    render_target_format,
                    depth_stencil_format,
                )
            }
            .is_ok()
            || unsafe {
                dx.d3d9.as_ref().unwrap().CheckDeviceFormat(
                    adapter.as_d3d9(),
                    D3DDEVTYPE_HAL,
                    D3DFMT_X8R8G8B8,
                    D3DUSAGE_DEPTHSTENCIL as _,
                    D3DRTYPE_TEXTURE,
                    depth_stencil_format,
                )
            }
            .is_ok()
        {
            valid = true;
            break;
        }
    }

    if !valid {
        dvar::set_bool("sm_enable", false).unwrap();
        dvar::set_bool("ui_showShadowMapOptions", false).unwrap();
        let mut gm = platform::render::d3d9::gfx_metrics_mut();
        gm.shadowmap_format_primary = D3DFMT_R32F;
        gm.shadowmap_format_secondary = D3DFMT_D24X8;
        gm.shadowmap_build_tech_type = Some(ShadowmapBuildTechType::Color);
        gm.has_hardware_shadowmap = false;
        gm.shadowmap_sampler_state = Some(ShadowmapSamplerState::A);
    } else {
        dvar::set_bool("ui_showShadowMapOptions", true).unwrap();
        let mut gm = platform::render::d3d9::gfx_metrics_mut();
        gm.shadowmap_format_primary = depth_stencil_format;
        gm.shadowmap_format_secondary = render_target_format;
        gm.shadowmap_build_tech_type = Some(ShadowmapBuildTechType::Depth);
        gm.has_hardware_shadowmap = true;
        gm.shadowmap_sampler_state = Some(ShadowmapSamplerState::B);
    }
}

#[cfg(d3d9)]
fn store_direct3d_caps(adapter: Adapter) {
    let caps = get_direct3d_caps(adapter);
    pick_renderer(&caps);
    let max_texture_dimension =
        if (caps.MaxTextureHeight as i32) < caps.MaxTextureHeight as i32 {
            caps.MaxTextureWidth
        } else {
            caps.MaxTextureHeight
        };
    {
        let mut vc = vid::config_mut();
        vc.device_supports_gamma =
            (caps.Caps2 & D3DCAPS2_FULLSCREENGAMMA as u32) != 0;
        vc.max_texture_maps = 16;
        vc.max_texture_size = max_texture_dimension as _;
    }

    {
        let mut gm = platform::render::d3d9::gfx_metrics_mut();
        gm.max_clip_planes = (caps.MaxUserClipPlanes as i32).clamp(6, i32::MAX);
        gm.has_anisotropic_min_filter =
            (caps.TextureFilterCaps & D3DPTFILTERCAPS_MINFANISOTROPIC) != 0;
        gm.has_anisotropic_mag_filter =
            (caps.TextureFilterCaps & D3DPTFILTERCAPS_MAGFANISOTROPIC) != 0;
        gm.max_anisotropy = caps.MaxAnisotropy as _;
        gm.slope_scale_depth_bias =
            caps.RasterCaps & D3DPRASTERCAPS_SLOPESCALEDEPTHBIAS as u32 != 0;
        gm.can_mip_cubemaps =
            caps.TextureCaps & D3DPTEXTURECAPS_MIPCUBEMAP as u32 != 0;
        gm.has_transparency_msaa = check_transparency_msaa(adapter);
        set_shadowmap_formats_dx(adapter);
    }
}

#[cfg(d3d9)]
fn enum_display_modes(adapter: Adapter) {
    let mut dx = platform::render::d3d9::dx_mut();
    let display_mode_count =
        unsafe { dx.d3d9.as_ref().unwrap().GetAdapterCount() };

    let mut i = 0;
    loop {
        if i >= display_mode_count
            || dx.display_modes.len() >= dx.display_modes.capacity()
        {
            break;
        }

        let mut mode = D3DDISPLAYMODE::default();
        if unsafe {
            dx.d3d9
                .as_ref()
                .unwrap()
                .EnumAdapterModes(
                    adapter.as_d3d9(),
                    D3DFMT_X8R8G8B8,
                    i,
                    addr_of_mut!(mode),
                )
                .is_ok()
        } {
            if mode.RefreshRate == 0 {
                mode.RefreshRate = 60;
            }
            dx.display_modes.push(mode);
        }

        i += 1;
    }

    dx.display_modes.sort_by(|a, b| {
        a.Width.cmp(&b.Width).then(
            a.Height
                .cmp(&b.Height)
                .then(a.RefreshRate.cmp(&b.RefreshRate)),
        )
    });
    let mut modes = dx
        .display_modes
        .iter()
        .filter(|&a| {
            a.Width >= MIN_HORIZONTAL_RESOLUTION
                && a.Height >= MIN_VERTICAL_RESOLUTION
        })
        .copied()
        .collect::<Vec<_>>();
    modes.dedup();
    for i in 0..modes.len() {
        dx.display_modes[i] = modes[i];
    }

    if dx.display_modes.is_empty() {
        fatal_init_error!(
            "No valid resolutions of {} x {} or above found",
            MIN_HORIZONTAL_RESOLUTION,
            MIN_VERTICAL_RESOLUTION
        );
    }

    let mut resolutions = dx
        .display_modes
        .iter()
        .map(|a| (a.Width, a.Height))
        .filter(|a| a.0 >= 640 && a.1 >= 480)
        .collect::<Vec<_>>();
    resolutions.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    resolutions.dedup();

    let mut refreshes = dx
        .display_modes
        .iter()
        .map(|a| a.RefreshRate)
        .collect::<Vec<_>>();
    refreshes.sort();
    refreshes.dedup();

    let mode_strings = resolutions
        .iter()
        .map(|a| format!("{}x{}", a.0, a.1))
        .collect::<Vec<_>>();
    for i in 0..mode_strings.len() {
        dx.resolution_name_table[i] = mode_strings[i].clone();
    }
    let refresh_strings = refreshes
        .iter()
        .map(|a| format!("{} Hz", *a))
        .collect::<Vec<_>>();
    for i in 0..refresh_strings.len() {
        dx.refresh_rate_name_table[i] = refresh_strings[i].clone();
    }

    dvar::register_enumeration(
        "r_mode",
        mode_strings.last().unwrap().clone(),
        Some(mode_strings),
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::LATCHED,
        Some("Direct X resolution mode"),
    )
    .unwrap();
    dvar::register_enumeration(
        "r_displayRefresh",
        refresh_strings.last().unwrap().clone(),
        Some(refresh_strings),
        dvar::DvarFlags::ARCHIVE
            | dvar::DvarFlags::LATCHED
            | dvar::DvarFlags::CHANGEABLE_RESET,
        Some("Refresh rate"),
    )
    .unwrap();
}

#[cfg(d3d9)]
fn pre_create_window() -> Result<(), ()> {
    let mut dx = platform::render::d3d9::dx_mut();
    assert!(
        dx.d3d9.is_none(),
        "D3D re-initialized before being shutdown"
    );

    com::println!(8.into(), "Getting Direct3D 9 interface...");
    let Some(d3d9) = (unsafe { Direct3DCreate9(D3D_SDK_VERSION) }) else {
        com::println!(8.into(), "Direct3D 9 failed to initialize");
        return Err(());
    };

    dx.d3d9 = Some(d3d9);

    dx.adapter = choose_adapter().unwrap_or_default();

    store_direct3d_caps(dx.adapter);
    enum_display_modes(dx.adapter);

    let mut identifier = D3DADAPTER_IDENTIFIER9::default();
    dx.vendor_id = identifier.VendorId;
    if unsafe {
        dx.d3d9.as_ref().unwrap().GetAdapterIdentifier(
            dx.adapter.as_d3d9(),
            0,
            addr_of_mut!(identifier),
        )
    }
    .is_ok()
        && identifier.VendorId == D3D_VENDOR_ID_NVIDIA
    {
        dx.nv_initialized = unsafe { NvAPI_Initialize() == NVAPI_OK }
    }

    Ok(())
}

#[cfg(vulkan)]
fn pre_create_window() -> Result<(), ()> {
    todo!()
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

#[cfg(wgpu)]
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
                fatal_init_error!("Couldn't initialize renderer")
            }
        }

        Ok(())
    } else {
        init_systems()
    }
}

#[cfg(d3d9)]
fn init_graphics_api() -> Result<(), ()> {
    let b = {
        let dx = platform::render::d3d9::dx();
        assert!(dx.device.is_some() == dx.d3d9.is_some());
        dx.device.is_some()
    };

    if b {
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
                fatal_init_error!("Couldn't initialize renderer")
            }
        }

        Ok(())
    } else {
        init_systems()
    }
}

#[cfg(vulkan)]
fn init_graphics_api() -> Result<(), ()> {
    let b = {
        let vk = platform::render::vulkan::vk();
        assert!(vk.device.is_some() == vk.physical_device.is_some() && vk.physical_device.is_some() == vk.instance.is_some());
        vk.device.is_some()
    };

    if b {
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
                fatal_init_error!("Couldn't initialize renderer")
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

#[cfg(d3d9)]
fn get_device_type() -> D3DDEVTYPE {
    let mut dx = platform::render::d3d9::dx_mut();
    dx.adapter = Adapter::from_d3d9(0);

    let mut i = 0;
    let mut d3d_id = D3DADAPTER_IDENTIFIER9::default();
    loop {
        let adapter_count =
            unsafe { dx.d3d9.as_ref().unwrap().GetAdapterCount() };
        if i > adapter_count {
            return D3DDEVTYPE_HAL;
        }

        if unsafe {
            dx.d3d9.as_ref().unwrap().GetAdapterIdentifier(
                i,
                0,
                addr_of_mut!(d3d_id),
            )
        }
        .is_ok()
        {
            let desc = CStr::from_bytes_until_nul(&d3d_id.Description).unwrap();
            if desc.to_string_lossy().contains("PerfHUD") {
                dx.adapter = Adapter::from_d3d9(i);
                return D3DDEVTYPE_REF;
            }
        }

        i += 1;
    }
}

#[cfg(d3d9)]
fn create_device_internal(
    hwnd: HWND,
    behavior_flags: u32,
    d3dpp: &mut D3DPRESENT_PARAMETERS,
) -> Result<(), windows::core::Error> {
    com::println!(8.into(), "Creating Direct3D device...");

    let mut d3ddm = D3DDISPLAYMODE::default();
    let mut i = 0;
    let r = loop {
        if let Some((w, h)) = get_monitor_dimensions() {
            let mut dx = platform::render::d3d9::dx_mut();
            dx.adapter_native_is_valid = true;
            dx.adapter_native_width = w as _;
            dx.adapter_native_height = h as _;
        } else {
            platform::render::d3d9::dx_mut().adapter_native_is_valid = false;
        }

        d3dpp.hDeviceWindow = HWND(0);
        let dev_type = get_device_type();
        let r = {
            let mut dx = platform::render::d3d9::dx_mut();
            let mut device = None;
            let r = unsafe {
                dx.d3d9.as_ref().unwrap().CreateDevice(
                    dx.adapter.as_d3d9(),
                    dev_type,
                    hwnd,
                    behavior_flags,
                    addr_of_mut!(*d3dpp),
                    addr_of_mut!(device),
                )
            };
            dx.device = device;
            if r.is_ok() {
                dx.nv_stereo_activated = false;
                // TODO - NvAPI_Stereo_CreateHandleFromIUnknown
                // TODO - NvAPI_Stereo_IsActivated

                if unsafe {
                    dx.d3d9.as_ref().unwrap().GetAdapterDisplayMode(
                        dx.adapter.as_d3d9(),
                        addr_of_mut!(d3ddm),
                    )
                }
                .is_ok()
                {
                    dx.adapter_fullscreen_width = d3ddm.Width as _;
                    dx.adapter_fullscreen_height = d3ddm.Height as _;
                    return Ok(());
                } else {
                    dx.adapter_fullscreen_width = d3dpp.BackBufferWidth as _;
                    dx.adapter_fullscreen_height = d3dpp.BackBufferHeight as _;
                    return Ok(());
                }
            };
            r
        };
        unsafe { Sleep(100) };
        if let Err(e) = r.clone() {
            if e.code() != D3DERR_INVALID_CALL {
                i += 1;
            }
        } else {
            i += 1;
        }

        if i >= 20 {
            break r;
        }
    };

    let mut dx = platform::render::d3d9::dx_mut();
    if dx.adapter.as_d3d9() != 0 {
        dx.adapter = Adapter::from_d3d9(0);
        drop(dx);
        return create_device_internal(hwnd, behavior_flags, d3dpp);
    }

    r
}

#[cfg(d3d9)]
fn is_depth_stencil_format_ok(
    rt: D3DFORMAT,
    ds: D3DFORMAT,
) -> Result<(), windows::core::Error> {
    let dx = platform::render::d3d9::dx();
    unsafe {
        dx.d3d9.as_ref().unwrap().CheckDeviceFormat(
            dx.adapter.as_d3d9(),
            D3DDEVTYPE_HAL,
            D3DFMT_X8R8G8B8,
            D3DUSAGE_DEPTHSTENCIL as _,
            D3DRTYPE_SURFACE,
            ds,
        )
    }?;
    unsafe {
        dx.d3d9.as_ref().unwrap().CheckDepthStencilMatch(
            dx.adapter.as_d3d9(),
            D3DDEVTYPE_HAL,
            D3DFMT_X8R8G8B8,
            rt,
            ds,
        )
    }
}

#[cfg(d3d9)]
fn get_depth_stencil_format(fmt: D3DFORMAT) -> D3DFORMAT {
    if is_depth_stencil_format_ok(fmt, D3DFMT_D24FS8).is_ok() {
        D3DFMT_D24FS8
    } else {
        D3DFMT_D24S8
    }
}

#[cfg(d3d9)]
fn setup_anti_aliasing(wnd_parms: &gfx::WindowParms) {
    assert!(
        wnd_parms.aa_samples >= 1 && wnd_parms.aa_samples <= 16,
        "wnd_parms->aa_samples not in [1, 16]\n\t{} not in [{}, {}]",
        wnd_parms.aa_samples,
        1,
        16
    );

    let mut ms_type =
        if dvar::get_bool("r_reflectionProbeGenerate").unwrap() == false {
            D3DMULTISAMPLE_TYPE(wnd_parms.aa_samples as _)
        } else {
            D3DMULTISAMPLE_NONMASKABLE
        };

    loop {
        let mut dx = platform::render::d3d9::dx_mut();
        if ms_type == D3DMULTISAMPLE_NONMASKABLE
            || ms_type == D3DMULTISAMPLE_NONE
        {
            dx.multi_sample_type = D3DMULTISAMPLE_NONE;
            dx.multi_sample_quality = 0;
            return;
        }
        dx.multi_sample_type = ms_type;
        let mut quality_levels = 0;
        if unsafe {
            dx.d3d9.as_ref().unwrap().CheckDeviceMultiSampleType(
                0,
                D3DDEVTYPE_HAL,
                D3DFMT_A8R8G8B8,
                wnd_parms.fullscreen == false,
                ms_type,
                addr_of_mut!(quality_levels),
            )
        }
        .is_ok()
        {
            break;
        }
        ms_type = D3DMULTISAMPLE_TYPE(ms_type.0 + 1);
    }
    com::println!(8.into(), "Using {}x anti-aliasing", ms_type.0);
    platform::render::d3d9::dx_mut().multi_sample_quality = 0;
}

#[cfg(d3d9)]
fn set_d3d_present_parameters(
    d3dpp: &mut D3DPRESENT_PARAMETERS,
    wnd_parms: &gfx::WindowParms,
) {
    setup_anti_aliasing(wnd_parms);
    *d3dpp = D3DPRESENT_PARAMETERS::default();
    d3dpp.BackBufferHeight = wnd_parms.display_height;
    d3dpp.BackBufferWidth = wnd_parms.display_width;
    d3dpp.BackBufferFormat = D3DFMT_A8R8G8B8;
    d3dpp.BackBufferCount = 1;
    d3dpp.MultiSampleType = platform::render::d3d9::dx().multi_sample_type;
    d3dpp.MultiSampleQuality =
        platform::render::d3d9::dx().multi_sample_quality;
    d3dpp.SwapEffect = D3DSWAPEFFECT_DISCARD;
    d3dpp.EnableAutoDepthStencil = BOOL(0);
    d3dpp.AutoDepthStencilFormat =
        platform::render::d3d9::dx().depth_stencil_format;
    d3dpp.PresentationInterval = if dvar::get_bool("r_vsync").unwrap() == true {
        D3DPRESENT_INTERVAL_ONE as _
    } else {
        D3DPRESENT_INTERVAL_IMMEDIATE as _
    };
    assert!(wnd_parms.window_handle.is_some());
    d3dpp.hDeviceWindow =
        HWND(wnd_parms.window_handle.unwrap().get_win32().unwrap().hwnd as _);
    d3dpp.Flags = 0;
    if wnd_parms.fullscreen == false {
        d3dpp.Windowed = BOOL(1);
        d3dpp.FullScreen_RefreshRateInHz = 0;
    } else {
        d3dpp.Windowed = BOOL(0);
        d3dpp.FullScreen_RefreshRateInHz = wnd_parms.hz as _;
    }
}

#[cfg(d3d9)]
fn create_device(wnd_parms: &gfx::WindowParms) -> Result<(), ()> {
    let mut dx = platform::render::d3d9::dx_mut();
    assert_eq!(dx.window_count, 0);
    assert!(wnd_parms.window_handle.is_some());
    assert!(dx.device.is_some());
    dx.depth_stencil_format = get_depth_stencil_format(D3DFMT_A8R8G8B8);
    let mut d3dpp = D3DPRESENT_PARAMETERS::default();
    set_d3d_present_parameters(&mut d3dpp, wnd_parms);
    let behavior_flags =
        if dvar::get_bool("r_multithreaded_device").unwrap() == true {
            D3DCREATE_MULTITHREADED | D3DCREATE_HARDWARE_VERTEXPROCESSING
        } else {
            D3DCREATE_HARDWARE_VERTEXPROCESSING
        };
    if let Err(e) = create_device_internal(
        HWND(wnd_parms.window_handle.unwrap().get_win32().unwrap().hwnd as _),
        behavior_flags as _,
        &mut d3dpp,
    ) {
        com::println!(
            8.into(),
            "Couldn't create a Direct3D device: {}",
            e.message()
        );
        Err(())
    } else {
        assert!(dx.device.is_some());
        Ok(())
    }
}

#[cfg(wgpu)]
fn create_device_internal(_wnd_parms: &gfx::WindowParms) -> Result<(), ()> {
    com::println!(8.into(), "Creating Render device...");

    let mut rg = RENDER_GLOBALS.write().unwrap();
    rg.device = block_on(Device::new(rg.adapter.as_ref().unwrap()));

    (rg.adapter_native_width, rg.adapter_native_height) =
        get_monitor_dimensions().unwrap();
    rg.device = block_on(platform::render::wgpu::Device::new(
        rg.adapter.as_ref().unwrap(),
    ));
    if rg.device.is_none() {
        return Err(());
    }

    Ok(())
}

#[cfg(wgpu)]
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

lazy_static! {
    static ref R_GLOB: RwLock<gfx::Globals> =
        RwLock::new(gfx::Globals::default());
}

pub fn r_glob() -> RwLockReadGuard<'static, gfx::Globals> {
    R_GLOB.read().unwrap()
}

pub fn r_glob_mut() -> RwLockWriteGuard<'static, gfx::Globals> {
    R_GLOB.write().unwrap()
}
