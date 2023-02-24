// This file is for any Unix-specific initialization that
// should be done before the rest of main() executes

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub mod linux;
        pub use linux as target;
    } else if #[cfg(target_os = "macos")] {
        pub mod macos;
        pub use macos as target;
    }
}

pub fn main() {
    target::main();
    println!("Exiting Unix main()!");
}
