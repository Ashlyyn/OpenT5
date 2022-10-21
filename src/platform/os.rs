// This file exports the platform-specific modules as module "target"
// to allow for easy execution in main()

#[cfg(target_os = "windows")]
pub mod win32;
#[cfg(target_os = "windows")]
pub use win32 as target;

#[cfg(target_os = "unix")]
pub mod unix;
#[cfg(target_os = "unix")]
pub use unix as target;

#[cfg(target_os = "linux")]
pub mod unix;
#[cfg(target_os = "linux")]
pub use unix as target;
