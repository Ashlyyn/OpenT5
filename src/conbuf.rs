#![allow(dead_code)]

use std::{sync::{atomic::{AtomicUsize}, Arc, RwLock}};

use arrayvec::ArrayString;
use lazy_static::lazy_static;
use cfg_if::cfg_if;
use crate::{platform::{FontHandle}};

cfg_if! {
    if #[cfg(target_os = "windows")] {
        use std::ptr::addr_of;
        use std::sync::atomic::Ordering;
        use windows::Win32::{UI::{WindowsAndMessaging::SendMessageA, Controls::{EM_SETSEL, EM_LINESCROLL, EM_SCROLLCARET, EM_REPLACESEL}}, Foundation::{WPARAM, LPARAM, HWND}};
        use crate::util::EasierWindowHandle;
    }
}

pub fn clean_text(text: &str) -> String {
    let mut clean = String::new();
    
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\n' {
            let c2 = match chars.next() {
                Some(c) => c,
                None => break,
            };

            clean.push_str("\r\n");
            if c2 != '\r' {
                clean.push(c2);
            }
        } else if c == '\r' {
            let c2 = match chars.next() {
                Some(c) => c,
                None => break,
            };

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

lazy_static! {
    static ref TEXT_APPENDED: AtomicUsize = AtomicUsize::new(0);
}

#[derive(Default, Debug)]
pub struct ConsoleData {
    pub window: Option<winit::window::Window>,
    pub buffer_window: Option<winit::window::Window>,
    pub cod_logo_window: Option<winit::window::Window>,
    pub buffer_font: Option<FontHandle>,
    pub input_line_window: Option<winit::window::Window>,
    pub error_string: String,
    console_text: ArrayString<512>,
    returned_text: ArrayString<512>,
    pub window_width: i16,
    pub window_height: i16,
}

lazy_static! {
    pub static ref S_WCD: Arc<RwLock<ConsoleData>> = Arc::new(RwLock::new(Default::default()));
}

pub fn s_wcd_set_window(window: winit::window::Window) {
    let lock = S_WCD.clone();
    let mut s_wcd = lock.write().unwrap();
    s_wcd.window = Some(window);
}

pub fn s_wcd_set_buffer_window(window: winit::window::Window) {
    let lock = S_WCD.clone();
    let mut s_wcd = lock.write().unwrap();
    s_wcd.buffer_window = Some(window);
}

pub fn s_wcd_set_input_line_window(window: winit::window::Window) {
    let lock = S_WCD.clone();
    let mut s_wcd = lock.write().unwrap();
    s_wcd.input_line_window = Some(window);
}

pub fn s_wcd_set_cod_logo_window(window: winit::window::Window) {
    let lock = S_WCD.clone();
    let mut s_wcd = lock.write().unwrap();
    s_wcd.cod_logo_window = Some(window);
}

cfg_if! {
    if #[cfg(target_os = "windows")] {
        pub fn append_text(text: &str) {
            let lock = S_WCD.clone();
            let mut s_wcd = lock.write().unwrap();
        
            let clean_text = clean_text(text);
            let clean_len = clean_text.len();
            TEXT_APPENDED.store(TEXT_APPENDED.load(Ordering::Relaxed) + clean_len, Ordering::Relaxed);

            let buffer_window_handle = s_wcd.buffer_window.as_mut().unwrap().window_handle();
            let hwnd = buffer_window_handle.get_win32().unwrap().hwnd;

            if TEXT_APPENDED.load(Ordering::Relaxed) > 0x4000 {
                unsafe { SendMessageA(HWND(hwnd as _), EM_SETSEL, WPARAM(0), LPARAM(-1)) };
                TEXT_APPENDED.store(clean_len, Ordering::Relaxed);
            } else {
                unsafe { SendMessageA(HWND(hwnd as _), EM_SETSEL, WPARAM(0xFFFF), LPARAM(0xFFFF)) };
            }

            unsafe { SendMessageA(HWND(hwnd as _), EM_LINESCROLL, WPARAM(0), LPARAM(0xFFFF)) };
            unsafe { SendMessageA(HWND(hwnd as _), EM_SCROLLCARET, WPARAM(0), LPARAM(0)) };
            unsafe { SendMessageA(HWND(hwnd as _), EM_REPLACESEL, WPARAM(0), LPARAM(addr_of!(clean_text) as isize)) };
        }
    } else {
        pub fn append_text(text: &str) {
            println!("{}", text);
        }
    }
}

pub fn append_text_in_main_thread(text: &str) {
    {
        let lock = S_WCD.clone();
        let s_wcd = lock.read().unwrap();
        if s_wcd.buffer_window.is_none() {
            return;
        }
    }

    //if sys::is_main_thread() {
        append_text(text);
    //}
}