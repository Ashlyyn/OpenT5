#![allow(non_snake_case, clippy::bool_comparison)]

use std::sync::atomic::AtomicBool;

mod cmd;
mod com;
mod common;
mod dvar;
mod gfx;
mod locale;
mod platform;
mod pmem;
mod sys;

#[allow(dead_code)]
static G_ALLOW_MATURE: AtomicBool = AtomicBool::new(true);

fn main() {
    platform::os::target::main();
    dvar::register_string(
        "test".to_string(),
        "abcd".to_string(),
        dvar::DvarFlags::empty(),
        "Testing 123...".to_string(),
    );
    let d = dvar::DVARS
        .read()
        .unwrap()
        .get(&"test".to_string())
        .unwrap()
        .clone();
    com::println(format!("{}", d));
}
