#![allow(dead_code)]

use lazy_static::lazy_static;
use std::sync::RwLock;
use std::fs::File;

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
    static ref LOG_FILE: RwLock<Option<File>> = RwLock::new(None);
}

pub fn log_file_open() -> bool {
    return LOG_FILE.read().unwrap().is_none();
}

pub fn error(err: String) {
    panic!("{}", err);
}

pub fn errorln(err: String) {
    error(format!("{}\n", err));
}