#![allow(dead_code)]

// This file exists to abstract filesystem-related functionalities

use crate::{util::EasierAtomic, *};
use cfg_if::cfg_if;
use core::{str::FromStr, sync::atomic::AtomicUsize};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

cfg_if! {
    if #[cfg(target_os = "windows")] {
        use windows::Win32::{
            UI::Shell::{
                SHGetFolderPathA, CSIDL_LOCAL_APPDATA, CSIDL_FLAG_CREATE,
                CSIDL_MYDOCUMENTS, CSIDL_PROFILE, SHGFP_TYPE_CURRENT,
            },
            Foundation::MAX_PATH};
            use core::ffi::CStr;
            use core::mem::transmute;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum OsFolder {
    UserConfig,
    UserData,
    Documents,
    Home,
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        // TODO - will panic if folder path contains invalid UTF-8 characters.
        // Fix later.
        #[allow(
            clippy::indexing_slicing,
            clippy::multiple_unsafe_ops_per_block
        )]
        pub fn get_os_folder_path(os_folder: OsFolder) -> Option<String> {
            let csidl: u32 = match os_folder {
                OsFolder::UserData
                    | OsFolder::UserConfig => CSIDL_LOCAL_APPDATA,
                OsFolder::Documents => CSIDL_MYDOCUMENTS,
                OsFolder::Home => CSIDL_PROFILE,
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
                    let Ok(c) = CStr::from_bytes_until_nul(&buf) else {
                        return None
                    };
                    Some(c.to_str().unwrap().to_owned())
                },
                Err(_) => None,
            }
        }
    } else if #[cfg(target_family = "unix")] {
        #[allow(clippy::needless_pass_by_value)]
        pub fn get_os_folder_path(os_folder: OsFolder) -> Option<String> {
            let envar = match os_folder {
                OsFolder::UserData => "XDG_DATA_HOME",
                OsFolder::UserConfig => "XDG_CONFIG_HOME",
                OsFolder::Documents => "XDG_DOCUMENTS_DIR",
                OsFolder::Home => "HOME",
            };

            let Ok(home) = std::env::var("HOME") else { return None };

            let envar_default = match os_folder {
                OsFolder::UserData => format!("{}/.local/share", home),
                OsFolder::UserConfig => format!("{}/.config", home),
                OsFolder::Documents => format!("{}/Documents", home),
                OsFolder::Home => home,
            };

            Some(std::env::var(envar).map_or(envar_default, |s| s))
        }
    } else {
        pub fn get_os_folder_path(os_folder: OsFolder) -> Option<String> {
            compile_error!(
                "get_os_folder_path unimplemented for OS or arch",
            );
        }
    }
}

// TODO - Will panic if `path` contains invalid UTF-8 characters.
// Fix at some point.
pub fn create_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, std::io::Error> {
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
        return Ok(PathBuf::from_str(path.to_str().unwrap()).unwrap());
    }

    if std::fs::File::create(path).is_ok() {
        Ok(PathBuf::from_str(path.to_str().unwrap()).unwrap())
    } else {
        let Some(dir_path) = path.parent() else {
            return Err(std::io::ErrorKind::InvalidFilename.into())
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
pub enum Thread {
    Main,
    Stream,
    Database,
    Backend,
    Server,
    Invalid,
}

// TODO - implement
pub const fn get_current_thread() -> Thread {
    Thread::Main
}

// TODO - implement
pub fn open_file_read_current_thread(
    path: &Path,
) -> Result<std::fs::File, std::io::Error> {
    let current_thread = get_current_thread();
    if current_thread == Thread::Invalid {
        com::print_errorln!(
            1.into(),
            "fs::open_file_read_current_thread for an unknown thread"
        );
        Err(std::io::ErrorKind::Other.into())
    } else {
        std::fs::File::open(path)
    }
}

static FS_LOADSTACK: AtomicUsize = AtomicUsize::new(0);

#[allow(clippy::verbose_file_reads)]
pub fn read_file(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    let mut f = open_file_read_current_thread(path)?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    FS_LOADSTACK.increment_wrapping();
    Ok(buf)
}

// TODO - implement
pub fn delete(path: &Path) -> Result<(), std::io::Error> {
    std::fs::remove_file(path)
}

// TODO - correctly implement
pub fn write_file(path: &Path, data: &[u8]) -> Result<usize, std::io::Error> {
    let Ok(mut file) = std::fs::File::create(path) else {
        com::println!(10.into(), "Failed to open {}", path.display());
        return Err(std::io::ErrorKind::NotFound.into());
    };

    let count = file.write(data)?;
    if count != data.len() {
        Err(match delete(path) {
            Ok(_) => std::io::ErrorKind::Other.into(),
            Err(e) => e,
        })
    } else {
        Ok(count)
    }
}
