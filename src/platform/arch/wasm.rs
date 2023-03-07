use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub mod wasm32;
        pub use wasm32 as target;
    } else if #[cfg(target_arch = "wasm64")] {
        pub mod wasm64;
        pub use wasm64 as target;
    }
}

pub const fn main() {
    target::main();
}