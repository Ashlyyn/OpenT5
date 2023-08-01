use crate::platform::WindowHandle;
use cfg_if::cfg_if;
use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, RawDisplayHandle, RawWindowHandle,
    XlibDisplayHandle, XlibWindowHandle,
};

cfg_if! {
    if #[cfg(appkit)] {
        use icrate::AppKit::NSWindow;
        use objc2::rc::Id;
    }
}

pub fn main() {}

#[cfg(appkit)]
pub trait AppKitWindowHandleExt {
    fn ns_window(&self) -> Id<NSWindow>;
}

#[cfg(appkit)]
impl AppKitWindowHandleExt for AppKitWindowHandle {
    fn ns_window(&self) -> Id<NSWindow> {
        unsafe { Id::new(self.ns_window as *mut NSWindow) }.unwrap()
    }
}

impl WindowHandle {
    pub const fn new(handle: RawWindowHandle) -> Self {
        Self(handle)
    }

    pub const fn get(&self) -> RawWindowHandle {
        self.0
    }

    pub const fn get_appkit(&self) -> Option<AppKitWindowHandle> {
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
    AppKit(AppKitDisplayHandle),
}

#[allow(clippy::missing_trait_methods)]
impl Ord for MonitorHandle {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match *self {
            Self::Xlib(handle) => handle
                .display
                .cmp(&other.get_xlib().unwrap().display)
                .then(handle.screen.cmp(&other.get_xlib().unwrap().screen)),
            Self::AppKit(_) => ().cmp(&()),
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
        if #[cfg(appkit)] {
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

    pub const fn get_appkit(&self) -> Option<AppKitDisplayHandle> {
        match *self {
            Self::AppKit(handle) => Some(handle),
            _ => None,
        }
    }
}
