#![allow(dead_code)]

pub mod os;

use std::sync::{Arc, RwLock};

use lazy_static::lazy_static;

use raw_window_handle::{RawWindowHandle, Win32WindowHandle, XlibWindowHandle, XcbWindowHandle, WaylandWindowHandle, UiKitWindowHandle, AppKitWindowHandle};

#[derive(Copy, Clone, Debug)]
pub struct WindowHandle(RawWindowHandle);

unsafe impl Sync for WindowHandle {}
unsafe impl Send for WindowHandle {}

impl WindowHandle {
    pub fn new(handle: RawWindowHandle) -> Self {
        Self(handle)
    }

    pub fn get(&self) -> RawWindowHandle {
        self.0
    }

    pub fn get_win32(&self) -> Option<Win32WindowHandle> {
        match self.get() {
            RawWindowHandle::Win32(handle) => Some(handle),
            _ => None,
        }
    }

    pub fn get_xlib(&self) -> Option<XlibWindowHandle> {
        match self.get() {
            RawWindowHandle::Xlib(handle) => Some(handle),
            _ => None,
        }
    }

    pub fn get_xcb(&self) -> Option<XcbWindowHandle> {
        match self.get() {
            RawWindowHandle::Xcb(handle) => Some(handle),
            _ => None,
        }
    }

    pub fn get_wayland(&self) -> Option<WaylandWindowHandle> {
        match self.get() {
            RawWindowHandle::Wayland(handle) => Some(handle),
            _ => None,
        }
    }

    pub fn get_ui_kit(&self) -> Option<UiKitWindowHandle> {
        match self.get() {
            RawWindowHandle::UiKit(handle) => Some(handle),
            _ => None,
        }
    }

    pub fn get_app_kit(&self) -> Option<AppKitWindowHandle> {
        match self.get() {
            RawWindowHandle::AppKit(handle) => Some(handle),
            _ => None,
        }
    }
}

#[derive(Copy, Clone)]
pub struct PlatformVars {
    pub window_handle: Option<WindowHandle>,
    pub active_app: bool,
    pub is_minimized: bool,
    pub recenter_mouse: bool,
    pub sys_msg_time: isize,
}

impl PlatformVars {
    pub fn new(active_app: bool, is_minimized: bool) -> Self {
        PlatformVars {
            window_handle: None,
            active_app,
            is_minimized,
            recenter_mouse: false,
            sys_msg_time: 0,
        }
    }
}

lazy_static! {
    static ref G_PLATFORM_VARS: Arc<RwLock<PlatformVars>> =
        Arc::new(RwLock::new(PlatformVars::new(false, false)));
}

pub fn get_platform_vars() -> PlatformVars {
    *G_PLATFORM_VARS.try_read().expect("")
}

pub fn set_platform_vars(vars: PlatformVars) {
    *G_PLATFORM_VARS.try_write().expect("") = vars;
}

pub fn get_window_handle() -> Option<WindowHandle> {
    G_PLATFORM_VARS.try_read().expect("").window_handle
}

pub fn set_window_handle(window_handle: WindowHandle) {
    G_PLATFORM_VARS.try_write().expect("").window_handle = Some(window_handle);
}

pub fn clear_window_handle() {
    G_PLATFORM_VARS.try_write().expect("").window_handle = None;
}

pub fn get_active_app() -> bool {
    G_PLATFORM_VARS.try_read().expect("").active_app
}

pub fn set_active_app() {
    G_PLATFORM_VARS.try_write().expect("").active_app = true;
}

pub fn clear_active_app() {
    G_PLATFORM_VARS.try_write().expect("").active_app = false;
}

pub fn get_minimized() -> bool {
    G_PLATFORM_VARS.try_read().expect("").active_app
}

pub fn set_minimized() {
    G_PLATFORM_VARS.try_write().expect("").is_minimized = true;
}

pub fn clear_minimized() {
    G_PLATFORM_VARS.try_write().expect("").is_minimized = false;
}

pub fn get_msg_time() -> isize {
    G_PLATFORM_VARS.try_read().expect("").sys_msg_time
}

pub fn set_msg_time(msg_time: isize) {
    G_PLATFORM_VARS.try_write().expect("").sys_msg_time = msg_time;
}

#[derive(Copy, Clone, Debug)]
pub struct FontHandle(pub isize);