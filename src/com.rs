#![allow(dead_code)]

use lazy_static::lazy_static;
use std::fs::File;
use std::sync::{Arc, RwLock};

// Needs to be actually implemented
// Just a wrapper around print! currently
pub fn print(message: String) {
    print!("{}", message);
}

pub fn println(message: String) {
    print(format!("{}\n", message));
}

pub fn print_warning(message: String) {
    print(format!("^3{}", message));
}

lazy_static! {
    static ref LOG_FILE: Arc<RwLock<Option<File>>> =
        Arc::new(RwLock::new(None));
}

// Check if log file is open
pub fn log_file_open() -> bool {
    return LOG_FILE.clone().read().unwrap().is_some();
}

// Also needs to be actually implemented
// Currently just a wrapper for panic
pub fn error(err: String) {
    panic!("{}", err);
}

pub fn errorln(err: String) {
    error(format!("{}\n", err));
}

// Implement these two later
// (not integral to the program, look annoying to implement)
#[allow(unused_variables, unreachable_code)]
pub fn filter(string: String, name: String, case_sensitive: bool) -> bool {
    return true;

    todo!("com::filter");
}

#[allow(unused_variables, unreachable_code)]
pub fn dvar_dump(channel: i32, param_2: String) {
    return;

    todo!("com::dvar_dump");
}
