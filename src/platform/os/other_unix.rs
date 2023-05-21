// This file is for any Unix-specific initialization that
// should be done before the rest of main() executes

pub fn main() {}

impl WindowHandle {
    pub const fn new(handle: RawWindowHandle) -> Self {
        Self(handle)
    }

    pub const fn get(&self) -> RawWindowHandle {
        self.0
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
}

// Win32 => Win32
// Linux => Xlib, Wayland
// macOS => Xlib, AppKit
// Other Unix => Xlib
impl MonitorHandle {
    pub const fn get_xlib(&self) -> Option<(*mut c_void, i32)> {
        match *self {
            Self::Xlib { display, screen } => Some((display, screen)),
            _ => None,
        }
    }
}
