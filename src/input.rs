#![allow(dead_code)]

pub mod gpad;
pub mod keyboard;
pub mod mouse;

use crate::*;

use core::sync::atomic::Ordering;

use lazy_static::lazy_static;

lazy_static! {
    static ref APP_ACTIVE: AtomicBool = AtomicBool::new(false);
}

pub fn activate(app_active: bool) {
    APP_ACTIVE.store(app_active, Ordering::SeqCst);
    if app_active == false {
        mouse::deactivate();
    } else {
        mouse::activate(1);
    }
}

fn startup() {
    mouse::startup();
    gpad::startup();
    dvar::clear_modified("in_mouse").unwrap();
}

pub fn init() {
    dvar::register_bool(
        "in_mouse",
        true,
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::LATCHED,
        Some("Initialize the mouse drivers"),
    )
    .unwrap();
    startup();
}

fn is_foreground_window() -> bool {
    platform::get_platform_vars().active_app
}

pub const fn shutdown() {
    mouse::deactivate();
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
pub fn update_use_held() {
    let mut cgs = cl::get_local_client_globals_mut();
    let cg = cgs.iter_mut().nth(0).unwrap();
    if cg.use_held == false {
        if dvar::get_int("cl_dblTapMaxDelayTime").unwrap() <= com::frame_time() as i32 - cg.use_time {
            cg.use_count = 0;
        }
        cg.use_held = true;
        cg.use_time = com::frame_time() as _;
    }
}

pub fn update_use_count() {
    let mut cgs = cl::get_local_client_globals_mut();
    let cg = cgs.iter_mut().nth(0).unwrap();

    if (com::frame_time() - cg.use_time as isize) < (dvar::get_int("cl_dblTapMaxDelayTime").unwrap() as isize) {
        cg.use_count += 1;
    } else {
        cg.use_count = 0;
    }
    cg.use_held = false;
}