#![allow(non_snake_case, clippy::bool_comparison)]

use lazy_static::lazy_static;
use std::sync::{atomic::AtomicBool, Arc, RwLock};

mod cmd;
mod com;
mod common;
mod dvar;
mod gfx;
mod locale;
mod platform;
mod pmem;
mod sys;

lazy_static! {
    #[allow(dead_code)]
    static ref G_ALLOW_MATURE: AtomicBool = AtomicBool::new(true);
    static ref SYS_CMDLINE: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));
}

fn main() {
    platform::os::target::main();

    let mut cmd_line: String = String::new();
    for arg in std::env::args() {
        cmd_line.push_str(&format!("{} ", &arg));
    }
    cmd_line = cmd_line.trim().to_string();

    {
        let lock = SYS_CMDLINE.clone();
        let mut writer = lock.write().unwrap();
        writer.clear();
        writer.insert_str(0, &cmd_line);
    }

    pmem::init();
    dvar::init();
    dvar::register_string(
        "test".to_string(),
        "abcd".to_string(),
        dvar::DvarFlags::empty(),
        "Testing 123...".to_string(),
    );
    let d = dvar::find("test".to_string()).unwrap();
    com::println(format!("{}", d));
}
