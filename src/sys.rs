#![allow(dead_code)]

use crate::platform::WindowHandle;
use crate::util::SmpEvent;
use crate::*;
use num_derive::FromPrimitive;
use sysinfo::{CpuExt, SystemExt};

pub mod gpu;

use cfg_if::cfg_if;
use lazy_static::lazy_static;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{Read, Write};
use std::sync::RwLock;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{
    fmt::Display,
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicIsize, Ordering::SeqCst},
    time::SystemTime,
};
cfg_if! {
    if #[cfg(target_os = "windows")] {
        use std::ffi::{CStr, CString};
        use std::fs::OpenOptions;
        use std::os::windows::prelude::*;
        use windows::Win32::Foundation::MAX_PATH;
        use windows::Win32::System::LibraryLoader::GetModuleFileNameA;
        use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_HIDDEN;
        use windows::core::PCSTR;
        use windows::Win32::Foundation::HWND;
        use windows::Win32::UI::WindowsAndMessaging::{
            MessageBoxA, IDCANCEL, IDNO, IDOK, IDYES,
            MB_ICONINFORMATION, MB_ICONSTOP, MB_OK, MB_YESNO, MB_YESNOCANCEL,
            MESSAGEBOX_STYLE,
        };
        use windows::Win32::System::Diagnostics::Debug::OutputDebugStringA;
    } else if #[cfg(target_os = "linux")] {
        use gtk4::prelude::*;
        use gtk4::builders::MessageDialogBuilder;
        use std::cell::RefCell;
        use std::ffi::OsStr;
    }
}

fn in_restart_f() {
    input::shutdown();
    input::init();
}

fn net_restart_f() {
    net::restart();
}

fn movie_start_f() {
    todo!()
}

fn movie_stop_f() {
    todo!()
}

fn listen_f() {
    todo!()
}

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
    
    com::println(16.into(), &format!("CPU vendor is \"{}\"", get_cpu_vendor()));
    com::println(16.into(), &format!("CPU name is \"{}\"", get_cpu_name()));

    let info = find_info();

    let c = if info.logical_cpu_count == 1 {
        ""
    } else {
        "s"
    };
    com::println(16.into(), &format!("{} logical CPU{} reported", info.logical_cpu_count, c));

    let c = if info.physical_cpu_count == 1 {
        ""
    } else {
        "s"
    };
    com::println(16.into(), &format!("{} physical CPU{} detected", info.physical_cpu_count, c));
    com::println(16.into(), &format!("Measured CPU speed is {:.2} GHz", info.cpu_ghz));
    com::println(16.into(), &format!("Total CPU performance is estimated as {:.2} GHz", info.configure_ghz));
    com::println(16.into(), &format!("System memory is {} MB (capped at 1 GB)", info.sys_mb));
    com::println(16.into(), &format!("Video card is \"{}\"", info.gpu_description));
    // TODO - vector support
    com::println(16.into(), "");
    input::init();
}

lazy_static! {
    static ref BASE_TIME_ACQUIRED: AtomicBool = AtomicBool::new(false);
    pub static ref TIME_BASE: AtomicIsize = AtomicIsize::new(0);
}

pub fn milliseconds() -> isize {
    if BASE_TIME_ACQUIRED.load(SeqCst) == false {
        let now = SystemTime::now();
        let time = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        TIME_BASE.store(time.try_into().unwrap(), SeqCst);
    }

    let time: isize = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap();
    time - TIME_BASE.load(SeqCst)
}

cfg_if! {
    if #[cfg(target_os = "windows")] {
        pub fn get_executable_name() -> String {
            let mut buf: [u8; MAX_PATH as usize] = [0; MAX_PATH as usize];
            unsafe { GetModuleFileNameA(None, &mut buf) };
            let c_string = CStr::from_bytes_until_nul(&buf).unwrap();
            let s = c_string.to_str()
                .unwrap()
                .to_string();
            let p = PathBuf::from(s);
            let s = p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            match s.strip_suffix(".exe") {
                Some(s) => s.to_string(),
                None => s
            }
        }
    } else if #[cfg(target_os = "linux")] {
        pub fn get_executable_name() -> String {
            let pid = std::process::id();
            let proc_path = format!("/proc/{}/exe", pid);
            match std::fs::read_link(proc_path) {
                Ok(f) => {
                    let file_name = f.file_name()
                        .unwrap_or_else(|| OsStr::new(""))
                        .to_str()
                        .unwrap_or("")
                        .to_string();
                    let pos = file_name.find('.')
                        .unwrap_or(file_name.len());
                    file_name[..pos].to_string()
                },
                Err(_) => String::new()
            }
        }
    } else if #[cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    ))] {
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
                Ok(f) => f.file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                Err(_) => {
                    let pid = libc::getpid();
                    let kinfo_proc = unsafe { libc::kinfo_getproc(pid) };
                    if kinfo_proc.is_null() {
                        return String::new();
                    }

                    let s = CString::new((*kinfo_proc).ki_comm)
                        .unwrap_or(CString::new("")
                        .unwrap())
                        .to_str()
                        .unwrap_or("")
                        .to_string();
                    unsafe { libc::free(kinfo_proc) };
                    s
                }
            }
        }
    }
    else if #[cfg(target_os = "macos")] {
        pub fn get_executable_name() -> String {
            let buf: [u8; libc::PROC_PIDPATHINFO_MAXSIZE as _] =
                [0; libc::PROC_PIDPATHINFO_MAXSIZE as _];
            let pid = std::process::pid();
            unsafe {
                libc::proc_pidpath(
                    pid,
                    addr_of_mut!(buf) as *mut _,
                    buf.len()
                )
            };
            CString::from_vec_with_nul(buf.to_vec())
                .unwrap_or(
                    CString::new("")
                        .unwrap()
                )
                .to_str()
                .unwrap_or("")
                .to_string();
        }
    }
    // Fallback method - if no platform-specific method is used, try to get the executable name from argv[0]
    else {
        pub fn get_executable_name() -> String {
            let argv_0 = std::env::args().collect::<Vec<String>>()[0];
            let path = PathBuf::from(argv_0);
            let file_name = path.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            let pos = file_name.find('.')
                .unwrap_or(file_name.len());
            file_name[..pos].to_string()
        }
    }
}

fn get_application_name() -> &'static str {
    "Call of Duty(R) Singleplayer - Ship"
}

pub fn get_semaphore_file_path() -> Option<PathBuf> {
    let os_folder_path = fs::get_os_folder_path(fs::OsFolder::UserData)?;
    let p: PathBuf = [
        PathBuf::from(os_folder_path),
        PathBuf::from("CoD").join("Activision"),
    ]
    .iter()
    .collect();
    Some(p)
}

cfg_if! {
    if #[cfg(target_os = "windows")] {
        pub fn get_semaphore_file_name() -> String {
            format!("__{}", get_executable_name())
        }
    } else if #[cfg(target_family = "unix")] {
        pub fn get_semaphore_file_name() -> String {
            format!(".__{}", get_executable_name())
        }
    } else {
        pub fn get_semaphore_file_name() -> String {
            println!("sys::get_semaphore_file: using default implementation.");
            format!("__{}", get_executable_name())
        }
    }
}

pub fn no_free_files_error() -> ! {
    let msg_box_type = MessageBoxType::Ok;
    let msg_box_icon = MessageBoxIcon::Stop;
    let title = locale::localize_ref("WIN_DISK_FULL_TITLE");
    let text = locale::localize_ref("WIN_DISK_FULL_BODY");
    let handle = render::main_window_handle();
    message_box(handle, &title, &text, msg_box_type, Some(msg_box_icon));
    // DoSetEvent_UNK();
    std::process::exit(-1);
}

// TODO - implement
fn is_game_process(_pid: u32) -> bool {
    true
}

pub fn check_crash_or_rerun() -> bool {
    let semaphore_folder_path = match get_semaphore_file_path() {
        Some(s) => s,
        None => return true,
    };

    if !std::path::Path::new(&semaphore_folder_path).exists() {
        return std::fs::create_dir_all(&semaphore_folder_path).is_ok();
    }

    let semaphore_file_path =
        semaphore_folder_path.join(get_semaphore_file_name());
    let semaphore_file_exists = semaphore_file_path.exists();

    if semaphore_file_exists {
        if let Ok(mut f) = File::open(semaphore_file_path.clone()) {
            let mut buf = [0u8; 4];
            if let Ok(4) = f.read(&mut buf) {
                let pid_read = u32::from_ne_bytes(buf);
                if pid_read != std::process::id()
                    || is_game_process(pid_read) == false
                {
                    return true;
                }

                let msg_box_type = MessageBoxType::YesNoCanel;
                let msg_box_icon = MessageBoxIcon::Stop;
                let title = locale::localize_ref("WIN_IMPROPER_QUIT_TITLE");
                let text = locale::localize_ref("WIN_IMPROPER_QUIT_BODY");
                let handle = render::main_window_handle();
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
        if #[cfg(target_os = "windows")] {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .attributes(FILE_ATTRIBUTE_HIDDEN.0)
                .open(semaphore_file_path);
        } else {
            let file = File::create(semaphore_file_path);
        }
    }

    match file {
        Ok(mut f) => {
            let pid = std::process::id();
            if f.write_all(&pid.to_ne_bytes()).is_err() {
                no_free_files_error();
            } else {
                true
            }
        }
        Err(_) => {
            no_free_files_error();
        }
    }
}

pub fn get_cmdline() -> String {
    let mut cmd_line: String = String::new();
    std::env::args().for_each(|arg| {
        cmd_line.push_str(&arg);
    });
    cmd_line.trim().to_string()
}

pub fn start_minidump(b: bool) {
    com::println(0.into(), &format!("Starting minidump with b = {}...", b));
    com::println(0.into(), "TODO: implement.");
}

// Abstracted out in case a certain platform needs an implementation using
// something other than the num_cpus crate
pub fn get_logical_cpu_count() -> usize {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    system.cpus().len()
}

pub fn get_physical_cpu_count() -> usize {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    match system.physical_core_count() {
        Some(u) => u,
        None => get_logical_cpu_count(),
    }
}

pub fn get_system_ram_in_bytes() -> u64 {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    system.total_memory()
}

pub fn get_cpu_vendor() -> String {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    system.global_cpu_info().vendor_id().to_string()
}

pub fn get_cpu_name() -> String {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    system
        .global_cpu_info()
        .brand()
        .to_string()
        .trim()
        .to_string()
}

pub fn detect_video_card() -> String {
    let adapter =
        pollster::block_on(gpu::Adapter::new(&gpu::Instance::new(), None));
    adapter.get_info().name
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
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
            "GPU Description: {}\nCPU: {} ({})\nCores: {} ({} physical)\nSystem RAM: {}MiB",
            self.gpu_description, self.cpu_name, self.cpu_vendor,
            self.logical_cpu_count, self.physical_cpu_count, self.sys_mb)
    }
}

impl SysInfo {
    fn new() -> Self {
        SysInfo {
            ..Default::default()
        }
    }

    fn find(&mut self) -> &mut Self {
        self.gpu_description = detect_video_card();
        self.logical_cpu_count = get_logical_cpu_count();
        self.physical_cpu_count = get_physical_cpu_count();
        self.sys_mb = get_system_ram_in_bytes() / (1024 * 1024);
        self.cpu_vendor = get_cpu_vendor();
        self.cpu_name = get_cpu_name();
        self
    }
}

lazy_static! {
    static ref SYS_INFO: Arc<RwLock<Option<SysInfo>>> =
        Arc::new(RwLock::new(None));
}

pub fn find_info() -> SysInfo {
    let lock = SYS_INFO.clone();
    let mut sys_info = lock.write().unwrap();
    if sys_info.is_none() {
        *sys_info = Some(SysInfo::new().find().clone());
    }
    sys_info.as_ref().unwrap().clone()
}

pub enum EventType {
    None,
    Key(input::keyboard::KeyScancode, bool),
    Mouse(input::mouse::Scancode, bool),
    Console,
}

pub struct Event {
    time: isize,
    event_type: EventType,
    data: Vec<u8>,
}

impl Event {
    pub fn new(
        time: Option<isize>,
        event_type: EventType,
        data: Option<Vec<u8>>,
    ) -> Self {
        Event {
            time: time.unwrap_or_default(),
            event_type,
            data: data.unwrap_or_default(),
        }
    }
}

lazy_static! {
    static ref EVENT_QUEUE: Arc<RwLock<VecDeque<Event>>> =
        Arc::new(RwLock::new(VecDeque::new()));
}

pub fn enqueue_event(mut event: Event) {
    if event.time == 0 {
        event.time = milliseconds();
    }

    let lock = EVENT_QUEUE.clone();
    let mut writer = lock.try_write().expect("");
    writer.push_back(event);
}

pub fn render_fatal_error() -> ! {
    let msg_box_type = MessageBoxType::Ok;
    let msg_box_icon = MessageBoxIcon::Stop;
    let title = locale::localize_ref("WIN_RENDER_INIT_TITLE");
    let text = locale::localize_ref("WIN_RENDER_INIT_BODY");
    let handle = render::main_window_handle();
    message_box(handle, &title, &text, msg_box_type, Some(msg_box_icon));
    //DoSetEvent_UNK();
    std::process::exit(-1);
}

lazy_static! {
    static ref EVENTS: Arc<RwLock<HashMap<String, SmpEvent<()>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

pub fn create_event(manual_reset: bool, initial_state: bool, name: &str) {
    EVENTS.clone().try_write().expect("").insert(
        name.to_owned(),
        SmpEvent::new((), initial_state, manual_reset),
    );
    if initial_state {
        EVENTS
            .clone()
            .try_write()
            .expect("")
            .get_mut(&name.to_string())
            .unwrap()
            .send(())
            .unwrap();
    }
}

fn wait_for_event_timeout(name: &str, timeout: usize) -> bool {
    let lock = EVENTS.clone();
    let mut writer = lock.write().unwrap();
    let event = writer.get_mut(&name.to_string());
    match event {
        Some(e) => {
            e.acknowledge_timeout(Duration::from_millis(timeout as _));
            e.signaled().unwrap_or(false)
        }
        None => panic!("sys::wait_for_event_timeout: event not found."),
    }
}

pub fn query_event(name: &str) -> bool {
    wait_for_event_timeout(name, 0)
}

pub fn wait_event(name: &str, msec: usize) -> bool {
    wait_for_event_timeout(name, msec)
}

pub fn create_thread<T, F: Fn() -> T + Send + Sync + 'static>(
    name: &str,
    function: F,
) -> Option<JoinHandle<()>> {
    match std::thread::Builder::new()
        .name(name.to_string())
        .spawn(move || {
            std::thread::park();
            function();
        }) {
        Ok(h) => Some(h),
        Err(e) => {
            com::println(
                1.into(),
                &format!("error {} while creating thread {}", e, name),
            );
            None
        }
    }
}

pub fn spawn_render_thread<F: Fn() -> ! + Send + Sync + 'static>(
    function: F,
) -> bool {
    create_event(false, false, "renderPausedEvent");
    create_event(true, true, "renderCompletedEvent");
    create_event(true, false, "resourcesFlushedEvent");
    create_event(true, false, "resourcesQueuedEvent");
    create_event(true, true, "rendererRunningEvent");
    create_event(true, false, "backendEvent");
    create_event(false, false, "backendEvent1");
    create_event(true, true, "updateSpotLightEffectEvent");
    create_event(true, true, "updateEffectsEvent");
    create_event(true, true, "deviceOKEvent");
    create_event(true, false, "deviceHardStartEvent");
    create_event(true, false, "renderShutdownEvent");
    create_event(true, true, "deviceMessageEvent");
    create_event(true, false, "osQuitEvent");
    create_event(true, false, "osScriptDebuggerDrawEvent");
    create_event(true, false, "rgRegisteredEvent");
    create_event(true, false, "renderEvent");
    match create_thread("Backend", function) {
        Some(h) => {
            h.thread().unpark();
            true
        }
        None => false,
    }
}

/*
const MAX_CPUS: usize = 32;

lazy_static! {
    static ref S_CPU_COUNT: AtomicUsize = AtomicUsize::new(0);
    static ref S_AFFINITY_MASK_FOR_PROCESS: AtomicUsize = AtomicUsize::new(0);
    static ref S_AFFINITY_MASK_FOR_CPU: Arc<RwLock<ArrayVec<usize, MAX_CPUS>>> = Arc::new(RwLock::new(ArrayVec::new()));
}

cfg_if! {
    if #[cfg(target_os = "windows")] {
        pub fn init_threads() {
            let hprocess = unsafe { GetCurrentProcess() };
            let systemaffinitymask: c_ulonglong = 0;
            let processaffinitymask: c_ulonglong = 0;
            unsafe { GetProcessAffinityMask(hprocess, addr_of!(processaffinitymask) as *mut _, addr_of!(systemaffinitymask) as *mut _) };
            S_AFFINITY_MASK_FOR_PROCESS.store(processaffinitymask as _, Ordering::SeqCst);
            let mut cpu_count = 0usize;
            let mut affinity_mask_for_cpu: Vec<usize> = Vec::new();
            affinity_mask_for_cpu.push(1);
            while (!affinity_mask_for_cpu[0] + 1 & processaffinitymask as usize) != 0 {
                if (affinity_mask_for_cpu[0] & processaffinitymask as usize) != 0 {
                    affinity_mask_for_cpu[cpu_count + 1] = affinity_mask_for_cpu[0];
                    cpu_count += 1;
                    if cpu_count == MAX_CPUS { break; }
                }
                affinity_mask_for_cpu[0] = affinity_mask_for_cpu[0] << 1;
            }

            if cpu_count == 0 || cpu_count == 1 {
                S_CPU_COUNT.store(1, Ordering::SeqCst);
                S_AFFINITY_MASK_FOR_CPU.clone().write().unwrap()[0] = 0xFFFFFFFF;
                return;
            }

            S_CPU_COUNT.store(cpu_count, Ordering::SeqCst);
            let lock = S_AFFINITY_MASK_FOR_CPU.clone();
            let mut writer = lock.write().unwrap();
            writer[0] = affinity_mask_for_cpu[1];
            writer[1] = affinity_mask_for_cpu[cpu_count];
            if cpu_count > 2 {
                if cpu_count == 3 {
                    writer[2] = affinity_mask_for_cpu[2];
                } else if cpu_count == 4 {
                    writer[2] = affinity_mask_for_cpu[2];
                    writer[3] = affinity_mask_for_cpu[3];
                } else {
                    writer.iter_mut().for_each(|a| *a = 0xFFFFFFFF);
                    if cpu_count > MAX_CPUS {
                        S_CPU_COUNT.store(MAX_CPUS, Ordering::SeqCst);
                    }
                }
            }
        }
    }
}
*/

/*
cfg_if! {
    if #[cfg(target_os = "windows")] {
        lazy_static! {
            static ref THREAD_LOCK: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
            static ref H_THREADS: Arc<RwLock<ArrayVec<HANDLE, 15>>> = Arc::new(RwLock::new(ArrayVec::new()));
        }

        pub fn lock_thread_affinity() {
            let cpu_count = S_CPU_COUNT.load(Ordering::SeqCst);

            if cpu_count == 1 {
                return;
            }

            let thread_lock = THREAD_LOCK.clone();
            let _thread_lock_2 = thread_lock.lock().unwrap();

            let threads_lock = H_THREADS.clone();
            let threads_reader = threads_lock.read().unwrap();

            let affinity_mask_lock = S_AFFINITY_MASK_FOR_CPU.clone();
            let affinity_mask_reader = affinity_mask_lock.read().unwrap();

            if threads_reader[0].0 != 0 {
                unsafe { SetThreadAffinityMask(threads_reader[0], affinity_mask_reader[0]) };
            }

            if threads_reader[1].0 != 0 {
                unsafe { SetThreadAffinityMask(threads_reader[1], affinity_mask_reader[1]) };
            }

            if cpu_count < 3 && threads_reader[13].0 != 0 {
                unsafe { SetThreadAffinityMask(threads_reader[13], affinity_mask_reader[1]) };
            } else if threads_reader[13].0 != 0 {
                unsafe { SetThreadAffinityMask(threads_reader[13], affinity_mask_reader[2]) };
            }

            if cpu_count > 2 && threads_reader[2].0 != 0 {
                unsafe { SetThreadAffinityMask(threads_reader[2], affinity_mask_reader[2]) };
            }

            if cpu_count > 3 && threads_reader[3].0 != 0 {
                unsafe { SetThreadAffinityMask(threads_reader[3], affinity_mask_reader[3]) };
            }
        }
    }
}
*/

/*
cfg_if! {
    if #[cfg(target_os = "windows")] {
        pub fn unlock_thread_affinity() {
            let cpu_count = S_CPU_COUNT.load(Ordering::SeqCst);

            if cpu_count == 1 {
                return;
            }

            let thread_lock = THREAD_LOCK.clone();
            let _thread_lock_2 = thread_lock.lock().unwrap();

            let threads_lock = H_THREADS.clone();
            let threads_reader = threads_lock.read().unwrap();

            let affinity_mask = S_AFFINITY_MASK_FOR_PROCESS.load(Ordering::SeqCst);

            if threads_reader[0].0 != 0 {
                unsafe { SetThreadAffinityMask(threads_reader[0], affinity_mask) };
            }

            if threads_reader[1].0 != 0 {
                unsafe { SetThreadAffinityMask(threads_reader[1], affinity_mask) };
            }

            if threads_reader[13].0 != 0 {
                unsafe { SetThreadAffinityMask(threads_reader[0], affinity_mask) };
            }

            if threads_reader[2].0 != 0 {
                unsafe { SetThreadAffinityMask(threads_reader[1], affinity_mask) };
            }

            if threads_reader[3].0 != 0 {
                unsafe { SetThreadAffinityMask(threads_reader[1], affinity_mask) };
            }
        }
    }
}
*/

/*
fn register_info_dvars() {
    dvar::register_float(
        "sys_configureGHz",
        0.0,
        Some(f32::MIN),
        Some(f32::MAX),
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::WRITE_PROTECTED,
        Some("Normalized total CPU power, based on cpu type, count, and speed; used in autoconfigure")
    );
    dvar::register_int(
        "sys_sysMB",
        0,
        Some(i32::MIN),
        Some(i32::MAX),
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::WRITE_PROTECTED,
        Some("Physical memory in the system"),
    );
    dvar::register_string(
        "sys_gpu",
        "",
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::WRITE_PROTECTED,
        Some("GPU description"),
    );
    dvar::register_int(
        "sys_configSum",
        0,
        Some(i32::MIN),
        Some(i32::MAX),
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::WRITE_PROTECTED,
        Some("Configuration checksum"),
    );
    // TODO - SIMD support Dvar
    dvar::register_float(
        "sys_cpuGHz",
        info().unwrap().cpu_ghz,
        Some(f32::MIN),
        Some(f32::MAX),
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::WRITE_PROTECTED,
        Some("Measured CPU speed"),
    );
    dvar::register_string(
        "sys_cpuName",
        &info().unwrap().cpu_name,
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::WRITE_PROTECTED,
        Some("CPU name description"),
    );
}

fn archive_info(sum: i32) {
    register_info_dvars();
    dvar::set_float_internal("sys_configureGHz", info().unwrap().configure_ghz);
    dvar::set_int_internal("sys_sysMB", info().unwrap().sys_mb as _);
    dvar::set_string_internal("sys_gpu", &info().unwrap().gpu_description);
    dvar::set_int_internal("sys_configSum", sum);
}
*/

fn should_update_for_info_change() -> bool {
    let msg_box_type = MessageBoxType::YesNo;
    let msg_box_icon = MessageBoxIcon::Information;
    let title = locale::localize_ref("WIN_CONFIGURE_UPDATED_TITLE");
    let text = locale::localize_ref("WIN_CONFIGURE_UPDATED_BODY");
    let handle = render::main_window_handle();
    matches!(
        message_box(handle, &title, &text, msg_box_type, Some(msg_box_icon),),
        Some(MessageBoxResult::Yes)
    )
}

cfg_if! {
    if #[cfg(windows)] {
        #[derive(Copy, Clone, Default, Debug)]
        #[repr(u32)]
        pub enum MessageBoxType {
            #[default]
            Ok = MB_OK.0,
            YesNoCanel = MB_YESNOCANCEL.0,
            YesNo = MB_YESNO.0,
            // TODO - maybe implement Help?
        }
    } else if #[cfg(target_os = "linux")] {
        #[derive(Copy, Clone, Default, Debug)]
        #[repr(u32)]
        pub enum MessageBoxType {
            #[default]
            Ok,
            YesNoCanel,
            YesNo,
            // TODO - maybe implement Help?
        }

        impl TryInto<gtk4::ButtonsType> for MessageBoxType {
            type Error = ();
            fn try_into(self) -> Result<gtk4::ButtonsType, Self::Error> {
                match self {
                    MessageBoxType::Ok => Ok(gtk4::ButtonsType::Ok),
                    MessageBoxType::YesNo => Ok(gtk4::ButtonsType::YesNo),
                    _ => Err(())
                }
            }
        }
    } else {
        #[derive(Copy, Clone, Default, Debug)]
        #[repr(u32)]
        pub enum MessageBoxType {
            #[default]
            Ok,
            YesNoCanel,
            YesNo,
        }
    }
}

cfg_if! {
    if #[cfg(windows)] {
        #[derive(Copy, Clone, Default, Debug)]
        #[repr(u32)]
        pub enum MessageBoxIcon {
            #[default]
            None = 0x00000000,
            Stop = MB_ICONSTOP.0,
            Information = MB_ICONINFORMATION.0,
        }
    } else if #[cfg(target_os = "linux")]  {
        #[derive(Copy, Clone, Default, Debug)]
        #[repr(u32)]
        pub enum MessageBoxIcon {
            #[default]
            None,
            Stop,
            Information,
        }

        impl TryInto<gtk4::MessageType> for MessageBoxIcon {
            type Error = ();
            fn try_into(self) -> Result<gtk4::MessageType, Self::Error> {
                use gtk4::MessageType::*;
                match self {
                    MessageBoxIcon::Information => Ok(Info),
                    MessageBoxIcon::Stop => Ok(Error),
                    _ => Err(())
                }
            }
        }
    } else {
        #[derive(Copy, Clone, Default, Debug)]
        #[repr(u32)]
        pub enum MessageBoxIcon {
            #[default]
            None,
            Stop,
            Information,
        }
    }
}

cfg_if! {
    if #[cfg(windows)] {
        #[derive(Copy, Clone, FromPrimitive)]
        #[repr(i32)]
        pub enum MessageBoxResult {
            Ok = IDOK.0,
            Cancel = IDCANCEL.0,
            Yes = IDYES.0,
            No = IDNO.0,
            Unknown,
        }
    } else if #[cfg(target_os = "linux")] {
        #[derive(Copy, Clone, FromPrimitive, Debug)]
        #[repr(i32)]
        pub enum MessageBoxResult {
            Ok,
            Cancel,
            Yes,
            No,
            Unknown,
        }

        impl From<gtk4::ResponseType> for MessageBoxResult {
            fn from(value: gtk4::ResponseType) -> Self {
                match value {
                    gtk4::ResponseType::Ok => MessageBoxResult::Ok,
                    gtk4::ResponseType::Cancel => MessageBoxResult::Cancel,
                    gtk4::ResponseType::Yes => MessageBoxResult::Yes,
                    gtk4::ResponseType::No => MessageBoxResult::No,
                    _ => MessageBoxResult::Unknown
                }
            }
        }
    } else {
        #[derive(Copy, Clone, FromPrimitive, Debug)]
        #[repr(i32)]
        pub enum MessageBoxResult {
            Ok,
            Cancel,
            Yes,
            No,
            Unknown,
        }
    }
}

cfg_if! {
    if #[cfg(target_os = "windows")] {
        pub fn message_box(
            handle: Option<WindowHandle>,
            title: &str, text: &str,
            msg_box_type: MessageBoxType,
            msg_box_icon: Option<MessageBoxIcon>
        ) -> Option<MessageBoxResult> {
            let hwnd = match handle {
                Some(h) => h.get_win32().unwrap().hwnd,
                None => 0 as _,
            };

            let ctext = match CString::new(text) {
                Ok(s) => s,
                Err(_) => return None,
            };

            let ctitle = match CString::new(title) {
                Ok(s) => s,
                Err(_) => return None,
            };

            let ctype = MESSAGEBOX_STYLE(
                msg_box_type as u32
                | msg_box_icon.unwrap_or(MessageBoxIcon::None) as u32
            );

            let res: MessageBoxResult = num::FromPrimitive::from_i32(unsafe {
                MessageBoxA(
                    HWND(hwnd as _),
                    PCSTR(ctext.as_ptr() as *const _),
                    PCSTR(ctitle.as_ptr() as *const _),
                    ctype
                ) }.0).unwrap_or(MessageBoxResult::Unknown);
            Some(res)
        }
    } else if #[cfg(target_os = "linux")] {
        // The non-Windows implementations of message_box() will use GTK
        // by default, instead of targeting each, e.g. Wayland, X, Cocoa, etc.
        // For platforms that don't support GTK for some reason,
        // other implementations are welcome

        // The GTK implementation here is very much a work in progress. It's
        // super buggy on WSL2, but I can't tell if the issues are with the
        // application here, or with WSL2. Will try to test on native Linux
        // at some point
        lazy_static! {
            static ref GTK_WINDOW_TITLE: Arc<RwLock<String>>
                = Arc::new(RwLock::new(String::new()));
        }

        thread_local! {
            static GTK_RESPONSE_EVENT: RefCell<SmpEvent<gtk4::ResponseType>>
                = RefCell::new(SmpEvent::new(gtk4::ResponseType::Other(0xFFFF), false, false));
        }

        pub fn message_box(
            _handle: Option<WindowHandle>,
            text: &str,
            title: &str,
            msg_box_type: MessageBoxType,
            msg_icon_type: Option<MessageBoxIcon>
        ) -> Option<MessageBoxResult> {
            let dialog = MessageDialogBuilder::new()
                .buttons(gtk4::ButtonsType::None)
                .destroy_with_parent(true)
                .focusable(true)
                //.message_type(msg_icon_type.unwrap_or(MessageBoxIcon::None).try_into().unwrap_or(gtk4::MessageType::Other))
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
                    ("No", gtk4::ResponseType::No)
                ],
                MessageBoxType::YesNoCanel => vec![
                    ("Yes", gtk4::ResponseType::Yes),
                    ("No", gtk4::ResponseType::No),
                    ("Cancel", gtk4::ResponseType::Cancel)
                ],
            };

            dialog.add_buttons(buttons);
            dialog.run_async(|obj, answer| {
                obj.close();
                GTK_RESPONSE_EVENT.with_borrow_mut(|event| {
                    #[allow(unused_must_use)]
                    {
                        event.send(answer);
                    }
                })
            });

            let response = GTK_RESPONSE_EVENT.with_borrow(|event| {
                while !event.signaled().unwrap_or(false) {}
                event.get_state().unwrap()
            });

            Some(response.into())
        }
    } else {
        pub fn message_box(
            _handle: Option<WindowHandle>,
            text: &str,
            title: &str,
            msg_box_type: MessageBoxType,
            msg_icon_type: Option<MessageBoxIcon>
        ) -> Option<MessageBoxResult> {
            println!(
                "message_box: handle={:?}, text={}, title={}, type={:?}, icon={:?}",
                _handle,
                text,
                title,
                msg_box_type,
                msg_icon_type,
            );
            None
        }
    }
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
        fn output_debug_string(string: &str) {
            unsafe { OutputDebugStringA(PCSTR(string.as_ptr())) };
        }
    } else {
        fn output_debug_string(_string: &str) {

        }
    }
}

pub fn print(text: &str) {
    if DEBUG_OUTPUT.load(Ordering::Relaxed) {
        output_debug_string(text);
    }

    conbuf::append_text_in_main_thread(text);
}

fn create_console() {}

pub fn show_console() {
    if conbuf::s_wcd_window_is_none() {
        create_console();
    }

    conbuf::s_wcd_window_set_visible(true);
}

fn post_error(error: &str) {
    conbuf::s_wcd_set_error_string(error.into());
    conbuf::s_wcd_clear_input_line_window();

    // DestroyWindow(s_wcd.hwndInputLine);

    let handle = render::main_window_handle();
    message_box(
        handle,
        "Error",
        error,
        MessageBoxType::Ok,
        MessageBoxIcon::Stop.into(),
    )
    .unwrap();
}

pub fn error(error: &str) -> ! {
    com::ERROR_ENTERED.store(true, Ordering::Relaxed);
    // Sys_SuspendOtherThreads()

    // FixWindowsDesktop() (probably shouldn't be necessary)

    // if Sys_IsMainThread() (no clue how necessary this check is,
    // probably have to do some restructuring)
    show_console();
    conbuf::append_text(&format!("\n\n{}\n", error));
    post_error(error);
    // Finish processing events
    // DoSetEvent_UNK();
    std::process::exit(0);
}
