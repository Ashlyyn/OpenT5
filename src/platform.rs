#![allow(dead_code)]

pub mod arch;
pub mod os;
pub mod display_server;

use std::sync::RwLock;
extern crate alloc;
use alloc::sync::Arc;

use lazy_static::lazy_static;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct WindowHandle(pub RawWindowHandle);

// SAFETY:
// Really don't know if this is safe. It hasn't created any problems in
// testing, we'll see if any pop up later.
unsafe impl Sync for WindowHandle {}
// SAFETY:
// Really don't know if this is safe. It hasn't created any problems in
// testing, we'll see if any pop up later.
unsafe impl Send for WindowHandle {}

// SAFETY:
// Really don't know if this is safe. It hasn't created any problems in
// testing, we'll see if any pop up later.
unsafe impl HasRawWindowHandle for WindowHandle {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.get()
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
    pub const fn new(active_app: bool, is_minimized: bool) -> Self {
        Self {
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
    *G_PLATFORM_VARS.read().unwrap()
}

pub fn set_platform_vars(vars: PlatformVars) {
    *G_PLATFORM_VARS.write().unwrap() = vars;
}

pub fn get_window_handle() -> Option<WindowHandle> {
    G_PLATFORM_VARS.read().unwrap().window_handle
}

pub fn set_window_handle(window_handle: WindowHandle) {
    G_PLATFORM_VARS.write().unwrap().window_handle = Some(window_handle);
}

pub fn clear_window_handle() {
    G_PLATFORM_VARS.write().unwrap().window_handle = None;
}

pub fn get_active_app() -> bool {
    G_PLATFORM_VARS.read().unwrap().active_app
}

pub fn set_active_app() {
    G_PLATFORM_VARS.write().unwrap().active_app = true;
}

pub fn clear_active_app() {
    G_PLATFORM_VARS.write().unwrap().active_app = false;
}

pub fn get_minimized() -> bool {
    G_PLATFORM_VARS.read().unwrap().active_app
}

pub fn set_minimized() {
    G_PLATFORM_VARS.write().unwrap().is_minimized = true;
}

pub fn clear_minimized() {
    G_PLATFORM_VARS.write().unwrap().is_minimized = false;
}

pub fn get_msg_time() -> isize {
    G_PLATFORM_VARS.read().unwrap().sys_msg_time
}

pub fn set_msg_time(msg_time: isize) {
    G_PLATFORM_VARS.write().unwrap().sys_msg_time = msg_time;
}

#[derive(Copy, Clone, Debug)]
pub struct FontHandle(pub isize);
