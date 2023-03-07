#![allow(dead_code)]

use crate::common::{Vec3f32, Vec3i32};
use arrayvec::{ArrayString};
use bitflags::bitflags;

#[derive(Copy, Clone, Default, Debug)]
pub struct ExtraButtons;

#[derive(Copy, Clone, Default, Debug)]
pub struct PredictedVehicleInfo {
    in_vehicle: bool,
    origin: Vec3f32,
    angles: Vec3f32,
    tvel: Vec3f32,
    avel: Vec3f32,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Buttons;

#[derive(Copy, Clone, Default, Debug)]
pub struct UserCmd {
    server_time: i32,
    buttons: Buttons,
    angles: Vec3i32,
    weapon: u16,
    offhand_index: u16,
    last_weapon_alt_mode_switch: u16,
    forward_move: i8,
    right_move: i8,
    up_move: i8,
    pitch_move: i8,
    yaw_move: i8,
    melee_charge_yaw: f32,
    melee_charge_dist: u8,
    rollmove: f32,
    selected_location: [u8; 2],
    selected_yaw: u8,
}

bitflags! {
    #[derive(Default)]
    pub struct UiVisibilityFlags: i32 {

    }
}

#[derive(Copy, Clone, Debug)]
pub struct ArchivedMatchState {
    match_ui_visibility_flags: UiVisibilityFlags,
    bomb_timer: [i32; 2],
}

#[derive(Copy, Clone, Debug)]
pub enum ScoreboardColumnType {
    Invalid,
    None,
    Kills,
    Deaths,
    Assists,
    Defends,
    Plants,
    Defuses,
    Returns,
    Captures,
    Destructions,
    KdRatio,
    Survived,
    Stabs,
    Tomahawks,
    Humiliated,
    X2Score,
    Headshots
}

bitflags! {
    pub struct TalkFlags: u32 {

    }
}

#[derive(Copy, Clone, Debug)]
pub struct UnarchivedMatchState {
    allies_score: i32,
    axis_score: i32,
    score_limit: i32,
    match_ui_visibility_flags: UiVisibilityFlags,
    scoreboard_column_types: [ScoreboardColumnType; 4],
    map_center: Vec3f32,
    talk_flags: TalkFlags,
}

#[derive(Copy, Clone, Debug)]
pub struct MatchState {
    idx: usize,
    archived_state: ArchivedMatchState,
    unarchived_state: UnarchivedMatchState,
}

#[derive(Copy, Clone, Default, Debug)]
pub enum Team {
    #[default]
    FreeOrBad,
    Axis,
    Allies,
    Spectator,
    LocalPlayers,
}

#[derive(Copy, Clone, Default, Debug)]
pub enum FfaTeam {
    #[default]
    None,
    Axis,
    Allies,
}

#[derive(Copy, Clone, Debug)]
pub enum Xuid {
    Xuid(u64),
    Xuid32([u32; 2])
}

impl Default for Xuid {
    fn default() -> Self {
        Self::Xuid(0)
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub enum VehicleAnimState {
    #[default]
    Idle,
    Entry,
    ChangePos,
    Exit,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Score {
    ping: i32,
    status_icon: i32,
    place: i32,
    score: i32,
    kills: i32,
    assists: i32,
    deaths: i32,
    scoreboard_columns: [i32; 4],
}

#[derive(Clone, Default, Debug)]
pub struct ClientState {
    client_idx: usize,
    team: Team,
    ffa_team: FfaTeam,
    model_idx: usize,
    attach_model_idx: [i32; 6],
    attach_tag_idx: [i32; 6],
    name: ArrayString<32>,
    max_sprint_time_multiplier: f32,
    rank: i32,
    prestige: i32,
    last_damage_time: i32,
    last_stand_start_time: i32,
    xuid: Xuid,
    perks: [u32; 2],
    clan_abbrev: ArrayString<8>,
    attached_vehicle_ent_num: usize,
    attached_vehicle_seat: i32,
    needs_revive: bool,
    vehicle_anim_state: VehicleAnimState,
    score: Score,
    client_ui_visibility_flags: UiVisibilityFlags,
}