#![allow(dead_code)]

use crate::{
    util::{EasierAtomic, RwLockExt, StRwLock},
    *,
};
use bitflags::bitflags;
use cfg_if::cfg_if;
use libz_sys::{
    inflate, inflateInit_, z_stream, zlibVersion, Z_OK, Z_STREAM_END,
    Z_SYNC_FLUSH,
};

use std::{
    mem::{size_of, size_of_val, transmute},
    path::{Path, PathBuf},
    sync::atomic::{AtomicIsize, AtomicUsize},
};

use lazy_static::lazy_static;

cfg_if! {
    if #[cfg(windows)] {
        use core::ptr::addr_of_mut;
        use windows::Win32::{
            Foundation::{GetLastError, ERROR_HANDLE_EOF, HANDLE, CloseHandle},
            Storage::FileSystem::{ReadFileEx, GetFileSize},
            System::IO::OVERLAPPED,
        };
    } else if #[cfg(unix)] {
        use std::{pin::Pin, os::fd::RawFd};
        use nix::{errno::Errno, sys::{signal::SigevNotify, aio::{Aio, AioRead}}};

    }
}

const FASTFILE_HEADER_MAGIC_U: u64 = 0x4957_6666_75_313030;
const FASTFILE_HEADER_MAGIC_0: u64 = 0x4957_6666_30_313030;
const FASTFILE_EXPECTED_VERSION: u32 = 0x000001D9;

static G_TOTAL_WAIT: AtomicIsize = AtomicIsize::new(0);
static G_LOADED_SIZE: AtomicUsize = AtomicUsize::new(0);

#[cfg(windows)]
#[derive(Copy, Clone, Debug)]
struct FileHandle(HANDLE);

#[cfg(unix)]
#[derive(Copy, Clone, Debug)]
struct FileHandle(RawFd);

#[cfg(windows)]
fn get_file_size(f: &FileHandle) -> u32 {
    unsafe { GetFileSize(f.0, None) }
}

unsafe impl Send for FileHandle {}
unsafe impl Sync for FileHandle {}

const DEFLATE_BUFFER_SIZE: usize = 32768;

struct LoadData<'a> {
    f: Option<FileHandle>,
    outstanding_reads: usize,
    filename: PathBuf,
    flags: XFileLoadFlags,
    start_time: isize,
    compress_buffer: Option<&'a mut [u8]>,
    stream: z_stream,
    deflate_buffer_pos: usize,
    deflate_buffer: [u8; DEFLATE_BUFFER_SIZE],
    deflate_remaining_file_size: usize,

    #[cfg(windows)]
    overlapped: OVERLAPPED,

    #[cfg(unix)]
    aior: Option<Pin<Box<AioRead<'a>>>>,
}

impl<'a> Default for LoadData<'a> {
    fn default() -> Self {
        Self {
            f: None,
            outstanding_reads: 0,
            filename: PathBuf::new(),
            flags: XFileLoadFlags::empty(),
            start_time: 0,
            compress_buffer: None,
            stream: z_stream {
                next_in: core::ptr::null_mut(),
                avail_in: 0,
                total_in: 0,
                next_out: core::ptr::null_mut(),
                avail_out: 0,
                total_out: 0,
                msg: core::ptr::null_mut(),
                state: core::ptr::null_mut(),
                zalloc: unsafe { transmute(0usize) },
                zfree: unsafe { transmute(0usize) },
                opaque: core::ptr::null_mut(),
                data_type: 0,
                adler: 0,
                reserved: 0,
            },
            deflate_buffer_pos: 0,
            deflate_buffer: [0; 32768],
            deflate_remaining_file_size: 0,

            #[cfg(windows)]
            overlapped: OVERLAPPED::default(),

            #[cfg(unix)]
            aior: None,
        }
    }
}

#[derive(Debug)]
enum LoadDataReadError {
    StreamEnd,
}

impl<'a> LoadData<'a> {
    fn read<T: Sized + Copy>(&mut self) -> Result<T, LoadDataReadError> {
        if (self.stream.avail_in as i32 - size_of::<T>() as i32) < 0 {
            return Err(LoadDataReadError::StreamEnd);
        }

        let p = self.stream.next_in.cast::<T>().cast_const();

        let t = unsafe { *p };

        unsafe {
            self.stream.next_in = self.stream.next_in.add(size_of::<T>())
        };
        self.stream.avail_in -= size_of::<T>() as u32;

        Ok(t)
    }
}

lazy_static! {
    static ref G_LOAD: StRwLock<LoadData<'static>> = unsafe {
        StRwLock::new(LoadData::default(), "database", sys::is_database_thread)
    };
}

// We have to put this here instead of embedding it into LoadData because
// ReadFileEx and AioRead both require a valid reference to exist for longer
// than we can hold a write lock on G_LOAD with the way things are currently
// structured.
static mut G_FILE_BUF: [u8; 0x80000] = [0u8; 0x80000];

bitflags! {
    #[derive(Default)]
    struct XFileLoadFlags: u32 {

    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(C, packed)]
struct XFile {
    size: u32,
    external_size: u32,
    block_size: [u32; 7],
}

fn auth_load_inflate_init(stream: &mut z_stream, is_secure: bool) -> i32 {
    assert!(!is_secure);

    unsafe {
        inflateInit_(
            addr_of_mut!(*stream),
            zlibVersion(),
            size_of_val(stream) as _,
        )
    }
}

fn auth_load_inflate(stream: &mut z_stream, flush: i32) -> i32 {
    unsafe { inflate(addr_of_mut!(*stream), flush) }
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
    let mut g_load = G_LOAD.write();

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
    if G_LOAD.read().f.is_some() {
        assert_eq!(G_LOAD.read().outstanding_reads, 0);

        if read_data().is_err() && unsafe { GetLastError() != ERROR_HANDLE_EOF }
        {
            com::errorln!(
                com::ErrorParm::DROP,
                "Read error of file '{}'",
                G_LOAD.read().filename.display()
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

    let mut g_load = G_LOAD.write();

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

fn cancel_load_xfile() {
    let mut g_load = G_LOAD.write();

    while g_load.outstanding_reads != 0 {
        wait_xfile_stage();
    }

    if let Some(h) = g_load.f.as_ref() {
        assert_ne!(h.0 .0, 0xFFFFFFFF);
    } else {
        assert!(false);
    }

    unsafe { CloseHandle(g_load.f.as_mut().unwrap().0) };
}

fn load_xfile_data<T: Sized + Copy>() -> Result<T, ()> {
    assert_ne!(size_of::<T>(), 0);
    assert!(G_LOAD.read().f.is_some());
    assert_eq!(G_LOAD.read().stream.avail_out, 0);

    let mut data = Vec::new();
    let mut size = size_of::<T>();

    while size > 0 {
        if G_LOAD.read().deflate_buffer_pos + size > DEFLATE_BUFFER_SIZE {
            assert!(G_LOAD.read().deflate_buffer_pos <= DEFLATE_BUFFER_SIZE);

            if G_LOAD.read().deflate_buffer_pos < DEFLATE_BUFFER_SIZE {
                let c = DEFLATE_BUFFER_SIZE - G_LOAD.read().deflate_buffer_pos;
                let slice = &G_LOAD.read().deflate_buffer[G_LOAD
                    .read()
                    .deflate_buffer_pos
                    ..G_LOAD.read().deflate_buffer_pos + c];
                data.extend_from_slice(slice);
                size -= c;
            }
            let avail_out = if G_LOAD.read().deflate_remaining_file_size
                < DEFLATE_BUFFER_SIZE
            {
                G_LOAD.read().deflate_remaining_file_size
            } else {
                DEFLATE_BUFFER_SIZE
            };
            G_LOAD.write_with(|mut g_load| {
                g_load.stream.avail_out = avail_out as _;
                g_load.deflate_buffer_pos = DEFLATE_BUFFER_SIZE - avail_out;
                g_load.stream.next_out = unsafe {
                    g_load
                        .deflate_buffer
                        .as_ptr()
                        .add(g_load.deflate_buffer.len() - avail_out)
                        .cast_mut()
                };
            });

            let last_avail_out_size = avail_out.min(size);
            size -= last_avail_out_size;

            G_LOAD.write().deflate_remaining_file_size =
                G_LOAD.read().deflate_remaining_file_size - avail_out;

            loop {
                if G_LOAD.read().stream.avail_in != 0 {
                    let b = G_LOAD.write_with(|mut g_load| {
                        let res =
                            auth_load_inflate(&mut g_load.stream, Z_SYNC_FLUSH);

                        if res != Z_OK && res != Z_STREAM_END {
                            cancel_load_xfile();
                            com::errorln!(
                                com::ErrorParm::DROP,
                                "Fastfile for zone '{}' appears corrupt or \
                                 unreadable (code {}.)",
                                g_load.filename.display(),
                                res + 0x6E
                            );
                        }

                        if g_load.f.is_none() {
                            assert!(
                                unsafe {
                                    g_load.stream.next_in.sub_ptr(
                                        g_load
                                            .compress_buffer
                                            .as_ref()
                                            .unwrap()
                                            .as_ptr(),
                                    )
                                } <= g_load
                                    .compress_buffer
                                    .as_ref()
                                    .unwrap()
                                    .len()
                            );
                            if g_load.stream.next_in.cast_const()
                                == unsafe {
                                    g_load
                                        .compress_buffer
                                        .as_ref()
                                        .unwrap()
                                        .as_ptr()
                                        .add(
                                            g_load
                                                .compress_buffer
                                                .as_ref()
                                                .unwrap()
                                                .len(),
                                        )
                                }
                            {
                                g_load.stream.next_in = g_load
                                    .compress_buffer
                                    .as_ref()
                                    .unwrap()
                                    .as_ptr()
                                    .cast_mut();
                            }
                        }

                        if g_load.stream.avail_out == 0 {
                            assert!(last_avail_out_size <= DEFLATE_BUFFER_SIZE);

                            let slice = &G_LOAD.read().deflate_buffer[G_LOAD
                                .read()
                                .deflate_buffer_pos
                                ..G_LOAD.read().deflate_buffer_pos
                                    + last_avail_out_size];
                            data.extend_from_slice(slice);

                            g_load.deflate_buffer_pos += last_avail_out_size;
                            return true;
                        }

                        assert!(
                            res == Z_OK,
                            "Invalid fast file '{}' ({} != Z_OK)",
                            g_load.filename.display(),
                            res
                        );
                        false
                    });

                    if b {
                        continue;
                    }
                }
                wait_xfile_stage();
                read_xfile_stage();
            }
        }

        let slice =
            &G_LOAD.read().deflate_buffer[G_LOAD.read().deflate_buffer_pos
                ..G_LOAD.read().deflate_buffer_pos + size];
        data.extend_from_slice(slice);
        G_LOAD.write().deflate_buffer_pos += size;
    }

    Ok(unsafe { *data.get(0..size_of::<T>()).ok_or(())?.as_ptr().cast::<T>() })
}

fn load_xfile_set_size(size: usize) {
    assert_eq!(G_LOAD.read().deflate_remaining_file_size, 0);

    G_LOAD.write().deflate_remaining_file_size = size;
}

fn load_xfile_internal() {
    G_TOTAL_WAIT.store_relaxed(0);

    assert!(G_LOAD.read().f.is_none());

    read_xfile_stage();
    if G_LOAD.read().outstanding_reads == 0 {
        com::errorln!(
            com::ErrorParm::DROP,
            "Fastfile for zone '{}' is empty.",
            G_LOAD.read().filename.display()
        );
    }

    wait_xfile_stage();
    read_xfile_stage();

    G_LOAD.write_with(|mut g_load| {
        let magic = g_load.read::<u64>().unwrap();

        if magic != FASTFILE_HEADER_MAGIC_U && magic != FASTFILE_HEADER_MAGIC_0
        {
            com::errorln!(
                com::ErrorParm::DROP,
                "Fastfile for zone '{}' is corrupt or unreadable.",
                g_load.filename.display()
            );
        }

        let version = g_load.read::<u32>().unwrap();

        if version != FASTFILE_EXPECTED_VERSION {
            if version < FASTFILE_EXPECTED_VERSION {
                com::errorln!(
                    com::ErrorParm::DROP,
                    "Fastfile for zone '{}' is out of date (version {}, \
                     expecting {})",
                    g_load.filename.display(),
                    version,
                    FASTFILE_EXPECTED_VERSION
                );
            } else {
                com::errorln!(
                    com::ErrorParm::DROP,
                    "Fastfile for zone '{}' is newer than client executable \
                     (version {}, expecting {})",
                    g_load.filename.display(),
                    version,
                    FASTFILE_EXPECTED_VERSION
                );
            }
        }

        let is_secure = magic == FASTFILE_HEADER_MAGIC_U;

        let res = auth_load_inflate_init(&mut g_load.stream, is_secure);

        let s = if !is_secure {
            "authenticated file not supported"
        } else if res != Z_OK {
            "init failed"
        } else {
            ""
        };

        if s != "" {
            cancel_load_xfile();
            com::errorln!(
                com::ErrorParm::DROP,
                "Fastfile for zone '{}' could not be loaded (authenticated \
                 file not supported)",
                g_load.filename.display()
            );
        }
    });

    load_xfile_set_size(36);
    let xfile = load_xfile_data::<XFile>().unwrap();
    load_xfile_set_size(xfile.size as _);

    todo!()
}

fn load_xfile(
    f: FileHandle,
    filename: impl AsRef<Path>,
    flags: XFileLoadFlags,
) {
    G_LOAD.write_with(|mut g_load| {
        g_load.f = Some(f);
        g_load.filename = filename.as_ref().to_path_buf();
        g_load.flags = flags;
        g_load.start_time = sys::milliseconds();
        g_load.compress_buffer = Some(unsafe { &mut G_FILE_BUF });
        g_load.stream.next_in = unsafe { G_FILE_BUF.as_mut_ptr() };
        g_load.stream.avail_in = 0;
        g_load.deflate_buffer_pos = DEFLATE_BUFFER_SIZE;
    });

    load_xfile_internal();
}
