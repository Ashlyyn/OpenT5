#![allow(dead_code)]

// This file exists to abstract filesystem-related functionalities

use crate::*;
use std::{
    path::{Path, PathBuf},
};
use core::str::FromStr;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        use windows::Win32::{
            UI::Shell::{
                SHGetFolderPathA, CSIDL_LOCAL_APPDATA, CSIDL_FLAG_CREATE,
                CSIDL_MYDOCUMENTS, CSIDL_PROFILE, SHGFP_TYPE_CURRENT,
            },
            Foundation::MAX_PATH};
            use std::ffi::CStr;
    }
}

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
        pub fn get_os_folder_path(os_folder: OsFolder) -> Option<String> {
            let csidl: u32 = match os_folder {
                OsFolder::UserData => CSIDL_LOCAL_APPDATA,
                OsFolder::UserConfig => CSIDL_LOCAL_APPDATA,
                OsFolder::Documents => CSIDL_MYDOCUMENTS,
                OsFolder::Home => CSIDL_PROFILE,
            };

            let mut buf: [u8; MAX_PATH as usize] = [0; MAX_PATH as usize];
            match unsafe {
                SHGetFolderPathA(
                    None,
                    (csidl | CSIDL_FLAG_CREATE) as _,
                    None,
                    SHGFP_TYPE_CURRENT.0 as _,
                    &mut buf,
                )
            } {
                Ok(_) => {
                    // Null-terminate the string, in case the folder path
                    // was exactly MAX_PATH characters.
                    buf[buf.len() - 1] = 0x00;
                    let c = match CStr::from_bytes_until_nul(&buf) {
                        Ok(c) => c,
                        Err(_) => return None,
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
                "get_os_folder_path unimplemented for {} ({})",
                std::env::consts::OS,
                std::env::consts::FAMILY
            );
        }
    }
}

// TODO - Will panic if `path` contains invalid UTF-8 characters.
// Fix at some point.
pub fn create_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, std::io::Error> {
    let path = path.as_ref();

    if path.is_relative() {
        com::warnln(
            10.into(),
            &format!(
                "WARNING: refusing to create relative path \"{}\"",
                path.display()
            ),
        );
        return Err(std::io::ErrorKind::InvalidFilename.into());
    }

    if path.exists() {
        return Ok(PathBuf::from_str(path.to_str().unwrap()).unwrap());
    }

    if std::fs::File::create(path).is_ok() { 
        Ok(PathBuf::from_str(path.to_str().unwrap()).unwrap()) 
    } else {
        let Some(dir_path) = path.parent() else { return Err(std::io::ErrorKind::InvalidFilename.into()) };

            std::fs::create_dir_all(dir_path)?;

            match std::fs::File::create(path) {
                Ok(_) => Ok(path.to_path_buf()),
                Err(e) => Err(e),
            }
    }
}

struct Iwd {
    filename: String,
    basename: String,
    gamename: String,
    handle: Vec<u8>,
    checksum: usize,
    pure_checksum: usize,
    has_open_file: bool,
    num_files: usize,
    referenced: bool,
}
