#![allow(dead_code)]

extern crate alloc;
use alloc::sync::Arc;

use core::time::Duration;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use arrayvec::{ArrayString, ArrayVec};
use bitflags::bitflags;
use lazy_static::lazy_static;
use num_derive::FromPrimitive;

use crate::{
    cg::{self, Angles3, OffhandId, WeaponId},
    common::{StanceState, Vec3f32},
    util::{Angle, Point, Velocity},
};

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
    static ref CLIENT_UI_ACTIVES: Arc<RwLock<[ClientUiActive; 4]>> =
        Arc::new(RwLock::new([ClientUiActive::default(); 4]));
}

pub fn get_local_client_connection_state(local_client_num: usize) -> Connstate {
    CLIENT_UI_ACTIVES
        .clone()
        .read()
        .unwrap()
        .iter()
        .nth(local_client_num)
        .unwrap()
        .connection_state
}

pub fn get_local_client_ui_actives_mut(
) -> RwLockWriteGuard<'static, [ClientUiActive; 4]> {
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

const SKEL_MAX_MEM: usize = 0x0004_0000;

#[derive(Copy, Clone, Default, Debug)]
pub struct ServerId(i32);

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Default, Debug)]
pub struct ClientActive {
    pub using_ads: bool,
    pub timeout_count: i32,
    pub snap: Snapshot,
    pub server_time: Duration,
    pub old_server_time: Duration,
    pub old_frame_server_time: Duration,
    pub server_time_delta: Duration,
    pub old_snap_server_time: Duration,
    pub extrapolated_snapshot: i32,
    pub new_snapshots: i32,
    pub server_id: ServerId,
    pub mapname: ArrayString<64>,
    pub parse_match_state_num: i32,
    pub parse_entinties_num: i32,
    pub mouse_dx: [i32; 2],
    pub mouse_dy: [i32; 2],
    pub mouse_idx: i32,
    pub stance_held: bool,
    pub stance: StanceState,
    pub stance_position: StanceState,
    pub stance_time: Duration,
    pub cgame_user_cmd_weapon: WeaponId,
    pub cgame_user_cmd_offhand_index: OffhandId,
    pub cgame_user_cmd_last_weapon_for_alt: i32,
    pub cgame_fov_sensitivity_scale: f32,
    pub cgame_max_pitch_speed: f32,
    pub cgame_max_yaw_speed: f32,
    pub cgame_kick_angles: Angles3,
    pub cgame_origin: Point,
    pub cgame_velocity: Velocity,
    pub cgame_viewangles: Angles3,
    pub cgame_bob_cycle: i32,
    pub cgame_movement_dir: Angle,
    pub cgame_extra_buttons: cg::ExtraButtons,
    pub cgame_predicted_data_server_time: Duration,
    pub cgame_vehicle: cg::PredictedVehicleInfo,
    pub view_angles: Angles3,
    pub skel_timestamp: Duration,
    pub skel_mem_pos: isize,
    pub skel_memory: ArrayVec<u8, SKEL_MAX_MEM>,
    pub skel_memory_start: usize,
    pub allowed_alloc_skel: bool,
    pub cmds: ArrayVec<cg::UserCmd, 128>,
    pub cmd_num: i32,
    pub client_archive: ArrayVec<ClientArchiveData, 256>,
    pub client_archive_index: i32,
    pub out_packets: ArrayVec<OutPacket, 32>,
    pub snapshots: ArrayVec<Snapshot, 32>,
    pub entity_baselines: ArrayVec<EntityState, 1024>,
    pub parse_match_states: ArrayVec<cg::MatchState, 32>,
    pub parse_entities: ArrayVec<EntityState, 2048>,
    pub parse_clients: ArrayVec<cg::ClientState, 2048>,
    pub corrupted_translation_file: i32,
    pub translation_version: ArrayString<256>,
    pub last_fire_time: Duration,
    pub use_held: bool,
    pub use_time: Duration,
    pub use_count: i32,
    pub was_in_vehicle: i32,
}

lazy_static! {
    static ref CLIENTS: Arc<RwLock<Vec<ClientActive>>> =
        Arc::new(RwLock::new(Vec::new()));
}

pub fn get_local_client_globals() -> RwLockReadGuard<'static, Vec<ClientActive>>
{
    CLIENTS.read().unwrap()
}

pub fn get_local_client_globals_mut(
) -> RwLockWriteGuard<'static, Vec<ClientActive>> {
    CLIENTS.write().unwrap()
}
