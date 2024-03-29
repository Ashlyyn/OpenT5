// This file is for the gamepad subsystem. It will include everything
// necessary to use a gamepad as an input device.
//
// It is very much a work-in-progress. It's not feature-complete, and
// none of it has been tested yet.

use arrayvec::ArrayVec;
use std::sync::RwLock;

use crate::{common::Vec2f32, *};

#[derive(Copy, Clone, Default)]
struct GamePadRumble {
    left_motor_speed: usize,
    right_motor_speed: usize,
}

#[derive(Copy, Clone, Default)]
struct Feedback {
    rumble: GamePadRumble,
}

#[derive(Copy, Clone)]
struct GamePad {
    enabled: bool,
    port_index: GPadIdx,
    id: gilrs::GamepadId,
    feedback: Feedback,
    lstick: StickState,
    lstick_last: StickState,
    rstick: StickState,
    rstick_last: StickState,
}

lazy_static! {
    static ref S_GAMEPADS: RwLock<ArrayVec<GamePad, { MAX_GPADS as _ }>> =
        RwLock::new(ArrayVec::new());
}

pub fn startup() {
    init_all();
}

#[allow(clippy::too_many_lines)]
fn init_dvars() {
    dvar::register_int(
        "gpad_debug",
        0,
        Some(i32::MIN),
        Some(i32::MAX),
        dvar::DvarFlags::empty(),
        None,
    )
    .unwrap();
    dvar::register_float(
        "gpad_button_lstick_deflect_max",
        0.0,
        Some(0.0),
        Some(1.0),
        dvar::DvarFlags::empty(),
        None,
    )
    .unwrap();
    dvar::register_float(
        "gpad_button_rstick_deflect_max",
        0.0,
        Some(0.0),
        Some(1.0),
        dvar::DvarFlags::empty(),
        None,
    )
    .unwrap();
    dvar::register_float(
        "gpad_button_deadzone",
        0.13,
        Some(0.0),
        Some(1.0),
        dvar::DvarFlags::CHEAT_PROTECTED,
        None,
    )
    .unwrap();
    dvar::register_float(
        "gpad_stick_deadzone_min",
        0.2,
        Some(0.0),
        Some(1.0),
        dvar::DvarFlags::CHEAT_PROTECTED,
        None,
    )
    .unwrap();
    dvar::register_float(
        "gpad_stick_deadzone_max",
        0.01,
        Some(0.0),
        Some(1.0),
        dvar::DvarFlags::CHEAT_PROTECTED,
        None,
    )
    .unwrap();
    dvar::register_float(
        "gpad_stick_pressed",
        0.4,
        Some(0.0),
        Some(1.0),
        dvar::DvarFlags::CHEAT_PROTECTED,
        None,
    )
    .unwrap();
    dvar::register_float(
        "gpad_stick_hysteresis",
        0.1,
        Some(0.0),
        Some(1.0),
        dvar::DvarFlags::CHEAT_PROTECTED,
        None,
    )
    .unwrap();
    dvar::register_bool(
        "gpad_rumble",
        true,
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::ALLOW_SET_FROM_DEVGUI,
        None,
    )
    .unwrap();
    dvar::register_int(
        "gpad_menu_scroll_delay_first",
        420,
        Some(0),
        Some(1000),
        dvar::DvarFlags::ARCHIVE,
        None,
    )
    .unwrap();
    dvar::register_int(
        "gpad_menu_scroll_delay_rest",
        210,
        Some(0),
        Some(1000),
        dvar::DvarFlags::ARCHIVE,
        None,
    )
    .unwrap();
    dvar::register_string(
        "gpad_buttonsConfig",
        "buttons_default",
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::ALLOW_SET_FROM_DEVGUI,
        None,
    )
    .unwrap();
    dvar::register_string(
        "gpad_sticksConfig",
        "sticks_default",
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::ALLOW_SET_FROM_DEVGUI,
        None,
    )
    .unwrap();
    dvar::register_bool(
        "gpad_enabled",
        false,
        dvar::DvarFlags::ARCHIVE | dvar::DvarFlags::ALLOW_SET_FROM_DEVGUI,
        None,
    )
    .unwrap();
    dvar::register_bool(
        "gpad_present",
        false,
        dvar::DvarFlags::READ_ONLY,
        None,
    )
    .unwrap();
}

fn init_all() {
    init_dvars();
    S_GAMEPADS
        .write()
        .unwrap()
        .get_mut(0)
        .unwrap()
        .feedback
        .rumble
        .left_motor_speed = 0;
    S_GAMEPADS
        .write()
        .unwrap()
        .get_mut(0)
        .unwrap()
        .feedback
        .rumble
        .right_motor_speed = 0;

    for i in 0..MAX_GPADS {
        let mut gamepads = S_GAMEPADS.write().unwrap();
        let gpad = gamepads.get_mut(i as usize).unwrap();

        gpad.feedback.rumble.left_motor_speed = 0;
        gpad.feedback.rumble.right_motor_speed = 0;
    }
}

type GPadIdx = u8;

const MAX_GPADS: GPadIdx = 1;

fn port_index_to_id(port_index: GPadIdx) -> Option<gilrs::GamepadId> {
    S_GAMEPADS
        .read()
        .unwrap()
        .iter()
        .nth(port_index as _)
        .map(|g| g.id)
}

#[allow(clippy::cast_possible_truncation)]
fn id_to_port_index(id: gilrs::GamepadId) -> Option<GPadIdx> {
    let gamepads = S_GAMEPADS.read().unwrap();
    gamepads
        .iter()
        .enumerate()
        .find(|(_, &g)| g.id == id)
        .map(|(i, _)| i as GPadIdx)
}

pub fn is_active(port_index: GPadIdx) -> Option<bool> {
    S_GAMEPADS
        .read()
        .unwrap()
        .iter()
        .nth(port_index as _)
        .map(|g| g.enabled)
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum ButtonState {
    Pressed,
    Released,
}

impl From<bool> for ButtonState {
    fn from(b: bool) -> Self {
        if b {
            Self::Pressed
        } else {
            Self::Released
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Button {
    North,
    South,
    East,
    West,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    LeftTrigger,
    LeftBumper,
    RightTrigger,
    RightBumper,
    Menu,
    EtcLeft,  // Back on 360 Controller, Share on DS4, etc.
    EtcRight, // Pause/Start on most gamepads
}

// gilrs already implements a generic `TryFrom<T> for U`, so we can't
// manually implement `TryFrom<Button> for gilrs::Button` here.
// Thus we use `Into<gilrs::Button> for Button` and suppress the warning.
#[allow(clippy::from_over_into)]
impl Into<gilrs::Button> for Button {
    fn into(self) -> gilrs::Button {
        match self {
            Self::North => gilrs::Button::North,
            Self::South => gilrs::Button::South,
            Self::East => gilrs::Button::East,
            Self::West => gilrs::Button::West,
            Self::DPadUp => gilrs::Button::DPadUp,
            Self::DPadDown => gilrs::Button::DPadDown,
            Self::DPadLeft => gilrs::Button::DPadLeft,
            Self::DPadRight => gilrs::Button::DPadRight,
            Self::LeftTrigger => gilrs::Button::LeftTrigger,
            Self::LeftBumper => gilrs::Button::LeftTrigger2,
            Self::RightTrigger => gilrs::Button::RightTrigger,
            Self::RightBumper => gilrs::Button::RightTrigger2,
            Self::Menu => gilrs::Button::Mode,
            Self::EtcLeft => gilrs::Button::Select,
            Self::EtcRight => gilrs::Button::Start,
        }
    }
}

pub fn get_button(port_index: GPadIdx, button: Button) -> Option<ButtonState> {
    let Ok(gilrs) = gilrs::Gilrs::new() else {
        return None;
    };

    let mut gamepads = gilrs.gamepads();

    let gpad = gamepads.find(|(id, _)| match id_to_port_index(*id) {
        Some(idx) => idx,
        None => return false,
    } == port_index).map(|(_, g)| g);

    match gpad {
        Some(g) => {
            let button: gilrs::Button = match button.try_into() {
                Ok(b) => b,
                Err(_) => return None,
            };
            Some(g.is_pressed(button).into())
        }
        None => None,
    }
}

#[derive(Copy, Clone)]
pub struct StickState(Vec2f32, ButtonState);

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Stick {
    LStick,
    RStick,
}

// gilrs already implements a generic `TryFrom<T> for U`, so we can't
// manually implement `TryFrom<Stick> for gilrs::Button` here.
// Thus we use `Into<gilrs::Button> for Stick` and suppress the warning.
#[allow(clippy::from_over_into)]
impl Into<gilrs::Button> for Stick {
    fn into(self) -> gilrs::Button {
        match self {
            Self::LStick => gilrs::Button::LeftThumb,
            Self::RStick => gilrs::Button::RightThumb,
        }
    }
}

pub fn get_stick(port_index: GPadIdx, stick: Stick) -> Option<StickState> {
    let Ok(gilrs) = gilrs::Gilrs::new() else {
        return None;
    };

    let mut gamepads = gilrs.gamepads();

    let gpad = gamepads.find(|(id, _)| match id_to_port_index(*id) {
        Some(idx) => idx,
        None => return false,
    } == port_index).map(|(_, g)| g);

    gpad.map(|g| {
        let x_axis = match stick {
            Stick::LStick => gilrs::Axis::LeftStickX,
            Stick::RStick => gilrs::Axis::RightStickX,
        };

        let y_axis = match stick {
            Stick::LStick => gilrs::Axis::LeftStickY,
            Stick::RStick => gilrs::Axis::RightStickY,
        };

        let x = g
            .axis_data(x_axis)
            .map_or(0.0, gilrs::ev::state::AxisData::value);

        let y = g
            .axis_data(y_axis)
            .map_or(0.0, gilrs::ev::state::AxisData::value);

        let pressed = g.is_pressed(stick.into());

        StickState((x, y), pressed.into())
    })
}

pub fn is_button_pressed(port_index: GPadIdx, button: Button) -> Option<bool> {
    get_button(port_index, button).map(|b| b == ButtonState::Pressed)
}

pub fn is_button_released(port_index: GPadIdx, button: Button) -> Option<bool> {
    get_button(port_index, button).map(|b| b == ButtonState::Released)
}

pub fn is_stick_pressed(port_index: GPadIdx, stick: Stick) -> Option<bool> {
    get_stick(port_index, stick).map(|s| s.1 == ButtonState::Pressed)
}

pub fn is_stick_released(port_index: GPadIdx, stick: Stick) -> Option<bool> {
    get_stick(port_index, stick).map(|s| s.1 == ButtonState::Released)
}

// TODO - verify implementation is actually correct
fn update_sticks_down(port_index: GPadIdx) {
    let stick_pressed =
        dvar::get_float("gpad_stick_pressed").unwrap_or_default();

    let mut gamepads = S_GAMEPADS.write().unwrap();
    let mut iter = gamepads.iter_mut();
    let gpad = iter.nth(port_index as _).unwrap();

    gpad.lstick_last.1 = gpad.lstick.1;

    let s = if !is_stick_pressed(port_index, Stick::LStick).unwrap_or_default()
    {
        stick_pressed
            + dvar::get_float("gpad_stick_pressed_hysteresis")
                .unwrap_or_default()
    } else {
        stick_pressed
            - dvar::get_float("gpad_stick_pressed_hysteresis")
                .unwrap_or_default()
    };

    gpad.lstick.1 = (s < gpad.lstick.0 .0).into();

    gpad.rstick_last.1 = gpad.rstick.1;

    let s = if !is_stick_pressed(port_index, Stick::RStick).unwrap_or_default()
    {
        stick_pressed
            + dvar::get_float("gpad_stick_pressed_hysteresis")
                .unwrap_or_default()
    } else {
        stick_pressed
            - dvar::get_float("gpad_stick_pressed_hysteresis")
                .unwrap_or_default()
    };

    gpad.rstick.1 = (s < gpad.rstick.0 .0).into();
}

#[allow(clippy::semicolon_outside_block)]
pub fn update_sticks(port_index: GPadIdx) {
    let lstick = get_stick(port_index, Stick::LStick).unwrap();
    let rstick = get_stick(port_index, Stick::RStick).unwrap();
    let lx = lstick.0 .0;
    let ly = lstick.0 .1;
    let rx = rstick.0 .0;
    let ry = rstick.0 .1;

    {
        let mut gpads = S_GAMEPADS.write().unwrap();
        let gpad = gpads.iter_mut().nth(port_index as _).unwrap();
        gpad.lstick_last = gpad.lstick;
        gpad.rstick_last = gpad.rstick;
        gpad.lstick.0 = (lx, ly);
        gpad.rstick.0 = (rx, ry);
    }
    update_sticks_down(port_index);
}
