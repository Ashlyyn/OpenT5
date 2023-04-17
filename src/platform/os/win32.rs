// This file is for any Windows-specific initialization that
// should be done before the rest of main() executes

#![allow(non_snake_case)]
use std::{collections::VecDeque, mem::size_of_val, ptr::addr_of};

use raw_window_handle::{
    HasRawDisplayHandle, RawDisplayHandle, RawWindowHandle, Win32WindowHandle,
    WindowsDisplayHandle,
};
use windows::{
    core::PCSTR,
    s,
    Win32::{
        Foundation::{
            BOOL, COLORREF, HMODULE, HWND, LPARAM, LRESULT, RECT, WPARAM,
        },
        Graphics::Gdi::{CreateSolidBrush, HDC, HMONITOR},
        System::{
            Diagnostics::Debug::{SetErrorMode, SEM_FAILCRITICALERRORS},
            Environment::GetCommandLineA,
            LibraryLoader::GetModuleHandleA,
            Threading::{GetStartupInfoW, STARTUPINFOW},
        },
        UI::{
            Input::KeyboardAndMouse::{
                MapVirtualKeyW, MAPVK_VSC_TO_VK_EX, VIRTUAL_KEY, VK_ADD,
                VK_BACK, VK_CAPITAL, VK_CONTROL, VK_DECIMAL, VK_DELETE,
                VK_DIVIDE, VK_DOWN, VK_END, VK_ESCAPE, VK_F1, VK_F10, VK_F11,
                VK_F12, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9,
                VK_HOME, VK_INSERT, VK_LBUTTON, VK_LCONTROL, VK_LEFT, VK_LMENU,
                VK_LSHIFT, VK_LWIN, VK_MBUTTON, VK_MENU, VK_MULTIPLY, VK_NEXT,
                VK_NUMLOCK, VK_NUMPAD0, VK_NUMPAD1, VK_NUMPAD2, VK_NUMPAD3,
                VK_NUMPAD4, VK_NUMPAD5, VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8,
                VK_NUMPAD9, VK_OEM_1, VK_OEM_2, VK_OEM_3, VK_OEM_4, VK_OEM_5,
                VK_OEM_6, VK_OEM_7, VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD,
                VK_OEM_PLUS, VK_PAUSE, VK_PRIOR, VK_RBUTTON, VK_RCONTROL,
                VK_RETURN, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_RWIN,
                VK_SEPARATOR, VK_SHIFT, VK_SNAPSHOT, VK_SPACE, VK_SUBTRACT,
                VK_TAB, VK_UP, VK_XBUTTON1, VK_XBUTTON2,
            },
            WindowsAndMessaging::{
                DefWindowProcA, DestroyWindow, GetSystemMetrics, LoadCursorA,
                LoadIconA, MessageBoxA, PostQuitMessage, RegisterClassExA,
                IDC_ARROW, MB_OK, MSG, SM_REMOTESESSION, WA_INACTIVE,
                WM_ACTIVATE, WM_CHAR, WM_CLOSE, WM_CREATE, WM_DESTROY,
                WM_DISPLAYCHANGE, WM_KEYDOWN, WM_KEYUP, WM_KILLFOCUS,
                WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
                WM_MOVE, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SETFOCUS,
                WM_SYSKEYDOWN, WM_SYSKEYUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
                WNDCLASSEXA, ShowWindow, SW_SHOW,
            },
        },
    },
};

use libc::c_int;

use crate::{
    com::{self, ErrorParm},
    platform::WindowHandle,
    sys::{self, KeyboardScancode, Modifiers, MouseScancode, WindowEvent},
    util::{CharFromUtf16Char, HighWord, LowWord},
};

// Get info for WinMain (Rust doesn't do this automatically), then call it
#[allow(
    clippy::panic,
    clippy::semicolon_outside_block,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
pub fn main() {
    // Get hInstance
    // SAFETY:
    // GetModuleHandleA is an FFI function, requiring use of unsafe.
    // GetModuleHandleA itself should never create UB, violate memory
    // safety, etc.
    let hInstance = unsafe {
        match GetModuleHandleA(None) {
            Ok(h) => Some(h),
            Err(n) => panic!("failed to get module handle, exiting ({})!", n),
        }
    };

    let mut info = STARTUPINFOW {
        cb: core::mem::size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };

    // Get command line
    // SAFETY:
    // GetCommandLineA is an FFI function, requiring use of unsafe.
    // GetCommandLineA itself should never create UB, violate memory
    // safety, etc.
    let p = unsafe { GetCommandLineA() };
    let pCmdLine = if p.is_null() {
        panic!("failed to get command line, exiting!")
    } else {
        Some(p)
    };

    // Get nCmdShow
    // SAFETY:
    // GetStartupInfoW is an FFI function, requiring use of unsafe.
    // GetStartupInfoW itself should never create UB, violate memory
    // safety, etc., provided a valid &STARUPINFOW is passed.
    unsafe {
        GetStartupInfoW(&mut info);
    }
    let nCmdShow = u32::from(info.wShowWindow);

    // Call actual WinMain
    // hPrevInstance always NULL for Win32 platforms
    WinMain(hInstance, None, pCmdLine, nCmdShow);
}

unsafe extern "system" fn main_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let mut mesg = MSG::default();
    mesg.hwnd = hwnd;
    mesg.message = msg;
    mesg.wParam = wparam;
    mesg.lParam = lparam;

    if msg == WM_DESTROY {
        sys::MAIN_WINDOW_EVENTS
            .lock()
            .unwrap()
            .push_back(WindowEvent::Destroyed);
        PostQuitMessage(0);
        LRESULT(0)
    } else if msg == WM_CLOSE {
        sys::MAIN_WINDOW_EVENTS
            .lock()
            .unwrap()
            .push_back(WindowEvent::CloseRequested);
        DestroyWindow(hwnd);
        LRESULT(0)
    } else if let Ok(ev) = mesg.try_into() {
        sys::MAIN_WINDOW_EVENTS.lock().unwrap().push_back(ev);
        LRESULT(0)
    } else {
        DefWindowProcA(hwnd, msg, wparam, lparam)
    }
}

fn register_class(hinstance: HMODULE) {
    let mut wnd_class = WNDCLASSEXA::default();
    wnd_class.cbSize = size_of_val(&wnd_class) as _;
    wnd_class.lpfnWndProc = Some(main_wnd_proc);
    wnd_class.hInstance = hinstance;
    wnd_class.hIcon = unsafe { LoadIconA(hinstance, PCSTR(0x00000001 as _)) }
        .unwrap_or_default();
    wnd_class.hCursor =
        unsafe { LoadCursorA(hinstance, PCSTR(IDC_ARROW.0 as _)) }
            .unwrap_or_default();
    wnd_class.hbrBackground = unsafe { CreateSolidBrush(COLORREF(0)) };
    wnd_class.lpszClassName = s!("CoDBlackOps");
    if unsafe { RegisterClassExA(addr_of!(wnd_class)) } == 0 {
        com::error(ErrorParm::FATAL, "EXE_ERR_COULDNT_REGISTER_WINDOW");
    }
}

impl TryFrom<MSG> for WindowEvent {
    type Error = ();
    fn try_from(value: MSG) -> Result<Self, Self::Error> {
        match value.message {
            WM_CREATE => {
                let mut handle = Win32WindowHandle::empty();
                handle.hwnd = value.hwnd.0 as _;
                handle.hinstance = unsafe { GetModuleHandleA(None) }
                    .unwrap_or(HMODULE(0))
                    .0 as _;
                Ok(Self::Created(WindowHandle(RawWindowHandle::Win32(handle))))
            }
            WM_DESTROY => Ok(Self::Destroyed),
            WM_CLOSE => Ok(Self::CloseRequested),
            WM_MOVE => Ok(Self::Moved {
                x: value.lParam.low_word() as _,
                y: value.lParam.high_word() as _,
            }),
            WM_ACTIVATE => {
                if value.wParam.0 == WA_INACTIVE as usize {
                    Ok(Self::Deactivate)
                } else {
                    Ok(Self::Activate)
                }
            }
            WM_SETFOCUS => Ok(Self::SetFocus),
            WM_KILLFOCUS => Ok(Self::KillFocus),
            WM_DISPLAYCHANGE => Ok(Self::DisplayChange {
                bit_depth: value.wParam.0 as _,
                horz_res: value.lParam.low_word() as _,
                vert_res: value.lParam.high_word() as _,
            }),
            WM_LBUTTONDOWN => Ok(Self::MouseButtonDown(MouseScancode::LClick)),
            WM_LBUTTONUP => Ok(Self::MouseButtonUp(MouseScancode::LClick)),
            WM_RBUTTONDOWN => Ok(Self::MouseButtonDown(MouseScancode::RClick)),
            WM_RBUTTONUP => Ok(Self::MouseButtonUp(MouseScancode::RClick)),
            WM_MBUTTONDOWN => Ok(Self::MouseButtonDown(MouseScancode::MClick)),
            WM_MBUTTONUP => Ok(Self::MouseButtonUp(MouseScancode::MClick)),
            WM_XBUTTONDOWN => {
                if value.wParam.high_word() == 0x01 {
                    Ok(Self::MouseButtonDown(MouseScancode::Button4))
                } else {
                    Ok(Self::MouseButtonDown(MouseScancode::Button5))
                }
            }
            WM_XBUTTONUP => {
                if value.wParam.high_word() == 0x01 {
                    Ok(Self::MouseButtonUp(MouseScancode::Button4))
                } else {
                    Ok(Self::MouseButtonUp(MouseScancode::Button5))
                }
            }
            WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP => {
                let down = value.message == WM_KEYDOWN
                    || value.message == WM_SYSKEYDOWN;
                let kpi = KeyPressInfo::from_lparam(value.lParam);
                let vk = VIRTUAL_KEY(value.wParam.0 as _);
                let physical_scancode: Option<KeyboardScancode> =
                    OemScancode(kpi.scancode).try_into().ok();

                if let Ok(k) = TryInto::<KeyboardScancode>::try_into(vk) {
                    if !down {
                        return Ok(Self::KeyUp {
                            logical_scancode: k,
                            physical_scancode,
                        });
                    }

                    return Ok(Self::KeyDown {
                        logical_scancode: k,
                        physical_scancode,
                    });
                }

                if let Ok(k) = TryInto::<MouseScancode>::try_into(vk) {
                    return Ok(if down {
                        Self::MouseButtonDown(k)
                    } else {
                        Self::MouseButtonUp(k)
                    });
                }

                if let Some(k) = Modifiers::try_from_vk(vk, kpi.scancode) {
                    return Ok(Self::ModifiersChanged { modifier: k, down });
                }

                Err(())
            }
            WM_CHAR => {
                let c = (value.wParam.low_word()).try_as_char();

                if let Some(c) = c {
                    Ok(Self::Character(c))
                } else {
                    Err(())
                }
            }
            _ => Err(()),
        }
    }
}

enum KeyState {
    Up,
    Down,
}

impl KeyState {
    fn from_bool(b: bool) -> Self {
        if b {
            Self::Down
        } else {
            Self::Up
        }
    }
}

struct KeyPressInfo {
    repeat_count: u16,
    scancode: u16,
    context_code: bool,
    previous_state: KeyState,
}

impl KeyPressInfo {
    fn from_lparam(lparam: LPARAM) -> Self {
        Self::from_isize(lparam.0)
    }

    fn from_isize(i: isize) -> Self {
        let repeat_count = (i & 0x0000FFFF) as u16;
        let scancode = ((i & 0x00FF0000) >> 16) as u16;
        let extended = i & 0x01000000 != 0;
        let scancode = if extended {
            scancode | 0xE000
        } else {
            scancode
        };
        let context_code = i & 0x10000000 != 0;
        let previous_state = KeyState::from_bool(i & 0x40000000 != 0);

        Self {
            repeat_count,
            scancode,
            context_code,
            previous_state,
        }
    }
}

#[derive(Copy, Clone, Debug)]
struct OemScancode(u16);

impl TryFrom<OemScancode> for KeyboardScancode {
    type Error = ();
    fn try_from(value: OemScancode) -> Result<Self, Self::Error> {
        match value.0 {
            0x001E => Ok(Self::A),
            0x0030 => Ok(Self::B),
            0x002E => Ok(Self::C),
            0x0020 => Ok(Self::D),
            0x0012 => Ok(Self::E),
            0x0021 => Ok(Self::F),
            0x0022 => Ok(Self::G),
            0x0023 => Ok(Self::H),
            0x0017 => Ok(Self::I),
            0x0024 => Ok(Self::J),
            0x0025 => Ok(Self::K),
            0x0026 => Ok(Self::L),
            0x0032 => Ok(Self::M),
            0x0031 => Ok(Self::N),
            0x0018 => Ok(Self::O),
            0x0019 => Ok(Self::P),
            0x0010 => Ok(Self::Q),
            0x0013 => Ok(Self::R),
            0x001F => Ok(Self::S),
            0x0014 => Ok(Self::T),
            0x0016 => Ok(Self::U),
            0x002F => Ok(Self::V),
            0x0011 => Ok(Self::W),
            0x002D => Ok(Self::X),
            0x0015 => Ok(Self::Y),
            0x002C => Ok(Self::Z),

            0x0002 => Ok(Self::Key1),
            0x0003 => Ok(Self::Key2),
            0x0004 => Ok(Self::Key3),
            0x0005 => Ok(Self::Key4),
            0x0006 => Ok(Self::Key5),
            0x0007 => Ok(Self::Key6),
            0x0008 => Ok(Self::Key7),
            0x0009 => Ok(Self::Key8),
            0x000A => Ok(Self::Key9),
            0x000B => Ok(Self::Key0),

            0x001C => Ok(Self::Enter),
            0x0001 => Ok(Self::Esc),
            0x000E => Ok(Self::Backspace),
            0x000F => Ok(Self::Tab),

            0x0039 => Ok(Self::Space),
            0x000C => Ok(Self::Hyphen),
            0x000D => Ok(Self::Equals),
            0x001A => Ok(Self::OpenBracket),
            0x001B => Ok(Self::CloseBracket),
            0x002B => Ok(Self::BackSlash),
            0x0027 => Ok(Self::Semicolon),
            0x0028 => Ok(Self::Apostrophe),
            0x0029 => Ok(Self::Tilde),
            0x0033 => Ok(Self::Comma),
            0x0034 => Ok(Self::Period),
            0x0035 => Ok(Self::ForwardSlash),
            0x003A => Ok(Self::CapsLk),

            0x003B => Ok(Self::F1),
            0x003C => Ok(Self::F2),
            0x003D => Ok(Self::F3),
            0x003E => Ok(Self::F4),
            0x003F => Ok(Self::F5),
            0x0040 => Ok(Self::F6),
            0x0041 => Ok(Self::F7),
            0x0042 => Ok(Self::F8),
            0x0043 => Ok(Self::F9),
            0x0044 => Ok(Self::F10),
            0x0057 => Ok(Self::F11),
            0x0058 => Ok(Self::F12),

            0x0046 => Ok(Self::ScrLk),
            0xE052 => Ok(Self::Insert),
            0xE047 => Ok(Self::Home),
            0xE049 => Ok(Self::PgUp),
            0xE053 => Ok(Self::Del),
            0xE04F => Ok(Self::End),
            0xE051 => Ok(Self::PgDn),
            0xE04D => Ok(Self::ArrowRight),
            0xE04B => Ok(Self::ArrowLeft),
            0xE050 => Ok(Self::ArrowDown),
            0xE048 => Ok(Self::ArrowUp),

            0xE035 => Ok(Self::NumSlash),
            0x0037 => Ok(Self::NumAsterisk),
            0x004A => Ok(Self::NumHyphen),
            0x004E => Ok(Self::NumPlus),
            0xE01C => Ok(Self::NumEnter),
            0x0053 => Ok(Self::NumPeriod),

            0x004F => Ok(Self::Num1),
            0x0050 => Ok(Self::Num2),
            0x0051 => Ok(Self::Num3),
            0x004B => Ok(Self::Num4),
            0x004C => Ok(Self::Num5),
            0x004D => Ok(Self::Num6),
            0x0047 => Ok(Self::Num7),
            0x0048 => Ok(Self::Num8),
            0x0049 => Ok(Self::Num9),
            0x0052 => Ok(Self::Num0),

            0x001D => Ok(Self::LCtrl),
            0x002A => Ok(Self::LShift),
            0x0038 => Ok(Self::LAlt),
            0xE05B => Ok(Self::LSys),
            0xE01D => Ok(Self::RCtrl),
            0x0036 => Ok(Self::RShift),
            0xE038 => Ok(Self::RAlt),
            0xE05C => Ok(Self::RSys),

            _ => Err(()),
        }
    }
}

impl TryFrom<VIRTUAL_KEY> for MouseScancode {
    type Error = ();
    fn try_from(value: VIRTUAL_KEY) -> Result<Self, Self::Error> {
        match value {
            VK_LBUTTON => Ok(Self::LClick),
            VK_RBUTTON => Ok(Self::RClick),
            VK_MBUTTON => Ok(Self::MClick),
            VK_XBUTTON1 => Ok(Self::Button4),
            VK_XBUTTON2 => Ok(Self::Button5),
            _ => Err(()),
        }
    }
}

trait ModifiersExt {
    fn try_from_vk(vk: VIRTUAL_KEY, scancode: u16) -> Option<Modifiers>;
}

impl ModifiersExt for Modifiers {
    fn try_from_vk(vk: VIRTUAL_KEY, scancode: u16) -> Option<Self> {
        let vk = if vk == VK_SHIFT || vk == VK_MENU || vk == VK_CONTROL {
            VIRTUAL_KEY(unsafe {
                MapVirtualKeyW(scancode as _, MAPVK_VSC_TO_VK_EX)
            } as _)
        } else {
            vk
        };

        match vk {
            VK_LSHIFT => Some(Modifiers::LSHIFT),
            VK_RSHIFT => Some(Modifiers::RSHIFT),
            VK_LMENU => Some(Modifiers::LALT),
            VK_RMENU => Some(Modifiers::RALT),
            VK_LCONTROL => Some(Modifiers::LCTRL),
            VK_RCONTROL => Some(Modifiers::RCTRL),
            VK_LWIN => Some(Modifiers::LSYS),
            VK_RWIN => Some(Modifiers::RSYS),
            VK_CAPITAL => Some(Modifiers::CAPSLOCK),
            VK_NUMLOCK => Some(Modifiers::NUMLOCK),
            _ => None,
        }
    }
}

impl TryFrom<VIRTUAL_KEY> for KeyboardScancode {
    type Error = ();
    fn try_from(value: VIRTUAL_KEY) -> Result<Self, Self::Error> {
        match value {
            VK_BACK => Ok(Self::Backspace),
            VK_TAB => Ok(Self::Tab),
            VK_RETURN => Ok(Self::Enter),
            VK_PAUSE => Ok(Self::PauseBreak),
            VK_ESCAPE => Ok(Self::Esc),
            VK_SPACE => Ok(Self::Space),
            VK_PRIOR => Ok(Self::PgUp),
            VK_NEXT => Ok(Self::PgDn),
            VK_END => Ok(Self::End),
            VK_HOME => Ok(Self::Home),
            VK_LEFT => Ok(Self::ArrowLeft),
            VK_UP => Ok(Self::ArrowUp),
            VK_DOWN => Ok(Self::ArrowDown),
            VK_RIGHT => Ok(Self::ArrowRight),
            VK_SNAPSHOT => Ok(Self::PrtScSysRq),
            VK_INSERT => Ok(Self::Insert),
            VK_DELETE => Ok(Self::Del),
            VIRTUAL_KEY(0x30) => Ok(Self::Key0),
            VIRTUAL_KEY(0x31) => Ok(Self::Key1),
            VIRTUAL_KEY(0x32) => Ok(Self::Key2),
            VIRTUAL_KEY(0x33) => Ok(Self::Key3),
            VIRTUAL_KEY(0x34) => Ok(Self::Key4),
            VIRTUAL_KEY(0x35) => Ok(Self::Key5),
            VIRTUAL_KEY(0x36) => Ok(Self::Key6),
            VIRTUAL_KEY(0x37) => Ok(Self::Key7),
            VIRTUAL_KEY(0x38) => Ok(Self::Key8),
            VIRTUAL_KEY(0x39) => Ok(Self::Key9),
            VIRTUAL_KEY(0x41) => Ok(Self::A),
            VIRTUAL_KEY(0x42) => Ok(Self::B),
            VIRTUAL_KEY(0x43) => Ok(Self::C),
            VIRTUAL_KEY(0x44) => Ok(Self::D),
            VIRTUAL_KEY(0x45) => Ok(Self::E),
            VIRTUAL_KEY(0x46) => Ok(Self::F),
            VIRTUAL_KEY(0x47) => Ok(Self::G),
            VIRTUAL_KEY(0x48) => Ok(Self::H),
            VIRTUAL_KEY(0x49) => Ok(Self::I),
            VIRTUAL_KEY(0x4A) => Ok(Self::J),
            VIRTUAL_KEY(0x4B) => Ok(Self::K),
            VIRTUAL_KEY(0x4C) => Ok(Self::L),
            VIRTUAL_KEY(0x4D) => Ok(Self::M),
            VIRTUAL_KEY(0x4E) => Ok(Self::N),
            VIRTUAL_KEY(0x4F) => Ok(Self::O),
            VIRTUAL_KEY(0x50) => Ok(Self::P),
            VIRTUAL_KEY(0x51) => Ok(Self::Q),
            VIRTUAL_KEY(0x52) => Ok(Self::R),
            VIRTUAL_KEY(0x53) => Ok(Self::S),
            VIRTUAL_KEY(0x54) => Ok(Self::T),
            VIRTUAL_KEY(0x55) => Ok(Self::U),
            VIRTUAL_KEY(0x56) => Ok(Self::V),
            VIRTUAL_KEY(0x57) => Ok(Self::W),
            VIRTUAL_KEY(0x58) => Ok(Self::X),
            VIRTUAL_KEY(0x59) => Ok(Self::Y),
            VIRTUAL_KEY(0x5A) => Ok(Self::Z),
            VK_NUMPAD0 => Ok(Self::Num0),
            VK_NUMPAD1 => Ok(Self::Num1),
            VK_NUMPAD2 => Ok(Self::Num2),
            VK_NUMPAD3 => Ok(Self::Num3),
            VK_NUMPAD4 => Ok(Self::Num4),
            VK_NUMPAD5 => Ok(Self::Num5),
            VK_NUMPAD6 => Ok(Self::Num6),
            VK_NUMPAD7 => Ok(Self::Num7),
            VK_NUMPAD8 => Ok(Self::Num8),
            VK_NUMPAD9 => Ok(Self::Num9),
            VK_MULTIPLY => Ok(Self::NumAsterisk),
            VK_ADD => Ok(Self::NumPlus),
            VK_SEPARATOR => Ok(Self::NumPeriod),
            VK_SUBTRACT => Ok(Self::NumHyphen),
            VK_DECIMAL => Ok(Self::NumPeriod),
            VK_DIVIDE => Ok(Self::NumSlash),
            VK_F1 => Ok(Self::F1),
            VK_F2 => Ok(Self::F2),
            VK_F3 => Ok(Self::F3),
            VK_F4 => Ok(Self::F4),
            VK_F5 => Ok(Self::F5),
            VK_F6 => Ok(Self::F6),
            VK_F7 => Ok(Self::F7),
            VK_F8 => Ok(Self::F8),
            VK_F9 => Ok(Self::F9),
            VK_F10 => Ok(Self::F10),
            VK_F11 => Ok(Self::F11),
            VK_F12 => Ok(Self::F12),

            VK_OEM_1 => Ok(Self::Semicolon),
            VK_OEM_PLUS => Ok(Self::Equals),
            VK_OEM_COMMA => Ok(Self::Comma),
            VK_OEM_MINUS => Ok(Self::Hyphen),
            VK_OEM_PERIOD => Ok(Self::Period),
            VK_OEM_2 => Ok(Self::ForwardSlash),
            VK_OEM_3 => Ok(Self::Tilde),
            VK_OEM_4 => Ok(Self::OpenBracket),
            VK_OEM_5 => Ok(Self::BackSlash),
            VK_OEM_6 => Ok(Self::CloseBracket),
            VK_OEM_7 => Ok(Self::Apostrophe),

            _ => Err(()),
        }
    }
}

pub unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _place: *mut RECT,
    data: LPARAM,
) -> BOOL {
    let monitors = data.0 as *mut VecDeque<HMONITOR>;
    (*monitors).push_back(hmonitor);
    true.into() // continue enumeration
}

impl WindowHandle {
    pub const fn new(handle: RawWindowHandle) -> Self {
        Self(handle)
    }

    pub const fn get(&self) -> RawWindowHandle {
        self.0
    }

    pub const fn get_win32(&self) -> Option<Win32WindowHandle> {
        match self.get() {
            RawWindowHandle::Win32(handle) => Some(handle),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum MonitorHandle {
    Win32(isize),
}

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

// Win32 => Win32
// Linux => Xlib, Wayland
// macOS => Xlib, AppKit, UiKit
// Other Unix => Xlib
impl MonitorHandle {
    pub const fn get_win32(&self) -> Option<HMONITOR> {
        match *self {
            Self::Win32(handle) => Some(HMONITOR(handle)),
        }
    }
}

pub fn show_window(handle: WindowHandle) {
    unsafe { ShowWindow(HWND(handle.get_win32().unwrap().hwnd as _), SW_SHOW) };
}

#[allow(unused_variables, clippy::semicolon_outside_block)]
fn WinMain(
    hInstance: Option<HMODULE>,
    hPrevInstance: Option<HMODULE>,
    pCmdLine: Option<PCSTR>,
    nCmdShow: u32,
) -> c_int {
    // SAFETY:
    // GetSystemMetrics is an FFI function, requiring use of unsafe.
    // GetSystemMetrics itself should never create UB, violate memory
    // safety, etc.
    if unsafe { GetSystemMetrics(SM_REMOTESESSION) != 0 } {
        // SAFETY:
        // MessageBoxA is an FFI function, requiring use of unsafe.
        // MessageBoxA itself should never create UB, violate memory
        // safety, etc., regardless of the parameters passed to it.
        unsafe {
            MessageBoxA(
                None,
                s!("The game can not be run over a remote desktop connection."),
                None,
                MB_OK,
            );
        }
        return 0;
    }

    if hPrevInstance.is_some() {
        return 0;
    }

    register_class(hInstance.unwrap());

    // SAFETY:
    // SetErrorMode is an FFI function, requiring use of unsafe.
    // SetErrorMode itself should never create UB, violate memory
    // safety, etc.
    unsafe {
        SetErrorMode(SEM_FAILCRITICALERRORS);
    }

    0
}
