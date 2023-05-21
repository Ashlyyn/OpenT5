use crate::platform::WindowHandle;
use cfg_if::cfg_if;
use core::ffi::c_void;
use raw_window_handle::{
    AppKitWindowHandle, RawDisplayHandle, RawWindowHandle, XlibDisplayHandle,
    XlibWindowHandle,
};

pub fn main() {}

impl WindowHandle {
    pub const fn new(handle: RawWindowHandle) -> Self {
        Self(handle)
    }

    pub const fn get(&self) -> RawWindowHandle {
        self.0
    }

    pub const fn get_app_kit(&self) -> Option<AppKitWindowHandle> {
        match self.get() {
            RawWindowHandle::AppKit(handle) => Some(handle),
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
    AppKit(()),
}

#[allow(clippy::missing_trait_methods)]
impl Ord for MonitorHandle {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match *self {
            Self::Xlib(handle) => handle
                .display
                .cmp(&other.get_xlib().unwrap().display)
                .then(handle.screen.cmp(&other.get_xlib().unwrap().screen)),
            Self::AppKit(()) => ().cmp(&()),
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl PartialOrd for MonitorHandle {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl MonitorHandle {
    cfg_if! {
        if #[cfg(feature = "macos_use_appkit")] {
            pub const fn get(&self) -> RawDisplayHandle {
                match *self {
                    Self::AppKit(handle) => RawDisplayHandle::AppKit(handle),
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

    pub const fn get_app_kit(&self) -> Option<()> {
        match *self {
            Self::AppKit(_) => Some(()),
            _ => None,
        }
    }
}
