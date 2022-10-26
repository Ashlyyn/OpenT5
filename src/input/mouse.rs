#![allow(dead_code)]

use std::sync::Arc;

use crate::*;
use bitflags::bitflags;
use lazy_static::lazy_static;
use no_deadlocks::RwLock;

use super::gpad;

#[allow(unused_variables)]
pub fn activate(param_1: isize) {}

pub fn deactivate() {}

bitflags! {
    #[derive(Default)]
    struct ButtonState: u32 {
        const LBUTTON = 0x01;
        const RBUTTON = 0x02;
        const MBUTTON = 0x04;
        const XBUTTON1 = 0x08;
        const XBUTTON2 = 0x10;
    }
}

#[derive(Copy, Clone, Default)]
struct MouseVars {
    old_button_state: ButtonState,
    old_pos: (u16, u16),
    mouse_active: bool,
    mouse_initialized: bool,
}

lazy_static! {
    static ref S_MV: Arc<RwLock<MouseVars>> =
        Arc::new(RwLock::new(Default::default()));
}

pub fn startup() {
    S_MV.clone().write().unwrap().mouse_initialized = false;
    if dvar::get_bool("in_mouse").unwrap_or(false) == false {
        com::println("Mouse control not active.");
    } else {
        S_MV.clone().write().unwrap().mouse_initialized = true;
        // FUN_004682b0();
    }

    gpad::startup();
    dvar::clear_modified("in_mouse");
}

pub enum Scancode {
    LClick,
    RClick,
    MClick,
    MWheelUp,
    MWheelDown,
}
