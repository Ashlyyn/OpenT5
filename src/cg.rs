#![allow(dead_code)]

use core::time::Duration;

use crate::{
    dvar,
    util::{Angle, Point, Velocity},
};
use arrayvec::ArrayString;
use bitflags::bitflags;

#[derive(Copy, Clone, Default, Debug)]
pub struct ExtraButtons;

pub type Angles3 = (Angle, Angle, Angle);

#[derive(Copy, Clone, Default, Debug)]
pub struct PredictedVehicleInfo {
    in_vehicle: bool,
    origin: Point,
    angles: Angles3,
    tvel: Velocity,
    avel: Velocity,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Buttons;

#[derive(Copy, Clone, Default, Debug)]
pub struct WeaponId(u16);

#[derive(Copy, Clone, Default, Debug)]
pub struct OffhandId(u16);

#[derive(Copy, Clone, Default, Debug)]
pub struct UserCmd {
    server_time: Duration,
    buttons: Buttons,
    angles: Angles3,
    weapon: WeaponId,
    offhand_index: OffhandId,
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
    Headshots,
}

bitflags! {
    pub struct TalkFlags: u32 {

    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct ScoreCount(pub i32);

#[derive(Copy, Clone, Debug)]
pub struct UnarchivedMatchState {
    allies_score: ScoreCount,
    axis_score: ScoreCount,
    score_limit: ScoreCount,
    match_ui_visibility_flags: UiVisibilityFlags,
    scoreboard_column_types: [ScoreboardColumnType; 4],
    map_center: Point,
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

#[derive(Copy, Clone, Default, Debug)]
pub struct Xuid(pub u64);

#[derive(Copy, Clone, Default, Debug)]
pub enum VehicleAnimState {
    #[default]
    Idle,
    Entry,
    ChangePos,
    Exit,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Ping(pub i32);

#[derive(Copy, Clone, Default, Debug)]
pub struct StatusIconId(pub i32);

#[derive(Copy, Clone, Default, Debug)]
pub struct ScoreboardPlacement(pub i32);

#[derive(Copy, Clone, Debug)]
pub struct Score {
    ping: Ping,
    status_icon: StatusIconId,
    place: ScoreboardPlacement,
    score: ScoreCount,
    kills: i32,
    assists: i32,
    deaths: i32,
    scoreboard_columns: [ScoreboardColumnType; 4],
}

#[derive(Copy, Clone, Default, Debug)]
pub struct ClientId(pub u8);

#[derive(Copy, Clone, Default, Debug)]
pub struct ModelId(pub usize);

#[derive(Copy, Clone, Default, Debug)]
pub struct Rank(pub u16);

#[derive(Copy, Clone, Default, Debug)]
pub struct Prestige(pub u8);

#[derive(Clone, Default, Debug)]
pub struct ClientState {
    client_id: ClientId,
    team: Team,
    ffa_team: FfaTeam,
    model_idx: ModelId,
    attach_model_idx: [i32; 6],
    attach_tag_idx: [i32; 6],
    name: ArrayString<32>,
    max_sprint_time_multiplier: f32,
    rank: Rank,
    prestige: Prestige,
    last_damage_time: Duration,
    last_stand_start_time: Duration,
    xuid: Xuid,
    perks: [u32; 2],
    clan_abbrev: ArrayString<8>,
    attached_vehicle_ent_num: usize,
    attached_vehicle_seat: i32,
    needs_revive: bool,
    vehicle_anim_state: VehicleAnimState,
    score: ScoreCount,
    client_ui_visibility_flags: UiVisibilityFlags,
}

pub fn register_dvars() {
    dvar::register_int(
        "developer",
        0,
        Some(0),
        Some(2),
        dvar::DvarFlags::empty(),
        Some("Turn on Development systems"),
    )
    .unwrap();
}
