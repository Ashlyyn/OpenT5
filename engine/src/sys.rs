#![allow(dead_code, non_upper_case_globals)]

extern crate alloc;

use crate::{
    cl::Connstate,
    platform::{display_server::target::WindowHandleExt, WindowHandle},
    util::{EasierAtomic, EasierAtomicBool, SignalState, SmpEvent},
    *,
};

#[allow(unused_imports)]
use num_derive::FromPrimitive;

use alloc::collections::VecDeque;
#[allow(unused_imports)]
use cfg_if::cfg_if;
use core::{
    fmt::Display,
    sync::atomic::{AtomicBool, AtomicIsize, Ordering::SeqCst},
};
#[allow(unused_imports)]
use lazy_static::lazy_static;
use std::path::Path;
#[allow(unused_imports)]
use std::{
    io::{Read, Write},
    path::PathBuf,
    sync::{Mutex, RwLock},
    thread::{JoinHandle, ThreadId},
};

#[cfg(not(windows))]
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(unused_imports)]
use alloc::ffi::CString;
#[allow(unused_imports)]
use core::ffi::CStr;
#[allow(unused_imports)]
use core::{
    mem::{size_of_val, transmute},
    ptr::{addr_of, addr_of_mut},
};

#[allow(unused_imports)]
use std::fs::OpenOptions;

cfg_if! {
    if #[cfg(windows)] {
        use windows::{
            core::PCSTR,
            s, w,
            Win32::{
                Media::timeGetTime,
                Foundation::{HWND, LPARAM, MAX_PATH, RECT, WPARAM, CloseHandle},
                Graphics::Gdi::{
                    CreateFontW, GetDC, GetDeviceCaps, ReleaseDC,
                    CLIP_DEFAULT_PRECIS, COLOR_WINDOW, DEFAULT_CHARSET,
                    DEFAULT_PITCH, DEFAULT_QUALITY, FF_MODERN, FW_LIGHT,
                    HBRUSH, HORZRES, LOGPIXELSY, OUT_DEFAULT_PRECIS, VERTRES,
                },
                Storage::FileSystem::FILE_ATTRIBUTE_HIDDEN,
                System::{
                    Diagnostics::{
                        Debug::OutputDebugStringA,
                        ToolHelp::{
                            CreateToolhelp32Snapshot, TH32CS_SNAPMODULE,
                            MODULEENTRY32W, Module32FirstW, Module32NextW
                        }
                    },
                    LibraryLoader::{
                        GetModuleFileNameA,
                        GetModuleFileNameW,
                        GetModuleHandleA
                    },
                    Performance::QueryPerformanceFrequency,
                    SystemInformation::{
                        GetNativeSystemInfo, GlobalMemoryStatus, MEMORYSTATUS,
                        SYSTEM_INFO,
                    },
                    Threading::{OpenProcess, PROCESS_ALL_ACCESS, Sleep},
                    WindowsProgramming::MulDiv,
                },
                UI::{
                    Controls::EM_LINESCROLL,
                    Input::KeyboardAndMouse::SetFocus,
                    WindowsAndMessaging::{
                        AdjustWindowRect, CloseWindow, CreateWindowExA,
                        DestroyWindow, DispatchMessageA, GetDesktopWindow,
                        GetMessageA, LoadCursorA, LoadIconA, LoadImageA,
                        MessageBoxA, PeekMessageA, RegisterClassA,
                        SendMessageA, SetWindowLongPtrA, SetWindowTextA,
                        ShowWindow, TranslateMessage, ES_AUTOHSCROLL,
                        ES_AUTOVSCROLL, ES_MULTILINE, ES_READONLY,
                        GWLP_WNDPROC, HMENU, IDCANCEL, IDC_ARROW, IDNO, IDOK,
                        IDYES, IMAGE_BITMAP, LR_LOADFROMFILE,
                        MB_ICONINFORMATION, MB_ICONSTOP, MB_OK, MB_YESNO,
                        MB_YESNOCANCEL, MESSAGEBOX_STYLE, MSG, PM_NOREMOVE,
                        STM_SETIMAGE, SW_HIDE, SW_SHOW, WINDOW_EX_STYLE,
                        WINDOW_STYLE, WM_SETFONT, WNDCLASSA, WS_BORDER,
                        WS_CAPTION, WS_CHILD, WS_POPUPWINDOW, WS_VISIBLE,
                        WS_VSCROLL,
                    },
                },
            },
        };
        use std::os::windows::prelude::*;
        use platform::{
            os::win32::{con_wnd_proc, input_line_wnd_proc},
            FontHandle,
        };
    } else if #[cfg(xlib)] {
        use x11::xlib::{
            CurrentTime, RevertToParent, XMapWindow, XOpenDisplay,
            XSetInputFocus, XCloseDisplay, ClientMessage, XDestroyWindow,
            XEvent, XNextEvent, XPending,
        };
        use platform::display_server::target::{
            WindowEventExtXlib, XlibContext, WM_DELETE_WINDOW
        };
    } else if #[cfg(appkit)] {
        use platform::display_server::appkit::AppKitWindowHandleExt;
        use icrate::{
            AppKit::{NSApp, NSAlert},
            Foundation::{NSDefaultRunLoopMode, NSDate, NSString}
        };
        use objc2::ffi::NSUIntegerMax;
    }
}

cfg_if! {
    if #[cfg(not(any(wasm, windows)))] {
        use sysinfo::{CpuExt, SystemExt};
    }
}

cfg_if! {
    if #[cfg(linux)] {
        use gtk4::prelude::*;
        use gtk4::builders::MessageDialogBuilder;
        use core::cell::RefCell;
        use std::ffi::OsStr;
        use std::io::BufReader;
    } else if #[cfg(macos)] {
        use std::ffi::CString;
        use cstr::cstr;
        use core::mem::size_of_val;
    }
}

#[cfg(all(windows, any(x86_64, i686)))]
use platform::arch::x86::target::cpuid;

cfg_if! {
    if #[cfg(d3d9)] {
        use cstr::cstr;
        use windows::Win32::Graphics::Direct3D9::{
            Direct3DCreate9, D3D_SDK_VERSION, D3DADAPTER_IDENTIFIER9,
            D3DADAPTER_DEFAULT
        };
    } else if #[cfg(vulkan)] {
        use cstr::cstr;
    }
}

use bitflags::bitflags;

fn in_restart_f() {
    input::shutdown();
    input::init();
}

fn net_restart_f() {
    net::restart();
}

#[allow(clippy::todo)]
fn movie_start_f() {
    todo!()
}

#[allow(clippy::todo)]
fn movie_stop_f() {
    todo!()
}

#[allow(clippy::todo)]
fn listen_f() {
    todo!()
}

#[allow(clippy::todo)]
fn connect_f() {
    todo!()
}

pub fn init() {
    cmd::add_internal("in_restart", in_restart_f).unwrap();
    cmd::add_internal("net_restart", net_restart_f).unwrap();
    cmd::add_internal("movie_start", movie_start_f).unwrap();
    cmd::add_internal("movie_stop", movie_stop_f).unwrap();
    cmd::add_internal("net_listen", listen_f).unwrap();
    cmd::add_internal("net_connect", connect_f).unwrap();

    com::println!(16.into(), "CPU vendor is \"{}\"", get_cpu_vendor(),);
    com::println!(16.into(), "CPU name is \"{}\"", get_cpu_name());

    let info = find_info();

    let c = if info.logical_cpu_count == 1 { "" } else { "s" };
    com::println!(
        16.into(),
        "{} logical CPU{} reported",
        info.logical_cpu_count,
        c,
    );

    let c = if info.physical_cpu_count == 1 {
        ""
    } else {
        "s"
    };
    com::println!(
        16.into(),
        "{} physical CPU{} detected",
        info.physical_cpu_count,
        c,
    );
    com::println!(16.into(), "Measured CPU speed is {:.2} GHz", info.cpu_ghz,);
    com::println!(
        16.into(),
        "Total CPU performance is estimated as {:.2} GHz",
        info.configure_ghz,
    );
    com::println!(
        16.into(),
        "System memory is {} MB (capped at 1 GB)",
        info.sys_mb,
    );
    com::println!(16.into(), "Video card is \"{}\"", info.gpu_description,);
    // TODO - vector support
    com::println!(16.into(), "");
    input::init();
}

lazy_static! {
    static ref BASE_TIME_ACQUIRED: AtomicBool = AtomicBool::new(false);
    static ref TIME_BASE: AtomicIsize = AtomicIsize::new(0);
}

#[cfg(windows)]
pub fn milliseconds() -> isize {
    if BASE_TIME_ACQUIRED.load(SeqCst) == false {
        let now = unsafe { timeGetTime() };
        TIME_BASE.store_relaxed(now as _);
        BASE_TIME_ACQUIRED.store_relaxed(true);
    }

    let now = unsafe { timeGetTime() };
    now as isize - TIME_BASE.load_relaxed()
}

#[cfg(not(windows))]
pub fn milliseconds() -> isize {
    if BASE_TIME_ACQUIRED.load(SeqCst) == false {
        let now = SystemTime::now();
        let time = now.duration_since(UNIX_EPOCH).unwrap().as_millis();
        TIME_BASE.store(time.try_into().unwrap(), SeqCst);
        BASE_TIME_ACQUIRED.store_relaxed(true);
    }

    let time: isize = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap();
    time - TIME_BASE.load(SeqCst)
}

pub fn directory_has_contents(dir: impl AsRef<Path>) -> bool {
    if let Ok(mut d) = dir.as_ref().read_dir() {
        d.next().is_none()
    } else {
        false
    }
}

pub fn list_files(
    dir: impl AsRef<Path>,
    ext: impl AsRef<str>,
    _filter: impl AsRef<str>,
    _pure: bool,
) -> Vec<PathBuf> {
    if let Ok(d) = dir.as_ref().read_dir() {
        d.filter(|d| d.is_ok())
            .map(|d| d.unwrap().path())
            .filter(|d| {
                if ext.as_ref() == "" {
                    true
                } else if let Some(e) = d.extension() {
                    e == ext.as_ref()
                } else {
                    false
                }
            })
            .collect()
    } else {
        Vec::new()
    }
}

#[cfg(windows)]
#[allow(clippy::semicolon_outside_block)]
pub fn get_executable_name() -> String {
    let mut buf = [0u8; MAX_PATH as usize];
    // SAFETY:
    // GetModuleFileNameA is an FFI function, requiring use of unsafe.
    // GetModuleFileNameA itself should never create UB, violate memory
    // safety, etc., provided the buffer passed is long enough,
    // which we've guaranteed is true.
    unsafe {
        GetModuleFileNameA(None, &mut buf);
    }
    let c_string = CStr::from_bytes_until_nul(&buf).unwrap();
    let s = c_string.to_str().unwrap().to_owned();
    let p = PathBuf::from(s);
    let s = p.file_name().unwrap().to_str().unwrap().to_owned();
    s.strip_suffix(".exe")
        .map_or(s.clone(), alloc::borrow::ToOwned::to_owned)
}

#[cfg(linux)]
pub fn get_executable_name() -> String {
    let pid = std::process::id();
    let proc_path = format!("/proc/{pid}/exe");
    std::fs::read_link(proc_path).map_or_else(
        |_| String::new(),
        |f| {
            let file_name = f
                .file_name()
                .unwrap_or_else(|| OsStr::new(""))
                .to_str()
                .unwrap_or("")
                .to_owned();
            let pos = file_name.find('.').unwrap_or(file_name.len());
            file_name.get(..pos).unwrap().to_owned()
        },
    )
}

#[cfg(bsd)]
pub fn get_executable_name() -> String {
    cfg_if! {
        if #[cfg(target_os = "netbsd")] {
            const PROC_PATH: &'static str = "/proc/curproc/exe";
        }
        else {
            const PROC_PATH: &'static str = "/proc/curproc/file";
        }
    }
    // kinfo_getproc method hasn't been tested yet. Not even sure it
    // compiles (don't have a BSD machine to test it on). Probably
    // doesn't work even if it does compile, but the general idea
    // is here
    match std::fs::read_link(proc_path) {
        Ok(f) => f.file_name().unwrap().to_str().unwrap().to_owned(),
        Err(_) => {
            let pid = libc::getpid();
            let kinfo_proc = unsafe { libc::kinfo_getproc(pid) };
            if kinfo_proc.is_null() {
                return String::new();
            }

            let s = CString::new((*kinfo_proc).ki_comm)
                .unwrap_or(CString::new("").unwrap())
                .to_str()
                .unwrap_or("")
                .to_owned();
            unsafe { libc::free(kinfo_proc) };
            s
        }
    }
}

#[cfg(macos)]
pub fn get_executable_name() -> String {
    let mut buf = [0u8; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
    let pid = std::process::id();
    unsafe {
        libc::proc_pidpath(
            pid as libc::c_int,
            addr_of_mut!(buf) as *mut _,
            buf.len() as u32,
        )
    };
    CString::from_vec_with_nul(buf.to_vec())
        .unwrap_or(CString::new("").unwrap())
        .to_str()
        .unwrap_or("")
        .to_owned()
}

// Fallback method - if no platform-specific method is used,
// try to get the executable name from argv[0]
#[cfg(other_os)]
pub fn get_executable_name() -> String {
    let argv_0 = std::env::args()
        .collect::<Vec<String>>()
        .get(0)
        .unwrap()
        .clone();
    let path = PathBuf::from(argv_0);
    let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
    let pos = file_name.find('.').unwrap_or(file_name.len());
    file_name.get(..pos).unwrap().to_owned()
}

const fn get_application_name() -> &'static str {
    "Call of Duty(R) Singleplayer - Ship"
}

pub fn get_semaphore_folder_path() -> Option<PathBuf> {
    let os_folder_path = fs::get_os_folder_path(fs::OsFolder::UserData)?;
    let p: PathBuf = [
        PathBuf::from(os_folder_path),
        PathBuf::from("CoD").join("Activision"),
    ]
    .iter()
    .collect();
    Some(p)
}

#[cfg(windows)]
pub fn get_semaphore_file_name() -> String {
    format!("__{}", get_executable_name())
}

#[cfg(unix)]
pub fn get_semaphore_file_name() -> String {
    format!(".__{}", get_executable_name())
}

#[cfg(other_os)]
pub fn get_semaphore_file_name() -> String {
    com::dprintln!(
        0.into(),
        "sys::get_semaphore_file: using default implementation."
    );
    format!("__{}", get_executable_name())
}

pub fn no_free_files_error() -> ! {
    let msg_box_type = MessageBoxType::Ok;
    let msg_box_icon = MessageBoxIcon::Stop;
    let title = locale::localize_ref("WIN_DISK_FULL_TITLE");
    let text = locale::localize_ref("WIN_DISK_FULL_BODY");
    let handle = None;
    message_box(handle, &title, &text, msg_box_type, Some(msg_box_icon));
    // DoSetEvent_UNK();
    std::process::exit(-1);
}

#[cfg(windows)]
fn is_game_process(pid: u32) -> bool {
    let Ok(hprocess) = (unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, pid) })
    else {
        return false;
    };

    unsafe { CloseHandle(hprocess) };

    let Ok(hsnapshot) =
        (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, pid) })
    else {
        return false;
    };

    let mut me = MODULEENTRY32W::default();
    me.dwSize = size_of_val(&me) as _;
    if unsafe { Module32FirstW(hsnapshot, addr_of_mut!(me)) }.0 != 0 {
        let mut buf = [0u16; MAX_PATH as _];
        unsafe { GetModuleFileNameW(None, &mut buf) };
        buf[buf.len() - 1] = 0x0000;
        let filename = PathBuf::from(String::from_utf16_lossy(&buf));
        let exe = filename.file_name().unwrap();
        let ret = loop {
            if String::from_utf16_lossy(&me.szModule) == exe.to_string_lossy() {
                break true;
            }

            if unsafe { Module32NextW(hsnapshot, addr_of_mut!(me)) }.0 == 0 {
                break false;
            }
        };

        unsafe { CloseHandle(hsnapshot) };

        ret
    } else {
        false
    }
}

#[cfg(linux)]
fn is_game_process(pid: u32) -> bool {
    let this_pid = std::process::id();
    let this_proc_path = format!("/proc/{this_pid}/exe");
    let this = std::fs::read_link(this_proc_path).map_or_else(
        |_| String::new(),
        |f| {
            let file_name = f
                .file_name()
                .unwrap_or_else(|| OsStr::new(""))
                .to_str()
                .unwrap_or("")
                .to_owned();
            let pos = file_name.find('.').unwrap_or(file_name.len());
            file_name.get(..pos).unwrap().to_owned()
        },
    );

    let other_proc_path = format!("/proc/{pid}/exe");
    let other = std::fs::read_link(other_proc_path).map_or_else(
        |_| String::new(),
        |f| {
            let file_name = f
                .file_name()
                .unwrap_or_else(|| OsStr::new(""))
                .to_str()
                .unwrap_or("")
                .to_owned();
            let pos = file_name.find('.').unwrap_or(file_name.len());
            file_name.get(..pos).unwrap().to_owned()
        },
    );

    this == other
}

#[cfg(not(any(windows, linux)))]
const fn is_game_process(_pid: u32) -> bool {
    true
}

pub fn check_crash_or_rerun() -> bool {
    let Some(semaphore_folder_path) = get_semaphore_folder_path() else {
        return true;
    };

    if !std::path::Path::new(&semaphore_folder_path).exists() {
        return std::fs::create_dir_all(&semaphore_folder_path).is_ok();
    }

    let semaphore_file_path =
        semaphore_folder_path.join(get_semaphore_file_name());
    let semaphore_file_exists = semaphore_file_path.exists();

    if semaphore_file_exists {
        if let Ok(mut f) = std::fs::File::open(semaphore_file_path.clone()) {
            let mut buf = [0u8; 4];
            if let Ok(4) = f.read(&mut buf) {
                // let pid_read = u32::from_ne_bytes(buf);
                // if pid_read != std::process::id()
                // || is_game_process(pid_read) == false
                // {
                // return true;
                // }

                let msg_box_type = MessageBoxType::YesNoCancel;
                let msg_box_icon = MessageBoxIcon::Stop;
                let title = locale::localize_ref("WIN_IMPROPER_QUIT_TITLE");
                let text = locale::localize_ref("WIN_IMPROPER_QUIT_BODY");
                let handle = None;
                match message_box(
                    handle,
                    &title,
                    &text,
                    msg_box_type,
                    Some(msg_box_icon),
                ) {
                    Some(MessageBoxResult::Yes) => com::force_safe_mode(),
                    Some(MessageBoxResult::Cancel) | None => return false,
                    _ => {}
                };
            };
        }
    }

    // Create file with hidden attribute on Windows
    // On Unix platforms, the equivalent operation
    // (prefixing the file's name with a '.') should
    // already have been done by get_semaphore_file_name.
    cfg_if! {
        if #[cfg(windows)] {
            let file = std::fs::File::options()
                .write(true)
                .create(true)
                .attributes(FILE_ATTRIBUTE_HIDDEN.0)
                .create(true)
                .open(semaphore_file_path);
        } else {
            let file = std::fs::File::create(semaphore_file_path);
        }
    }

    file.map_or_else(
        |_| no_free_files_error(),
        |mut f| {
            let pid = std::process::id();
            if f.write_all(&pid.to_ne_bytes()).is_err() {
                no_free_files_error();
            } else {
                true
            }
        },
    )
}

pub fn get_cmdline() -> String {
    let mut cmd_line = String::new();
    std::env::args().for_each(|arg| {
        cmd_line.push_str(&arg);
    });
    cmd_line.trim().to_owned()
}

pub fn start_minidump(b: bool) {
    com::println!(0.into(), "Starting minidump with b = {}...", b);
    com::println!(0.into(), "TODO: implement.");
}

fn normal_exit() {
    let semaphore_file_path = get_semaphore_folder_path()
        .unwrap()
        .join(get_semaphore_file_name());
    std::fs::remove_file(semaphore_file_path).unwrap();
}

// TODO - use processor affinity masks to get the number of logical
// CPUs actually available to the process
#[cfg(all(windows, x86))]
pub fn get_logical_cpu_count() -> usize {
    let mut system_info = SYSTEM_INFO::default();
    unsafe { GetNativeSystemInfo(addr_of_mut!(system_info)) };
    system_info.dwNumberOfProcessors as _
}

// TODO - actually get number of physical CPUs
// (SYSTEM_INFO::dwNumberOfProcessors returns the logical CPU count)
#[cfg(all(windows, x86))]
pub fn get_physical_cpu_count() -> usize {
    let mut system_info = SYSTEM_INFO::default();
    unsafe { GetNativeSystemInfo(addr_of_mut!(system_info)) };
    system_info.dwNumberOfProcessors as _
}

// TODO - use GlobalMemoryStatusEx
#[cfg(all(windows, x86))]
pub fn system_memory_mb() -> u64 {
    let mut memory_status = MEMORYSTATUS::default();
    memory_status.dwLength = size_of_val(&memory_status) as _;
    unsafe { GlobalMemoryStatus(addr_of_mut!(memory_status)) };
    memory_status.dwAvailPhys as _
}

#[cfg(all(windows, x86))]
pub fn get_cpu_vendor() -> String {
    // Make the buffer large enough to contain a null-terminator in
    // the case that the CPU vendor string is 12-bytes long (AMD and
    // Intel CPUs both are)
    //
    // Theoretically, we could declare it as [0u8; 13], but then
    // we'd have to do some annoying casts to get the CpuidResult's
    // fields into it. I'd rather waste three bytes.
    let mut vendor_buf = [0u32; 4];
    let cpuid_result = cpuid(0x0000_0000);
    vendor_buf[0] = cpuid_result.ebx;
    vendor_buf[1] = cpuid_result.ecx;
    vendor_buf[2] = cpuid_result.edx;
    // vendor_buf[3] was zeroed in name_buf's initialization
    let bytes: [u8; 16] = unsafe { transmute(vendor_buf) };
    CStr::from_bytes_until_nul(&bytes)
        .unwrap()
        .to_string_lossy()
        .to_string()
}

#[cfg(all(windows, x86))]
pub fn get_cpu_name() -> String {
    // Make the buffer large enough to contain a null-terminator in
    // the case that the CPU brand string is 48-bytes long
    //
    // Theoretically, we could declare it as [0u8; 49], but then
    // we'd have to do some annoying casts to get the CpuidResult's
    // fields into it. I'd rather waste three bytes.
    let mut name_buf = [0u32; 13];
    let cpuid_result = cpuid(0x8000_0002);
    name_buf[0] = cpuid_result.eax;
    name_buf[1] = cpuid_result.ebx;
    name_buf[2] = cpuid_result.ecx;
    name_buf[3] = cpuid_result.edx;
    let cpuid_result = cpuid(0x8000_0003);
    name_buf[4] = cpuid_result.eax;
    name_buf[5] = cpuid_result.ebx;
    name_buf[6] = cpuid_result.ecx;
    name_buf[7] = cpuid_result.edx;
    let cpuid_result = cpuid(0x8000_0004);
    name_buf[8] = cpuid_result.eax;
    name_buf[9] = cpuid_result.ebx;
    name_buf[10] = cpuid_result.ecx;
    name_buf[11] = cpuid_result.edx;
    // name_buf[12] was zeroed in name_buf's initialization
    let bytes: [u8; 52] = unsafe { transmute(name_buf) };
    CStr::from_bytes_until_nul(&bytes)
        .unwrap()
        .to_string_lossy()
        .to_string()
}

#[cfg(not(all(windows, x86)))]
pub fn get_logical_cpu_count() -> usize {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    system.cpus().len()
}

#[cfg(not(all(windows, x86)))]
pub fn get_physical_cpu_count() -> usize {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    system
        .physical_core_count()
        .map_or_else(get_logical_cpu_count, |u| u)
}

#[cfg(not(all(windows, x86)))]
pub fn system_memory_mb() -> u64 {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    system.total_memory()
}

#[cfg(not(all(windows, x86)))]
pub fn get_cpu_vendor() -> String {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    system.global_cpu_info().vendor_id().to_owned()
}

#[cfg(not(all(windows, x86)))]
pub fn get_cpu_name() -> String {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    system
        .global_cpu_info()
        .brand()
        .to_owned()
        .trim()
        .to_owned()
}

#[cfg(wasm)]
#[allow(clippy::missing_const_for_fn)]
pub fn get_logical_cpu_count() -> usize {
    0
}

#[cfg(wasm)]
#[allow(clippy::missing_const_for_fn)]
pub fn get_physical_cpu_count() -> usize {
    0
}

#[cfg(wasm)]
#[allow(clippy::missing_const_for_fn)]
pub fn system_memory_mb() -> u64 {
    0
}

#[cfg(wasm)]
pub fn get_cpu_vendor() -> String {
    "Unknown CPU vendor".to_owned()
}

#[cfg(wasm)]
pub fn get_cpu_name() -> String {
    "Unknown CPU name".to_owned()
}

#[cfg(wgpu)]
pub fn detect_video_card() -> String {
    let adapter = pollster::block_on(platform::render::wgpu::Adapter::new(
        &platform::render::wgpu::Instance::new(),
        None,
    ));
    adapter.get_info().name
}

#[cfg(d3d9)]
pub fn detect_video_card() -> String {
    let Some(d3d9) = (unsafe { Direct3DCreate9(D3D_SDK_VERSION) }) else {
        return String::from("Unknown video card");
    };

    let mut identifier = D3DADAPTER_IDENTIFIER9::default();
    if unsafe {
        d3d9.GetAdapterIdentifier(
            D3DADAPTER_DEFAULT,
            0,
            addr_of_mut!(identifier),
        )
    }
    .is_ok()
    {
        CStr::from_bytes_until_nul(&identifier.Description)
            .unwrap_or_else(|_| cstr!("Unknown video card"))
            .to_string_lossy()
            .to_string()
    } else {
        String::from("Unknown video card")
    }
}

#[cfg(vulkan)]
pub fn detect_video_card() -> String {
    use ash::{vk, Entry};

    let entry = unsafe { Entry::load().unwrap() };
    let app_info = vk::ApplicationInfo {
        api_version: vk::make_api_version(0, 1, 3, 0),
        ..Default::default()
    };
    let create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        ..Default::default()
    };
    let instance =
        unsafe { entry.create_instance(&create_info, None) }.unwrap();
    let pdevs = unsafe { instance.enumerate_physical_devices() }.unwrap();
    assert!(!pdevs.is_empty());
    let pdev = pdevs[0];
    let props = unsafe { instance.get_physical_device_properties(pdev) };
    CStr::from_bytes_until_nul(&props.device_name.map(|c| c as _))
        .unwrap_or_else(|_| cstr!("Unknown video card"))
        .to_string_lossy()
        .to_string()
}

#[cfg(windows)]
fn seconds_per_tick() -> f64 {
    unsafe { Sleep(0) };
    let mut frequency = 0i64;
    let _ =
        unsafe { QueryPerformanceFrequency(addr_of_mut!(frequency)) }.unwrap();
    1.0f64 / frequency as f64
}

#[cfg(linux)]
fn seconds_per_tick() -> f64 {
    let Ok(cpuinfo) = std::fs::File::open("/proc/cpuinfo") else {
        return 0.0f64;
    };

    // We can't just use std::fs::read_to_string since there's no *guarantee*
    // the contents of /proc/cpuinfo are valid UTF-8 (it probably is, but we
    // don't want to rely on that). So instead, we read it into a CString and
    // use CString::to_string_lossy and parse whatever valid UTF-8 it gives.
    let mut reader = BufReader::new(cpuinfo);
    let mut buf = Vec::new();
    let _ = reader.read_to_end(&mut buf).unwrap();
    // The file isn't guaranteed to be null-terminated, so tack a
    // null-terminator on here just in case
    buf.push(b'\0');
    let buf_str = CString::from_vec_with_nul(buf).unwrap();
    let s = buf_str.to_string_lossy();

    // Now that we have some valid UTF-8, find the line containing the CPU
    // frequency.
    let mut lines = s.lines();
    let line = lines.find(|l| l.contains("cpu MHz\t\t: ")).unwrap();

    // scanf! failed kept giving a MatchFailed error, so we're doing this the
    // old fashioned way. split_whitespace will yield the frequency as its 3rd
    // element
    let toks = line.split_whitespace().collect::<Vec<_>>();
    let mhz = toks[3].parse::<f64>().unwrap();

    let hz = mhz * 1_000_000f64;

    1.0f64 / hz
}

#[cfg(macos)]
fn seconds_per_tick() -> f64 {
    let mut hz = 0;
    libc::sysctlbyname(
        cstr!("hw.cpufrequency").as_ptr(),
        addr_of_mut!(hz),
        size_of_val(&hz),
        core::ptr::null_mut(),
        0,
    );
    if hz == 0 {
        return 0.0f64;
    }

    1.0f64 / hz as f64
}

#[cfg(any(other_unix, other_os, no_os))]
fn seconds_per_tick() -> f64 {
    todo!()
}

pub fn init_timing() {
    *MSEC_PER_RAW_TIMER_TICK.write().unwrap() = seconds_per_tick() * 1000.0f64;
}

#[derive(Clone, Default)]
pub struct SysInfo {
    pub gpu_description: String,
    pub logical_cpu_count: usize,
    pub physical_cpu_count: usize,
    pub sys_mb: u64,
    pub cpu_vendor: String,
    pub cpu_name: String,
    pub cpu_ghz: f32,
    pub configure_ghz: f32,
}

impl Display for SysInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "GPU Description: {}\nCPU: {} ({})\nCores: {} ({} \
             physical)\nSystem RAM: {}MiB",
            self.gpu_description,
            self.cpu_name,
            self.cpu_vendor,
            self.logical_cpu_count,
            self.physical_cpu_count,
            self.sys_mb
        )
    }
}

impl SysInfo {
    fn new() -> Self {
        Self::default()
    }
}

lazy_static! {
    static ref SYS_INFO: RwLock<Option<SysInfo>> = RwLock::new(None);
    static ref MSEC_PER_RAW_TIMER_TICK: RwLock<f64> = RwLock::new(0.0f64);
}

#[allow(
    clippy::cast_precision_loss,
    clippy::as_conversions,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]
pub fn find_info() -> SysInfo {
    let mut sys_info = SYS_INFO.write().unwrap();
    if sys_info.is_none() {
        let gpu_description = detect_video_card();
        let logical_cpu_count = get_logical_cpu_count();
        let physical_cpu_count = get_physical_cpu_count();
        let sys_mb = (system_memory_mb() as f64 / (1024f64 * 1024f64))
            .clamp(0f64, f64::MAX) as u64;
        let cpu_vendor = get_cpu_vendor();
        let cpu_name = get_cpu_name();
        let cpu_ghz = 1.0f64
            / (*MSEC_PER_RAW_TIMER_TICK.read().unwrap() * 1_000_000.0f64);
        let configure_ghz = cpu_ghz;

        *sys_info = Some(SysInfo {
            gpu_description,
            logical_cpu_count,
            physical_cpu_count,
            sys_mb,
            cpu_vendor,
            cpu_name,
            cpu_ghz: cpu_ghz as _,
            configure_ghz: configure_ghz as _,
        });
    }
    sys_info.as_ref().unwrap().clone()
}

#[derive(Clone, Debug)]
pub enum EventType {
    None,
    Key(KeyboardScancode, bool),
    Mouse(MouseScancode, bool),
    Character(char),
    Console(String),
}

#[derive(Clone, Debug)]
pub struct Event {
    time: isize,
    event_type: EventType,
}

impl Event {
    pub fn new(time: Option<isize>, event_type: EventType) -> Self {
        Self {
            time: time.unwrap_or_default(),
            event_type,
        }
    }
}

lazy_static! {
    static ref EVENT_QUEUE: RwLock<VecDeque<Event>> =
        RwLock::new(VecDeque::new());
}

pub fn enqueue_event(mut ev: Event) {
    if ev.time == 0 {
        ev.time = milliseconds();
    }

    EVENT_QUEUE.write().unwrap().push_back(ev);
}

pub fn render_fatal_error() -> ! {
    let msg_box_type = MessageBoxType::Ok;
    let msg_box_icon = MessageBoxIcon::Stop;
    let title = locale::localize_ref("WIN_RENDER_INIT_TITLE");
    let text = locale::localize_ref("WIN_RENDER_INIT_BODY");
    let handle = None;
    message_box(handle, &title, &text, msg_box_type, Some(msg_box_icon));
    // DoSetEvent_UNK();
    std::process::exit(-1);
}

lazy_static! {
    static ref QUIT_EVENT: Mutex<SmpEvent> =
        Mutex::new(SmpEvent::new(SignalState::Cleared, true));
}

pub fn set_quit_event() {
    let mut ev = QUIT_EVENT.lock().unwrap().clone();
    ev.set();
}

pub fn query_quit_event() -> SignalState {
    let mut ev = QUIT_EVENT.lock().unwrap().clone();
    ev.query()
}

lazy_static! {
    static ref RG_REGISTERED_EVENT: Mutex<SmpEvent> =
        Mutex::new(SmpEvent::new(SignalState::Cleared, true));
}

pub fn clear_rg_registered_event() {
    let mut ev = RG_REGISTERED_EVENT.lock().unwrap().clone();
    ev.clear();
}

pub fn query_rg_registered_event() -> SignalState {
    let mut ev = RG_REGISTERED_EVENT.lock().unwrap().clone();
    ev.query()
}

pub fn set_rg_registered_event() {
    let mut ev = RG_REGISTERED_EVENT.lock().unwrap().clone();
    ev.set();
}

pub fn wait_rg_registered_event() {
    let mut ev = RG_REGISTERED_EVENT.lock().unwrap().clone();
    ev.wait();
}

lazy_static! {
    static ref BACKEND_EVENT: Mutex<SmpEvent> =
        Mutex::new(SmpEvent::new(SignalState::Cleared, true));
}

pub fn query_backend_event() -> SignalState {
    let mut ev = BACKEND_EVENT.lock().unwrap().clone();
    ev.query()
}

pub fn set_backend_event() {
    let mut ev = BACKEND_EVENT.lock().unwrap().clone();
    ev.set();
}

pub fn wait_backend_event() {
    let mut ev = BACKEND_EVENT.lock().unwrap().clone();
    ev.wait();
}

lazy_static! {
    static ref RENDER_DEVICE_OK_EVENT: Mutex<SmpEvent> =
        Mutex::new(SmpEvent::new(SignalState::Signaled, true));
}

pub fn query_render_device_ok_event() -> SignalState {
    let mut ev = RENDER_DEVICE_OK_EVENT.lock().unwrap().clone();
    ev.query()
}

lazy_static! {
    static ref RENDER_COMPLETED_EVENT: Mutex<SmpEvent> =
        Mutex::new(SmpEvent::new(SignalState::Signaled, true));
}

pub fn query_render_completed_event() -> SignalState {
    let mut ev = RENDER_COMPLETED_EVENT.lock().unwrap().clone();
    ev.query()
}

pub fn create_thread<T, F: Fn() -> T + Send + Sync + 'static>(
    name: &str,
    function: F,
) -> Option<JoinHandle<()>> {
    match std::thread::Builder::new()
        .name(name.into())
        .spawn(move || {
            std::thread::park();
            function();
        }) {
        Ok(h) => Some(h),
        Err(e) => {
            com::println!(
                1.into(),
                "error {} while creating thread {}",
                e,
                name,
            );
            None
        }
    }
}

pub fn spawn_render_thread<F: Fn() -> ! + Send + Sync + 'static>(
    function: F,
) -> bool {
    create_thread("Backend", function).map_or(false, |h| {
        h.thread().unpark();
        true
    })
}

// const MAX_CPUS: usize = 32;
//
// lazy_static! {
// static ref S_CPU_COUNT: AtomicUsize = AtomicUsize::new(0);
// static ref S_AFFINITY_MASK_FOR_PROCESS: AtomicUsize = AtomicUsize::new(0);
// static ref S_AFFINITY_MASK_FOR_CPU: Arc<RwLock<ArrayVec<usize, MAX_CPUS>>> =
// Arc::new(RwLock::new(ArrayVec::new())); }
//
// cfg_if! {
// if #[cfg(target_os = "windows")] {
// pub fn init_threads() {
// let hprocess = unsafe { GetCurrentProcess() };
// let systemaffinitymask: c_ulonglong = 0;
// let processaffinitymask: c_ulonglong = 0;
// unsafe { GetProcessAffinityMask(hprocess, addr_of!(processaffinitymask) as
// *mut _, addr_of!(systemaffinitymask) as *mut _) };
// S_AFFINITY_MASK_FOR_PROCESS.store(processaffinitymask as _,
// Ordering::SeqCst); let mut cpu_count = 0usize;
// let mut affinity_mask_for_cpu: Vec<usize> = Vec::new();
// affinity_mask_for_cpu.push(1);
// while (!affinity_mask_for_cpu[0] + 1 & processaffinitymask as usize) != 0 {
// if (affinity_mask_for_cpu[0] & processaffinitymask as usize) != 0 {
// affinity_mask_for_cpu[cpu_count + 1] = affinity_mask_for_cpu[0];
// cpu_count += 1;
// if cpu_count == MAX_CPUS { break; }
// }
// affinity_mask_for_cpu[0] = affinity_mask_for_cpu[0] << 1;
// }
//
// if cpu_count == 0 || cpu_count == 1 {
// S_CPU_COUNT.store(1, Ordering::SeqCst);
// S_AFFINITY_MASK_FOR_CPU.clone().write().unwrap()[0] = 0xFFFFFFFF;
// return;
// }
//
// S_CPU_COUNT.store(cpu_count, Ordering::SeqCst);
// let lock = S_AFFINITY_MASK_FOR_CPU.clone();
// let mut writer = lock.write().unwrap();
// writer[0] = affinity_mask_for_cpu[1];
// writer[1] = affinity_mask_for_cpu[cpu_count];
// if cpu_count > 2 {
// if cpu_count == 3 {
// writer[2] = affinity_mask_for_cpu[2];
// } else if cpu_count == 4 {
// writer[2] = affinity_mask_for_cpu[2];
// writer[3] = affinity_mask_for_cpu[3];
// } else {
// writer.iter_mut().for_each(|a| *a = 0xFFFFFFFF);
// if cpu_count > MAX_CPUS {
// S_CPU_COUNT.store(MAX_CPUS, Ordering::SeqCst);
// }
// }
// }
// }
// }
// }

// cfg_if! {
// if #[cfg(target_os = "windows")] {
// lazy_static! {
// static ref THREAD_LOCK: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
// static ref H_THREADS: Arc<RwLock<ArrayVec<HANDLE, 15>>> =
// Arc::new(RwLock::new(ArrayVec::new())); }
//
// pub fn lock_thread_affinity() {
// let cpu_count = S_CPU_COUNT.load(Ordering::SeqCst);
//
// if cpu_count == 1 {
// return;
// }
//
// let thread_lock = THREAD_LOCK.clone();
// let _thread_lock_2 = thread_lock.lock().unwrap();
//
// let threads_lock = H_THREADS.clone();
// let threads_reader = threads_lock.read().unwrap();
//
// let affinity_mask_lock = S_AFFINITY_MASK_FOR_CPU.clone();
// let affinity_mask_reader = affinity_mask_lock.read().unwrap();
//
// if threads_reader[0].0 != 0 {
// unsafe { SetThreadAffinityMask(threads_reader[0], affinity_mask_reader[0]) };
// }
//
// if threads_reader[1].0 != 0 {
// unsafe { SetThreadAffinityMask(threads_reader[1], affinity_mask_reader[1]) };
// }
//
// if cpu_count < 3 && threads_reader[13].0 != 0 {
// unsafe { SetThreadAffinityMask(threads_reader[13], affinity_mask_reader[1])
// }; } else if threads_reader[13].0 != 0 {
// unsafe { SetThreadAffinityMask(threads_reader[13], affinity_mask_reader[2])
// }; }
//
// if cpu_count > 2 && threads_reader[2].0 != 0 {
// unsafe { SetThreadAffinityMask(threads_reader[2], affinity_mask_reader[2]) };
// }
//
// if cpu_count > 3 && threads_reader[3].0 != 0 {
// unsafe { SetThreadAffinityMask(threads_reader[3], affinity_mask_reader[3]) };
// }
// }
// }
// }

// cfg_if! {
// if #[cfg(target_os = "windows")] {
// pub fn unlock_thread_affinity() {
// let cpu_count = S_CPU_COUNT.load(Ordering::SeqCst);
//
// if cpu_count == 1 {
// return;
// }
//
// let thread_lock = THREAD_LOCK.clone();
// let _thread_lock_2 = thread_lock.lock().unwrap();
//
// let threads_lock = H_THREADS.clone();
// let threads_reader = threads_lock.read().unwrap();
//
// let affinity_mask = S_AFFINITY_MASK_FOR_PROCESS.load(Ordering::SeqCst);
//
// if threads_reader[0].0 != 0 {
// unsafe { SetThreadAffinityMask(threads_reader[0], affinity_mask) };
// }
//
// if threads_reader[1].0 != 0 {
// unsafe { SetThreadAffinityMask(threads_reader[1], affinity_mask) };
// }
//
// if threads_reader[13].0 != 0 {
// unsafe { SetThreadAffinityMask(threads_reader[0], affinity_mask) };
// }
//
// if threads_reader[2].0 != 0 {
// unsafe { SetThreadAffinityMask(threads_reader[1], affinity_mask) };
// }
//
// if threads_reader[3].0 != 0 {
// unsafe { SetThreadAffinityMask(threads_reader[1], affinity_mask) };
// }
// }
// }
// }

// fn register_info_dvars() {
// dvar::register_float(
// "sys_configureGHz",
// 0.0,
// Some(f32::MIN),
// Some(f32::MAX),
// dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::WRITE_PROTECTED,
// Some("Normalized total CPU power, based on cpu type, count, and speed; used
// in autoconfigure") );
// dvar::register_int(
// "sys_sysMB",
// 0,
// Some(i32::MIN),
// Some(i32::MAX),
// dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::WRITE_PROTECTED,
// Some("Physical memory in the system"),
// );
// dvar::register_string(
// "sys_gpu",
// "",
// dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::WRITE_PROTECTED,
// Some("GPU description"),
// );
// dvar::register_int(
// "sys_configSum",
// 0,
// Some(i32::MIN),
// Some(i32::MAX),
// dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::WRITE_PROTECTED,
// Some("Configuration checksum"),
// );
// TODO - SIMD support Dvar
// dvar::register_float(
// "sys_cpuGHz",
// info().unwrap().cpu_ghz,
// Some(f32::MIN),
// Some(f32::MAX),
// dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::WRITE_PROTECTED,
// Some("Measured CPU speed"),
// );
// dvar::register_string(
// "sys_cpuName",
// &info().unwrap().cpu_name,
// dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::WRITE_PROTECTED,
// Some("CPU name description"),
// );
// }
//
// fn archive_info(sum: i32) {
// register_info_dvars();
// dvar::set_float_internal("sys_configureGHz", info().unwrap().configure_ghz);
// dvar::set_int_internal("sys_sysMB", info().unwrap().sys_mb as _);
// dvar::set_string_internal("sys_gpu", &info().unwrap().gpu_description);
// dvar::set_int_internal("sys_configSum", sum);
// }

fn should_update_for_info_change() -> bool {
    let msg_box_type = MessageBoxType::YesNo;
    let msg_box_icon = MessageBoxIcon::Information;
    let title = locale::localize_ref("WIN_CONFIGURE_UPDATED_TITLE");
    let text = locale::localize_ref("WIN_CONFIGURE_UPDATED_BODY");
    let handle = None;
    matches!(
        message_box(handle, &title, &text, msg_box_type, Some(msg_box_icon)),
        Some(MessageBoxResult::Yes)
    )
}

#[cfg(windows)]
#[derive(Copy, Clone, Default, Debug)]
#[repr(u32)]
pub enum MessageBoxType {
    #[default]
    Ok = MB_OK.0,
    YesNoCancel = MB_YESNOCANCEL.0,
    YesNo = MB_YESNO.0,
    // TODO - maybe implement Help?
}

#[cfg(windows)]
#[derive(Copy, Clone, Default, Debug)]
#[repr(u32)]
pub enum MessageBoxIcon {
    #[default]
    None = 0x0000_0000,
    Stop = MB_ICONSTOP.0,
    Information = MB_ICONINFORMATION.0,
}

#[cfg(windows)]
#[derive(Copy, Clone, FromPrimitive)]
#[repr(i32)]
pub enum MessageBoxResult {
    Ok = IDOK.0,
    Cancel = IDCANCEL.0,
    Yes = IDYES.0,
    No = IDNO.0,
    Unknown,
}

#[cfg(linux)]
#[derive(Copy, Clone, Default, Debug)]
#[repr(u32)]
pub enum MessageBoxType {
    #[default]
    Ok,
    YesNoCancel,
    YesNo,
    // TODO - maybe implement Help?
}

#[cfg(linux)]
impl TryInto<gtk4::ButtonsType> for MessageBoxType {
    type Error = ();
    fn try_into(self) -> Result<gtk4::ButtonsType, Self::Error> {
        match self {
            Self::Ok => Ok(gtk4::ButtonsType::Ok),
            Self::YesNo => Ok(gtk4::ButtonsType::YesNo),
            _ => Err(()),
        }
    }
}

#[cfg(linux)]
#[derive(Copy, Clone, Default, Debug)]
#[repr(u32)]
pub enum MessageBoxIcon {
    #[default]
    None,
    Stop,
    Information,
}

#[cfg(linux)]
impl TryInto<gtk4::MessageType> for MessageBoxIcon {
    type Error = ();
    fn try_into(self) -> Result<gtk4::MessageType, Self::Error> {
        use gtk4::MessageType::*;
        match self {
            Self::Information => Ok(Info),
            Self::Stop => Ok(Error),
            _ => Err(()),
        }
    }
}

#[cfg(linux)]
#[derive(Copy, Clone, FromPrimitive, Debug)]
#[repr(i32)]
pub enum MessageBoxResult {
    Ok,
    Cancel,
    Yes,
    No,
    Unknown,
}

#[cfg(linux)]
impl From<gtk4::ResponseType> for MessageBoxResult {
    fn from(value: gtk4::ResponseType) -> Self {
        match value {
            gtk4::ResponseType::Ok => Self::Ok,
            gtk4::ResponseType::Cancel => Self::Cancel,
            gtk4::ResponseType::Yes => Self::Yes,
            gtk4::ResponseType::No => Self::No,
            _ => Self::Unknown,
        }
    }
}

#[cfg(not(any(windows, linux)))]
#[derive(Copy, Clone, Default, Debug)]
#[repr(u32)]
pub enum MessageBoxType {
    #[default]
    Ok,
    YesNoCancel,
    YesNo,
}

#[cfg(not(any(windows, linux)))]
#[derive(Copy, Clone, Default, Debug)]
#[repr(u32)]
pub enum MessageBoxIcon {
    #[default]
    None,
    Stop,
    Information,
}

#[cfg(not(any(windows, linux)))]
#[derive(Copy, Clone, FromPrimitive, Debug)]
#[repr(i32)]
pub enum MessageBoxResult {
    Ok,
    Cancel,
    Yes,
    No,
    Unknown,
}

#[cfg(windows)]
pub fn message_box(
    handle: Option<WindowHandle>,
    title: &str,
    text: &str,
    msg_box_type: MessageBoxType,
    msg_box_icon: Option<MessageBoxIcon>,
) -> Option<MessageBoxResult> {
    let hwnd = handle.map_or(0 as _, |h| h.get_win32().unwrap().hwnd);

    let Ok(ctext) = CString::new(text) else {
        return None;
    };

    let Ok(ctitle) = CString::new(title) else {
        return None;
    };

    let ctype = MESSAGEBOX_STYLE(
        msg_box_type as u32
            | msg_box_icon.unwrap_or(MessageBoxIcon::None) as u32,
    );

    // SAFETY:
    // MessageBoxA is an FFI function, requiring use of unsafe.
    // MessageBoxA itself should never create UB, violate memory
    // safety, etc., regardless of the parameters passed to it.
    let res: MessageBoxResult = num::FromPrimitive::from_i32(
        unsafe {
            MessageBoxA(
                HWND(hwnd as _),
                PCSTR(ctext.as_ptr().cast()),
                PCSTR(ctitle.as_ptr().cast()),
                ctype,
            )
        }
        .0,
    )
    .unwrap_or(MessageBoxResult::Unknown);
    Some(res)
}

#[cfg(linux)]
// The non-Windows implementations of message_box() will use GTK
// by default, instead of targeting each, e.g. Wayland, X, Cocoa, etc.
// For platforms that don't support GTK for some reason,
// other implementations are welcome

// The GTK implementation here is very much a work in progress. It's
// super buggy on WSL2, but I can't tell if the issues are with the
// application here, or with WSL2. Will try to test on native Linux
// at some point
lazy_static! {
    static ref GTK_WINDOW_TITLE: Arc<RwLock<String>> =
        Arc::new(RwLock::new(String::new()));
}

#[cfg(linux)]
thread_local! {
    static GTK_RESPONSE_EVENT: RefCell<SmpEvent>
        = RefCell::new(SmpEvent::new(SignalState::Cleared, false));

    static GTK_RESPONSE_EVENT_VALUE:
        RefCell<Option<gtk4::ResponseType>>
            = RefCell::new(None);
}

#[cfg(linux)]
#[allow(clippy::unnecessary_wraps)]
pub fn message_box(
    _handle: Option<WindowHandle>,
    text: &str,
    title: &str,
    msg_box_type: MessageBoxType,
    msg_icon_type: Option<MessageBoxIcon>,
) -> Option<MessageBoxResult> {
    let dialog = MessageDialogBuilder::new()
        .buttons(gtk4::ButtonsType::None)
        .destroy_with_parent(true)
        .focusable(true)
        //.message_type(msg_icon_type.unwrap_or(MessageBoxIcon::None)
        //.try_into().unwrap_or(gtk4::MessageType::Other))
        .message_type(msg_icon_type.unwrap().try_into().unwrap())
        .modal(false)
        .name(title)
        .resizable(false)
        .title(title)
        .text(text)
        .visible(true)
        .build();

    let buttons = &match msg_box_type {
        MessageBoxType::Ok => vec![("Ok", gtk4::ResponseType::Ok)],
        MessageBoxType::YesNo => vec![
            ("Yes", gtk4::ResponseType::Yes),
            ("No", gtk4::ResponseType::No),
        ],
        MessageBoxType::YesNoCancel => vec![
            ("Yes", gtk4::ResponseType::Yes),
            ("No", gtk4::ResponseType::No),
            ("Cancel", gtk4::ResponseType::Cancel),
        ],
    };

    dialog.add_buttons(buttons);
    dialog.run_async(|obj, answer| {
        obj.close();
        GTK_RESPONSE_EVENT.with(|event| {
            #[allow(unused_must_use)]
            {
                GTK_RESPONSE_EVENT_VALUE.with(|value| {
                    *value.borrow_mut() = Some(answer);
                });
                event.borrow_mut().set();
            }
        });
    });

    let response = GTK_RESPONSE_EVENT.with(|event| {
        event.borrow_mut().wait();
        GTK_RESPONSE_EVENT_VALUE.with(|value| value.borrow().unwrap())
    });

    Some(response.into())
}

#[cfg(appkit)]
pub fn message_box(
    _handle: Option<WindowHandle>,
    text: &str,
    title: &str,
    msg_box_type: MessageBoxType,
    _msg_icon_type: Option<MessageBoxIcon>,
) -> Option<MessageBoxResult> {
    // defining these here the NSAlertXXXButtonReturn constants are defined as
    // statics instead of consts and wouldn't play nice
    const FIRST_BUTTON: isize = 1000;
    const SECOND_BUTTON: isize = 1001;
    const THIRD_BUTTON: isize = 1002;

    let alert = unsafe { NSAlert::new() };
    unsafe {
        alert.setMessageText(&NSString::from_str(&format!(
            "{}\n\n{}",
            title, text
        )))
    };
    let buttons = match msg_box_type {
        MessageBoxType::Ok => vec!["Ok"],
        MessageBoxType::YesNo => vec!["Yes", "No"],
        MessageBoxType::YesNoCancel => vec!["Yes", "No", "Cancel"],
    };

    for button in buttons {
        unsafe { alert.addButtonWithTitle(&NSString::from_str(button)) };
    }

    match unsafe { alert.runModal() } {
        FIRST_BUTTON => match msg_box_type {
            MessageBoxType::Ok => Some(MessageBoxResult::Ok),
            MessageBoxType::YesNo | MessageBoxType::YesNoCancel => {
                Some(MessageBoxResult::Yes)
            }
        },
        SECOND_BUTTON => match msg_box_type {
            MessageBoxType::Ok => panic!("where the fuck i am"),
            MessageBoxType::YesNo | MessageBoxType::YesNoCancel => {
                Some(MessageBoxResult::No)
            }
        },
        THIRD_BUTTON => match msg_box_type {
            MessageBoxType::Ok | MessageBoxType::YesNo => {
                panic!("where the fuck i am")
            }
            MessageBoxType::YesNoCancel => Some(MessageBoxResult::Cancel),
        },
        _ => panic!("where the fuck i am"),
    }
}

#[cfg(not(any(windows, linux, appkit)))]
pub fn message_box(
    handle: Option<WindowHandle>,
    text: &str,
    title: &str,
    msg_box_type: MessageBoxType,
    msg_icon_type: Option<MessageBoxIcon>,
) -> Option<MessageBoxResult> {
    println!(
        "message_box: handle={:?} text={}, title={}, type={:?}, icon={:?}",
        handle, text, title, msg_box_type, msg_icon_type,
    );
    None
}

cfg_if! {
    if #[cfg(debug_assertions)] {
        static DEBUG_OUTPUT: AtomicBool = AtomicBool::new(true);
    } else {
        static DEBUG_OUTPUT: AtomicBool = AtomicBool::new(false);
    }
}

cfg_if! {
    if #[cfg(target_os = "windows")] {
        fn output_debug_string(string: impl ToString) {
            // SAFETY:
            // OutputDebugStringA is an FFI function, requiring use of unsafe.
            // OutputDebugStringA itself should never create UB, violate memory
            // safety, etc., in any scenario.
            unsafe { OutputDebugStringA(PCSTR(string.to_string().as_ptr())); }
        }
    } else {
        fn output_debug_string(string: impl ToString) {
            com::dprint!(0.into(), "sys::print: {}", string.to_string());
        }
    }
}

#[doc(hidden)]
pub fn _print_internal(arguments: core::fmt::Arguments) {
    if DEBUG_OUTPUT.load(Ordering::Relaxed) {
        output_debug_string(arguments);
    }

    conbuf::append_text_in_main_thread(arguments);
}

#[macro_export]
macro_rules! __sys_print {
    ($($arg:tt)*) => {{
        $crate::sys::_print_internal(core::format_args!($($arg)*));
    }};
}
pub use __sys_print as print;

#[macro_export]
macro_rules! __sys_println {
    () => {
        $crate::sys::print!("\n")
    };
    ($($arg:tt)*) => {{
        $crate::sys::print!("{}\n", core::format_args!($($arg)*));
    }};
}
pub use __sys_println as println;

#[cfg(windows)]
pub fn create_console() {
    let hinstance = unsafe { GetModuleHandleA(None) }.unwrap_or_default();
    let class_name = s!("CoD Black Ops WinConsole");

    let mut wnd_class = WNDCLASSA::default();
    wnd_class.hInstance = hinstance;
    wnd_class.hIcon =
        unsafe { LoadIconA(hinstance, PCSTR(1 as _)) }.unwrap_or_default();
    wnd_class.hCursor = unsafe { LoadCursorA(None, PCSTR(IDC_ARROW.0 as _)) }
        .unwrap_or_default();
    wnd_class.hbrBackground = HBRUSH(COLOR_WINDOW.0 as _);
    wnd_class.lpszClassName = class_name;
    wnd_class.lpfnWndProc = Some(con_wnd_proc);
    if unsafe { RegisterClassA(addr_of!(wnd_class)) } == 0 {
        return;
    }

    let dwstyle = WS_POPUPWINDOW | WS_CAPTION;
    let mut rect = RECT::default();
    rect.right = 620;
    rect.bottom = 450;
    unsafe { AdjustWindowRect(addr_of_mut!(rect), dwstyle, false) };
    let desktop_wnd = unsafe { GetDesktopWindow() };
    let hdc = unsafe { GetDC(desktop_wnd) };
    let x = unsafe { GetDeviceCaps(hdc, HORZRES) };
    let y = unsafe { GetDeviceCaps(hdc, VERTRES) };
    unsafe { ReleaseDC(desktop_wnd, hdc) };
    let width = rect.right - rect.left + 1 as i32;
    let height = rect.bottom - rect.top + 1 as i32;
    conbuf::s_wcd_mut().window_width = width as _;
    conbuf::s_wcd_mut().window_height = height as _;
    let hwnd = unsafe {
        CreateWindowExA(
            WINDOW_EX_STYLE(0),
            class_name,
            s!("CoD Black Ops Console"),
            dwstyle,
            (x - 600) / 2,
            (y - 450) / 2,
            width,
            height,
            None,
            None,
            hinstance,
            None,
        )
    };

    conbuf::s_wcd_mut().window =
        Some(WindowHandle::from_win32(hwnd, Some(hinstance)));

    if hwnd.0 == 0 {
        return;
    }

    let hdc = unsafe { GetDC(hwnd) };
    let font_height = unsafe { MulDiv(8, GetDeviceCaps(hdc, LOGPIXELSY), 72) };
    conbuf::s_wcd_mut().buffer_font = Some(FontHandle(unsafe {
        CreateFontW(
            font_height,
            0,
            0,
            0,
            FW_LIGHT.0 as _,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as _,
            OUT_DEFAULT_PRECIS.0 as _,
            CLIP_DEFAULT_PRECIS.0 as _,
            DEFAULT_QUALITY.0 as _,
            DEFAULT_PITCH.0 as u32 + FF_MODERN.0 as u32,
            w!("Courier New"),
        )
        .0
    }));

    unsafe { ReleaseDC(hwnd, hdc) };
    if let Ok(image) = unsafe {
        LoadImageA(
            hinstance,
            s!("codlogo.bmp"),
            IMAGE_BITMAP,
            0,
            0,
            LR_LOADFROMFILE,
        )
    } {
        let cod_logo = unsafe {
            CreateWindowExA(
                WINDOW_EX_STYLE(0),
                s!("Static"),
                None,
                WS_CHILD | WS_VISIBLE | WINDOW_STYLE(0x0000000E),
                5,
                5,
                0,
                0,
                hwnd,
                HMENU(1),
                hinstance,
                None,
            )
        };

        conbuf::s_wcd_mut().cod_logo_window =
            Some(WindowHandle::from_win32(cod_logo, Some(hinstance)));

        unsafe {
            SendMessageA(cod_logo, STM_SETIMAGE, WPARAM(0), LPARAM(image.0))
        };
    }

    let hwnd_input_line = unsafe {
        CreateWindowExA(
            WINDOW_EX_STYLE(0),
            s!("edit"),
            None,
            WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | WINDOW_STYLE(ES_AUTOHSCROLL as _),
            6,
            400,
            608,
            20,
            hwnd,
            HMENU(101),
            hinstance,
            None,
        )
    };

    conbuf::s_wcd_mut().input_line_window =
        Some(WindowHandle::from_win32(hwnd_input_line, Some(hinstance)));

    let hwnd_buffer = unsafe {
        CreateWindowExA(
            WINDOW_EX_STYLE(0),
            s!("edit"),
            None,
            WS_CHILD
                | WS_VISIBLE
                | WS_BORDER
                | WS_VSCROLL
                | WINDOW_STYLE(
                    ES_READONLY as u32
                        | ES_AUTOVSCROLL as u32
                        | ES_MULTILINE as u32,
                ),
            6,
            70,
            606,
            324,
            hwnd,
            HMENU(100),
            hinstance,
            None,
        )
    };

    conbuf::s_wcd_mut().buffer_window =
        Some(WindowHandle::from_win32(hwnd_buffer, Some(hinstance)));

    unsafe {
        SendMessageA(
            hwnd_buffer,
            WM_SETFONT,
            WPARAM(conbuf::s_wcd().buffer_font.unwrap().0 as _),
            LPARAM(0),
        )
    };
    conbuf::s_wcd_mut().sys_input_line_wnd_proc = unsafe {
        transmute(SetWindowLongPtrA(
            hwnd_input_line,
            GWLP_WNDPROC,
            input_line_wnd_proc as _,
        ) as *const ())
    };
    unsafe {
        SendMessageA(
            hwnd_input_line,
            WM_SETFONT,
            WPARAM(conbuf::s_wcd().buffer_font.unwrap().0 as _),
            LPARAM(0),
        )
    };
    unsafe { SetFocus(hwnd_input_line) };
    unsafe {
        SetWindowTextA(
            hwnd_buffer,
            PCSTR(conbuf::clean_text(&console::get_text_copy(0x4000)).as_ptr()),
        )
    };
}

#[cfg(not(windows))]
pub fn create_console() {
    unimplemented!()
}

#[cfg(windows)]
pub fn show_console() {
    if conbuf::s_wcd().window.is_none() {
        create_console();
        assert!(!conbuf::s_wcd().window.is_none());
    }

    show_window(conbuf::s_wcd().buffer_window.unwrap());
    unsafe {
        SendMessageA(
            HWND(
                conbuf::s_wcd()
                    .buffer_window
                    .unwrap()
                    .get_win32()
                    .unwrap()
                    .hwnd as _,
            ),
            EM_LINESCROLL,
            WPARAM(0),
            LPARAM(0xFFFF),
        )
    };
}

#[cfg(windows)]
pub fn destroy_console() {
    if conbuf::s_wcd().window.is_some() {
        let hwnd = HWND(
            conbuf::s_wcd().window.unwrap().get_win32().unwrap().hwnd as _,
        );
        unsafe { ShowWindow(hwnd, SW_HIDE) };
        unsafe { CloseWindow(hwnd) };
        unsafe { DestroyWindow(hwnd) };
        conbuf::s_wcd_mut().window = None;
    }
}

#[cfg(windows)]
fn set_error_text(error: &str) {
    conbuf::s_wcd_mut().error_string = error.into();
    destroy_window(conbuf::s_wcd().input_line_window.unwrap());
    conbuf::s_wcd_mut().input_line_window = None;

    message_box(
        None,
        "Error",
        error,
        MessageBoxType::Ok,
        Some(MessageBoxIcon::Stop),
    )
    .unwrap();
}

#[cfg(not(windows))]
pub fn show_console() {
    todo!()
}

#[cfg(not(windows))]
pub fn destroy_console() {
    todo!()
}

#[cfg(not(windows))]
#[allow(clippy::missing_const_for_fn)]
pub fn set_error_text(_error: &str) {
    todo!()
}

pub fn error(error: &str) -> ! {
    com::ERROR_ENTERED.store(true, Ordering::Relaxed);
    // Sys_SuspendOtherThreads()

    // FixWindowsDesktop() (probably shouldn't be necessary)

    // if Sys_IsMainThread() (no clue how necessary this check is,
    // probably have to do some restructuring)
    show_console();
    conbuf::append_text(&format!("\n\n{}\n", error));
    set_error_text(error);
    // Finish processing events
    // DoSetEvent_UNK();
    std::process::exit(0);
}

pub const fn default_cd_path() -> &'static str {
    ""
}

pub fn cwd() -> PathBuf {
    std::env::current_dir().unwrap()
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum KeyboardScancode {
    Esc,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    PrtScSysRq,
    ScrLk,
    PauseBreak,

    Tilde,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,
    Hyphen,
    Equals,
    Backspace,
    Insert,
    Home,
    PgUp,
    NumLk,
    NumSlash,
    NumAsterisk,
    NumHyphen,

    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    OpenBracket,
    CloseBracket,
    BackSlash,
    Del,
    End,
    PgDn,
    Num7,
    Num8,
    Num9,
    NumPlus,

    CapsLk,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Semicolon,
    Apostrophe,
    Enter,
    Num4,
    Num5,
    Num6,

    LShift,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,
    Period,
    ForwardSlash,
    RShift,
    ArrowUp,
    Num1,
    Num2,
    Num3,
    NumEnter,

    LCtrl,
    LSys,
    LAlt,
    Space,
    RAlt,
    RSys,
    Fn,
    Menu,
    RCtrl,
    ArrowLeft,
    ArrowDown,
    ArrowRight,
    Num0,
    NumPeriod,
}

bitflags! {
    #[non_exhaustive]
    pub struct Modifiers: u16 {
        const LCTRL = 0x0001;
        const LSYS = 0x0002;
        const LALT = 0x0004;
        const LSHIFT = 0x0008;
        const RSHIFT = 0x0010;
        const RALT = 0x0020;
        const RSYS = 0x0040;
        const RCTRL = 0x0080;
        const CAPSLOCK = 0x0100;
        const NUMLOCK = 0x0200;
        const SCRLOCK = 0x0400;
    }
}

impl TryFrom<Modifiers> for KeyboardScancode {
    type Error = ();
    fn try_from(value: Modifiers) -> Result<Self, Self::Error> {
        match value {
            Modifiers::CAPSLOCK => Ok(Self::CapsLk),
            Modifiers::LALT => Ok(Self::LAlt),
            Modifiers::LCTRL => Ok(Self::LCtrl),
            Modifiers::LSHIFT => Ok(Self::LShift),
            Modifiers::LSYS => Ok(Self::LSys),
            Modifiers::NUMLOCK => Ok(Self::NumLk),
            Modifiers::RALT => Ok(Self::RAlt),
            Modifiers::RCTRL => Ok(Self::RCtrl),
            Modifiers::RSHIFT => Ok(Self::RShift),
            Modifiers::RSYS => Ok(Self::RSys),
            Modifiers::SCRLOCK => Ok(Self::ScrLk),
            _ => Err(()),
        }
    }
}

impl TryInto<Modifiers> for KeyboardScancode {
    type Error = ();
    fn try_into(self) -> Result<Modifiers, Self::Error> {
        match self {
            Self::CapsLk => Ok(Modifiers::CAPSLOCK),
            Self::LAlt => Ok(Modifiers::LALT),
            Self::LCtrl => Ok(Modifiers::LCTRL),
            Self::LShift => Ok(Modifiers::LSHIFT),
            Self::LSys => Ok(Modifiers::LSYS),
            Self::NumLk => Ok(Modifiers::NUMLOCK),
            Self::RAlt => Ok(Modifiers::RALT),
            Self::RCtrl => Ok(Modifiers::RCTRL),
            Self::RShift => Ok(Modifiers::RSHIFT),
            Self::RSys => Ok(Modifiers::RSYS),
            Self::ScrLk => Ok(Modifiers::SCRLOCK),
            _ => Err(()),
        }
    }
}

impl Modifiers {
    pub fn each(self) -> Vec<Modifiers> {
        let mut v = vec![];

        if self.contains(Self::CAPSLOCK) {
            v.push(Self::CAPSLOCK)
        }
        if self.contains(Self::LALT) {
            v.push(Self::LALT)
        }
        if self.contains(Self::LCTRL) {
            v.push(Self::LCTRL)
        }
        if self.contains(Self::LSYS) {
            v.push(Self::LSYS)
        }
        if self.contains(Self::NUMLOCK) {
            v.push(Self::NUMLOCK)
        }
        if self.contains(Self::RALT) {
            v.push(Self::RALT)
        }
        if self.contains(Self::RCTRL) {
            v.push(Self::RCTRL)
        }
        if self.contains(Self::RSHIFT) {
            v.push(Self::RSHIFT)
        }
        if self.contains(Self::RSYS) {
            v.push(Self::RSYS)
        }
        if self.contains(Self::SCRLOCK) {
            v.push(Self::SCRLOCK)
        }

        v
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MouseScancode {
    LClick,
    RClick,
    MClick,
    Button4,
    Button5,
    ButtonN(u8),
}

impl KeyboardScancode {
    pub const fn affected_by_num_lock(self) -> bool {
        matches!(
            self,
            Self::Num0
                | Self::Num1
                | Self::Num2
                | Self::Num3
                | Self::Num4
                | Self::Num5
                | Self::Num6
                | Self::Num7
                | Self::Num8
                | Self::Num9
                | Self::NumPeriod,
        )
    }
}

bitflags! {
    #[non_exhaustive]
    pub struct MouseButtons: u8 {
        const LCLICK = 0x01;
        const RCLICK = 0x02;
        const MCLICK = 0x04;
        const BUTTON_4 = 0x08;
        const BUTTON_5 = 0x10;
    }
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum WindowEvent {
    Created(WindowHandle),
    Destroyed,
    Moved {
        x: u32,
        y: u32,
    },
    Resized {
        width: u32,
        height: u32,
    },
    Activate,
    Deactivate,
    SetFocus,
    KillFocus,
    CloseRequested,
    DisplayChange {
        bits_per_pixel: u32,
        horz_res: u32,
        vert_res: u32,
    },
    #[non_exhaustive]
    KeyDown {
        logical_scancode: KeyboardScancode,
        physical_scancode: Option<KeyboardScancode>,
    },
    #[non_exhaustive]
    KeyUp {
        logical_scancode: KeyboardScancode,
        physical_scancode: Option<KeyboardScancode>,
    },
    Character(char),
    CursorMoved {
        x: f64,
        y: f64,
    },
    MouseButtonDown(MouseScancode),
    MouseButtonUp(MouseScancode),
    MouseWheelScroll(f32),
    ModifiersChanged {
        modifiers: Modifiers,
    },
}

lazy_static! {
    pub static ref MAIN_WINDOW_EVENTS: Mutex<VecDeque<WindowEvent>> =
        Mutex::new(VecDeque::new());
}

#[cfg(windows)]
#[allow(clippy::undocumented_unsafe_blocks, clippy::cast_possible_wrap)]
pub fn next_main_window_event() -> Option<WindowEvent> {
    if query_quit_event() == SignalState::Signaled {
        com::quit_f();
    }

    if MAIN_WINDOW_EVENTS.lock().unwrap().is_empty() {
        let mut msg = MSG::default();

        if unsafe { PeekMessageA(addr_of_mut!(msg), None, 0, 0, PM_NOREMOVE) }
            .as_bool()
        {
            if unsafe { GetMessageA(addr_of_mut!(msg), None, 0, 0) }.0 == 0 {
                set_quit_event();
            }
            platform::set_msg_time(msg.time as _);
            unsafe {
                TranslateMessage(addr_of!(msg));
            }
            unsafe {
                DispatchMessageA(addr_of!(msg));
            }
        }
        None
    } else {
        MAIN_WINDOW_EVENTS.lock().unwrap().pop_front()
    }
}

#[cfg(wayland)]
pub fn next_main_window_event() -> Option<WindowEvent> {
    None
}

#[cfg(appkit)]
pub fn next_main_window_event() -> Option<WindowEvent> {
    if query_quit_event() == SignalState::Signaled {
        com::quit_f();
    }

    if MAIN_WINDOW_EVENTS.lock().unwrap().is_empty() {
        let ns_app = unsafe { NSApp }.unwrap();
        if let Some(ev) = unsafe {
            ns_app.nextEventMatchingMask_untilDate_inMode_dequeue(
                NSUIntegerMax as _,
                Some(&NSDate::distantPast()),
                NSDefaultRunLoopMode,
                true,
            )
        } {
            WindowEvent::try_from(ev.as_ref()).ok()
        } else {
            None
        }
    } else {
        MAIN_WINDOW_EVENTS.lock().unwrap().pop_front()
    }
}

#[cfg(xlib)]
lazy_static! {
    static ref XLIB_CONTEXT: RwLock<XlibContext> =
        RwLock::new(XlibContext::default());
}

// All uses of unsafe in the following function are either for FFI
// or for accessing the members of the XEvent union. All of the
// functions should be safe as called, and all of the union accesses
// should be safe since XEvent is a tagged union thanks to its
// `type_` member. No reason to comment them individually.
#[cfg(xlib)]
#[allow(
    clippy::undocumented_unsafe_blocks,
    clippy::cast_sign_loss,
    clippy::get_first,
    clippy::if_then_some_else_none,
    clippy::single_match_else,
    clippy::cast_possible_wrap
)]
pub fn next_main_window_event() -> Option<WindowEvent> {
    if query_quit_event() == SignalState::Signaled {
        com::quit_f();
    }

    if MAIN_WINDOW_EVENTS.lock().unwrap().is_empty() {
        let mut ev =
            unsafe { core::mem::MaybeUninit::<XEvent>::zeroed().assume_init() };
        let display = unsafe {
            XOpenDisplay(platform::display_server::xlib::display_name())
        };
        if unsafe { XPending(display) } == 0 {
            return None;
        }

        unsafe {
            XNextEvent(display, addr_of_mut!(ev));
        }
        unsafe {
            XCloseDisplay(display);
        }

        // Since XEvents don't have a timestamp associated with them
        // like Windows MSGs do, we do the next best thing and acquire
        // a timestamp immediately after retrieving the event.
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as isize;
        let any = unsafe { ev.any };
        match any.type_ {
            ClientMessage => {
                let ev = unsafe { ev.client_message };
                if *ev.data.as_longs().get(0).unwrap() as u64
                    == WM_DELETE_WINDOW.load_relaxed()
                {
                    unsafe {
                        XDestroyWindow(ev.display, ev.window);
                    }
                    platform::set_msg_time(time);
                    Some(WindowEvent::CloseRequested)
                } else {
                    None
                }
            }
            _ => {
                let context = *XLIB_CONTEXT.read().unwrap();
                if let Ok((mut evs, new_context)) =
                    WindowEvent::try_from_xevent(ev, context)
                {
                    if let Some(n) = new_context {
                        *XLIB_CONTEXT.write().unwrap() = n;
                    }

                    platform::set_msg_time(time);
                    let ev = evs.pop_front();
                    MAIN_WINDOW_EVENTS.lock().unwrap().append(&mut evs);

                    ev
                } else {
                    None
                }
            }
        }
    } else {
        MAIN_WINDOW_EVENTS.lock().unwrap().pop_front()
    }
}

#[cfg(windows)]
pub fn show_window(handle: WindowHandle) {
    #[allow(clippy::undocumented_unsafe_blocks)]
    unsafe {
        ShowWindow(HWND(handle.get_win32().unwrap().hwnd as _), SW_SHOW);
    }
}

#[cfg(windows)]
pub fn focus_window(handle: WindowHandle) {
    #[allow(clippy::undocumented_unsafe_blocks)]
    unsafe {
        SetFocus(HWND(handle.get_win32().unwrap().hwnd as _));
    }
}

#[cfg(windows)]
pub fn destroy_window(handle: WindowHandle) {
    #[allow(clippy::undocumented_unsafe_blocks)]
    unsafe {
        DestroyWindow(HWND(handle.get_win32().unwrap().hwnd as _));
    }
}

#[cfg(wayland)]
pub fn show_window(handle: WindowHandle) {
    let handle = handle.get_wayland().unwrap();
    todo!()
}

#[cfg(wayland)]
pub fn focus_window(handle: WindowHandle) {
    let handle = handle.get_wayland().unwrap();
    todo!()
}

#[cfg(wayland)]
pub fn destroy_window(handle: WindowHandle) {
    let handle = handle.get_wayland().unwrap();
    todo!()
}

#[cfg(appkit)]
pub fn show_window(handle: WindowHandle) {
    unsafe {
        handle
            .get_appkit()
            .unwrap()
            .ns_window()
            .orderFrontRegardless()
    };
}

#[cfg(appkit)]
pub fn focus_window(handle: WindowHandle) {
    unsafe { handle.get_appkit().unwrap().ns_window().makeKeyWindow() };
}

#[cfg(appkit)]
pub fn destroy_window(handle: WindowHandle) {
    unsafe { handle.get_appkit().unwrap().ns_window().close() };
}

#[cfg(xlib)]
#[allow(clippy::undocumented_unsafe_blocks)]
pub fn show_window(handle: WindowHandle) {
    let handle = handle.get_xlib().unwrap();
    let display =
        unsafe { XOpenDisplay(platform::display_server::xlib::display_name()) };
    unsafe {
        XMapWindow(display, handle.window);
    }
}

#[cfg(xlib)]
#[allow(clippy::undocumented_unsafe_blocks)]
pub fn focus_window(handle: WindowHandle) {
    let handle = handle.get_xlib().unwrap();
    let display =
        unsafe { XOpenDisplay(platform::display_server::xlib::display_name()) };
    unsafe {
        XSetInputFocus(display, handle.window, RevertToParent, CurrentTime);
    }
}

#[cfg(xlib)]
pub fn destroy_window(handle: WindowHandle) {
    let handle = handle.get_xlib().unwrap();
    let display =
        unsafe { XOpenDisplay(platform::display_server::xlib::display_name()) };
    unsafe {
        XDestroyWindow(display, handle.window);
    }
}

static MODIFIERS: RwLock<Modifiers> = RwLock::new(Modifiers::empty());

pub fn handle_main_window_event(ev: WindowEvent) {
    match ev {
        WindowEvent::Created(handle) => {
            platform::set_window_handle(handle);
            if dvar::get_bool("r_reflectionProbeGenerate").unwrap()
                && dvar::get_bool("r_fullscreen").unwrap()
            {
                dvar::set_bool("r_fullscreen", false).unwrap();
                cbuf::add_textln(0, "vid_restart");
            }
            dvar::register_bool(
                "r_autopriority",
                false,
                dvar::DvarFlags::ARCHIVE,
                Some(
                    "Automatically set the priority of the windows process \
                     when the game is minimized",
                ),
            )
            .unwrap();
        }
        WindowEvent::CloseRequested => {
            cbuf::add_textln(0, "quit");
            sys::set_quit_event();
        }
        WindowEvent::Destroyed => {
            // FUN_004dfd60()
            platform::clear_window_handle();
        }
        WindowEvent::Moved { x, y } => {
            if dvar::get_bool("r_fullscreen").unwrap() {
                input::mouse::activate(0);
            } else {
                dvar::set_int_internal("vid_xpos", x as _).unwrap();
                dvar::set_int_internal("vid_ypos", y as _).unwrap();
                dvar::clear_modified("vid_xpos").unwrap();
                dvar::clear_modified("vid_ypos").unwrap();
                if platform::get_platform_vars().active_app {
                    input::activate(true);
                }
            }
        }
        WindowEvent::ModifiersChanged { modifiers } => {
            let diff = *MODIFIERS.read().unwrap() ^ modifiers;

            if diff.is_empty() {
                return;
            }

            for m in diff.each() {
                sys::enqueue_event(sys::Event::new(
                    Some(platform::get_msg_time() as _),
                    // diff will have all the modifiers that changed set
                    // however, to detect if they were pressed or released
                    // we have to check if the new modifiers, not diff,
                    // contains them
                    sys::EventType::Key(
                        m.try_into().unwrap(),
                        modifiers.contains(m),
                    ),
                ));
            }

            *MODIFIERS.write().unwrap() = modifiers;
        }
        WindowEvent::KeyDown {
            logical_scancode, ..
        } => {
            if logical_scancode == KeyboardScancode::Enter
                && MODIFIERS.read().unwrap().contains(Modifiers::LALT)
            {
                if cl::get_local_client_connection_state(0)
                    == Connstate::LOADING
                {
                    return;
                }

                if dvar::get_int("developer").unwrap() != 0 {
                    // FUN_005a5360()
                    dvar::set_bool(
                        "r_fullscreen",
                        dvar::get_bool("r_fullscreen").unwrap() == false,
                    )
                    .unwrap();
                    cbuf::add_textln(0, "vid_restart");
                }
            }
            sys::enqueue_event(sys::Event::new(
                Some(platform::get_msg_time() as _),
                sys::EventType::Key(logical_scancode, true),
            ));
        }
        WindowEvent::KeyUp {
            logical_scancode, ..
        } => {
            sys::enqueue_event(sys::Event::new(
                Some(platform::get_msg_time() as _),
                sys::EventType::Key(logical_scancode, false),
            ));
        }
        _ => {}
    }
}

static THREAD_ID: RwLock<[Option<ThreadId>; 15]> = RwLock::new([None; 15]);

struct ThreadContext(usize);

impl Display for ThreadContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                0 => "Main",
                1 => "Backend",
                2 => "Worker0",
                3 => "Worker1",
                4 => "Worker2",
                5 => "Worker3",
                6 => "Worker4",
                7 => "Worker5",
                8 => "Worker6",
                9 => "Worker7",
                10 => "Server",
                11 => "occlusion",
                12 => "TitleServer",
                13 => "Database",
                14 => "Stream",
                _ => unreachable!(),
            }
        )
    }
}

fn get_current_thread_id() -> ThreadId {
    std::thread::current().id()
}

fn get_thread_context() -> ThreadContext {
    let tid = get_current_thread_id();
    let tids = THREAD_ID.read().unwrap();
    for (i, t) in tids.iter().enumerate() {
        if let Some(id) = *t && id == tid {
            return ThreadContext(i);
        }
    }

    com::println!(1.into(), "Current thread is not in thread table");
    panic!()
}

pub fn get_current_thread_name() -> String {
    get_thread_context().to_string()
}

pub fn init_main_thread() {
    *THREAD_ID.write().unwrap().get_mut(0).unwrap() =
        Some(get_current_thread_id());
}

pub fn is_main_thread() -> bool {
    Some(get_current_thread_id()) == *THREAD_ID.read().unwrap().get(0).unwrap()
}

pub fn is_render_thread() -> bool {
    Some(get_current_thread_id()) == *THREAD_ID.read().unwrap().get(1).unwrap()
}
pub fn is_server_thread() -> bool {
    Some(get_current_thread_id()) == *THREAD_ID.read().unwrap().get(10).unwrap()
}

pub fn is_database_thread() -> bool {
    Some(get_current_thread_id()) == *THREAD_ID.read().unwrap().get(13).unwrap()
}

pub fn is_stream_thread() -> bool {
    Some(get_current_thread_id()) == *THREAD_ID.read().unwrap().get(14).unwrap()
}

pub fn notify_renderer() {
    set_backend_event();
}

pub fn quit() -> ! {
    normal_exit();
    std::process::exit(0);
}

pub fn wait_renderer() {
    // TODO - TLS shit, maybe?
    if !is_main_thread()
        || query_render_device_ok_event() == SignalState::Signaled
    {
        return;
    }

    while query_render_completed_event() == SignalState::Cleared {
        if com::ERROR_ENTERED.load_relaxed() == true {
            std::thread::sleep(Duration::from_millis(100));
            return;
        } else if query_render_device_ok_event() == SignalState::Cleared {
            return;
        }
    }
}
