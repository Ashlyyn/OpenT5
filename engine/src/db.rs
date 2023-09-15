use crate::*;
use cfg_if::cfg_if;

use std::{
    path::PathBuf,
    ptr::addr_of_mut,
    sync::{atomic::AtomicUsize, RwLock},
};

use lazy_static::lazy_static;

cfg_if! {
    if #[cfg(windows)] {
        use windows::Win32::{
            Foundation::{GetLastError, ERROR_HANDLE_EOF, HANDLE},
            Storage::FileSystem::ReadFileEx,
            System::IO::OVERLAPPED,
        };
    }
}

static G_TOTAL_WAIT: AtomicUsize = AtomicUsize::new(0);

#[cfg(windows)]
#[derive(Debug)]
struct FileHandle(HANDLE);

unsafe impl Send for FileHandle {}
unsafe impl Sync for FileHandle {}

#[derive(Default)]
struct LoadData<'a> {
    f: Option<FileHandle>,
    compress_buffer: &'a mut [u8],
    outstanding_reads: usize,
    filename: PathBuf,

    #[cfg(windows)]
    overlapped: OVERLAPPED,
}

unsafe impl<'a> Send for LoadData<'a> {}
unsafe impl<'a> Sync for LoadData<'a> {}

lazy_static! {
    static ref G_LOAD: RwLock<LoadData<'static>> =
        RwLock::new(LoadData::default());
}

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
        g_load.compress_buffer.as_mut_ptr().add(
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
