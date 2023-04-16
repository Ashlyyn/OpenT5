#![allow(dead_code)]

use crate::console::Channel;
use crate::util::EasierAtomic;
use crate::*;
use arrayvec::ArrayVec;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::AtomicUsize;
use core::time::Duration;
use lazy_static::lazy_static;
use std::fs::File;
use std::sync::Mutex;
use std::sync::RwLock;
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

// Not sure what to call this, think of a better name later.
#[doc(hidden)]
pub enum MessageType {
    Print,
    Warn,
    Error,
}

#[doc(hidden)]
#[allow(clippy::print_stdout, clippy::needless_pass_by_value)]
pub fn _print_internal(
    channel: Channel,
    _message_type: MessageType,
    arguments: core::fmt::Arguments,
) {
    if channel.get() > 32 {
        return;
    }

    std::print!("({}) - {}", channel.get(), arguments);
}

/// Prints text.
///
/// Currently just a wrapper around [`std::print!`], will get a proper
/// implementation in the future.
///
/// # Panics
///
/// Currently panics if [`std::print!`] panics.
///
/// # Example
///
/// ```
/// com::print!("Hello to com!");
/// ```
#[macro_export]
macro_rules! __print {
    ($channel:expr, $($arg:tt)*) => {{
        $crate::com::_print_internal($channel, $crate::com::MessageType::Print, core::format_args!($($arg)*));
    }};
}
pub use __print as print;

/// Prints text with a newline.
///
/// Invokes [`com::print`] with the supplied text and a newline appended.
/// Analogous to [`std::println!`]
///
/// # Panics
///
/// Currently panics if [`com::print!`] panics.
///
/// # Example
///
/// ```
/// com::println!("Hello to com!");
/// ```
#[macro_export]
macro_rules! __println {
    ($channel:expr) => {
        $crate::com::print!($channel, "\n")
    };
    ($channel:expr, $($arg:tt)*) => {{
        $crate::com::print!($channel, "{}\n", core::format_args!($($arg)*));
    }};
}
pub use __println as println;

cfg_if! {
    if #[cfg(debug_assertions)] {
        #[doc(hidden)]
        pub fn _dprint(channel: Channel, arguments: core::fmt::Arguments) {
            _print_internal(channel, MessageType::Print, arguments);
        }
    } else {
        pub fn _dprint(_channel: Channel, _arguments: core::fmt::Arguments) {

        }
    }
}

/// Prints text if the executable is compiled in debug mode.
///
/// Does nothing in release mode.
///
/// Currently just a wrapper around [`std::print!`], will get a proper
/// implementation in the future.
///
/// # Panics
///
/// Currently panics if [`std::print!`] panics.
///
/// # Example
///
/// ```
/// com::dprint!("Hello to com from debug mode!");
/// ```
#[macro_export]
macro_rules! __dprint {
    ($channel:expr, $($arg:tt)*) => {{
        $crate::com::_dprint($channel, core::format_args!($($arg)*));
    }};
}
#[allow(unused_imports)]
pub use __dprint as dprint;

/// Prints text with a newline appended if the executable is compiled
/// in debug mode.
///
/// Does nothing in release mode.
///
/// Currently just a wrapper around [`com::print!`], will get a proper
/// implementation in the future.
///
/// # Panics
///
/// Currently panics if [`com::print!`] panics.
///
/// # Example
///
/// ```
/// com::dprintln!("Hello to com from debug mode!");
/// ```
#[macro_export]
macro_rules! __dprintln {
    ($channel:expr) => {
        $crate::com::dprint!($channel, "\n")
    };
    ($channel:expr, $($arg:tt)*) => {{
        $crate::com::dprint!($channel, "{}\n", core::format_args!($($arg)*));
    }};
}
pub use __dprintln as dprintln;

#[doc(hidden)]
#[allow(clippy::needless_pass_by_value)]
pub fn _warn(channel: Channel, arguments: core::fmt::Arguments) {
    _print_internal(
        channel,
        MessageType::Warn,
        format_args!("^3{}", arguments),
    );
}

/// Prints a warning.
///
/// Implemented simply as a wrapper around [`com::print!`].
///
/// # Panics
///
/// Currently panics if [`com::print!`] panics.
///
/// # Example
///
/// ```
/// com::warn!("Warning to com!");
/// ```
macro_rules! __warn {
    ($channel:expr, $($arg:tt)*) => {{
        $crate::com::_warn($channel, core::format_args!($($arg)*));
    }};
}
#[allow(unused_imports)]
pub(crate) use __warn as warn;

/// Prints a warning with a newline appended.
///
/// Implemented simply as a wrapper around [`com::warn!`].
///
/// # Panics
///
/// Currently panics if [`com::warn!`] panics.
///
/// # Example
///
/// ```
/// com::warnln!("Warning to com!");
/// ```
macro_rules! __warnln {
    ($channel:expr) => {
        $crate::com::warn!(channel, "\n")
    };
    ($channel:expr, $($arg:tt)*) => {{
        $crate::com::warn!($channel, "{}\n", core::format_args!($($arg)*));
    }};
}
pub(crate) use __warnln as warnln;

static COM_ERROR_PRINTS_COUNT: AtomicUsize = AtomicUsize::new(0);

#[doc(hidden)]
#[allow(clippy::needless_pass_by_value)]
pub fn _print_error(channel: Channel, arguments: core::fmt::Arguments) {
    let prefix = if arguments.to_string().contains("error") {
        "^1Error: "
    } else {
        "^1"
    };

    COM_ERROR_PRINTS_COUNT.increment_wrapping();
    _print_internal(
        channel,
        MessageType::Error,
        format_args!("{}{}", prefix, arguments),
    );
}

/// Prints an error.
///
/// Implemented simply as a wrapper around [`com::print!`].
///
/// # Panics
///
/// Currently panics if [`com::print!`] panics.
///
/// # Example
///
/// ```
/// com::print_error!("Error to com!");
/// ```
macro_rules! __print_error {
    ($channel:expr, $($arg:tt)*) => {{
        $crate::com::_print_error($channel, core::format_args!($($arg)*));
    }};
}
#[allow(unused_imports)]
pub(crate) use __print_error as print_error;

/// Prints an error with a newline appended.
///
/// Implemented simply as a wrapper around [`com::print_error!`].
///
/// # Panics
///
/// Currently panics if [`com::print_error!`] panics.
///
/// # Example
///
/// ```
/// com::print_errorln!("Error to com!");
/// ```
macro_rules! __print_errorln {
    ($channel:expr) => {
        $crate::com::print_error!(channel, "\n")
    };
    ($channel:expr, $($arg:tt)*) => {{
        $crate::com::print_error!($channel, "{}\n", core::format_args!($($arg)*));
    }};
}
pub(crate) use __print_errorln as print_errorln;

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
#[allow(clippy::panic, clippy::needless_pass_by_value)]
pub fn error(err_type: ErrorParm, err: impl ToString) {
    panic!("{} ({:?})", err.to_string(), err_type);
}

#[allow(clippy::needless_pass_by_value)]
pub fn errorln(err_type: ErrorParm, err: impl ToString) {
    error(err_type, format!("{}\n", err.to_string()));
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

    dvar::register_bool(
        "onlinegame",
        true,
        dvar::DvarFlags::READ_ONLY,
        Some("Current game is an online game with stats, custom classes, unlocks"),
    )
    .unwrap();
}

fn init_try_block_function() {
    init_dvars();
    fs::init_filesystem(true);
    cl::init_once_for_all_clients();
    render::init_threads();
    cl::init_renderer();
    self::println!(16.into(), "--- Common Initialization Complete ---");
}

#[allow(clippy::todo)]
pub fn touch_memory() {
    todo!("com::touch_memory");
}

// TODO - implement
pub const fn get_icon_rgba() -> Option<Vec<u8>> {
    None
}

// TODO - implement
pub fn startup_variable(name: &str) {
    com::println!(16.into(), "com::startup_variable: {}", name);
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

static FRAME_TIME: AtomicU64 = AtomicU64::new(0);

pub fn frame_time() -> Duration {
    Duration::from_millis(FRAME_TIME.load_relaxed())
}

pub fn quit_f() -> ! {
    self::println!(0.into(), "quitting...");
    if ERROR_ENTERED.load(Ordering::Relaxed) {}
    sys::quit();
}

pub fn frame() {}
