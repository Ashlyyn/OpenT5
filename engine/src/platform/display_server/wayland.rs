// Linux can use Wayland with the `linux_use_wayland` feature enabled. Otherwise
// we default to Xlib.

// It's probably going to be a *very* hot minute before we implement Wayland
// support just due to the effort it's going to take and the state of the
// various Wayland crates.

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle,
    WaylandWindowHandle,
};

use crate::platform::WindowHandle;

pub fn init() {}

pub trait WindowHandleExt {
    fn get_wayland(&self) -> Option<WaylandWindowHandle>;
}

impl WindowHandleExt for WindowHandle {
    fn get_wayland(&self) -> Option<WaylandWindowHandle> {
        match self.get() {
            RawWindowHandle::Wayland(handle) => Some(handle),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MonitorHandle {
    Wayland(WaylandDisplayHandle),
}

#[allow(clippy::missing_trait_methods)]
impl Ord for MonitorHandle {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match *self {
            Self::Wayland(handle) => {
                handle.display.cmp(&other.get_wayland().unwrap().display)
            }
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
    pub const fn get(&self) -> RawDisplayHandle {
        match *self {
            Self::Wayland(handle) => RawDisplayHandle::Wayland(handle),
            _ => panic!(),
        }
    }

    pub const fn get_wayland(&self) -> Option<WaylandDisplayHandle> {
        match *self {
            Self::Wayland(handle) => Some(handle),
            _ => None,
        }
    }
}
