#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]

use cfg_if::cfg_if;

use crate::util::EasierAtomic;
cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        use lazy_static::lazy_static;
        use raw_window_handle::{RawWindowHandle};
        use crate::{
            platform::{FontHandle, WindowHandle},
        };
        use std::sync::RwLock;
        extern crate alloc;
        use core::sync::atomic::AtomicUsize;
        use arrayvec::ArrayString;
    }
}

cfg_if! {
    if #[cfg(all(not(target_os = "windows"), not(target_arch = "wasm32")))] {
        use crate::*;
    }
}

cfg_if! {
    if #[cfg(target_os = "windows")] {
        use core::ptr::addr_of;
        use core::sync::atomic::Ordering;
        use windows::Win32::{
            UI::{
                WindowsAndMessaging::SendMessageA, Controls::{
                    EM_SETSEL, EM_LINESCROLL, EM_SCROLLCARET, EM_REPLACESEL
                }
            },
            Foundation::{WPARAM, LPARAM, HWND}};
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

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        lazy_static! {
            static ref TEXT_APPENDED: AtomicUsize = AtomicUsize::new(0);
        }

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
        }

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
                }
            }
        }

        lazy_static! {
            pub static ref S_WCD: RwLock<ConsoleData> =
                RwLock::new(ConsoleData::default());
        }

        pub fn s_wcd_set_window(window: WindowHandle) {
            let mut s_wcd = S_WCD.write().unwrap();
            s_wcd.window = Some(window);
        }

        pub fn s_wcd_set_buffer_window(window: WindowHandle) {
            let mut s_wcd = S_WCD.write().unwrap();
            s_wcd.buffer_window = Some(window);
        }

        pub fn s_wcd_set_input_line_window(window: WindowHandle) {
            let mut s_wcd = S_WCD.write().unwrap();
            s_wcd.input_line_window = Some(window);
        }

        pub fn s_wcd_clear_input_line_window() {
            let mut s_wcd = S_WCD.write().unwrap();
            s_wcd.input_line_window = None;
        }

        pub fn s_wcd_set_cod_logo_window(window: WindowHandle) {
            let mut s_wcd = S_WCD.write().unwrap();
            s_wcd.cod_logo_window = Some(window);
        }

        pub fn s_wcd_set_error_string(error: String) {
            let mut s_wcd = S_WCD.write().unwrap();
            s_wcd.error_string = error;
        }

        pub fn s_wcd_window_is_none() -> bool {
            let s_wcd = S_WCD.read().unwrap();
            s_wcd.window.is_none()
        }

        pub fn s_wcd_window_set_visible(visible: bool) {
            let mut s_wcd = S_WCD.write().unwrap();
            //s_wcd.window.as_mut().unwrap().set_visible(visible);
        }

        pub fn s_wcd_buffer_window_handle() -> WindowHandle {
            let s_wcd = S_WCD.read().unwrap();
            *s_wcd.buffer_window.as_ref().unwrap()
        }

        pub fn s_wcd_buffer_is_none() -> bool {
            let s_wcd = S_WCD.read().unwrap();
            s_wcd.buffer_window.is_none()
        }

        pub fn s_wcd_set_window_width(width: i16) {
            let mut s_wcd = S_WCD.write().unwrap();
            s_wcd.window_width = width;
        }

        pub fn s_wcd_window_width() -> i16 {
            let s_wcd = S_WCD.read().unwrap();
            s_wcd.window_width
        }

        pub fn s_wcd_window_height() -> i16 {
            let s_wcd = S_WCD.read().unwrap();
            s_wcd.window_height
        }

        pub fn s_wcd_window_handle() -> RawWindowHandle {
            let s_wcd = S_WCD.read().unwrap();
            s_wcd.window.as_ref().unwrap().get()
        }
    }
}

cfg_if! {
    if #[cfg(target_os = "windows")] {
        #[allow(
            clippy::undocumented_unsafe_blocks,
            clippy::unnecessary_safety_comment
        )]
        pub fn append_text(text: &str) {
            let clean_text = clean_text(text);
            let clean_len = clean_text.len();
            TEXT_APPENDED.store_relaxed(
                TEXT_APPENDED.load_relaxed() + clean_len
            );

            let buffer_window_handle = s_wcd_buffer_window_handle();
            let hwnd = buffer_window_handle.get_win32().unwrap().hwnd;

            // SAFETY:
            // SendMessageA is an FFI function, requiring use of unsafe.
            // SendMessageA itself might be able to create unsafe behavior
            // with certain messages, but the ones we're passing here
            // are safe.
            if TEXT_APPENDED.load(Ordering::Relaxed) > 0x4000 {
                unsafe { SendMessageA(
                    HWND(hwnd as _), EM_SETSEL, WPARAM(0), LPARAM(-1)
                ); }
                TEXT_APPENDED.store(clean_len, Ordering::Relaxed);
            } else {
                unsafe { SendMessageA(
                    HWND(hwnd as _), EM_SETSEL, WPARAM(0xFFFF), LPARAM(0xFFFF)
                ); }
            }

            unsafe { SendMessageA(
                HWND(hwnd as _), EM_LINESCROLL, WPARAM(0), LPARAM(0xFFFF)
            ); }
            unsafe { SendMessageA(
                HWND(hwnd as _), EM_SCROLLCARET, WPARAM(0), LPARAM(0)
            ); }
            unsafe { SendMessageA(
                HWND(hwnd as _), EM_REPLACESEL,
                WPARAM(0), LPARAM(addr_of!(clean_text) as isize)
            ); }
        }
    } else {
        pub fn append_text(text: &str) {
            com::println!(0.into(), "conbuf: {}", text);
        }
    }
}

cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        pub fn append_text_in_main_thread(text: &str) {
            if s_wcd_buffer_is_none() {
                return;
            }

            //if sys::is_main_thread() {
            append_text(text);
            //}
        }
    } else {
        pub fn append_text_in_main_thread(text: &str) {
            //if sys::is_main_thread() {
            append_text(text);
            //}
        }
    }
}
