// This file is for any Windows-specific initialization that
// should be done before the rest of main() executes

#![allow(non_snake_case)]
use windows::{
    core::*,
    Win32::{
        Foundation::HINSTANCE,
        System::{
            Diagnostics::Debug::{SetErrorMode, SEM_FAILCRITICALERRORS},
            Environment::GetCommandLineA,
            LibraryLoader::GetModuleHandleA,
            Threading::{GetStartupInfoW, STARTUPINFOW},
        },
        UI::WindowsAndMessaging::{
            GetSystemMetrics, MessageBoxA, MB_OK, SM_REMOTESESSION,
        },
    },
};

use libc::c_int;

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
    let hInstance: Option<HINSTANCE> = unsafe {
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

#[allow(unused_variables, clippy::semicolon_outside_block)]
fn WinMain(
    hInstance: Option<HINSTANCE>,
    hPrevInstance: Option<HINSTANCE>,
    pCmdLine: Option<PSTR>,
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

    // SAFETY:
    // SetErrorMode is an FFI function, requiring use of unsafe.
    // SetErrorMode itself should never create UB, violate memory
    // safety, etc.
    unsafe {
        SetErrorMode(SEM_FAILCRITICALERRORS);
    }

    0
}
