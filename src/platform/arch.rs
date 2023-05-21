// There's only a slight bit of ISA-specific code currently present in the
// codebase. We might have some odd issues pop up in regards to pointer size,
// endianness, etc., things we can't account for by only (currently) testing 
// one ISA on three OSes, but they should be minor. 

#![allow(clippy::pub_use)]
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(any(target_arch = "x86", target_arch = "x86_64"))] {
        pub mod x86;
        pub use x86 as target;
    } else if #[cfg(any(target_arch = "wasm32", target_arch = "wasm64"))] {
        pub mod wasm;
        pub use wasm as target;
    } else {
        pub mod other;
        pub use other as target;
    }
}

pub const fn main() {
    target::main();
}
