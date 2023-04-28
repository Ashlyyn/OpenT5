use raw_window_handle::{RawWindowHandle, WaylandWindowHandle, XlibWindowHandle, XlibDisplayHandle, RawDisplayHandle};
use cfg_if::cfg_if;
use x11::xlib::{XMapWindow, XOpenDisplay};

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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MonitorHandle {
    Xlib(XlibDisplayHandle),
    Wayland(()),
}

impl Ord for MonitorHandle {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match *self {
            Self::Xlib(handle) => handle.display.cmp(&other.get_xlib().unwrap().display).then(handle.screen.cmp(&other.get_xlib().unwrap().screen)),
            Self::Wayland(()) => ().cmp(&()),
        }
    }
}

impl PartialOrd for MonitorHandle {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Win32 => Win32
// Linux => Xlib, Wayland
// macOS => Xlib, AppKit, UiKit
// Other Unix => Xlib
impl MonitorHandle {
    cfg_if! {
        if #[cfg(feature = "linux_use_wayland")] {
            pub const fn get(&self) -> RawDisplayHandle {
                match *self {
                    Self::Wayland(handle) => RawDisplayHandle::Wayland(handle),
                    _ => panic!()
                }  
            }
        } else {
            pub const fn get(&self) -> RawDisplayHandle {
                match *self {
                    Self::Xlib(handle) => RawDisplayHandle::Xlib(handle),
                    _ => panic!()
                }  
            }
        }
    }

    pub const fn get_xlib(&self) -> Option<XlibDisplayHandle> {
        match *self {
            Self::Xlib(handle) => Some(handle),
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