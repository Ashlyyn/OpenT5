// This file exports the platform-specific modules as module "target"
// to allow for easy execution in main()

#![allow(clippy::pub_use)]

mod linux;
mod macos;

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
