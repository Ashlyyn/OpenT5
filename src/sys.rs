#![allow(dead_code)]

use crate::*;
use sysinfo::{CpuExt, SystemExt};

mod gpu;

use lazy_static::lazy_static;
use std::path::Path;
use std::{
    fmt::Display,
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicIsize, Ordering::SeqCst},
    time::SystemTime,
};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::MAX_PATH;
#[cfg(target_os = "windows")]
use windows::Win32::System::LibraryLoader::GetModuleFileNameA;
//#[cfg(not(target_os="windows"))]

lazy_static! {
    static ref BASE_TIME_ACQUIRED: AtomicBool = AtomicBool::new(false);
    pub static ref TIME_BASE: AtomicIsize = AtomicIsize::new(0);
}

pub fn init() {
    gpu::init();
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

#[cfg(target_os = "windows")]
#[allow(unreachable_code)]
pub fn get_executable_name() -> String {
    todo!("Not working correctly on Windows (path can't be stripped)");
    let mut buf: [u8; MAX_PATH as usize] = [0; MAX_PATH as usize];
    unsafe { GetModuleFileNameA(None, &mut buf) };
    let s = String::from_utf8(buf.to_vec()).unwrap().to_string();
    println!("\"{}\"", s);
    let s = s.strip_suffix(".exe").unwrap().to_string();
    match s.rfind("\\.:") {
        Some(pos) => s[pos + 1..].to_string(),
        None => s,
    }
}

#[cfg(target_os = "linux")]
pub fn get_executable_name() -> String {
    let pid = std::process::id();

    let proc_path = format!("/proc/{}/exe", pid);
    let path = std::fs::read_link(proc_path)
        .expect("sys::get_executable_name: readlink() failed")
        .to_str()
        .unwrap()
        .to_string();
    let s = if path.ends_with('/') {
        &path[..path.len() - 1]
    } else {
        &path
    };
    let pos = s.rfind('/').unwrap();
    s[pos + 1..].to_string()
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd"
))]
pub fn get_executable_name() -> String {
    #[cfg(target_os = "netbsd")]
    const PROC_PATH: String = "/proc/curproc/exe";
    #[cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd"
    ))]
    const PROC_PATH: String = "/proc/curproc/file";

    // TODO - implement kinfo_getproc method if procfs method fails
    let path = std::fs::read_link(proc_path)
        .expect("sys::get_executable_name: readlink() failed")
        .to_str()
        .unwrap()
        .to_string();
    let s = if path.ends_with('/') {
        &path[..path.len() - 1]
    } else {
        &path
    };
    let pos = s.rfind('/').unwrap();
    s[pos + 1..].to_string()
}

// TODO - implement for macOS

// Fallback method - if no platform-specific method is used, try to get the executable name from argv[0]
#[cfg(not(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "linux",
    target_os = "windows"
)))]
pub fn get_executable_name() -> String {
    let s = if path.ends_with('/') {
        &path[..path.len() - 1]
    } else {
        &path
    };
    let s = std::env::args().to_string();
    let pos = s.rfind('/').unwrap();
    s[pos + 1..].to_string()
}

pub fn get_semaphore_file_path() -> PathBuf {
    Path::new(&fs::get_os_folder_path(fs::OsFolder::UserData))
        .join("/Activision/CoD")
}

pub fn get_semaphore_file_name() -> String {
    format!("__{}", get_executable_name())
}

pub fn check_crash_or_rerun() -> bool {
    let semaphore_folder_path = get_semaphore_file_path();

    if !std::path::Path::new(&semaphore_folder_path).exists() {
        std::fs::create_dir_all(&semaphore_folder_path).unwrap();
        return true;
    }

    let semaphore_file_path =
        semaphore_folder_path.join(&get_semaphore_file_name());
    let semaphore_file_exists = semaphore_file_path.exists();
    if semaphore_file_exists {
        com::print_warning("check_crash_or_rerun: Semaphore file found, game probably crashed on last run.".to_string());
    }
    // TODO - implement message box functionality and ref localization
    true
}

pub fn get_cmdline() -> String {
    let mut cmd_line: String = String::new();
    for arg in std::env::args() {
        write!(&mut cmd_line, "{} ", &arg).unwrap();
    }
    cmd_line.trim().to_string()
}

pub fn start_minidump(b: bool) {
    com::println(format!("Starting minidump with b = {}...", b));
    com::println("TODO: implement.".to_string());
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
    system.total_memory() * 1024
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

pub async fn detect_video_card() -> String {
    let adapter = gpu::Adapter::new(&gpu::Instance::new(), None).await;
    adapter.get_info().name
}

pub struct SysInfo {
    pub gpu_description: String,
    pub logical_cpu_count: usize,
    pub physical_cpu_count: usize,
    pub sys_mb: u64,
    pub cpu_vendor: String,
    pub cpu_name: String,
}

impl Display for SysInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,
            "GPU Description: {}\nCPU: {} ({})\nCores: {} ({} physical)\nSystem RAM: {}MiB\n",
            self.gpu_description, self.cpu_name, self.cpu_vendor,
            self.logical_cpu_count, self.physical_cpu_count, self.sys_mb)
    }
}

impl SysInfo {
    async fn new() -> Self {
        let gpu_description = detect_video_card().await;
        let logical_cpu_count = get_logical_cpu_count();
        let physical_cpu_count = get_physical_cpu_count();
        let sys_mb = get_system_ram_in_bytes() / (1024 * 1024);
        let cpu_vendor = get_cpu_vendor();
        let cpu_name = get_cpu_name();

        SysInfo {
            gpu_description,
            logical_cpu_count,
            physical_cpu_count,
            sys_mb,
            cpu_vendor,
            cpu_name,
        }
    }
}

pub async fn find_info() -> SysInfo {
    SysInfo::new().await
}
