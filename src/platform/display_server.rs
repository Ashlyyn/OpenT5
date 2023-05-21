use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(windows)] {
        pub mod win32;
        pub use win32 as target;
    } else if #[cfg(all(target_os = "linux", feature = "linux_use_wayland"))] {
        pub mod wayland;
        pub use wayland as target;
    } else if #[cfg(all(target_os = "macos", feature = "macos_use_appkit"))] {
        pub mod appkit;
        pub use appkit as target;
    } else if #[cfg(unix)] {
        pub mod xlib;
        pub use xlib as target;
    }
}
