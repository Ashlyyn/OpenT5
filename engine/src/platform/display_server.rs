// Windows has only one option for a display server.
//
// Linux can use Xlib or Wayland (default is Xlib).
//
// macOS can use Xlib or AppKit (default is Xlib).
//
// Other Unix and Unix-like OSes will only use Xlib
// (maybe add Wayland support for them later).
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
    } else if #[cfg(wayland)] {
        pub mod wayland;
        pub use wayland as target;
    } else if #[cfg(appkit)] {
        pub mod appkit;
        pub use appkit as target;
    } else if #[cfg(xlib)] {
        pub mod xlib;
        pub use xlib as target;
    }
}
