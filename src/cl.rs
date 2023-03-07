#![allow(dead_code)]

extern crate alloc;
use alloc::sync::Arc;

use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use arrayvec::{ArrayString, ArrayVec};
use bitflags::bitflags;
use lazy_static::lazy_static;
use num_derive::FromPrimitive;

use crate::{common::{StanceState, Vec3f32}, cg};

#[derive(Copy, Clone, Default, Debug, FromPrimitive)]
#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
pub enum Connstate {
    #[default]
    DISCONNECTED = 0,
    CINEMATIC = 1,
    UICINEMATIC = 2,
    LOGO = 3,
    CONNECTING = 4,
    CHALLENGING = 5,
    CONNECTED = 6,
    SENDINGSTATS = 7,
    LOADING = 8,
    PRIMED = 9,
    ACTIVE = 10,
}

bitflags! { 
    #[derive(Default)]
    pub struct ClientUiActiveFlags: i32 {

    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct KeyCatchers;

#[derive(Copy, Clone, Default, Debug)]
pub struct ClientUiActive {
    flags: ClientUiActiveFlags,
    key_catchers: KeyCatchers,
    connection_state: Connstate,
    next_scroll_time: i32,
}

lazy_static! {
    static ref CLIENT_UI_ACTIVES: Arc<RwLock<[ClientUiActive; 4]>> = Arc::new(RwLock::new([ClientUiActive::default(); 4]));
}

pub fn get_local_client_connection_state(local_client_num: usize) -> Connstate {
    CLIENT_UI_ACTIVES.clone().read().unwrap().iter().nth(local_client_num).unwrap().connection_state
}

pub fn get_local_client_ui_actives_mut() -> RwLockWriteGuard<'static, [ClientUiActive; 4]> {
    CLIENT_UI_ACTIVES.write().unwrap()
}

// TODO - implement
#[derive(Copy, Clone, Default, Debug)]
pub struct Snapshot;

#[derive(Copy, Clone, Default, Debug)]
pub struct ClientArchiveData {
    server_time: i32,
    origin: Vec3f32,
    velocity: Vec3f32,
    bob_cycle: i32,
    movement_dir: i32,
    view_angles: Vec3f32,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct OutPacket {
    cmd_num: i32,
    server_time: i32,
    real_time: i32,
}

// TODO - implement
#[derive(Copy, Clone, Default, Debug)]
pub struct EntityState;

#[derive(Clone, Default, Debug)]
pub struct ClientActive {
    using_ads: bool,
    timeout_count: i32,
    snap: Snapshot,
    server_time: i32,
    old_server_time: i32,
    old_frame_server_time: i32,
    server_time_delta: i32,
    old_snap_server_time: i32,
    extrapolated_snapshot: i32,
    new_snapshots: i32,
    server_id: i32,
    mapname: ArrayString<64>,
    parse_match_state_num: i32,
    parse_entinties_num: i32,
    mouse_dx: [i32; 2],
    mouse_dy: [i32; 2],
    mouse_idx: i32,
    stance_held: bool,
    stance: StanceState,
    stance_position: StanceState,
    stance_time: i32,
    cgame_user_cmd_weapon: i32,
    cgame_user_cmd_offhand_index: i32,
    cgame_user_cmd_last_weapon_for_alt: i32,
    cgame_fov_sensitivity_scale: f32,
    cgame_max_pitch_speed: f32,
    cgame_max_yaw_speed: f32,
    cgame_kick_angles: Vec3f32,
    cgame_origin: Vec3f32,
    cgame_velocity: Vec3f32,
    cgame_viewangles: Vec3f32,
    cgame_bob_cycle: i32,
    cgame_movement_dir: i32,
    cgame_extra_buttons: cg::ExtraButtons,
    cgame_predicted_data_server_time: i32,
    cgame_vehicle: cg::PredictedVehicleInfo,
    view_angles: Vec3f32,
    skel_timestamp: i32,
    skel_mem_pos: isize,
    skel_memory: ArrayVec<u8, 262144>,
    skel_memory_start: usize,
    allowed_alloc_skel: bool,
    cmds: ArrayVec<cg::UserCmd, 128>,
    cmd_num: i32,
    client_archive: ArrayVec<ClientArchiveData, 256>,
    client_archive_index: i32,
    out_packets: ArrayVec<OutPacket, 32>,
    snapshots: ArrayVec<Snapshot, 32>,
    entity_baselines: ArrayVec<EntityState, 1024>,
    parse_match_states: ArrayVec<cg::MatchState, 32>,
    parse_entities: ArrayVec<EntityState, 2048>,
    parse_clients: ArrayVec<cg::ClientState, 2048>,
    corrupted_translation_file: i32,
    translation_version: ArrayString<256>,
    last_fire_time: i32,
    pub use_held: bool,
    pub use_time: i32,
    pub use_count: i32,
    was_in_vehicle: i32,
}

lazy_static! {
    static ref CLIENTS: Arc<RwLock<Vec<ClientActive>>> = Arc::new(RwLock::new(Vec::new()));
}   

pub fn get_local_client_globals() -> RwLockReadGuard<'static, Vec<ClientActive>> {
    CLIENTS.read().unwrap()
}

pub fn get_local_client_globals_mut() -> RwLockWriteGuard<'static, Vec<ClientActive>> {
    CLIENTS.write().unwrap()
}