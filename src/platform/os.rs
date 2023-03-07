// This file exports the platform-specific modules as module "target"
// to allow for easy execution in main()

#![allow(clippy::pub_use)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_family = "windows")] {
        pub mod win32;
        pub use win32 as target;
    } else if #[cfg(target_family = "unix")] {
        pub mod unix;
        pub use unix as target;
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
