#![allow(dead_code)]

use crate::*;
use arrayvec::ArrayVec;
use lazy_static::lazy_static;
use std::fs::File;
use std::sync::{Arc, RwLock};

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum ErrorParm {
    FATAL,
    DROP,
    SERVERDISCONNECT,
    DISCONNECT,
    SCRIPT,
    SCRIPT_DROP,
    LOCALIZATION,
}

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum ParseTokenType {
    UNKNOWN,
    NUMBER,
    STRING,
    NAME,
    HASH,
    PUNCTUATION,
}

struct ParseInfo {
    token: String,
    token_type: ParseTokenType,
    lines: i32,
    unget_token: bool,
    space_delimited: bool,
    keep_string_quotes: bool,
    csv: bool,
    negative_numbers: bool,
    error_prefix: String,
    warning_prefix: String,
    backup_lines: i32,
    backup_text: String,
    parse_file: String,
}

impl ParseInfo {
    fn new() -> Self {
        ParseInfo {
            token: "".to_string(),
            token_type: ParseTokenType::UNKNOWN,
            lines: 1,
            unget_token: false,
            space_delimited: true,
            keep_string_quotes: false,
            csv: false,
            negative_numbers: false,
            error_prefix: "".to_string(),
            warning_prefix: "".to_string(),
            backup_lines: 0,
            backup_text: String::new(),
            parse_file: String::new(),
        }
    }
}

struct ParseThreadInfo {
    parse_info: ArrayVec<ParseInfo, 16>,
    parse_info_num: isize,
    token_pos: isize,
    prev_token_pos: isize,
    line: String,
}

impl ParseThreadInfo {
    fn new() -> Self {
        ParseThreadInfo {
            parse_info: ArrayVec::new(),
            parse_info_num: 0,
            token_pos: 0,
            prev_token_pos: 0,
            line: String::new(),
        }
    }
}

// Needs to be actually implemented
// Just a wrapper around print! currently
pub fn print(message: String) {
    print!("{}", message);
}

pub fn println(message: &str) {
    print(format!("{}\n", message));
}

pub fn warn(message: &str) {
    print(format!("^3{}", message));
}

pub fn warnln(message: &str) {
    warn(&format!("{}\n", message));
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
pub fn error(err_type: ErrorParm, err: &str) {
    panic!("{} ({:?})", err, err_type);
}

pub fn errorln(err_type: ErrorParm, err: &str) {
    error(err_type, &format!("{}\n", err));
}

// Implement these two later
// (not integral to the program, look annoying to implement)
#[allow(unused_variables, unreachable_code)]
pub fn filter(string: &str, name: &str, case_sensitive: bool) -> bool {
    return true;

    todo!("com::filter");
}

#[allow(unused_variables, unreachable_code)]
pub fn dvar_dump(channel: i32, param_2: String) {
    return;

    todo!("com::dvar_dump");
}

lazy_static! {
    static ref G_PARSE: Arc<RwLock<ArrayVec<ParseThreadInfo, 16>>> =
        Arc::new(RwLock::new(ArrayVec::new()));
}

pub fn get_official_build_name_r() -> String {
    "Call of Duty: BlackOps".to_string()
}

pub fn init() {
    init_try_block_function();
}

fn init_try_block_function() {
    render::init_threads();
}

pub fn touch_memory() {
    todo!("com::touch_memory");
}

// TODO - implement
pub fn get_icon_rgba() -> Option<winit::window::Icon> {
    None
}

// TODO - implement
pub fn startup_variable(name: &str) {
    println(&format!("com::startup_variable: {}", name));
}

lazy_static! {
    static ref SAFE_MODE: AtomicBool = AtomicBool::new(false);
}

pub fn safe_mode() -> bool {
    SAFE_MODE.load(Ordering::SeqCst)
}

pub fn force_safe_mode() {
    SAFE_MODE.store(true, Ordering::SeqCst);
}
