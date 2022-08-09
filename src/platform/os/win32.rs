#![allow(non_snake_case)]
use std::sync::RwLock;

use lazy_static::lazy_static;

use windows::{
    core::*,
    Win32::{
        Foundation::{HINSTANCE, HWND},
        System::{
            Diagnostics::Debug::{SetErrorMode, SEM_FAILCRITICALERRORS},
            Environment::GetCommandLineA,
            LibraryLoader::GetModuleHandleA,
            Threading::{GetStartupInfoW, STARTUPINFOW},
        },
        UI::WindowsAndMessaging::{GetSystemMetrics, MessageBoxA, MB_OK, SM_REMOTESESSION},
    },
};

use libc::c_int;

#[allow(dead_code)]
#[derive(Default)]
struct WinVars {
    reflib_library: Option<HINSTANCE>, // seems to be unused
    reflib_active: i32,                // also seems to be unused
    hWnd: Option<HWND>,
    hInstance: Option<HINSTANCE>,
    activeApp: i32,
    isMinimized: i32,
    recenterMouse: i32,
    sysMsgTime: u32,
}

lazy_static! {
    static ref G_WV: RwLock<WinVars> = RwLock::new(WinVars {
        reflib_library: None,
        reflib_active: 0,
        hWnd: None,
        hInstance: None,
        activeApp: 0,
        isMinimized: 0,
        recenterMouse: 0,
        sysMsgTime: 0
    });
}

// Get info for WinMain (Rust doesn't do this automatically), then call it
pub fn main() {
    // Get hInstance
    let hInstance: Option<HINSTANCE> = unsafe {
        match GetModuleHandleA(None) {
            Ok(h) => Some(h),
            Err(n) => panic!("failed to get module handle, exiting ({})!", n),
        }
    };

    let mut info = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };

    // Get command line
    let p = unsafe { GetCommandLineA() };
    let pCmdLine = match p.is_null() {
        true => panic!("failed to get command line, exiting!"),
        false => Some(p),
    };

    // Get nCmdShow
    unsafe { GetStartupInfoW(&mut info) };
    let nCmdShow = info.wShowWindow as c_int as u32;

    // Call actual WinMain
    // hPrevInstance always NULL for Win32 platforms
    WinMain(hInstance, None, pCmdLine, nCmdShow);
}

#[allow(unused_variables)]
fn WinMain(
    hInstance: Option<HINSTANCE>,
    hPrevInstance: Option<HINSTANCE>,
    pCmdLine: Option<PSTR>,
    nCmdShow: u32,
) -> c_int {
    if unsafe { GetSystemMetrics(SM_REMOTESESSION) != 0 } {
        unsafe {
            MessageBoxA(
                None,
                s!("The game can not be run over a remote desktop connection."),
                None,
                MB_OK,
            )
        };
        return 0;
    }

    if hPrevInstance.is_some() {
        return 0;
    }

    G_WV.write().unwrap().hInstance = hInstance;
    unsafe { SetErrorMode(SEM_FAILCRITICALERRORS) };

    println!("Exiting WinMain()!");
    0
}
