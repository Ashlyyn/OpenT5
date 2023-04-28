#![feature(never_type)]
#![feature(io_error_more)]
#![feature(const_option)]
#![feature(int_roundings)]
#![feature(const_fn_floating_point_arithmetic)]
#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    missing_docs,
    clippy::uninlined_format_args,
    clippy::bool_comparison,
    clippy::missing_docs_in_private_items,
    clippy::unwrap_used,
    clippy::default_numeric_fallback,
    clippy::implicit_return,
    clippy::wildcard_imports,
    clippy::shadow_reuse,
    clippy::blanket_clippy_restriction_lints,
    clippy::exit,
    clippy::wildcard_enum_match_arm,
    clippy::match_wildcard_for_single_variants,
    clippy::enum_glob_use,
    clippy::as_underscore,
    clippy::float_arithmetic,
    clippy::module_name_repetitions,
    clippy::unseparated_literal_suffix,
    clippy::as_conversions,
    clippy::integer_arithmetic,
    clippy::arithmetic_side_effects,
    clippy::shadow_unrelated,
    clippy::get_unwrap,
    clippy::unused_async,
    clippy::if_not_else,
    clippy::integer_division,
    clippy::multiple_crate_versions,
    clippy::cargo_common_metadata,
    clippy::single_char_lifetime_names,
    clippy::similar_names,
    clippy::else_if_without_else,
    clippy::self_named_module_files,
    clippy::equatable_if_let,
    clippy::pattern_type_mismatch,
    clippy::semicolon_outside_block,
    clippy::iter_nth_zero,
    clippy::missing_inline_in_public_items,
    clippy::semicolon_if_nothing_returned,
    clippy::let_underscore_untyped,
    clippy::let_unit_value,
    clippy::question_mark_used,
    clippy::impl_trait_in_params
)]
#![deny(missing_debug_implementations, clippy::separated_literal_suffix)]

use cfg_if::cfg_if;
use core::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use lazy_static::lazy_static;
extern crate alloc;
use alloc::sync::Arc;

use crate::sys::focus_window;

cfg_if! {
    if #[cfg(target_arch="wasm32")] {
        use wasm_bindgen::prelude::*;
    } else {
        use discord_rich_presence::activity::Activity;

        mod discord_rpc;
        mod pmem;
    }
}

mod cbuf;
mod cg;
mod cl;
mod cmd;
mod com;
mod common;
mod conbuf;
mod console;
mod dvar;
mod fs;
mod gfx;
mod input;
mod key;
mod locale;
mod net;
mod pb;
mod platform;
mod rb;
mod render;
mod seh;
mod sys;
mod util;
mod vid;

lazy_static! {
    #[allow(dead_code)]
    static ref G_ALLOW_MATURE: AtomicBool = AtomicBool::new(true);
    static ref S_NOSND: AtomicBool = AtomicBool::new(false);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
#[allow(
    clippy::collapsible_else_if,
    clippy::missing_panics_doc,
    clippy::expect_used
)]
pub fn run() {
    platform::os::target::main();
    let cmdline = sys::get_cmdline();
    if cmdline.contains("autominidump") {
        sys::start_minidump(false);
    } else {
        if cmdline.contains("minidump") {
            sys::start_minidump(true);
        } else {
            // Windows top-level exception handler bullshit
        }
    }

    sys::init_main_thread();

    cfg_if! {
        if #[cfg(not(target_arch = "wasm32"))] {
            pmem::init();
        }
    }
    locale::init();

    #[allow(clippy::collapsible_if)]
    if cmdline.get(0..9).unwrap_or_default() != "allowdupe"
        || cmdline.chars().nth(9).unwrap_or(' ') > ' '
    {
        if !cmdline.contains("g_connectpaths 3") {
            if sys::check_crash_or_rerun() == false {
                return;
            }
        }
    }

    if cmdline.contains("nosnd") {
        S_NOSND.store(true, Ordering::SeqCst);
    }

    dvar::init();

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
            // Set Discord activity on a different thread so that it doesn't block main
            std::thread::spawn(|| {
                discord_rpc::set_activity(Activity::new().state("Testing...")).unwrap();
            });
        }
    }

    com::init();
    com::println!(16.into(), "Working directory: {}", sys::cwd().as_os_str().to_string_lossy());
    focus_window(platform::get_window_handle().unwrap());
    loop {
        if platform::get_minimized() {
            std::thread::sleep(Duration::from_millis(5));
        }
        com::frame();
        // FUN_005cc940();
        if dvar::get_bool("onlinegame").unwrap() {
            // PbProcessServerEvents();
        }
    }
}
