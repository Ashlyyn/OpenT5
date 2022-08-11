#![allow(dead_code)]

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::MAX_PATH;
#[cfg(target_os = "windows")]
use windows::Win32::UI::Shell::{
    SHGetFolderPathA, CSIDL_APPDATA, CSIDL_FLAG_CREATE, CSIDL_MYDOCUMENTS,
    CSIDL_PROFILE,
};

pub enum OsFolder {
    UserConfig,
    UserData,
    Documents,
    Home,
}

#[cfg(target_os = "windows")]
pub fn get_os_folder_path(os_folder: OsFolder) -> String {
    let csidl: u32 = match os_folder {
        OsFolder::UserData => CSIDL_APPDATA,
        OsFolder::UserConfig => CSIDL_APPDATA,
        OsFolder::Documents => CSIDL_MYDOCUMENTS,
        OsFolder::Home => CSIDL_PROFILE,
    };

    let mut buf: [u8; MAX_PATH as usize] = [0; MAX_PATH as usize];
    unsafe {
        SHGetFolderPathA(
            None,
            (csidl | CSIDL_FLAG_CREATE) as i32,
            None,
            0,
            &mut buf,
        )
        .unwrap()
    };
    String::from_utf8(buf.to_vec()).unwrap()
}

#[cfg(not(target_os = "windows"))]
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
        OsFolder::Home => format!("{}", home),
    };

    match std::env::var(envar) {
        Ok(s) => s,
        Err(_) => envar_default,
    }
}
