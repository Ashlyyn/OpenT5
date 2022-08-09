#![allow(dead_code)]

use lazy_static::lazy_static;
use std::fs::File;
use std::sync::{Arc, RwLock};

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

pub fn log_file_open() -> bool {
    return LOG_FILE.clone().read().unwrap().is_some();
}

pub fn error(err: String) {
    panic!("{}", err);
}

pub fn errorln(err: String) {
    error(format!("{}\n", err));
}

#[allow(unused_variables, unreachable_code)]
pub fn filter(string: String, name: String, case_sensitive: bool) -> bool {
    return true;

    todo!("com::filter");
}

pub fn dvar_dump(_channel: i32, _param_2: String) {
    todo!("com::dvar_dump");
}
