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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum MonitorHandle {
    Xlib { display: *mut c_void, screen: i32 },
    AppKit(()),
}

cfg_if! {
    if #[cfg(feature = "macos_use_appkit")] {
        pub fn show_window(handle: WindowHandle) {
            let _handle = handle.get_wayland().unwrap();
            todo!()
        }

        pub fn focus_window(handle: WindowHandle) {
            let _handle = handle.get_wayland().unwrap();
            todo!()
        }
    } else {
        pub fn show_window(handle: WindowHandle) {
            let handle = handle.get_xlib().unwrap();
            let display = unsafe { XOpenDisplay(core::ptr::null()) };
            unsafe { XMapWindow(display, handle.window) };
        }

        pub fn focus_window(handle: WindowHandle) {
            let handle = handle.get_xlib().unwrap();
            let display = unsafe { XOpenDisplay(core::ptr::null()) };
            unsafe { XSetInputFocus(display, handle.window, RevertToParent, CurrentTime) };
        }
    }
}