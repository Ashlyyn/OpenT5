#![allow(dead_code)]

// This file exists to abstract filesystem-related functionalities

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        use windows::Win32::{
            UI::Shell::{
                SHGetFolderPathA, CSIDL_LOCAL_APPDATA, CSIDL_FLAG_CREATE, CSIDL_MYDOCUMENTS,
                CSIDL_PROFILE, SHGFP_TYPE_CURRENT,
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
        pub fn get_os_folder_path(os_folder: OsFolder) -> String {
            let csidl: u32 = match os_folder {
                OsFolder::UserData => CSIDL_LOCAL_APPDATA,
                OsFolder::UserConfig => CSIDL_LOCAL_APPDATA,
                OsFolder::Documents => CSIDL_MYDOCUMENTS,
                OsFolder::Home => CSIDL_PROFILE,
            };

            let mut buf: [u8; MAX_PATH as usize] = [0; MAX_PATH as usize];
            unsafe {
                SHGetFolderPathA(
                    None,
                    (csidl | CSIDL_FLAG_CREATE) as _,
                    None,
                    SHGFP_TYPE_CURRENT.0 as _,
                    &mut buf,
                )
                .unwrap()
            };
            let c = CStr::from_bytes_until_nul(&buf).unwrap();
            c.to_str().unwrap().to_string()
        }
    } else {
        pub fn get_os_folder_path(os_folder: OsFolder) -> String {
            let envar = match os_folder {
                OsFolder::UserData => "XDG_DATA_HOME",
                OsFolder::UserConfig => "XDG_CONFIG_HOME",
                OsFolder::Documents => "XDG_DOCUMENTS_DIR",
                OsFolder::Home => "HOME",
            };

            let home = std::env::var("HOME")
                .expect("sys::get_os_folder_path: envar \"HOME\" not set.");

            let envar_default = match os_folder {
                OsFolder::UserData => format!("{}/.local/share", home),
                OsFolder::UserConfig => format!("{}/.config", home),
                OsFolder::Documents => format!("{}/Documents", home),
                OsFolder::Home => home,
            };

            match std::env::var(envar) {
                Ok(s) => s,
                Err(_) => envar_default,
            }
        }
    }
}
