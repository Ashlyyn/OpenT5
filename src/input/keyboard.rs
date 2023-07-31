#![allow(dead_code)]

use std::{collections::HashMap, sync::RwLock};

extern crate alloc;
use alloc::sync::Arc;

use lazy_static::lazy_static;
use num_derive::FromPrimitive;

use super::{super::*, update_use_count, update_use_held};

#[repr(u8)]
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum KeybindCode {
    None,
    Activate,
    Attack,
    Back,
    Breath,
    BreathSprint,
    Down,
    Forward,
    Frag,
    Gas,
    GoCrouch,
    GoProne,
    GoStandUp,
    Handbrake,
    LeanLeft,
    LeanRight,
    Left,
    LookDown,
    LookUp,
    LowerStance,
    Melee,
    MeleeBreath,
    MoveLeft,
    MoveRight,
    Prone,
    RaiseStance,
    Reload,
    Reverse,
    Right,
    Smoke,
    SpecNext,
    SpecPrev,
    SpeedThrow,
    Speed,
    Sprint,
    Stance,
    Strafe,
    SwitchSeat,
    Talk,
    Throw,
    ToggleAds,
    ToggleAdsThrow,
    ToggleCrouch,
    ToggleProne,
    ToggleSpec,
    ToggleView,
    Up,
    UseReload,
    VehicleAttack,
    VehicleAttackSecond,
    VehicleBoost,
    VehicleDropDeployable,
    VehicleFirePickup,
    VehicleMoveDown,
    VehicleMoveUp,
    VehicleSpecialAbility,
    VehicleSwapPickup,
    MLook,
}

#[derive(Copy, Clone)]
struct Keybind {
    down: [isize; 2],
    downtime: usize,
    msec: usize,
    active: bool,
    was_pressed: bool,
    val: f32,
}

impl Keybind {
    const fn new() -> Self {
        Self {
            down: [0, 0],
            downtime: 0,
            msec: 0,
            active: false,
            was_pressed: false,
            val: 0.0,
        }
    }
}

lazy_static! {
    static ref KEYBINDS: RwLock<HashMap<KeybindCode, Keybind>> =
        RwLock::new(HashMap::with_capacity(47));
}

fn find_keybind(code: KeybindCode) -> Option<Keybind> {
    KEYBINDS.read().unwrap().get(&code).copied()
}

fn key_down(code: KeybindCode) {
    let Some(mut bind) = find_keybind(code) else {
        return;
    };

    let arg = cmd::argv(1);
    let i = if arg.is_empty() {
        -1isize
    } else {
        arg.parse::<isize>().unwrap_or(-1isize)
    };

    let arg = cmd::argv(3);
    bind.val = if arg.is_empty() {
        1.0f32
    } else {
        arg.parse::<f32>().unwrap_or(1.0f32)
    };

    if bind.down[0] == i || bind.down[1] == i {
        KEYBINDS.write().unwrap().insert(code, bind);
        return;
    }

    if bind.down[0] == 0 {
        bind.down[0] = i;
    } else {
        if bind.down[1] != 0 {
            com::print!(14.into(), "Three keys down for a button!");
            return;
        }
        bind.down[1] = i;
    }

    if !bind.active {
        let arg = cmd::argv(2);
        let Ok(downtime) = arg.parse::<usize>() else {
            return;
        };
        bind.downtime = downtime;
        bind.active = true;
        bind.was_pressed = true;
    }

    KEYBINDS.write().unwrap().insert(code, bind);
}

fn key_up(code: KeybindCode) {
    let Some(mut bind) = find_keybind(code) else {
        return;
    };

    let arg = cmd::argv(1);
    if arg.is_empty() {
        bind.down = [0, 0];
        bind.active = false;

        KEYBINDS.write().unwrap().insert(code, bind);
        return;
    }

    let i = arg.parse::<isize>().unwrap_or(-1isize);
    if bind.down[0] == i {
        bind.down[0] = 0;
    } else {
        if bind.down[1] != i {
            return;
        }
        bind.down[1] = 0;
    }

    if bind.down != [0, 0] {
        KEYBINDS.write().unwrap().insert(code, bind);
    }

    bind.active = false;
    let arg = cmd::argv(2);
    let Ok(i) = arg.parse::<usize>() else { return };

    if i == 0 {
        // bind.msec = (frame_msec >> 1) + bind.msec;
    } else {
        bind.msec += i - bind.downtime;
    }
    bind.val = 0.0;
    bind.active = false;

    KEYBINDS.write().unwrap().insert(code, bind);
}

fn activate_down() {
    update_use_held();
    key_down(KeybindCode::Activate);
}

fn activate_up() {
    update_use_count();
    key_up(KeybindCode::Activate);
}

#[allow(clippy::todo)]
fn attack_down() {
    todo!();
}

#[allow(clippy::todo)]
fn attack_up() {
    todo!();
}

fn back_down() {
    key_down(KeybindCode::Back);
}

fn back_up() {
    key_up(KeybindCode::Back);
}

fn breath_down() {
    key_down(KeybindCode::Breath);
}

fn breath_up() {
    key_up(KeybindCode::Breath);
}

fn breath_sprint_down() {
    key_down(KeybindCode::Breath);
    key_down(KeybindCode::Sprint);
}

fn breath_sprint_up() {
    key_up(KeybindCode::Breath);
    key_up(KeybindCode::Sprint);
}

fn down_down() {
    key_down(KeybindCode::Down);
}

fn down_up() {
    key_up(KeybindCode::Down);
}

fn forward_down() {
    key_down(KeybindCode::Forward);
}

fn forward_up() {
    key_up(KeybindCode::Forward);
}

fn frag_down() {
    key_down(KeybindCode::Frag);
}

fn frag_up() {
    key_up(KeybindCode::Frag);
}

fn gas_down() {
    key_down(KeybindCode::Gas);
}

fn gas_up() {
    key_up(KeybindCode::Gas);
}

#[allow(clippy::todo)]
fn go_crouch() {
    todo!();
}

#[allow(clippy::todo)]
fn go_prone() {
    todo!();
}

#[allow(clippy::todo)]
fn go_stand_down() {
    todo!();
}

#[allow(clippy::todo)]
fn go_stand_up() {
    todo!();
}

fn handbrake_down() {
    key_down(KeybindCode::Handbrake);
}

fn handbrake_up() {
    key_up(KeybindCode::Handbrake);
}

fn lean_left_down() {
    key_down(KeybindCode::LeanLeft);
}

fn lean_left_up() {
    key_up(KeybindCode::LeanLeft);
}

fn lean_right_down() {
    key_down(KeybindCode::LeanRight);
}

fn lean_right_up() {
    key_up(KeybindCode::LeanRight);
}

fn left_down() {
    key_down(KeybindCode::Left);
}

fn left_up() {
    key_up(KeybindCode::Left);
}

fn lookdown_down() {
    key_down(KeybindCode::LookDown);
}

fn lookdown_up() {
    key_up(KeybindCode::LookDown);
}

fn lookup_down() {
    key_down(KeybindCode::LookUp);
}

fn lookup_up() {
    key_up(KeybindCode::LookUp);
}

#[allow(clippy::todo)]
fn lower_stance() {
    todo!();
}

fn melee_down() {
    key_down(KeybindCode::Melee);
}

fn melee_up() {
    key_up(KeybindCode::Melee);
}

fn melee_breath_down() {
    key_down(KeybindCode::Melee);
    key_down(KeybindCode::Breath);
}

fn melee_breath_up() {
    key_up(KeybindCode::Melee);
    key_up(KeybindCode::Breath);
}

fn mlook_down() {
    if let Some(r) = KEYBINDS.write().unwrap().get_mut(&KeybindCode::MLook) {
        r.active = true;
    };
}

#[allow(clippy::todo)]
fn mlook_up() {
    todo!();
}

fn move_left_down() {
    key_down(KeybindCode::MoveLeft);
}

fn move_left_up() {
    key_up(KeybindCode::MoveLeft);
}

fn move_right_down() {
    key_down(KeybindCode::MoveRight);
}

fn move_right_up() {
    key_up(KeybindCode::MoveRight);
}

fn prone_down() {
    key_down(KeybindCode::Prone);
}

fn prone_up() {
    key_up(KeybindCode::Prone);
}

#[allow(clippy::todo)]
fn raise_stance() {
    todo!();
}

fn reload_down() {
    update_use_held();
    key_down(KeybindCode::Reload);
}

fn reload_up() {
    update_use_count();
    key_up(KeybindCode::Reload);
}

fn reverse_down() {
    key_down(KeybindCode::Reverse);
}

fn reverse_up() {
    key_up(KeybindCode::Reverse);
}

fn right_down() {
    key_down(KeybindCode::Right);
}

fn right_up() {
    key_up(KeybindCode::Right);
}

#[allow(clippy::todo)]
fn smoke_down() {
    todo!();
}

#[allow(clippy::todo)]
fn smoke_up() {
    todo!();
}

fn spec_next_down() {
    key_down(KeybindCode::SpecNext);
}

fn spec_next_up() {
    key_up(KeybindCode::SpecNext);
}

fn spec_prev_down() {
    key_down(KeybindCode::SpecPrev);
}

fn spec_prev_up() {
    key_up(KeybindCode::SpecPrev);
}

fn speed_throw_down() {
    key_down(KeybindCode::Speed);
    key_down(KeybindCode::Throw);
}

fn speed_throw_up() {
    key_up(KeybindCode::Speed);
    key_up(KeybindCode::Throw);
}

#[allow(clippy::todo)]
fn speed_down() {
    todo!();
}

#[allow(clippy::todo)]
fn speed_up() {
    todo!();
}

fn sprint_down() {
    key_down(KeybindCode::Sprint);
}

fn sprint_up() {
    key_up(KeybindCode::Sprint);
}

#[allow(clippy::todo)]
fn stance_down() {
    todo!();
}

#[allow(clippy::todo)]
fn stance_up() {
    todo!();
}

fn strafe_down() {
    key_down(KeybindCode::Strafe);
}

fn strafe_up() {
    key_up(KeybindCode::Strafe);
}

fn switch_seat_down() {
    key_down(KeybindCode::SwitchSeat);
}

fn switch_seat_up() {
    key_up(KeybindCode::SwitchSeat);
}

fn talk_down() {
    key_down(KeybindCode::Talk);
}

fn talk_up() {
    key_up(KeybindCode::Talk);
}

fn throw_down() {
    key_down(KeybindCode::Throw);
}

fn throw_up() {
    key_up(KeybindCode::Throw);
}

#[allow(clippy::todo)]
fn toggle_ads() {
    todo!();
}

fn toggle_ads_throw_down() {
    key_down(KeybindCode::Throw);
    toggle_ads();
}

fn toggle_ads_throw_up() {
    key_up(KeybindCode::Throw);
    toggle_ads();
}

#[allow(clippy::todo)]
fn toggle_crouch() {
    todo!();
}

#[allow(clippy::todo)]
fn toggle_prone() {
    todo!();
}

fn toggle_spec_down() {
    key_down(KeybindCode::ToggleSpec);
}

fn toggle_spec_up() {
    key_up(KeybindCode::ToggleSpec);
}

#[allow(clippy::todo)]
fn toggle_view() {
    todo!();
}

#[allow(clippy::todo)]
fn up_down() {
    todo!();
}

#[allow(clippy::todo)]
fn up_up() {
    todo!();
}

fn use_reload_down() {
    update_use_held();
    key_down(KeybindCode::Reload);
}

fn use_reload_up() {
    update_use_count();
    key_up(KeybindCode::Reload);
}

fn vehicle_attack_down() {
    key_down(KeybindCode::VehicleAttack);
}

fn vehicle_attack_up() {
    key_up(KeybindCode::VehicleAttack);
}

fn vehicle_attack_second_down() {
    key_down(KeybindCode::VehicleAttackSecond);
}

fn vehicle_attack_second_up() {
    key_up(KeybindCode::VehicleAttackSecond);
}

fn vehicle_boost_down() {
    key_down(KeybindCode::VehicleBoost);
}

fn vehicle_boost_up() {
    key_up(KeybindCode::VehicleBoost);
}

fn vehicle_drop_deployable_down() {
    key_down(KeybindCode::VehicleDropDeployable);
}

fn vehicle_drop_deployable_up() {
    key_up(KeybindCode::VehicleDropDeployable);
}

fn vehicle_fire_pickup_down() {
    key_down(KeybindCode::VehicleFirePickup);
}

fn vehicle_fire_pickup_up() {
    key_up(KeybindCode::VehicleFirePickup);
}

fn vehicle_move_down_down() {
    key_down(KeybindCode::VehicleMoveDown);
}

fn vehicle_move_down_up() {
    key_up(KeybindCode::VehicleMoveDown);
}

fn vehicle_move_up_down() {
    key_down(KeybindCode::VehicleMoveUp);
}

fn vehicle_move_up_up() {
    key_up(KeybindCode::VehicleMoveUp);
}

fn vehicle_special_ability_down() {
    key_down(KeybindCode::VehicleSpecialAbility);
}

fn vehicle_special_ability_up() {
    key_up(KeybindCode::VehicleSpecialAbility);
}

fn vehicle_swap_pickup_down() {
    key_down(KeybindCode::VehicleSwapPickup);
}

fn vehicle_swap_pickup_up() {
    key_up(KeybindCode::VehicleSwapPickup);
}

fn is_talk_key_held() -> bool {
    dvar::get_bool("cl_talking").unwrap_or(false)
        && KEYBINDS
            .read()
            .unwrap()
            .get(&KeybindCode::Talk)
            .unwrap_or(&Keybind::new())
            .active
            == false
}

#[repr(u8)]
#[derive(Copy, Clone, Default, Hash, Eq, PartialEq, FromPrimitive)]
pub enum KeyScancode {
    #[default]
    None,
    // First row
    Esc,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    PrtScrSysRq,
    ScrLk,
    PauseBreak,
    // Second row
    Tilde,
    Number0,
    Number1,
    Number2,
    Number3,
    Number4,
    Number5,
    Number6,
    Number7,
    Number8,
    Number9,
    Minus,
    Equals,
    Backspace,
    Insert,
    Home,
    PageUp,
    NumLk,
    NumpadSlash,
    NumpadAsterisk,
    NumpadMinus,
    // Third row
    Tab,
    Q,
    W,
    E,
    R,
    T,
    Y,
    U,
    I,
    O,
    P,
    OpenBracket,
    CloseBracket,
    BackSlash,
    Del,
    End,
    PageDown,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadPlus,
    // Fourth row
    CapsLock,
    A,
    S,
    D,
    F,
    G,
    H,
    J,
    K,
    L,
    Semicolon,
    Apostrophe,
    Enter,
    Numpad4,
    Numpad5,
    Numpad6,
    // Fifth row
    LShift,
    Z,
    X,
    C,
    V,
    B,
    N,
    M,
    Comma,
    Period,
    ForwardSlash,
    RShift,
    ArrowUp,
    Numpad1,
    Numpad2,
    Numpad3,
    NumpadEnter,
    // Sixth row
    LCtrl,
    Super,
    LAlt,
    Space,
    RAlt,
    Fn,
    RCtrl,
    LeftArrow,
    DownArrow,
    RightArrow,
    Numpad0,
    NumpadDot,
}

lazy_static! {
    static ref KEY_CODES: Arc<RwLock<HashMap<KeyScancode, KeybindCode>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

fn find_keybind_code(code: KeyScancode) -> Option<KeybindCode> {
    let lock = KEY_CODES.clone();
    let reader = lock.read().unwrap();
    reader.get(&code).copied()
}
