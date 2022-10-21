use arrayvec::ArrayVec;
use no_deadlocks::RwLock;

use crate::*;

#[derive(Copy, Clone, Default)]
struct GamePadRumble {
    left_motor_speed: usize,
    right_motor_speed: usize,
}

#[derive(Copy, Clone, Default)]
struct Feedback {
    rumble: GamePadRumble,
}

#[derive(Copy, Clone, Default)]
struct GamePad {
    feedback: Feedback,
}

lazy_static! {
    static ref S_GAMEPADS: Arc<RwLock<ArrayVec<GamePad, 1>>> = Arc::new(RwLock::new(ArrayVec::new()));
}

pub fn startup() {
    init_all();
}

fn init_all() {
    dvar::register_int("gpad_debug", 0, Some(i32::MIN), Some(i32::MAX), dvar::DvarFlags::empty(), None);
    dvar::register_float("gpad_button_lstick_deflect_max", 0.0, Some(0.0), Some(1.0), dvar::DvarFlags::empty(), None);
    dvar::register_float("gpad_button_rstick_deflect_max", 0.0, Some(0.0), Some(1.0), dvar::DvarFlags::empty(), None);
    dvar::register_float("gpad_button_deadzone", 0.13, Some(0.0), Some(1.0), dvar::DvarFlags::CHEAT_PROTECTED, None);
    dvar::register_float("gpad_stick_deadzone_min", 0.2, Some(0.0), Some(1.0), dvar::DvarFlags::CHEAT_PROTECTED, None);
    dvar::register_float("gpad_stick_deadzone_max", 0.01, Some(0.0), Some(1.0), dvar::DvarFlags::CHEAT_PROTECTED, None);
    dvar::register_float("gpad_stick_pressed", 0.4, Some(0.0), Some(1.0), dvar::DvarFlags::CHEAT_PROTECTED, None);
    dvar::register_float("gpad_stick_hysteresis", 0.1, Some(0.0), Some(1.0), dvar::DvarFlags::CHEAT_PROTECTED, None);
    dvar::register_bool("gpad_rumble", true, dvar::DvarFlags::UNKNOWN_00000001_A | dvar::DvarFlags::ALLOW_SET_FROM_DEVGUI, None);
    dvar::register_int("gpad_menu_scroll_delay_first", 420, Some(0), Some(1000), dvar::DvarFlags::UNKNOWN_00000001_A, None);
    dvar::register_int("gpad_menu_scroll_delay_rest", 210, Some(0), Some(1000), dvar::DvarFlags::UNKNOWN_00000001_A, None);
    dvar::register_string("gpad_buttonsConfig", "buttons_default", dvar::DvarFlags::UNKNOWN_00000001_A | dvar::DvarFlags::ALLOW_SET_FROM_DEVGUI, None);
    dvar::register_string("gpad_sticksConfig", "sticks_default", dvar::DvarFlags::UNKNOWN_00000001_A | dvar::DvarFlags::ALLOW_SET_FROM_DEVGUI, None);
    dvar::register_bool("gpad_enabled", false, dvar::DvarFlags::UNKNOWN_00000001_A | dvar::DvarFlags::ALLOW_SET_FROM_DEVGUI, None);
    dvar::register_bool("gpad_present", false, dvar::DvarFlags::READ_ONLY, None);
    S_GAMEPADS.clone().write().unwrap()[0].feedback.rumble.left_motor_speed = 0;
    S_GAMEPADS.clone().write().unwrap()[0].feedback.rumble.right_motor_speed = 0;
    // TODO - XInputSetState/equivalent
}
