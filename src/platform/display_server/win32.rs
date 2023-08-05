// Don't know if this file is actually necessary since the display server
// is so tightly-coupled with the rest of the OS. Just here now for
// compeletness's sake.

use raw_window_handle::{
    HasRawDisplayHandle, RawDisplayHandle, RawWindowHandle, Win32WindowHandle,
    WindowsDisplayHandle,
};

pub fn init() {}

pub trait WindowHandleExt {
    fn get_win32(&self) -> Option<Win32WindowHandle>;
    fn from_win32(hwnd: HWND, hinstance: Option<HMODULE>) -> Self;
}

impl WindowHandleExt for WindowHandle {
    const fn get_win32(&self) -> Option<Win32WindowHandle> {
        match self.get() {
            RawWindowHandle::Win32(handle) => Some(handle),
            _ => None,
        }
    }

    fn from_win32(hwnd: HWND, hinstance: Option<HMODULE>) -> Self {
        let mut h = Win32WindowHandle::empty();
        h.hwnd = hwnd.0 as _;
        h.hinstance = hinstance.unwrap_or_default().0 as _;
        Self(RawWindowHandle::Win32(h))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MonitorHandle {
    Win32(isize),
}

#[allow(clippy::missing_trait_methods)]
impl Ord for MonitorHandle {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.get_win32()
            .unwrap()
            .0
            .cmp(&other.get_win32().unwrap().0)
    }
}

#[allow(clippy::missing_trait_methods)]
impl PartialOrd for MonitorHandle {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// SAFETY:
// Always safe somce WindowsDisplayHandle is a unit struct
unsafe impl HasRawDisplayHandle for WindowHandle {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        RawDisplayHandle::Windows(WindowsDisplayHandle::empty())
    }
}

// SAFETY:
// Really don't know if this is safe. It hasn't created any problems in
// testing, we'll see if any pop up later.
unsafe impl Sync for MonitorHandle {}
// SAFETY:
// Really don't know if this is safe. It hasn't created any problems in
// testing, we'll see if any pop up later.
unsafe impl Send for MonitorHandle {}

impl MonitorHandle {
    #[allow(clippy::unnecessary_wraps, clippy::trivially_copy_pass_by_ref)]
    pub const fn get_win32(&self) -> Option<HMONITOR> {
        match *self {
            Self::Win32(hmonitor) => Some(HMONITOR(hmonitor)),
        }
    }
}