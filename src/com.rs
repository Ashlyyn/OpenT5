#![allow(dead_code)]

use crate::console::Channel;
use crate::*;
use crate::util::EasierAtomic;
use arrayvec::ArrayVec;
use lazy_static::lazy_static;
use std::fs::File;
use std::sync::Mutex;
use std::sync::RwLock;
use std::sync::atomic::AtomicUsize;
extern crate alloc;
use alloc::sync::Arc;
use cfg_if::cfg_if;

pub static ERROR_ENTERED: AtomicBool = AtomicBool::new(false);

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug)]
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
#[derive(Copy, Clone, Debug)]
pub enum ParseTokenType {
    UNKNOWN,
    NUMBER,
    STRING,
    NAME,
    HASH,
    PUNCTUATION,
}

#[allow(clippy::struct_excessive_bools)]
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
    const fn new() -> Self {
        Self {
            token: String::new(),
            token_type: ParseTokenType::UNKNOWN,
            lines: 1,
            unget_token: false,
            space_delimited: true,
            keep_string_quotes: false,
            csv: false,
            negative_numbers: false,
            error_prefix: String::new(),
            warning_prefix: String::new(),
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
        Self {
            parse_info: ArrayVec::new(),
            parse_info_num: 0,
            token_pos: 0,
            prev_token_pos: 0,
            line: String::new(),
        }
    }
}

lazy_static! {
    static ref PRINT_LOCK: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
}

#[allow(clippy::print_stdout)]
fn print_internal(channel: Channel, _param_2: i32, message: &str) {
    if channel.get() > 32 {
        return;
    }

    print!("({}) - {}", channel.get(), message);
}

// Needs to be actually implemented
// Just a wrapper around print! currently
pub fn print(channel: Channel, message: &str) {
    let lock = PRINT_LOCK.clone();
    let _lock = lock.lock().unwrap();

    print_internal(channel, 0, message);
}

pub fn println(channel: Channel, message: &str) {
    print(channel, &format!("{}\n", message));
}

cfg_if! {
    if #[cfg(debug_assertions)] {
        pub fn dprint(channel: Channel, message: &str) {
            let lock = PRINT_LOCK.clone();
            let _lock = lock.lock().unwrap();

            print_internal(channel, 0, message);
        }

        pub fn dprintln(channel: Channel, message: &str) {
            print(channel, &format!("{}\n", message));
        }
    } else {
        #[allow(unused)]
        pub fn dprint(channel: Channel, message: &str) {

        }

        #[allow(unused)]
        pub fn dprintln(channel: Channel, message: &str) {

        }
    }
}

pub fn warn(channel: Channel, message: &str) {
    print(channel, &format!("^3{}", message));
}

pub fn warnln(channel: Channel, message: &str) {
    warn(channel, &format!("{}\n", message));
}

static COM_ERROR_PRINTS_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn print_error(channel: Channel, message: &str) {
    let prefix = if message.contains("error") {
        "^1Error: "
    } else {
        "^1"
    };

    let message = format!("{}{}", prefix, message);
    COM_ERROR_PRINTS_COUNT.increment().unwrap_or_else(|| COM_ERROR_PRINTS_COUNT.store_relaxed(0));
    print_internal(channel, 3, &message);
}

pub fn print_errorln(channel: Channel, message: &str) {
    print_error(channel, &format!("{}\n", message));
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
#[allow(clippy::panic)]
pub fn error(err_type: ErrorParm, err: &str) {
    panic!("{} ({:?})", err, err_type);
}

pub fn errorln(err_type: ErrorParm, err: &str) {
    error(err_type, &format!("{}\n", err));
}

// Implement these two later
// (not integral to the program, look annoying to implement)
#[allow(unused_variables, unreachable_code, clippy::todo)]
pub const fn filter(string: &str, name: &str, case_sensitive: bool) -> bool {
    return true;

    todo!("com::filter");
}

#[allow(
    unused_variables,
    unreachable_code,
    clippy::todo,
    clippy::needless_pass_by_value
)]
pub const fn dvar_dump(channel: i32, param_2: &str) {
    return;

    todo!("com::dvar_dump");
}

lazy_static! {
    static ref G_PARSE: Arc<RwLock<ArrayVec<ParseThreadInfo, 16>>> =
        Arc::new(RwLock::new(ArrayVec::new()));
}

pub const fn get_official_build_name_r() -> &'static str {
    "Call of Duty: BlackOps"
}

pub const fn get_build_display_name() -> &'static str {
    "Call of Duty Singleplayer - Ship"
}

thread_local! {
    static G_ERROR: Arc<RwLock<ArrayVec<i32, 16>>> = Arc::new(RwLock::new(ArrayVec::new()));
}

lazy_static! {
    static ref ERROR_MESSAGE: Arc<RwLock<String>> =
        Arc::new(RwLock::new(String::new()));
}

pub fn init() {
    let com_error = 0; // TODO - implement sys::get_value correctly

    if com_error != 0 {
        //sys::error(&format!("Error during initialization:\n{}", *ERROR_MESSAGE.clone().read().unwrap()));
    }

    init_try_block_function();
}

fn init_dvars() {
    dvar::register_bool(
        "wideScreen",
        true,
        dvar::DvarFlags::READ_ONLY,
        Some("True if the game video is running in 16x9 aspect, false if 4x3."),
    )
    .unwrap();
}

fn init_try_block_function() {
    init_dvars();
    render::init_threads();
}

#[allow(clippy::todo)]
pub fn touch_memory() {
    todo!("com::touch_memory");
}

// TODO - implement
pub const fn get_icon_rgba() -> Option<winit::window::Icon> {
    None
}

// TODO - implement
pub fn startup_variable(name: &str) {
    println(16.into(), &format!("com::startup_variable: {}", name));
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
