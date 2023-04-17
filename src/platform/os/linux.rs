use libc::c_void;
use raw_window_handle::{RawWindowHandle, WaylandWindowHandle, XlibWindowHandle};
use cfg_if::cfg_if;

use crate::platform::WindowHandle;

pub fn main() {
    gtk4::init().unwrap();
}

impl WindowHandle {
    pub const fn new(handle: RawWindowHandle) -> Self {
        Self(handle)
    }

    pub const fn get(&self) -> RawWindowHandle {
        self.0
    }

    pub const fn get_wayland(&self) -> Option<WaylandWindowHandle> {
        match self.get() {
            RawWindowHandle::Wayland(handle) => Some(handle),
            _ => None,
        }
    }

    pub const fn get_xlib(&self) -> Option<XlibWindowHandle> {
        match self.get() {
            RawWindowHandle::Xlib(handle) => Some(handle),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum MonitorHandle {
    Xlib { display: *mut c_void, screen: i32 },
    Wayland(()),
}

// Win32 => Win32
// Linux => Xlib, Wayland
// macOS => Xlib, AppKit, UiKit
// Other Unix => Xlib
impl MonitorHandle {
    pub const fn get_xlib(&self) -> Option<(*mut c_void, i32)> {
        match *self {
            Self::Xlib { display, screen }=> Some((display, screen)),
            _ => None,
        }
    }

    pub const fn get_wayland(&self) -> Option<()> {
        match *self {
            Self::Wayland(_) => Some(()),
            _ => None,
        }
    }
}

cfg_if! {
    if #[cfg(feature = "linux_use_wayland")] {
        fn show_window(handle: WindowHandle) {
            let handle = handle.as_wayland().unwrap();
            todo!()
        }
    } else {
        fn show_window(handle: WindowHandle) {
            let handle = handle.as_xlib().unwrap();
            todo!()
        }
    }
}