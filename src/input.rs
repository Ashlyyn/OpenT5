#![allow(dead_code)]

pub mod gamepad;
pub mod keyboard;
pub mod mouse;

use crate::*;

use std::sync::atomic::{AtomicIsize, Ordering};

use lazy_static::lazy_static;

lazy_static! {
    static ref APP_ACTIVE: AtomicIsize = AtomicIsize::new(0);
}

pub fn activate(app_active: isize) {
    APP_ACTIVE.store(app_active, Ordering::SeqCst);
    if app_active == 0 {
        mouse::deactivate();
    } else {
        mouse::activate(1);
    }
}

fn startup() {
    mouse::startup();
    gamepad::startup();
    dvar::clear_modified("in_mouse");
}

fn init() {
    dvar::register_bool(
        "in_mouse",
        true,
        Some(dvar::DvarFlags::UNKNOWN_00000001_A | dvar::DvarFlags::LATCHED),
        Some("Initialize the mouse drivers"),
    );
    startup();
}

fn is_foreground_window() -> bool {
    platform::get_platform_vars().active_app
}
