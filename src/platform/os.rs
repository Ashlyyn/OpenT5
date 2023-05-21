// Windows, Linux, macOS, other Unix and Unix-like OSes (e.g., the BSDs) are 
// currently supported, in addition to wasm. Other OSes might be supported in 
// the future, but these are the ones we're supporting right now.

#![allow(clippy::pub_use)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(windows)] {
        pub mod win32;
        pub use win32 as target;
    } else if #[cfg(target_os = "linux")] {
        pub mod linux;
        pub use linux as target;
    } else if #[cfg(target_os = "macos")] {
        pub mod macos;
        pub use macos as target;
    } else if #[cfg(unix)] {
        pub mod other_unix;
        pub use other_unix as target;
    } else if #[cfg(target_arch = "wasm32")] {
        pub mod none;
        pub use none as target;
    } else {
        pub mod other;
        pub use other as target;
    }
}

pub fn main() {
    target::main();
}
