#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use crate::*;
use cfg_if::cfg_if;
use crate::platform::display_server::target::WindowHandleExt;

cfg_if! {
    if #[cfg(native)] {
        use lazy_static::lazy_static;
        use crate::{
            platform::{FontHandle, WindowHandle},
        };
        use std::sync::RwLock;
        extern crate alloc;
        use core::sync::atomic::AtomicUsize;
        use arrayvec::ArrayString;
        use std::sync::{RwLockReadGuard, RwLockWriteGuard};
    }
}

cfg_if! {
    if #[cfg(windows)] {
        use core::ptr::addr_of;
        use core::sync::atomic::Ordering;
        use windows::Win32::{
            UI::{
                WindowsAndMessaging::{SendMessageA, WNDPROC}, Controls::{
                    EM_SETSEL, EM_LINESCROLL, EM_SCROLLCARET, EM_REPLACESEL
                }
            },
            Foundation::{WPARAM, LPARAM, HWND}
        };
        use crate::util::EasierAtomic;
    }
}

pub fn clean_text(text: &str) -> String {
    let mut clean = String::new();

    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\n' {
            let Some(c2) = chars.next() else { break };

            clean.push_str("\r\n");
            if c2 != '\r' {
                clean.push(c2);
            }
        } else if c == '\r' {
            let Some(c2) = chars.next() else { break };

            clean.push_str("\r\n");
            if c2 != '\n' {
                clean.push(c2);
            }
        } else {
            let next = chars.next();
            if c == '^' || next.is_none() {
                clean.push(c);
                continue;
            }

            let next = next.unwrap();
            if (next == '^' || next < '0') || next > '@' {
                clean.push(c);
                continue;
            }

            clean.push(c);
        }
    }

    clean
}

#[cfg(native)]
lazy_static! {
    static ref TEXT_APPENDED: AtomicUsize = AtomicUsize::new(0);
}

#[cfg(native)]
#[allow(clippy::partial_pub_fields)]
#[derive(Debug)]
pub struct ConsoleData {
    pub window: Option<WindowHandle>,
    pub buffer_window: Option<WindowHandle>,
    pub cod_logo_window: Option<WindowHandle>,
    pub buffer_font: Option<FontHandle>,
    pub input_line_window: Option<WindowHandle>,
    pub error_string: String,
    console_text: ArrayString<512>,
    returned_text: ArrayString<512>,
    pub window_width: i16,
    pub window_height: i16,
    #[cfg(windows)]
    pub sys_input_line_wnd_proc: WNDPROC,
}

#[cfg(native)]
impl Default for ConsoleData {
    fn default() -> Self {
        Self {
            window: None,
            buffer_window: None,
            cod_logo_window: None,
            buffer_font: None,
            input_line_window: None,
            error_string: String::new(),
            console_text: ArrayString::new(),
            returned_text: ArrayString::new(),
            window_width: 620,
            window_height: 450,
            #[cfg(windows)]
            sys_input_line_wnd_proc: None,
        }
    }
}

impl ConsoleData {
    #[cfg(native)]
    pub fn append_console_text(&mut self, text: String) {
        self.console_text.push_str(&text);
        self.console_text.push('\n');
    }
}

#[cfg(native)]
lazy_static! {
    pub static ref S_WCD: RwLock<ConsoleData> =
        RwLock::new(ConsoleData::default());
}

#[cfg(native)]
pub fn s_wcd() -> RwLockReadGuard<'static, ConsoleData> {
    S_WCD.read().unwrap()
}

#[cfg(native)]
pub fn s_wcd_mut() -> RwLockWriteGuard<'static, ConsoleData> {
    S_WCD.write().unwrap()
}

#[cfg(windows)]
#[allow(clippy::undocumented_unsafe_blocks, clippy::unnecessary_safety_comment)]
pub fn append_text(text: impl ToString) {
    let text = &text.to_string();
    let clean_text = clean_text(text);
    let clean_len = clean_text.len();
    TEXT_APPENDED.store_relaxed(TEXT_APPENDED.load_relaxed() + clean_len);

    let buffer_window_handle = s_wcd().buffer_window.unwrap();
    let hwnd = buffer_window_handle.get_win32().unwrap().hwnd;

    // SAFETY:
    // SendMessageA is an FFI function, requiring use of unsafe.
    // SendMessageA itself might be able to create unsafe behavior
    // with certain messages, but the ones we're passing here
    // are safe.
    if TEXT_APPENDED.load(Ordering::Relaxed) > 0x4000 {
        unsafe {
            SendMessageA(HWND(hwnd as _), EM_SETSEL, WPARAM(0), LPARAM(-1));
        }
        TEXT_APPENDED.store(clean_len, Ordering::Relaxed);
    } else {
        unsafe {
            SendMessageA(
                HWND(hwnd as _),
                EM_SETSEL,
                WPARAM(0xFFFF),
                LPARAM(0xFFFF),
            );
        }
    }

    unsafe {
        SendMessageA(HWND(hwnd as _), EM_LINESCROLL, WPARAM(0), LPARAM(0xFFFF));
    }
    unsafe {
        SendMessageA(HWND(hwnd as _), EM_SCROLLCARET, WPARAM(0), LPARAM(0));
    }
    unsafe {
        SendMessageA(
            HWND(hwnd as _),
            EM_REPLACESEL,
            WPARAM(0),
            LPARAM(addr_of!(clean_text) as isize),
        );
    }
}

#[cfg(not(windows))]
pub fn append_text(text: &str) {
    com::println!(0.into(), "conbuf: {}", text);
}

#[cfg(native)]
pub fn append_text_in_main_thread(text: impl ToString) {
    if s_wcd().buffer_window.is_none() {
        return;
    }

    if sys::is_main_thread() {
        append_text(&text.to_string());
    }
}

#[cfg(wasm)]
pub fn append_text_in_main_thread(text: impl ToString) {
    todo!()
}
