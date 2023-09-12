#![allow(dead_code)]

// This file exists to abstract filesystem-related functionalities

use crate::{util::EasierAtomic, *};
use arrayvec::ArrayVec;
use cfg_if::cfg_if;
use core::{str::FromStr, sync::atomic::AtomicUsize};
use std::{
    ffi::OsStr,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::RwLock,
};
use zip::read::ZipArchive;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        use windows::Win32::{
            UI::Shell::{
                SHGetFolderPathA, CSIDL_LOCAL_APPDATA, CSIDL_FLAG_CREATE,
                CSIDL_PERSONAL, SHGFP_TYPE_CURRENT,
            },
            Foundation::MAX_PATH};
            use core::mem::transmute;
    }
}

#[cfg(windows)]
const MAX_PATH_LEN: usize = MAX_PATH as usize;

#[cfg(unix)]
const MAX_PATH_LEN: usize = 4096usize;

// Probably a safe value to default to.
#[cfg(not(any(windows, unix)))]
const MAX_PATH_LEN: usize = 256usize;

#[derive(Copy, Clone, Debug)]
pub enum OsFolder {
    UserData,
    Documents,
}

cfg_if! {
    if #[cfg(target_os = "windows")] {
        // TODO - will panic if folder path contains invalid UTF-8 characters.
        // Fix later.
        #[allow(
            clippy::indexing_slicing,
            clippy::multiple_unsafe_ops_per_block
        )]
        pub fn get_os_folder_path(os_folder: OsFolder) -> Option<PathBuf> {
            let csidl: u32 = match os_folder {
                OsFolder::UserData => CSIDL_LOCAL_APPDATA,
                OsFolder::Documents => CSIDL_PERSONAL,
            };

            let mut buf: [u8; MAX_PATH as usize] = [0; MAX_PATH as usize];
            // SAFETY:
            // SHGetFolderPathA is an FFI function, requiring use of unsafe.
            // SHGetFolderPathA itself should never create UB, violate memory
            // safety, etc., provided the supplied buffer is MAX_PATH bytes
            // or more, which we've ensure it is.
            match unsafe {
                SHGetFolderPathA(
                    None,
                    transmute(csidl | CSIDL_FLAG_CREATE),
                    None,
                    transmute(SHGFP_TYPE_CURRENT.0),
                    &mut buf,
                )
            } {
                Ok(_) => {
                    // Null-terminate the string, in case the folder path
                    // was exactly MAX_PATH characters.
                    buf[buf.len() - 1] = 0x00;
                    Some(PathBuf::new().join(unsafe { OsStr::from_os_str_bytes_unchecked(&buf) }).join("Activision").join("CoD"))
                },
                Err(_) => None,
            }
        }
    } else if #[cfg(target_family = "unix")] {
        #[allow(clippy::needless_pass_by_value)]
        pub fn get_os_folder_path(os_folder: OsFolder) -> Option<PathBuf> {
            let envar = match os_folder {
                OsFolder::UserData => "XDG_DATA_HOME",
                OsFolder::Documents => "XDG_DOCUMENTS_DIR",
            };

            let Ok(home) = std::env::var("HOME") else { return None };

            let envar_default = match os_folder {
                OsFolder::UserData => format!("{}/.local/share", home),
                OsFolder::Documents => format!("{}/Documents", home),
            };

            Some(PathBuf::from_str(&std::env::var(envar).map_or(envar_default, |s| s)).unwrap().join("Activision").join("CoD"))
        }
    } else {
        pub fn get_os_folder_path(os_folder: OsFolder) -> Option<PathBuf> {
            compile_error!(
                "get_os_folder_path unimplemented for OS or arch",
            );
        }
    }
}

pub fn create_path(path: impl AsRef<Path>) -> Result<PathBuf, std::io::Error> {
    let path = path.as_ref();

    if path.is_relative() {
        com::warnln!(
            10.into(),
            "WARNING: refusing to create relative path \"{}\"",
            path.display(),
        );
        return Err(std::io::ErrorKind::InvalidFilename.into());
    }

    if path.exists() {
        return Ok(path.to_path_buf());
    }

    if std::fs::File::create(path).is_ok() {
        Ok(path.to_path_buf())
    } else {
        let Some(dir_path) = path.parent() else {
            return Err(std::io::ErrorKind::InvalidFilename.into());
        };

        std::fs::create_dir_all(dir_path)?;

        match std::fs::File::create(path) {
            Ok(_) => Ok(path.to_path_buf()),
            Err(e) => Err(e),
        }
    }
}

// TODO - fully implement
pub fn init_filesystem(dev: bool) {
    // FSH.write().unwrap().fill(None);
    startup("main", dev);
}

// TODO - fully implement
fn startup(_param_1: &str, _dev: bool) {
    com::println!(16.into(), "----- fs::startup -----");
    register_dvars();
    com::println!(16.into(), "-----------------------");
}

fn register_dvars() {
    dvar::register_bool(
        "fs_ignoreLocalized",
        false,
        dvar::DvarFlags::LATCHED | dvar::DvarFlags::CHEAT_PROTECTED,
        "Ignore localized files".into(),
    )
    .unwrap();
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i32)]
enum Thread {
    Main,
    Stream,
    Database,
    Backend,
    Server,
}

impl Thread {
    #[must_use]
    fn as_i32(self) -> i32 {
        self as i32
    }
}

lazy_static! {
    static ref FS_GAMEDIR: RwLock<PathBuf> = RwLock::new(PathBuf::new());
}

fn build_os_path_for_thread(
    base: impl AsRef<Path>,
    gamedir: Option<impl AsRef<Path>>,
    qpath: impl AsRef<Path>,
    thread: Thread,
) -> PathBuf {
    let gamedir = if let Some(g) = gamedir {
        if g.as_ref() == Path::new("") {
            FS_GAMEDIR.read().unwrap().clone()
        } else {
            g.as_ref().to_path_buf()
        }
    } else {
        PathBuf::new()
    };

    let ospath = base.as_ref().to_path_buf().join(gamedir).join(qpath);

    if ospath.as_os_str().len() > MAX_PATH_LEN {
        if thread != Thread::Main {
            return PathBuf::new();
        }

        com::errorln!(
            com::ErrorParm::FATAL,
            "\x15fs::build_os_path: os path length exceeded"
        );
    }

    ospath
}

pub fn build_os_path(
    base: impl AsRef<Path>,
    gamedir: Option<impl AsRef<Path>>,
    qpath: impl AsRef<Path>,
) -> PathBuf {
    build_os_path_for_thread(base, gamedir, qpath, Thread::Main)
}

fn delete(filename: impl AsRef<Path>) -> Result<(), std::io::Error> {
    if filename.as_ref() == &PathBuf::new() {
        return Err(std::io::ErrorKind::InvalidFilename.into());
    }

    let homepath = dvar::get_string("fs_homepath").unwrap();
    let ospath = build_os_path(
        homepath,
        Some(FS_GAMEDIR.read().unwrap().clone()),
        filename,
    );
    std::fs::remove_file(ospath)
}

fn get_current_thread() -> Option<Thread> {
    if sys::is_main_thread() {
        Some(Thread::Main)
    } else if sys::is_database_thread() {
        Some(Thread::Database)
    } else if sys::is_stream_thread() {
        Some(Thread::Stream)
    } else if sys::is_render_thread() {
        Some(Thread::Backend)
    } else if sys::is_server_thread() {
        Some(Thread::Server)
    } else {
        None
    }
}

type Iwd = ZipArchive<std::fs::File>;

enum Qfile {
    ZipFile {
        archive: Arc<RwLock<Iwd>>,
        // Have to embed this here to make our implementation of Read work.
        // Can't embed the ZipFile instead, since it's only valid for as long
        // as the archive's RwLock is locked.
        name: PathBuf,
    },
    File {
        file: std::fs::File,
        name: PathBuf,
    },
}

impl Read for Qfile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Qfile::ZipFile { archive, name } => archive
                .write()
                .unwrap()
                .by_name(&name.to_string_lossy())?
                .read(buf),
            Qfile::File { ref mut file, .. } => file.read(buf),
        }
    }
}

struct FileHandleData {
    file: Qfile,
    handle_sync: bool,
    file_size: usize,
    streamed: bool,
}

impl FileHandleData {
    fn name(&self) -> &Path {
        match &self.file {
            Qfile::File { name, .. } => name.as_path(),
            Qfile::ZipFile { name, .. } => name.as_path(),
        }
    }
}

unsafe impl Sync for FileHandleData {}
unsafe impl Send for FileHandleData {}

lazy_static! {
    static ref FSH: RwLock<ArrayVec<Option<FileHandleData>, 70>> =
        RwLock::new(ArrayVec::new_const());
}

// Fd is neither [`Copy`] nor [`Clone`] so that our Drop implementation can
// ensure it's safe to set its corresponding element in FSH to [`None`]
// (wouldn't be able to do that if copies/clones of the Fd were floating
// around).
//
// We could also implement it as Fd(Arc<u8>) to make it [`Clone`] and then only
// clear it from FSH when the [`Arc`]'s refcount is 0, but that seems like a
// lot of overhead for something that shouldn't ever really *need* to be cloned

/// An opaque file handle used by functions in this module.
///
/// [`Fd`] is intentionally not [`Copy`] or [`Clone`] to ensure unique
/// ownership, and there is no way to retieve the underlying [`File`] it
/// represents.
///
/// It's only use is to be passed between functions in this module.
/// It can only be created by functions in this module, and it will
/// automatically clean up the resources it uses on [`Drop`].
#[derive(Debug)]
pub struct Fd(u8);

impl Fd {
    fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl Drop for Fd {
    fn drop(&mut self) {
        FSH.write().unwrap()[self.as_usize()] = None;
    }
}

fn handle_for_file(thread: Thread) -> std::io::Result<Fd> {
    let fd_range = match thread {
        Thread::Main => {
            assert!(sys::is_main_thread());
            1..=49
        }
        Thread::Stream => {
            assert!(sys::is_stream_thread());
            50..=61
        }
        Thread::Database => {
            assert!(sys::is_database_thread());
            61..=61
        }
        Thread::Backend => {
            assert!(sys::is_render_thread());
            62..=62
        }
        Thread::Server => {
            assert!(sys::is_server_thread());
            63..=64
        }
    };

    for fd in fd_range.clone() {
        if FSH.read().unwrap()[fd].is_none() {
            return Ok(Fd(fd as _));
        }
    }

    com::warnln!(
        10.into(),
        "FILE {:2}: '{}'",
        fd_range.start(),
        FSH.read().unwrap()[*fd_range.start()]
            .as_ref()
            .unwrap()
            .name()
            .to_string_lossy()
    );
    com::warnln!(
        10.into(),
        "fs::handle_for_file: none free ({})",
        thread.as_i32()
    );

    for (i, f) in FSH.read().unwrap().iter().enumerate() {
        com::println!(
            10.into(),
            "FILE {:2}: '{}'",
            i,
            f.as_ref().unwrap().name().to_string_lossy()
        );
    }

    com::errorln!(com::ErrorParm::DROP, "\x15fs::handle_for_file: none free");

    Err(std::io::ErrorKind::Other.into())
}

fn handle_for_file_current_thread() -> std::io::Result<Fd> {
    handle_for_file(
        get_current_thread()
            .expect("Does the fs need to support a new thread?"),
    )
}

struct Directory {
    path: PathBuf,
    gamedir: PathBuf,
}

enum Qdir {
    Iwd {
        iwd: Arc<RwLock<Iwd>>,
        iwd_name: PathBuf,
    },
    Dir {
        dir: Directory,
    },
}

impl Qdir {
    fn is_iwd(&self) -> bool {
        match self {
            Qdir::Iwd { .. } => true,
            _ => false,
        }
    }

    fn is_dir(&self) -> bool {
        match self {
            Qdir::Dir { .. } => true,
            _ => false,
        }
    }

    fn iwd(&self) -> Option<Arc<RwLock<Iwd>>> {
        match self {
            Qdir::Iwd { iwd, .. } => Some(iwd.clone()),
            _ => None,
        }
    }

    fn dir(&self) -> Option<&Directory> {
        match self {
            Qdir::Dir { dir } => Some(&dir),
            _ => None,
        }
    }
}

struct Searchpath {
    qdir: Qdir,
    ignore: bool,
    ignore_pure_check: bool,
    language: Option<locale::Language>,
}

impl Searchpath {
    fn is_localized(&self) -> bool {
        self.language.is_some()
    }
}

lazy_static! {
    static ref FS_SEARCHPATHS: RwLock<Vec<Searchpath>> =
        RwLock::new(Vec::new());
}

fn add_searchpath(sp: Searchpath) {
    FS_SEARCHPATHS.write().unwrap().push(sp)
}

fn use_searchpath(sp: &Searchpath) -> bool {
    if sp.is_localized() == false
        || dvar::get_bool("fs_ignoreLocalized").unwrap() == false
    {
        if let Some(lang) = sp.language && lang != seh::get_current_language()
        {
            false
        } else {
            true
        }
    } else {
        false
    }
}

fn load_zip_file(
    filename: impl AsRef<Path>,
    _basename: impl AsRef<Path>,
) -> std::io::Result<Iwd> {
    ZipArchive::new(file_open_read(filename)?)
        .map_err(|_| std::io::ErrorKind::Other.into())
}

fn add_iwd_files_for_game_directory(
    _base: impl AsRef<Path>,
    _gamedir: impl AsRef<Path>,
) -> std::io::Result<()> {
    todo!()
}

fn add_game_directory(
    base: impl AsRef<Path>,
    gamedir: impl AsRef<Path>,
    lang: Option<locale::Language>,
) -> std::io::Result<()> {
    let is_language_dir = lang.is_some();
    let gamedir = if is_language_dir {
        gamedir.as_ref().to_path_buf()
    } else {
        gamedir
            .as_ref()
            .to_path_buf()
            .join(lang.unwrap().to_string())
    };

    for sp in FS_SEARCHPATHS.read().unwrap().iter() {
        if let Some(dir) = &sp.qdir.dir() &&
            *dir.path.as_path() == *base.as_ref() && dir.gamedir == gamedir
        {
            if sp.is_localized() == is_language_dir {
                let s = if sp.is_localized() {
                    "localized"
                } else {
                    "non-localized"
                };
                com::warnln!(
                    10.into(),
                    "WARNING: game folder {}/{} added as both localized & non-localized. Using folder as {}",
                    base.as_ref().to_string_lossy(),
                    gamedir.to_string_lossy(),
                    s
                );
            }
            if sp.is_localized() && sp.language != lang {
                com::warnln!(
                    10.into(),
                    "WARNING: game folder {}/{} re-added as localized folder with different language", 
                    base.as_ref().to_string_lossy(),
                    gamedir.to_string_lossy()
                );
            }
            return Err(std::io::ErrorKind::Other.into());
        }
    }

    if is_language_dir {
        *FS_GAMEDIR.write().unwrap() = gamedir.clone();
    } else {
        let dir = build_os_path(&base, Some(&gamedir), "");
        if !sys::directory_has_contents(dir) {
            return Err(std::io::ErrorKind::Other.into());
        }
    }

    let dir = Directory {
        path: base.as_ref().to_path_buf(),
        gamedir: gamedir.clone(),
    };

    let (ignore, ignore_pure_check) =
        if dvar::get_bool("fs_usedevdir").unwrap() == false {
            let ignore = gamedir == PathBuf::from_str("main").unwrap();
            let ignore_pure_check = gamedir
                == PathBuf::from_str("players").unwrap()
                || gamedir == PathBuf::from_str("demos").unwrap();
            (ignore, ignore_pure_check)
        } else {
            (false, true)
        };

    let sp = Searchpath {
        language: lang,
        qdir: Qdir::Dir { dir },
        ignore,
        ignore_pure_check,
    };

    add_searchpath(sp);

    add_iwd_files_for_game_directory(base, gamedir)
}

fn pure_ignore_files(filename: impl AsRef<Path>) -> bool {
    if filename.as_ref() == Path::new("ban.txt") {
        true
    } else {
        if let Some(ext) = filename.as_ref().extension() {
            if ext == OsStr::new("cfg") {
                true
            } else if ext == OsStr::new(".dm_NETWORK_PROTOCOL_VERSION") {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

// TODO - implement
fn iwd_is_pure(_iwd: &Iwd) -> bool {
    true
}

fn add_iwd_pure_check_reference(_sp: &Searchpath) {}

fn files_are_loaded_globally(filename: impl AsRef<Path>) -> bool {
    const EXTS: [&'static str; 7] = [
        ".hlsl",
        ".txt",
        ".cfg",
        ".levelshots",
        ".menu",
        ".arena",
        ".str",
    ];

    if let Some(ext) = filename.as_ref().extension() {
        EXTS.iter().find(|&e| e == &ext).is_some()
    } else {
        false
    }
}

pub fn file_get_file_size(file: &std::fs::File) -> std::io::Result<u64> {
    Ok(file.metadata()?.len())
}

pub fn file_read(
    file: &mut impl Read,
    data: &mut [u8],
) -> std::io::Result<usize> {
    file.read(data)
}

fn file_write(file: &mut impl Write, data: &[u8]) -> std::io::Result<usize> {
    file.write(data)
}

lazy_static! {
    static ref FS_NUM_SERVER_IWDS: AtomicUsize = AtomicUsize::new(0);
    static ref FS_FAKE_CHK_SUM: AtomicUsize = AtomicUsize::new(0);
}

fn open_file_read_for_thread(
    filename: impl AsRef<Path>,
    thread: Thread,
) -> Result<(Fd, u64), std::io::Error> {
    let mut b = false;
    let impure_iwd = false;

    let fd = handle_for_file(thread)?;

    for sp in FS_SEARCHPATHS.read().unwrap().iter() {
        if !use_searchpath(sp) {
            continue;
        }

        match &sp.qdir {
            Qdir::Dir { dir } => {
                if (sp.ignore == false
                    && dvar::get_bool("fs_restrict").unwrap() == false
                    && FS_NUM_SERVER_IWDS.load_relaxed() == 0)
                    || (sp.is_localized()
                        || sp.ignore_pure_check
                        || pure_ignore_files(&filename))
                {
                    let ospath = build_os_path_for_thread(
                        &dir.path,
                        Some(&dir.gamedir),
                        &filename,
                        thread,
                    );
                    let file = file_open_read(&ospath)?;
                    let fh = FileHandleData {
                        file: Qfile::File {
                            file,
                            name: filename.as_ref().to_path_buf(),
                        },
                        handle_sync: false,
                        file_size: 0,
                        streamed: false,
                    };
                    FSH.write().unwrap()[fd.as_usize()] = Some(fh);
                    // fake check sum
                    if dvar::get_int("fs_debug").unwrap() != 0 {
                        com::println!(
                            10.into(),
                            "fs::open_file_read from thread '{}', handle \
                             '{}', {} (found in '{}/{}')",
                            sys::get_current_thread_name(),
                            fd.as_usize(),
                            filename.as_ref().to_string_lossy(),
                            dir.path.to_string_lossy(),
                            dir.gamedir.to_string_lossy()
                        );
                    }

                    if dvar::get_bool("fs_copyfiles").unwrap() == true
                        && dir.path
                            == PathBuf::from_str(
                                &dvar::get_string("fs_cdpath").unwrap(),
                            )
                            .unwrap()
                    {
                        let ospath_dest = build_os_path_for_thread(
                            dvar::get_string("fs_basepath").unwrap(),
                            Some(&dir.gamedir),
                            &filename,
                            thread,
                        );
                        let size = copy_file(ospath, ospath_dest)?;
                        return Ok((fd, size));
                    }
                } else if b == false {
                    let ospath = build_os_path_for_thread(
                        &dir.path,
                        Some(&dir.gamedir),
                        &filename,
                        thread,
                    );
                    b = file_open_read(ospath).is_ok();
                }
            }
            Qdir::Iwd { iwd, iwd_name } => {
                let mut archive = iwd.write().unwrap();
                if let Ok(zip_file) =
                    archive.by_name(&filename.as_ref().to_string_lossy())
                {
                    let archive = iwd.clone();
                    let handle_sync = false;
                    let file_size = zip_file.size();
                    let streamed = false;
                    let name = filename.as_ref().to_path_buf();
                    let fh = FileHandleData {
                        file: Qfile::ZipFile { archive, name },
                        handle_sync,
                        file_size: file_size as _,
                        streamed,
                    };
                    FSH.write().unwrap()[fd.as_usize()] = Some(fh);

                    if dvar::get_int("fs_debug").unwrap() != 0 {
                        com::println!(
                            10.into(),
                            "fs::open_file_read from thread '{}', handle \
                             '{}', {} (found in '{}')",
                            sys::get_current_thread_name(),
                            fd.as_usize(),
                            filename.as_ref().to_string_lossy(),
                            iwd_name.to_string_lossy()
                        );
                    }

                    return Ok((fd, file_size));
                };
            }
        };
    }

    if dvar::get_int("fs_debug").unwrap() != 0 && thread == Thread::Main {
        com::println!(
            10.into(),
            "Can't find {}",
            filename.as_ref().to_string_lossy()
        );
    }

    if impure_iwd {
        com::errorln!(
            com::ErrorParm::DROP,
            "Impure client detected. Invalid .IWD files referenced!\n{}",
            FSH.read().unwrap()[fd.as_usize()]
                .as_ref()
                .unwrap()
                .name()
                .to_string_lossy()
        );
    }

    if b == false {
        return Err(std::io::ErrorKind::Other.into());
    }

    if FS_NUM_SERVER_IWDS.load_relaxed() == 0
        && dvar::get_bool("fs_restrict").unwrap() == false
    {
        com::println!(
            10.into(),
            "Error: {} must be in an IWD or not in the main directory",
            filename.as_ref().to_string_lossy()
        );
        return Err(std::io::ErrorKind::Other.into());
    } else {
        com::println!(
            10.into(),
            "Error: {} must be in an IWD",
            filename.as_ref().to_string_lossy()
        );
        return Err(std::io::ErrorKind::Other.into());
    }
}

pub fn open_file_read_current_thread(
    filename: impl AsRef<Path>,
) -> Result<(Fd, u64), std::io::Error> {
    if let Some(thread) = get_current_thread() {
        open_file_read_for_thread(filename, thread)
    } else {
        com::print_errorln!(
            1.into(),
            "fs::open_file_read_current_thread for an unknown thread"
        );
        Err(std::io::ErrorKind::Other.into())
    }
}

pub fn open_file_read(
    filename: impl AsRef<Path>,
) -> std::io::Result<(Fd, u64)> {
    com::file_accessed().store_relaxed(1);
    open_file_read_current_thread(filename)
}

pub fn open_file_append(filename: impl AsRef<Path>) -> std::io::Result<Fd> {
    let ospath = build_os_path(
        dvar::get_string("fs_homepath").unwrap(),
        Some(&*FS_GAMEDIR.read().unwrap()),
        &filename,
    );
    if dvar::get_int("fs_debug").unwrap() != 0 {
        com::println!(
            10.into(),
            "fs::open_file_append: {}",
            ospath.to_string_lossy()
        );
    }
    let path = create_path(ospath)?;
    let file = file_open_append(path)?;
    let fd = handle_for_file_current_thread()?;

    let mut fsh = FSH.write().unwrap();
    let fh = &mut fsh[fd.as_usize()];
    *fh = Some(FileHandleData {
        file: Qfile::File {
            file,
            name: filename.as_ref().to_path_buf(),
        },
        handle_sync: false,
        file_size: 0,
        streamed: false,
    });

    Ok(fd)
}

fn file_open_read(
    filename: impl AsRef<Path>,
) -> std::io::Result<std::fs::File> {
    std::fs::File::options()
        .read(true)
        .create(false)
        .open(filename)
}

fn file_open_write(
    filename: impl AsRef<Path>,
) -> std::io::Result<std::fs::File> {
    std::fs::File::options()
        .write(true)
        .create(true)
        .open(filename)
}

fn file_open_append(
    filename: impl AsRef<Path>,
) -> std::io::Result<std::fs::File> {
    std::fs::File::options()
        .append(true)
        .create(true)
        .open(filename)
}

fn get_handle_and_open_file(
    qpath: impl AsRef<Path>,
    filename: impl AsRef<Path>,
    thread: Thread,
) -> std::io::Result<Fd> {
    let file = file_open_write(filename)?;
    let handle = handle_for_file(thread)?;

    let mut fsh = FSH.write().unwrap();
    let fh = &mut fsh[handle_for_file(thread)?.0 as usize];
    *fh = Some(FileHandleData {
        file: Qfile::File {
            file,
            name: qpath.as_ref().to_path_buf(),
        },
        handle_sync: false,
        file_size: 0,
        streamed: false,
    });

    Ok(handle)
}

fn open_file_write_to_dir_for_thread(
    qpath: impl AsRef<Path>,
    gamedir: Option<impl AsRef<Path>>,
    thread: Thread,
) -> std::io::Result<Fd> {
    let homepath = dvar::get_string("fs_homepath").unwrap();
    let ospath = build_os_path(homepath, gamedir, qpath.as_ref());
    if dvar::get_int("fs_debug").unwrap() != 0 {
        com::println!(
            10.into(),
            "fs::open_file_write_to_dir_for_thread: {}",
            ospath.clone().to_string_lossy()
        );
    }

    if let Err(e) = create_path(&ospath) {
        Err(e)
    } else {
        get_handle_and_open_file(qpath, ospath, thread)
    }
}

pub fn open_file_write(filename: impl AsRef<Path>) -> std::io::Result<Fd> {
    open_file_write_to_dir_for_thread(
        filename,
        Some(&*FS_GAMEDIR.read().unwrap()),
        Thread::Main,
    )
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Read,
    Write,
    Append,
    AppendSync,
}

pub fn open_file_by_mode(
    filename: impl AsRef<Path>,
    mode: Mode,
) -> std::io::Result<(Fd, usize)> {
    let r = match mode {
        Mode::Read => {
            open_file_read(filename).map(|(fd, size)| (fd, size as usize))
        }
        Mode::Write => open_file_write(filename).map(|fd| (fd, 0usize)),
        Mode::Append | Mode::AppendSync => {
            open_file_append(filename).map(|fd| (fd, 0usize))
        }
    };

    if let Ok((ref fd, file_size)) = r {
        let mut fsh = FSH.write().unwrap();
        let fh = fsh[fd.as_usize()].as_mut().unwrap();
        fh.file_size = file_size;
        fh.streamed = false;
        fh.handle_sync = mode == Mode::AppendSync;
    }

    r
}

pub fn read(fd: &Fd) -> Result<Vec<u8>, std::io::Error> {
    let mut buf = Vec::new();
    FSH.write().unwrap()[fd.as_usize()]
        .as_mut()
        .unwrap()
        .file
        .read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn write(fd: &Fd, data: &[u8]) -> std::io::Result<()> {
    let mut fsh = FSH.write().unwrap();
    let fh = &mut fsh[fd.as_usize()].as_mut().unwrap();
    match fh.file {
        Qfile::ZipFile { .. } => {
            Err(std::io::ErrorKind::InvalidFilename.into())
        }
        Qfile::File { ref mut file, .. } => {
            let r = file.write_all(data);

            if fh.handle_sync {
                file.flush()?;
            }

            r
        }
    }
}

static FS_LOADSTACK: AtomicUsize = AtomicUsize::new(0);

pub fn read_file(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    if path == Path::new("") {
        com::errorln!(
            com::ErrorParm::FATAL,
            "\x15fs::read_file with empty name"
        );
    }
    let (fd, _) = open_file_read_current_thread(path)?;
    FS_LOADSTACK.increment_wrapping();
    read(&fd)
}

pub fn write_file(path: impl AsRef<Path>, data: &[u8]) -> std::io::Result<()> {
    assert_ne!(path.as_ref(), Path::new(""));

    if let Ok(fd) = open_file_write(&path) {
        let r = write(&fd, data);
        if r.is_err() {
            delete(&path)?;
        }

        r
    } else {
        com::println!(
            10.into(),
            "Failed to open {}",
            path.as_ref().to_string_lossy()
        );
        Err(std::io::ErrorKind::NotFound.into())
    }
}

pub fn copy_file(
    src: impl AsRef<Path>,
    dest: impl AsRef<Path>,
) -> std::io::Result<u64> {
    let mut src = file_open_read(src)?;
    let src_size = file_get_file_size(&src)?;
    let mut data = Vec::with_capacity(src_size as _);
    let bytes_read = file_read(&mut src, &mut data)?;
    if bytes_read as u64 != src_size {
        com::errorln!(
            com::ErrorParm::FATAL,
            "\x15Short read in fs::copy_file()"
        );
    }

    create_path(&dest)?;
    let mut dst = file_open_write(&dest)?;
    let bytes_written = file_write(&mut dst, &data)?;
    if bytes_written as u64 != src_size {
        com::errorln!(
            com::ErrorParm::FATAL,
            "\x15Short write in fs::copy_file()"
        );
    }
    Ok(src_size)
}
