#![allow(dead_code)]

use crate::{util::EasierAtomic, *};
use cfg_if::cfg_if;

use std::{
    marker::PhantomData,
    path::PathBuf,
    sync::{
        atomic::{AtomicIsize, AtomicUsize},
        RwLock,
    },
};

use lazy_static::lazy_static;

cfg_if! {
    if #[cfg(windows)] {
        use core::ptr::addr_of_mut;
        use windows::Win32::{
            Foundation::{GetLastError, ERROR_HANDLE_EOF, HANDLE},
            Storage::FileSystem::ReadFileEx,
            System::IO::OVERLAPPED,
        };
    } else if #[cfg(unix)] {
        use std::{pin::Pin, os::fd::RawFd};
        use nix::{errno::Errno, sys::{signal::SigevNotify, aio::{Aio, AioRead}}};

    }
}

static G_TOTAL_WAIT: AtomicIsize = AtomicIsize::new(0);
static G_LOADED_SIZE: AtomicUsize = AtomicUsize::new(0);

#[cfg(windows)]
#[derive(Debug)]
struct FileHandle(HANDLE);

#[cfg(unix)]
#[derive(Debug)]
struct FileHandle(RawFd);

unsafe impl Send for FileHandle {}
unsafe impl Sync for FileHandle {}

#[derive(Default)]
struct LoadData<'a> {
    f: Option<FileHandle>,
    outstanding_reads: usize,
    filename: PathBuf,

    #[cfg(windows)]
    overlapped: OVERLAPPED,
    #[cfg(windows)]
    _p: PhantomData<&'a ()>,

    #[cfg(unix)]
    aior: Option<Pin<Box<AioRead<'a>>>>,
}

unsafe impl<'a> Send for LoadData<'a> {}
unsafe impl<'a> Sync for LoadData<'a> {}

lazy_static! {
    static ref G_LOAD: RwLock<LoadData<'static>> =
        RwLock::new(LoadData::default());
}

static mut G_FILE_BUF: [u8; 0x80000] = [0u8; 0x80000];

#[cfg(windows)]
unsafe extern "system" fn file_read_completion_dummy_callback(
    _: u32,
    _: u32,
    _: *mut OVERLAPPED,
) {
}

#[cfg(windows)]
fn read_data() -> Result<(), ()> {
    let mut g_load = G_LOAD.write().unwrap();

    let lpbuffer = unsafe {
        G_FILE_BUF.as_mut_ptr().add(
            g_load.overlapped.Anonymous.Anonymous.Offset as usize
                % 0x80000usize,
        )
    };

    if unsafe {
        ReadFileEx(
            g_load.f.as_ref().unwrap().0,
            Some(lpbuffer.cast()),
            0x40000,
            addr_of_mut!(g_load.overlapped),
            Some(file_read_completion_dummy_callback),
        )
    }
    .ok()
    .is_ok()
    {
        g_load.outstanding_reads += 1;
        g_load.overlapped.Anonymous.Pointer =
            (unsafe { g_load.overlapped.Anonymous.Anonymous.Offset } + 0x40000)
                as _;
        Ok(())
    } else {
        Err(())
    }
}

#[cfg(unix)]
fn read_data() -> Result<(), ()> {
    let mut g_load = G_LOAD.write().unwrap();

    let offset = (G_LOADED_SIZE.load_relaxed() & 1) * 0x40000;

    g_load.aior = Some(Box::pin(AioRead::new(
        g_load.f.as_ref().unwrap().0,
        0,
        unsafe { &mut G_FILE_BUF[offset..offset + 0x40000] },
        0,
        SigevNotify::SigevNone,
    )));

    if g_load.aior.as_mut().unwrap().as_mut().submit().is_ok() {
        g_load.outstanding_reads += 1;
        Ok(())
    } else {
        Err(())
    }
}

#[cfg(windows)]
fn read_xfile_stage() {
    if G_LOAD.read().unwrap().f.is_some() {
        assert_eq!(G_LOAD.read().unwrap().outstanding_reads, 0);

        if read_data().is_err() && unsafe { GetLastError() != ERROR_HANDLE_EOF }
        {
            com::errorln!(
                com::ErrorParm::DROP,
                "Read error of file '{}'",
                G_LOAD.read().unwrap().filename.display()
            );
        }
    }
}

#[cfg(unix)]
fn read_xfile_stage() {
    if G_LOAD.read().unwrap().f.is_some() {
        assert_eq!(G_LOAD.read().unwrap().outstanding_reads, 0);

        if read_data().is_err() {
            com::errorln!(
                com::ErrorParm::DROP,
                "Read error of file '{}'",
                G_LOAD.read().unwrap().filename.display()
            );
        }
    }
}

#[cfg(windows)]
fn wait_xfile_stage() {
    use windows::Win32::System::Threading::{SleepEx, INFINITE};

    let mut g_load = G_LOAD.write().unwrap();

    assert!(g_load.f.is_some());
    assert!(g_load.outstanding_reads > 0);

    g_load.outstanding_reads -= 1;

    let then = sys::milliseconds();

    unsafe { SleepEx(INFINITE, true) };

    let now = sys::milliseconds();

    G_TOTAL_WAIT.store_relaxed(G_TOTAL_WAIT.load_relaxed() + (now - then));

    G_LOADED_SIZE.increment_wrapping();
}

#[cfg(unix)]
fn wait_xfile_stage() {
    let mut g_load = G_LOAD.write().unwrap();

    assert!(g_load.f.is_some());
    assert!(g_load.outstanding_reads > 0);

    g_load.outstanding_reads -= 1;

    let then = sys::milliseconds();

    while g_load.aior.as_mut().unwrap().as_mut().error()
        == Err(Errno::EINPROGRESS)
    {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    let now = sys::milliseconds();

    G_TOTAL_WAIT.store_relaxed(G_TOTAL_WAIT.load_relaxed() + (now - then));

    G_LOADED_SIZE.increment_wrapping();
}
