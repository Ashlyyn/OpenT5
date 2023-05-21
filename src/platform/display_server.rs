// Windows has only one option for a display server.
//
// Linux can use Xlib or Wayland (default is Xlib).
//
// macOS can use Xlib or AppKit (default is Xlib).
//
// Other Unix and Unix-like OSes will only use Xlib.
//
// Other display servers might be supported in the future 
// (e.g. Redox's Orbital), but they definitely aren't supported right now.
//
// We'll also have to figure something out for wasm.

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
