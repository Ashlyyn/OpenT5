// This file exports the platform-specific modules as module "target"
// to allow for easy execution in main()

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_family = "windows")] {
        pub mod win32;
        pub use win32 as target;
    } else if #[cfg(target_family = "unix")] {
        pub mod unix;
        pub use unix as target;
    }
}
