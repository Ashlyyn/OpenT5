#![allow(dead_code)]

use std::sync::RwLock;
extern crate alloc;
use alloc::sync::Arc;

use arrayvec::ArrayVec;
use lazy_static::lazy_static;

#[derive(Copy, Clone, Default)]
enum LocSelInputState {
    #[default]
    None,
    Confirm,
    Yaw,
    Regroup,
    Defend,
    SquadCancel,
    Cancel,
}

#[derive(Clone, Default)]
struct KeyState {
    down: bool,
    repeats: i32,
    binding: String,
    binding2: String,
}

#[derive(Clone, Default)]
struct Field {
    cursor: i32,
    scroll: i32,
    draw_width: i32,
    width_in_pixels: i32,
    char_height: f32,
    fixed_size: i32,
    buffer: Vec<u8>,
}

#[derive(Clone, Default)]
struct PlayerKeyState {
    char_field: Field,
    char_team: i32,
    overstrike_mode: i32,
    any_key_down: i32,
    keys: ArrayVec<KeyState, 256>,
    loc_sel_input_state: LocSelInputState,
}

lazy_static! {
    static ref PLAYER_KEYS: Arc<RwLock<PlayerKeyState>> =
        Arc::new(RwLock::new(PlayerKeyState::default()));
}

#[allow(unused_variables, clippy::print_stdout)]
pub fn clear_states(ids: isize) {
    PLAYER_KEYS.clone().write().unwrap().any_key_down = 0;
    println!("TODO - key::clear_states");
}
