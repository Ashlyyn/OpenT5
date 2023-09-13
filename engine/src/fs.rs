#![allow(dead_code)]

// This file exists to abstract filesystem-related functionalities

use crate::{
    util::{EasierAtomic, EasierAtomicBool},
    *,
};
use arrayvec::ArrayVec;
use cfg_if::cfg_if;
use core::{str::FromStr, sync::atomic::AtomicUsize};
use std::{
    ffi::OsStr,
    io::{Read, Seek, Write},
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::RwLock,
};
use zip::read::ZipArchive;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        use windows::Win32::{
            UI::Shell::{
                SHGetFolderPathW, CSIDL_LOCAL_APPDATA, CSIDL_FLAG_CREATE,
                CSIDL_PERSONAL, SHGFP_TYPE_CURRENT,
            },
            Foundation::MAX_PATH
        };
        use core::mem::transmute;
        use std::ffi::OsString;
        use std::os::windows::prelude::OsStringExt;
    }
}

#[cfg(windows)]
const MAX_PATH_LEN: usize = MAX_PATH as usize;

#[cfg(unix)]
const MAX_PATH_LEN: usize = 4096usize;

// Probably a safe value to default to.
#[cfg(not(any(windows, unix)))]
const MAX_PATH_LEN: usize = 256usize;

/// A platform-independent representation of common directories.
///
/// Currently only consists of two variants [`OsFolder::UserData`] and
/// [`OsFolder::Documents`] since that's all we'll need to use.
///
/// [`OsFolder::UserData`] corresponds to [`CSIDL_LOCAL_APPDATA`] on Windows
/// and `$XDG_DATA_HOME` on Unix and Unix-like systems (defaults to
/// `$HOME/.local/share` if `$XDG_DATA_HOME` does not exist).
///
/// [`OsFolder::Documents`] corresponds to [`CSIDL_PERSONAL`] on Windows and
/// `$XDG_DOCUMENTS_DIR` on Unix and Unix-like systems (defaults to
/// `$HOME/Documents` if `$XDG_DOCUMENTS_DIR` does not exist).

#[derive(Copy, Clone, Debug)]
pub enum OsFolder {
    /// Corresponds to [`CSIDL_LOCAL_APPDATA`] on Windows
    /// and `$XDG_DATA_HOME` on Unix and Unix-like systems (defaults to
    /// `$HOME/.local/share` if `$XDG_DATA_HOME` does not exist).
    UserData,
    /// Corresponds to [`CSIDL_PERSONAL`] on Windows
    /// and `$XDG_DOCUMENTS_DIR` on Unix and Unix-like systems (defaults to
    /// `$HOME/Documents` if `$XDG_DOCUMENTS_DIR` does not exist).
    Documents,
}

/// Retrieves the absolute path of an [`OsFolder`].
///
/// Returns [`Some`] if the path is successfully retrieved, [`None`] if not.
///
/// [`None`] should never be returned on Windows in practice (it's probably
/// technically possible though).
#[cfg(windows)]
#[allow(clippy::indexing_slicing, clippy::multiple_unsafe_ops_per_block)]
pub fn get_os_folder_path(os_folder: OsFolder) -> Option<PathBuf> {
    let csidl: u32 = match os_folder {
        OsFolder::UserData => CSIDL_LOCAL_APPDATA,
        OsFolder::Documents => CSIDL_PERSONAL,
    };

    let mut buf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    // SAFETY:
    // SHGetFolderPathW is an FFI function, requiring use of unsafe.
    // SHGetFolderPathW itself should never create UB, violate memory
    // safety, etc., provided the supplied buffer is MAX_PATH bytes
    // or more, which we've ensure it is.
    match unsafe {
        SHGetFolderPathW(
            None,
            transmute(csidl | CSIDL_FLAG_CREATE),
            None,
            transmute(SHGFP_TYPE_CURRENT.0),
            &mut buf,
        )
    } {
        Ok(_) => {
            // Null-terminate the string, in case the folder path
            // was exactly [`MAX_PATH`] characters.
            buf[buf.len() - 1] = 0x0000;
            Some(
                PathBuf::new()
                    .join(OsString::from_wide(&buf))
                    .join("Activision")
                    .join("CoD"),
            )
        }
        Err(_) => None,
    }
}

/// Retrieves the absolute path of an [`OsFolder`].
///
/// Returns [`Some`] if the path is successfully retrieved, [`None`] if not.
///
/// [`None`] should never be returned in practice, as it will only happen if
/// both the XDG dir *and* $HOME cannot be retrieved.
#[cfg(unix)]
#[allow(clippy::needless_pass_by_value)]
pub fn get_os_folder_path(os_folder: OsFolder) -> Option<PathBuf> {
    let envar = match os_folder {
        OsFolder::UserData => "XDG_DATA_HOME",
        OsFolder::Documents => "XDG_DOCUMENTS_DIR",
    };

    let Ok(home) = std::env::var("HOME") else {
        return None;
    };

    let envar_default = match os_folder {
        OsFolder::UserData => format!("{}/.local/share", home),
        OsFolder::Documents => format!("{}/Documents", home),
    };

    Some(
        PathBuf::from_str(&std::env::var(envar).map_or(envar_default, |s| s))
            .unwrap()
            .join("Activision")
            .join("CoD"),
    )
}

#[cfg(not(any(windows, unix)))]
pub fn get_os_folder_path(os_folder: OsFolder) -> Option<PathBuf> {
    compile_error!("get_os_folder_path unimplemented for OS or arch",);
}

/// Creates the specified path and all parent directories.
///
/// Essentially the functional equivalent of `mkdir -p`.
///
/// Returns a [`PathBuf`] of the created path, or [`Err`] if
/// [`std::fs::File::create`] fails.
pub fn create_path(path: impl AsRef<Path>) -> Result<PathBuf, std::io::Error> {
    let path = path.as_ref();

    if path.is_relative() {
        com::warnln!(
            console::Channel::FILES,
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

fn remove_commands() {
    cmd::remove_command("path");
    cmd::remove_command("fullpath");
    cmd::remove_command("dir");
    cmd::remove_command("fdir");
    cmd::remove_command("touchFile");
}

fn shutdown() {
    for fh in FSH.write().unwrap().iter_mut() {
        *fh = None;
    }

    FS_SEARCHPATHS.write().unwrap().clear();
    remove_commands();
}

fn set_restrictions() {
    if dvar::get_bool("fs_restrict").unwrap() {
        com::println!(
            console::Channel::FILES,
            "\nRunning in restricted demo mode.\n"
        );
        shutdown();
        startup("demomain", true);
        for sp in FS_SEARCHPATHS.read().unwrap().iter() {
            if use_searchpath(sp) {
                // checksum
            }
        }
    }
}

// TODO - fully implement

/// Initializes the filesystem.
///
/// Should be called before using any other functions from this module.
pub fn init_filesystem(dev: bool) {
    for fh in FSH.write().unwrap().iter_mut() {
        *fh = None;
    }
    startup("main", dev);
}

fn display_path(_pure: bool) {
    todo!()
}

fn path_f() {
    display_path(true);
}

fn full_path_f() {
    display_path(false);
}

fn dir_f() {
    let argc = cmd::argc();
    if argc <= 1 || argc > 4 {
        com::println!(
            console::Channel::DONT_FILTER,
            "usage: dir <directory> [extension]"
        );
    }

    todo!()
}

fn new_dir_f() {
    let argc = cmd::argc();
    if argc < 2 {
        com::println!(console::Channel::DONT_FILTER, "usage: fdir <filter>");
        com::println!(
            console::Channel::DONT_FILTER,
            "example: fdir *q3dm*.bsp"
        );
        return;
    }

    todo!()
}

fn touch_file_f() {
    let argc = cmd::argc();
    if argc == 2 {
        let file = cmd::argv(1);
        let _ = touch_file(file);
    } else {
        com::println!(console::Channel::DONT_FILTER, "Usage: touchFile <file>");
    }
}

fn add_commands() {
    cmd::add_command_internal("path", path_f).unwrap();
    cmd::add_command_internal("fullpath", full_path_f).unwrap();
    cmd::add_command_internal("dir", dir_f).unwrap();
    cmd::add_command_internal("fdir", new_dir_f).unwrap();
    cmd::add_command_internal("touchFile", touch_file_f).unwrap();
}

// TODO - fully implement
fn startup(gamedir: impl AsRef<Path>, _dev: bool) {
    com::println!(console::Channel::SYSTEM, "----- fs::startup -----");
    register_dvars();
    if dvar::get_bool("fs_usedevdir").unwrap() {
        // add dev game dirs
        if !dvar::get_string("fs_basepath").unwrap().is_empty() {}
    }
    if !dvar::get_string("fs_cdpath").unwrap().is_empty()
        && dvar::get_string("fs_basepath").unwrap()
            != dvar::get_string("fs_cdpath").unwrap()
    {
        let _ = add_localized_game_directory(
            dvar::get_string("fs_cdpath").unwrap(),
            &gamedir,
        );
    }

    let basepath = dvar::get_string("fs_basepath").unwrap();

    if !basepath.is_empty() {
        let _ = add_localized_game_directory(&basepath, "players");
        let _ = add_localized_game_directory(
            &basepath,
            format!("{}_shared", gamedir.as_ref().display()),
        );
        let _ = add_localized_game_directory(&basepath, &gamedir);
    }

    let homepath = dvar::get_string("fs_homepath").unwrap();

    if !basepath.is_empty() && homepath != basepath {
        let _ = add_localized_game_directory(
            &basepath,
            format!("{}_shared", gamedir.as_ref().display()),
        );
        let _ = add_localized_game_directory(&homepath, &gamedir);
    }

    let basegame = dvar::get_string("fs_basegame").unwrap();
    let gamedir_var = dvar::get_string("fs_gameDirVar").unwrap();

    if !basegame.is_empty()
        && gamedir.as_ref() == Path::new("main")
        && Path::new(&gamedir_var) != gamedir.as_ref()
        && !basepath.is_empty()
    {
        let _ = add_game_directory(&basepath, basegame, None);
    }

    if !gamedir_var.is_empty()
        && gamedir.as_ref() == Path::new("main")
        && Path::new(&gamedir_var) != gamedir.as_ref()
        && !basepath.is_empty()
    {
        let _ = add_game_directory(&basepath, "usermaps", None);
        let _ = add_game_directory(basepath, gamedir_var, None);
    }

    add_commands();
    path_f();
    dvar::clear_modified("fs_gameDirVar").unwrap();
    com::println!(console::Channel::FILES, "-----------------------");
    com::println!(
        console::Channel::FILES,
        "{} files in iwd files",
        FS_IWD_FILE_COUNT.load_relaxed()
    )
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

/// Representation of threads that can call functions in this module.
///
/// Threads not listed here should never call anything in this modules.
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

/// Builds an OS path for [`thread`] with the supplied parameters.
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

/// Constructs an absolute path from the given parameters.
///
/// For typical usage, [`base`] represents the root directory of the game,
/// [`gamedir`] represents the top-level subdirectory (main, players, zone,
/// mods, etc.), and [`qpath`] represents a file or subdirectory therein.
pub fn build_os_path(
    base: impl AsRef<Path>,
    gamedir: Option<impl AsRef<Path>>,
    qpath: impl AsRef<Path>,
) -> PathBuf {
    build_os_path_for_thread(base, gamedir, qpath, Thread::Main)
}

/// Deletes [`filename`].
///
/// Only works for files, not directories.
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

/// Gets the [`Thread`] of the current thread.
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
        /// Ref-counted to allow [`Qdir`] and [`Searchpath`] to use the same
        /// archive.
        archive: Arc<RwLock<Iwd>>,

        // Have to embed this here to make our implementation of Read work.
        // Can't embed the ZipFile instead, since it's only valid for as long
        // as the archive's RwLock is locked.
        /// Path associated with [`Qfile::ZipFile::archive`].
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
/// ownership, and there is no way to retieve the underlying [`std::fs::File`]
/// or [`ZipFile`] it represents.
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
    // Each thread gets a specific range/number of [`Fd`]s it can use from
    // [`FSH`].
    //
    // So first, we get that range.
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

    // And then we try to find an [`Fd`] within said range.
    for fd in fd_range.clone() {
        if FSH.read().unwrap()[fd].is_none() {
            return Ok(Fd(fd as _));
        }
    }

    com::warnln!(
        console::Channel::FILES,
        "FILE {:2}: '{}'",
        fd_range.start(),
        FSH.read().unwrap()[*fd_range.start()]
            .as_ref()
            .unwrap()
            .name()
            .display()
    );
    com::warnln!(
        console::Channel::FILES,
        "fs::handle_for_file: none free ({})",
        thread.as_i32()
    );

    for (i, f) in FSH.read().unwrap().iter().enumerate() {
        com::println!(
            console::Channel::FILES,
            "FILE {:2}: '{}'",
            i,
            f.as_ref().unwrap().name().display(),
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

/// The directory equivalent of [`Qfile`].
///
/// Just like [`Qfile`] can be either a normal file *or* a file within a zip
/// file, [`Qdir`] can be a normal directory *or* a zip file.
enum Qdir {
    Iwd {
        /// Ref-counted to allow [`Qdir`] and [`Searchpath`] to use the same
        /// archive.
        iwd: Option<Arc<RwLock<Iwd>>>,
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

    fn iwd(&self) -> Option<&Arc<RwLock<Iwd>>> {
        match self {
            Qdir::Iwd { iwd, .. } => iwd.as_ref(),
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

/// Defines a path for functions in this mddule to search within.
///
/// Functions analogously to $PATH on Unix-like systems.
struct Searchpath {
    /// The directory of the [`Searchpath`]. Can be a normal dir or an IWD.
    qdir: Qdir,
    /// Whether the [`Searchpath`] should be ignored by functions using it.
    ignore: bool,
    /// Whether files within the [`Searchpath`] should have their pure check
    /// ignored.
    ignore_pure_check: bool,
    /// The language, if any, that the [`Searchpath`] should be restricted to.
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

/// Takes ownership of the supplied [`Searchpath`] and adds it to the global
/// list of [`Searchpath`]s.
fn add_searchpath(sp: Searchpath) {
    FS_SEARCHPATHS.write().unwrap().push(sp)
}

/// Checks whether a [`Searchpath`] should be used or not.
///
/// Returns false if localization is enabled (fs_ignoreLocalized is false)
/// *and* the localization of the [`Searchpath`] is different from the current
/// locale, true otherwise.
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

static FS_IWD_FILE_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Loads a zip file using the supplied file name.
///
/// Fails if the file is not a zip file, or if some underlying operation
/// (e.g. reading) fails.
fn load_zip_file(
    filename: impl AsRef<Path>,
    _basename: impl AsRef<Path>,
) -> std::io::Result<Iwd> {
    let r = ZipArchive::new(file_open_read(filename)?)
        .map_err(|_| std::io::ErrorKind::Other.into());

    if r.is_ok() {
        FS_IWD_FILE_COUNT.increment_wrapping();
    }

    r
}

const MAX_IWD_FILES_IN_GAME_DIRECTORY: usize = 1024;

/// Attempts to parse the language from an IWD file's name.
///
/// Returns [`Some`] if it can be successfully parsed, [`None`] otherwise.
fn iwd_file_language(iwd_path: impl AsRef<Path>) -> Option<String> {
    let file_name = iwd_path.as_ref().file_name()?;

    // All valid localized IWDs' names follow the format
    // "localized_{}_iw{:02}.iwd".
    //
    // "localized_" => 10
    // "{}" >= 1
    // "_iw{:02}.iwd" => 9
    //
    // Even with a one-letter language, the name would then have to be at least
    // 20 characters long. If it's shorter than that, it can't be valid.
    if file_name.len() <= 20 {
        return None;
    }

    // We can't use string manipulation functions with OsStrs, so we need to
    // convert the file name to a String. Given how the file names must be
    // formatted, to_string_lossy shouldn't drop any characters.
    let file_name = file_name.to_string_lossy();

    if !file_name.starts_with("localized_") {
        return None;
    }

    if !file_name.ends_with(".iwd") {
        return None;
    }

    // As noted above, the "localized_" prefix is 10 characters long, and the
    // "_iw{:02}.iwd" suffix is 9 characters long, so the language string
    // will be everything in between.
    //
    // We also convert it to lowercase since [`locale::lang_from_str`] expects
    // it to be lowercase.
    let lang_str = &file_name[10..file_name.len() - 9].to_ascii_lowercase();

    let lang = locale::lang_from_str(lang_str);

    lang.map(|s| s.to_string())
}

/// Adds the IWDs in a given directory to [`FS_SEARCHPATHS`].
///
/// If the IWD is non-localized (i.e. its name follows the format
/// "iw_{:02}.iwd"), it is unconditionally added.
///
/// If the IWD is localized (i.e. its name follows the format
/// "localized_{}_iw{:02}.iwd"), it will be added if the language matches the
/// current language.
///
/// Even though it returns a [`std::io::Result`], its current implementation is
/// infallible.
fn add_iwd_files_for_game_directory(
    base: impl AsRef<Path>,
    gamedir: impl AsRef<Path>,
) -> std::io::Result<()> {
    let lang_is_austrian = dvar::get_int("loc_language").unwrap()
        == locale::Language::AUSTRIAN as i32;

    let dir = build_os_path(&base, Some(&gamedir), "");

    let mut iwds = sys::list_files(dir, "iwd", Option::<&str>::None, false);

    if iwds.len() > MAX_IWD_FILES_IN_GAME_DIRECTORY {
        com::warnln!(
            console::Channel::FILES,
            "WARNING: Exceeded max number of iwd files in {}/{} ({}/{})",
            base.as_ref().display(),
            gamedir.as_ref().display(),
            iwds.len(),
            MAX_IWD_FILES_IN_GAME_DIRECTORY
        );
        iwds.truncate(MAX_IWD_FILES_IN_GAME_DIRECTORY);
    }

    let dir_is_main = gamedir.as_ref() == Path::new("main")
        && base.as_ref()
            == Path::new(&dvar::get_string("fs_basepath").unwrap());

    // TODO - sort the iwds

    for iwd_name in iwds {
        let file_name = iwd_name.file_name().unwrap().to_string_lossy();
        if &file_name[..10] == "localized_" {
            if let Some(lang_str) = iwd_file_language(&iwd_name) {
                if let Some(lang) = locale::lang_from_str(&lang_str) {
                    let lang = if lang == locale::Language::GERMAN
                        && lang_is_austrian
                    {
                        locale::Language::AUSTRIAN
                    } else {
                        lang
                    };

                    let filename =
                        build_os_path(&base, Some(&gamedir), &iwd_name);
                    let iwd = load_zip_file(&filename, &iwd_name)
                        .ok()
                        .map(|i| Arc::new(RwLock::new(i)));
                    let sp = Searchpath {
                        ignore: false,
                        ignore_pure_check: false,
                        language: Some(lang),
                        qdir: Qdir::Iwd { iwd, iwd_name },
                    };
                    add_searchpath(sp);
                } else {
                    com::warnln!(
                        console::Channel::FILES,
                        "WARNING: Localized assets iwd file {}/{}/{} has \
                         invalid name (bad language name specified). Proper \
                         naming convention is: localized_[language]_iwd#.iwd",
                        base.as_ref().display(),
                        gamedir.as_ref().display(),
                        iwd_name.display()
                    );

                    static LANGUAGES_LISTED: AtomicBool =
                        AtomicBool::new(false);
                    if LANGUAGES_LISTED.load_relaxed() == false {
                        com::println!(
                            console::Channel::FILES,
                            "Supported languages are:"
                        );
                        for i in 0..locale::Language::CZECH.as_u8() {
                            let lang =
                                locale::Language::try_from_u8(i).unwrap();
                            com::println!(
                                console::Channel::FILES,
                                "    {}",
                                lang
                            );
                        }
                        LANGUAGES_LISTED.store_relaxed(true);
                    }
                }
            } else {
                com::warnln!(
                    console::Channel::FILES,
                    "WARNING: Localized assets iwd file {}/{}/{} has invalid \
                     name (no language specified). Proper naming convention \
                     is: localized_[language]_iwd#.iwd",
                    base.as_ref().display(),
                    gamedir.as_ref().display(),
                    iwd_name.display(),
                );
            }
        } else if dir_is_main && &file_name[0..3] != "iw_" {
            com::warnln!(
                console::Channel::FILES,
                "WARNING: Invalid IWD {} in \\main.",
                iwd_name.display()
            );
        } else {
            let filename = build_os_path(&base, Some(&gamedir), &iwd_name);
            let iwd = load_zip_file(&filename, &iwd_name)
                .ok()
                .map(|i| Arc::new(RwLock::new(i)));
            let sp = Searchpath {
                ignore: false,
                ignore_pure_check: false,
                language: None,
                qdir: Qdir::Iwd { iwd, iwd_name },
            };
            add_searchpath(sp);
        }
    }

    Ok(())
}

/// Adds a game directory to [`FS_SEARCHPATHS`], and any IWD files within.
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
                    console::Channel::FILES,
                    "WARNING: game folder {}/{} added as both localized & non-localized. Using folder as {}",
                    base.as_ref().display(),
                    gamedir.display(),
                    s
                );
            }
            if sp.is_localized() && sp.language != lang {
                com::warnln!(
                    console::Channel::FILES,
                    "WARNING: game folder {}/{} re-added as localized folder with different language", 
                    base.as_ref().display(),
                    gamedir.display()
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

fn add_localized_game_directory(
    base: impl AsRef<Path>,
    gamedir: impl AsRef<Path>,
) -> std::io::Result<()> {
    for i in 0..=locale::Language::CZECH.as_u8() {
        #[allow(unused_must_use)]
        {
            add_game_directory(
                &base,
                &gamedir,
                Some(locale::Language::try_from_u8(i).unwrap()),
            );
        }
    }
    add_game_directory(base, gamedir, None)
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

// TODO - implement
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

/// Retrieves the size of a file in bytes.
pub fn file_get_file_size(file: &mut impl Seek) -> std::io::Result<u64> {
    file.seek(std::io::SeekFrom::End(0))
}

/// Reads the contents of a file into a supplied buffer.
///
/// At most [`data.len()`] bytes will be read.
///
/// Returns the number of bytes read on success.
pub fn file_read(
    file: &mut impl Read,
    data: &mut [u8],
) -> std::io::Result<usize> {
    file.read(data)
}

/// Writes the contents of [`data`] into [`file`].
///
/// Returns the number of bytes written on success.
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
                            console::Channel::FILES,
                            "fs::open_file_read from thread '{}', handle \
                             '{}', {} (found in '{}/{}')",
                            sys::get_current_thread_name(),
                            fd.as_usize(),
                            filename.as_ref().display(),
                            dir.path.display(),
                            dir.gamedir.display()
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
                if let Some(ref iwd) = iwd {
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
                                console::Channel::FILES,
                                "fs::open_file_read from thread '{}', handle \
                                 '{}', {} (found in '{}')",
                                sys::get_current_thread_name(),
                                fd.as_usize(),
                                filename.as_ref().display(),
                                iwd_name.display()
                            );
                        }
                        return Ok((fd, file_size));
                    };
                }
            }
        };
    }

    if dvar::get_int("fs_debug").unwrap() != 0 && thread == Thread::Main {
        com::println!(
            console::Channel::FILES,
            "Can't find {}",
            filename.as_ref().display()
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
                .display()
        );
    }

    if b == false {
        return Err(std::io::ErrorKind::Other.into());
    }

    if FS_NUM_SERVER_IWDS.load_relaxed() == 0
        && dvar::get_bool("fs_restrict").unwrap() == false
    {
        com::println!(
            console::Channel::FILES,
            "Error: {} must be in an IWD or not in the main directory",
            filename.as_ref().display()
        );
        return Err(std::io::ErrorKind::Other.into());
    } else {
        com::println!(
            console::Channel::FILES,
            "Error: {} must be in an IWD",
            filename.as_ref().display()
        );
        return Err(std::io::ErrorKind::Other.into());
    }
}

/// Opens a file for reading for the calling thread.
///
/// Returns an opaque file descriptor and the file's size on success.
pub fn open_file_read_current_thread(
    filename: impl AsRef<Path>,
) -> Result<(Fd, u64), std::io::Error> {
    if let Some(thread) = get_current_thread() {
        open_file_read_for_thread(filename, thread)
    } else {
        com::print_errorln!(
            console::Channel::ERROR,
            "fs::open_file_read_current_thread for an unknown thread"
        );
        Err(std::io::ErrorKind::Other.into())
    }
}

/// Opens a file for reading.
///
/// Returns an opaque file descriptor and the file's size on success.
pub fn open_file_read(
    filename: impl AsRef<Path>,
) -> std::io::Result<(Fd, u64)> {
    com::file_accessed().store_relaxed(1);
    open_file_read_current_thread(filename)
}

/// Opens a file for appending.
///
/// Returns an opaque file descriptor on success.
pub fn open_file_append(filename: impl AsRef<Path>) -> std::io::Result<Fd> {
    let ospath = build_os_path(
        dvar::get_string("fs_homepath").unwrap(),
        Some(&*FS_GAMEDIR.read().unwrap()),
        &filename,
    );
    if dvar::get_int("fs_debug").unwrap() != 0 {
        com::println!(
            console::Channel::FILES,
            "fs::open_file_append: {}",
            ospath.display()
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

/// Opens [`filename`] in read mode.
///
/// Fails if the file does not exist.
fn file_open_read(
    filename: impl AsRef<Path>,
) -> std::io::Result<std::fs::File> {
    std::fs::File::options()
        .read(true)
        .create(false)
        .open(filename)
}

/// Opens [`filename`] in write mode.
///
/// Creates the file it it does not exist.
fn file_open_write(
    filename: impl AsRef<Path>,
) -> std::io::Result<std::fs::File> {
    std::fs::File::options()
        .write(true)
        .create(true)
        .open(filename)
}

/// Opens [`filename`] in append mode.
///
/// Creates the file if it does not exist.
fn file_open_append(
    filename: impl AsRef<Path>,
) -> std::io::Result<std::fs::File> {
    std::fs::File::options()
        .append(true)
        .create(true)
        .open(filename)
}

/// Opens [`filename`], failing if it does not exist, and returns a
/// corresponding [`Fd`].
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

/// Opens a file for [`thread`], creating it if necessary, in write mode and
/// returns a corresponding [`Fd`].
fn open_file_write_to_dir_for_thread(
    qpath: impl AsRef<Path>,
    gamedir: Option<impl AsRef<Path>>,
    thread: Thread,
) -> std::io::Result<Fd> {
    let homepath = dvar::get_string("fs_homepath").unwrap();
    let ospath = build_os_path(homepath, gamedir, qpath.as_ref());
    if dvar::get_int("fs_debug").unwrap() != 0 {
        com::println!(
            console::Channel::FILES,
            "fs::open_file_write_to_dir_for_thread: {}",
            ospath.clone().display()
        );
    }

    if let Err(e) = create_path(&ospath) {
        Err(e)
    } else {
        get_handle_and_open_file(qpath, ospath, thread)
    }
}

/// Opens a file for writing.
///
/// Returns an opaque file descriptor on success.
pub fn open_file_write(filename: impl AsRef<Path>) -> std::io::Result<Fd> {
    open_file_write_to_dir_for_thread(
        filename,
        Some(&*FS_GAMEDIR.read().unwrap()),
        Thread::Main,
    )
}

/// The mode to open a file in, used by [`open_file_by_mode`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Opens a file in read mode.
    Read,
    /// Opens a file in write mode.
    Write,
    /// Opens a file in append mode.
    Append,
    /// Opens a file in append mode and manually flushes the stream after each
    /// write.
    AppendSync,
}

/// Opens the specified file in the specified mode.
///
/// [`Mode::Read`] opens the file in read mode.
/// [`Mode::Write`] opens the file in write mode.
/// [`Mode::Append`] opens the file in append mode.
/// [`Mode::AppendSync`] opens the file in append mode and manually flushes the
/// stream after each write.
///
/// Returns an opaque file descriptor and the file's length on success.
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

/// Reads contents of the file represented by [`fd`] into [`buf`].
///
/// Reads at most [`buf.len()`] bytes.
pub fn read(fd: &Fd, buf: &mut [u8]) -> std::io::Result<usize> {
    FSH.write().unwrap()[fd.as_usize()]
        .as_mut()
        .unwrap()
        .file
        .read(buf)
}

/// Writes [`data`] into the file represented by [`fd`].
///
/// Writes at most [`data.len()`] bytes.
pub fn write(fd: &Fd, data: &[u8]) -> std::io::Result<usize> {
    let mut fsh = FSH.write().unwrap();
    let fh = &mut fsh[fd.as_usize()].as_mut().unwrap();
    match fh.file {
        Qfile::ZipFile { .. } => {
            Err(std::io::ErrorKind::InvalidFilename.into())
        }
        Qfile::File { ref mut file, .. } => {
            let r = file.write(data);

            if fh.handle_sync {
                file.flush()?;
            }

            r
        }
    }
}

/// Number of files currently opened by [`read_file`].
///
/// Incremented by  [`read_file`] and decremented when the [`ReadFile`]
/// returned by [`read_file`] is dropped.
static FS_LOADSTACK: AtomicUsize = AtomicUsize::new(0);

/// Wrapper around [`Vec<u8>`] returned by [`read_file`].
///
/// Necessary since [`read_file`] increments a global counter, and said counter
/// needs to be decremented on [`Drop`].
///
/// [`Deref`] and [`DerefMut`] are implemented, so it can be used wherever a
/// [`Vec<u8>`] can be used.
///
/// It is not, however, [`repr(transparent)`] because
/// the [`Drop`] implementation would be pointless if the data could just be
/// cloned or the data moved out of the struct.
///
/// (Also, why the hell would you be *cloning* the contents of a
/// potentially-massive file anyways?)
///
/// We can always change it later if it becomes a problem.
pub struct ReadFile(Vec<u8>);

impl Deref for ReadFile {
    type Target = Vec<u8>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ReadFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for ReadFile {
    fn drop(&mut self) {
        FS_LOADSTACK.decrement_wrapping();
    }
}

/// Reads the contents of the specified file.
///
/// Returns a buffer containing the data read from the file.
pub fn read_file(
    filename: impl AsRef<Path>,
) -> Result<ReadFile, std::io::Error> {
    if filename.as_ref() == Path::new("") {
        com::errorln!(
            com::ErrorParm::FATAL,
            "\x15fs::read_file with empty name"
        );
    }
    let (fd, file_size) = open_file_read_current_thread(filename)?;
    FS_LOADSTACK.increment_wrapping();
    let mut buf = Vec::with_capacity(file_size as _);
    read(&fd, &mut buf).map(|_| ReadFile(buf))
}

/// Writes [`data`] into the specified file.
///
/// Writes at most [`data.len()`] bytes and returns the number of bytes written
/// on success.
pub fn write_file(
    path: impl AsRef<Path>,
    data: &[u8],
) -> std::io::Result<usize> {
    assert_ne!(path.as_ref(), Path::new(""));

    if let Ok(fd) = open_file_write(&path) {
        let r = write(&fd, data);
        if r.is_err() {
            delete(&path)?;
        }

        r
    } else {
        com::println!(
            console::Channel::FILES,
            "Failed to open {}",
            path.as_ref().display()
        );
        Err(std::io::ErrorKind::NotFound.into())
    }
}

/// Copies [`src`] into [`dest`], creating [`dest`] if it doesn't already
/// exist.
///
/// Returns the number of bytes written on success.
///
/// If the copy fails, [`dest`] is deleted, rather than being left partially
/// written.
pub fn copy_file(
    src: impl AsRef<Path>,
    dest: impl AsRef<Path>,
) -> std::io::Result<u64> {
    let mut src = file_open_read(src)?;
    let src_size = file_get_file_size(&mut src)?;
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

fn touch_file(filename: impl AsRef<Path>) -> std::io::Result<bool> {
    open_file_read(filename).map(|(_, size)| size != 0xFFFF_FFFF_FFFF_FFFF)
}
