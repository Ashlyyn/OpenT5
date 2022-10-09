#![allow(dead_code)]

use crate::*;
use bitflags::bitflags;
use common::*;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, RwLock};
use std::vec::Vec;

// DvarLimitsXXXXX hold the domain for each possible type of Dvar
// Display is impl'ed to print said domain
// Default is impl'ed where possible, and should always resolve to the largest
// possible domain

// bool has no custom-definable domain, it'll always be 0 or 1/true or false
// DvarLimitsBool still needs to be defined for printing the domain
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct DvarLimitsBool {}

impl Display for DvarLimitsBool {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Domain is 0 or 1")
    }
}

impl DvarLimitsBool {
    pub fn new() -> Self {
        DvarLimitsBool {}
    }
}

// The domain for a float consists of a min and a max
// Min should never be greater than max, but they may be equal
#[derive(Copy, Clone, PartialEq)]
pub struct DvarLimitsFloat {
    pub min: f32,
    pub max: f32,
}

impl Default for DvarLimitsFloat {
    fn default() -> Self {
        DvarLimitsFloat {
            min: f32::MIN,
            max: f32::MAX,
        }
    }
}

impl Display for DvarLimitsFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.min == f32::MIN {
            if self.max == f32::MAX {
                write!(f, "Domain is any number")
            } else {
                write!(f, "Domain is any number {} or smaller", self.max)
            }
        } else if self.max == f32::MAX {
            write!(f, "Domain is any number {} or bigger", self.min)
        } else {
            write!(f, "Domain is any number from {} to {}", self.min, self.max)
        }
    }
}

impl DvarLimitsFloat {
    pub fn new(min: f32, max: f32) -> Self {
        // Panic if max is greater than min
        // (possibly implement better error handling in the future)
        if min > max {
            panic!("DvarLimitsFloat::new(): supplied min is greater than max");
        }

        DvarLimitsFloat { min, max }
    }
}

// The domain for a vector follows the same rules as a float
// Each element is bound the same min/max
#[derive(Copy, Clone, PartialEq)]
pub struct DvarLimitsVector2 {
    pub min: f32,
    pub max: f32,
}

impl Default for DvarLimitsVector2 {
    fn default() -> Self {
        DvarLimitsVector2 {
            min: f32::MIN,
            max: f32::MAX,
        }
    }
}

impl Display for DvarLimitsVector2 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.min == f32::MIN {
            if self.max == f32::MAX {
                write!(f, "Domain is any 2D vector")
            } else {
                write!(
                    f,
                    "Domain is any 2D vector with components {} or smaller",
                    self.max
                )
            }
        } else if self.max == f32::MAX {
            write!(
                f,
                "Domain is any 2D vector with components {} or bigger",
                self.min
            )
        } else {
            write!(
                f,
                "Domain is any 2D vector with components from {} to {}",
                self.min, self.max
            )
        }
    }
}

impl DvarLimitsVector2 {
    pub fn new(min: f32, max: f32) -> Self {
        DvarLimitsVector2 { min, max }
    }
}

// Same rules as Vector2
#[derive(Copy, Clone, PartialEq)]
pub struct DvarLimitsVector3 {
    pub min: f32,
    pub max: f32,
}

impl Default for DvarLimitsVector3 {
    fn default() -> Self {
        DvarLimitsVector3 {
            min: f32::MIN,
            max: f32::MAX,
        }
    }
}

impl Display for DvarLimitsVector3 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.min == f32::MIN {
            if self.max == f32::MAX {
                write!(f, "Domain is any 3D vector")
            } else {
                write!(
                    f,
                    "Domain is any 3D vector with components {} or smaller",
                    self.max
                )
            }
        } else if self.max == f32::MAX {
            write!(
                f,
                "Domain is any 3D vector with components {} or bigger",
                self.min
            )
        } else {
            write!(
                f,
                "Domain is any 3D vector with components from {} to {}",
                self.min, self.max
            )
        }
    }
}

impl DvarLimitsVector3 {
    pub fn new(n: f32, m: f32) -> Self {
        DvarLimitsVector3 { min: n, max: m }
    }
}

// Same rules as Vector2/Vector3
#[derive(Copy, Clone, PartialEq)]
pub struct DvarLimitsVector4 {
    pub min: f32,
    pub max: f32,
}

impl Default for DvarLimitsVector4 {
    fn default() -> Self {
        DvarLimitsVector4 {
            min: f32::MIN,
            max: f32::MAX,
        }
    }
}

impl Display for DvarLimitsVector4 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.min == f32::MIN {
            if self.max == f32::MAX {
                write!(f, "Domain is any 4D vector")
            } else {
                write!(
                    f,
                    "Domain is any 4D vector with components {} or smaller",
                    self.max
                )
            }
        } else if self.max == f32::MAX {
            write!(
                f,
                "Domain is any 4D vector with components {} or bigger",
                self.min
            )
        } else {
            write!(
                f,
                "Domain is any 4D vector with components from {} to {}",
                self.min, self.max
            )
        }
    }
}

impl DvarLimitsVector4 {
    pub fn new(n: f32, m: f32) -> Self {
        DvarLimitsVector4 { min: n, max: m }
    }
}

// Same rules as Float
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct DvarLimitsInt {
    pub min: i32,
    pub max: i32,
}

impl Default for DvarLimitsInt {
    fn default() -> Self {
        DvarLimitsInt {
            min: i32::MIN,
            max: i32::MAX,
        }
    }
}

impl Display for DvarLimitsInt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.min == i32::MIN {
            if self.max == i32::MAX {
                write!(f, "Domain is any integer")
            } else {
                write!(f, "Domain is any integer {} or smaller", self.max)
            }
        } else if self.max == i32::MAX {
            write!(f, "Domain is any integer {} or bigger", self.min)
        } else {
            write!(f, "Domain is any integer from {} to {}", self.min, self.max)
        }
    }
}

impl DvarLimitsInt {
    pub fn new(n: i32, m: i32) -> Self {
        DvarLimitsInt { min: n, max: m }
    }
}

// String has no custom-definable domain
// Still implemented for the same reason as Bool
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct DvarLimitsString {}

impl Display for DvarLimitsString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Domain is any text")
    }
}

impl DvarLimitsString {
    pub fn new() -> Self {
        DvarLimitsString {}
    }
}

// Domain for enumeration is a set of possible strings
#[derive(Clone, Default, PartialEq, Eq)]
pub struct DvarLimitsEnumeration {
    pub strings: Vec<String>,
}

impl Display for DvarLimitsEnumeration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Domain is any one of the following:")
            .unwrap_or_else(|e| panic!("{}", e));
        for (i, s) in self.strings.iter().enumerate() {
            write!(f, "\n  {:2}: {}", i, s).unwrap_or_else(|e| panic!("{}", e));
        }

        std::fmt::Result::Ok(())
    }
}

impl DvarLimitsEnumeration {
    pub fn new(s: &[String]) -> Self {
        DvarLimitsEnumeration {
            strings: s.to_vec(),
        }
    }
}

// Color has no custom-definable domain
// All valid RGBA values are allowed
#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct DvarLimitsColor {}

impl Display for DvarLimitsColor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Domain is any 4-component color, in RGBA format")
    }
}

impl DvarLimitsColor {
    pub fn new() -> Self {
        DvarLimitsColor {}
    }
}

// Same rules as Int
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct DvarLimitsInt64 {
    pub min: i64,
    pub max: i64,
}

impl Default for DvarLimitsInt64 {
    fn default() -> Self {
        DvarLimitsInt64 {
            min: i64::MIN,
            max: i64::MAX,
        }
    }
}

impl Display for DvarLimitsInt64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.min == i64::MIN {
            if self.max == i64::MAX {
                write!(f, "Domain is any integer")
            } else {
                write!(f, "Domain is any integer {} or smaller", self.max)
            }
        } else if self.max == i64::MAX {
            write!(f, "Domain is any integer {} or bigger", self.min)
        } else {
            write!(f, "Domain is any integer from {} to {}", self.min, self.max)
        }
    }
}

impl DvarLimitsInt64 {
    pub fn new(n: i64, m: i64) -> Self {
        DvarLimitsInt64 { min: n, max: m }
    }
}

// Same rules as Vector3
#[derive(Copy, Clone, PartialEq)]
pub struct DvarLimitsLinearColorRGB {
    pub min: f32,
    pub max: f32,
}

impl Default for DvarLimitsLinearColorRGB {
    fn default() -> Self {
        DvarLimitsLinearColorRGB {
            min: f32::MIN,
            max: f32::MAX,
        }
    }
}

impl Display for DvarLimitsLinearColorRGB {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.min == f32::MIN {
            if self.max == f32::MAX {
                write!(f, "Domain is any 3D vector")
            } else {
                write!(
                    f,
                    "Domain is any 3D vector with components {} or smaller",
                    self.max
                )
            }
        } else if self.max == f32::MAX {
            write!(
                f,
                "Domain is any 3D vector with components {} or bigger",
                self.min
            )
        } else {
            write!(
                f,
                "Domain is any 3D vector with components from {} to {}",
                self.min, self.max
            )
        }
    }
}

impl DvarLimitsLinearColorRGB {
    pub fn new(n: f32, m: f32) -> Self {
        DvarLimitsLinearColorRGB { min: n, max: m }
    }
}

// Same rules as Vector3 and LinearColorRGB
#[derive(Copy, Clone, PartialEq)]
pub struct DvarLimitsColorXYZ {
    pub min: f32,
    pub max: f32,
}

impl Default for DvarLimitsColorXYZ {
    fn default() -> Self {
        DvarLimitsColorXYZ {
            min: f32::MIN,
            max: f32::MAX,
        }
    }
}

impl Display for DvarLimitsColorXYZ {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.min == f32::MIN {
            if self.max == f32::MAX {
                write!(f, "Domain is any 3D vector")
            } else {
                write!(
                    f,
                    "Domain is any 3D vector with components {} or smaller",
                    self.max
                )
            }
        } else if self.max == f32::MAX {
            write!(
                f,
                "Domain is any 3D vector with components {} or bigger",
                self.min
            )
        } else {
            write!(
                f,
                "Domain is any 3D vector with components from {} to {}",
                self.min, self.max
            )
        }
    }
}

impl DvarLimitsColorXYZ {
    pub fn new(n: f32, m: f32) -> Self {
        DvarLimitsColorXYZ { min: n, max: m }
    }
}

// Enum to tie all the DvarLimitsXXXX's together
#[derive(Clone)]
pub enum DvarLimits {
    Bool(DvarLimitsBool),
    Float(DvarLimitsFloat),
    Vector2(DvarLimitsVector2),
    Vector3(DvarLimitsVector3),
    Vector4(DvarLimitsVector4),
    Int(DvarLimitsInt),
    String(DvarLimitsString),
    Enumeration(DvarLimitsEnumeration),
    Color(DvarLimitsColor),
    Int64(DvarLimitsInt64),
    LinearColorRGB(DvarLimitsLinearColorRGB),
    ColorXYZ(DvarLimitsColorXYZ),
}

// Display should display the domain of the current value
impl Display for DvarLimits {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{}", b),
            Self::Float(n) => write!(f, "{}", n),
            Self::Vector2(v) => write!(f, "{}", v),
            Self::Vector3(v) => write!(f, "{}", v),
            Self::Vector4(v) => write!(f, "{}", v),
            Self::Int(i) => write!(f, "{}", i),
            Self::String(s) => write!(f, "{}", s),
            Self::Enumeration(e) => write!(f, "{}", e),
            Self::Color(c) => write!(f, "{}", c),
            Self::Int64(i) => write!(f, "{}", i),
            Self::LinearColorRGB(c) => write!(f, "{}", c),
            Self::ColorXYZ(c) => write!(f, "{}", c),
        }
    }
}

impl DvarLimits {
    // A bunch of helper functions to extract the domain
    // Useful if a given Dvar is known to be a specific type
    // Otherwise long match expressions would be required
    pub fn as_bool_limits(&self) -> Option<DvarLimitsBool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_float_limits(&self) -> Option<DvarLimitsFloat> {
        match self {
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_vector2_limits(&self) -> Option<DvarLimitsVector2> {
        match self {
            Self::Vector2(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_vector3_limits(&self) -> Option<DvarLimitsVector3> {
        match self {
            Self::Vector3(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_vector4_limits(&self) -> Option<DvarLimitsVector4> {
        match self {
            Self::Vector4(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_int_limits(&self) -> Option<DvarLimitsInt> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_string_limits(&self) -> Option<DvarLimitsString> {
        match self {
            Self::String(s) => Some(*s),
            _ => None,
        }
    }

    pub fn as_enumeration_limits(&self) -> Option<DvarLimitsEnumeration> {
        match self {
            Self::Enumeration(v) => Some(v.clone()),
            _ => None,
        }
    }

    pub fn as_color_limits(&self) -> Option<DvarLimitsColor> {
        match self {
            Self::Color(c) => Some(*c),
            _ => None,
        }
    }

    pub fn as_int64_limits(&self) -> Option<DvarLimitsInt64> {
        match self {
            Self::Int64(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_linear_color_rgb_limits(
        &self,
    ) -> Option<DvarLimitsLinearColorRGB> {
        match self {
            Self::LinearColorRGB(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_color_xyz_limits(&self) -> Option<DvarLimitsColorXYZ> {
        match self {
            Self::ColorXYZ(v) => Some(*v),
            _ => None,
        }
    }
}

// Enum to hold all possible Dvar values
#[derive(Clone, PartialEq)]
pub enum DvarValue {
    Bool(bool),
    Float(f32),
    Vector2(Vec2f32),
    Vector3(Vec3f32),
    Vector4(Vec4f32),
    Int(i32),
    String(String),
    Enumeration(String),
    Color(Vec4f32),
    Int64(i64),
    LinearColorRGB(Vec3f32),
    ColorXYZ(Vec3f32),
}

// Display should display the current value
impl Display for DvarValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{}", b),
            Self::Float(v) => write!(f, "{}", v),
            Self::Vector2(v) => write!(f, "({}, {})", v.0, v.1),
            Self::Vector3(v) => write!(f, "({}, {}, {})", v.0, v.1, v.2),
            Self::Vector4(v) => {
                write!(f, "({}, {}, {}, {})", v.0, v.1, v.2, v.3)
            }
            Self::Int(i) => write!(f, "{}", i),
            Self::String(s) => write!(f, "{}", s),
            Self::Enumeration(s) => write!(f, "{}", s),
            Self::Color(c) => write!(f, "({}, {}, {})", c.0, c.1, c.2),
            Self::Int64(i) => write!(f, "{}", i),
            Self::LinearColorRGB(c) => write!(f, "({}, {}, {})", c.0, c.1, c.2),
            Self::ColorXYZ(c) => write!(f, "({}, {}, {})", c.0, c.1, c.2),
        }
    }
}

impl DvarValue {
    // Helper functions defined for the same reason as in DvarLimits
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_vector2(&self) -> Option<Vec2f32> {
        match self {
            Self::Vector2(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_vector3(&self) -> Option<Vec3f32> {
        match self {
            Self::Vector3(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_vector4(&self) -> Option<Vec4f32> {
        match self {
            Self::Vector4(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }
    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::String(s) => Some(s.to_string()),
            _ => None,
        }
    }

    pub fn as_enumeration(&self) -> Option<String> {
        match self {
            Self::Enumeration(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn as_color(&self) -> Option<Vec4f32> {
        match self {
            Self::Color(c) => Some(*c),
            _ => None,
        }
    }

    pub fn as_int64(&self) -> Option<i64> {
        match self {
            Self::Int64(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_linear_color_rgb(&self) -> Option<Vec3f32> {
        match self {
            Self::LinearColorRGB(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_color_xyz(&self) -> Option<Vec3f32> {
        match self {
            Self::ColorXYZ(v) => Some(*v),
            _ => None,
        }
    }
}

// Enum for the possible sources a Dvar may be set from
#[derive(PartialEq, Eq)]
pub enum SetSource {
    Internal,
    External,
    Script,
    Devgui,
}

bitflags! {
    #[derive(Default)]
    pub struct DvarFlags: u32 {
        /// Flag with unknown purpose. Never(?) manually set by a function,
        /// but some Dvars are registered with it set by default
        const UNKNOWN_00000001_A    = 0x00000001;
        /// Flag with unknown purpose. Never(?) manually set by a function,
        /// but some Dvars are registered with it set by default
        const UNKNOWN_00000002_U    = 0x00000002;
        /// Flag with unknown purpose. Only used once or twice
        const UNKNOWN_00000004      = 0x00000004;
        /// Flag related to system info (not quite sure what "system info"
        /// means in this context)
        const SYSTEM_INFO           = 0x00000008;
        /// Flag to write-protect a Dvar. Dvar cannot be written to if flag
        /// is set, but may still be updated internally
        const WRITE_PROTECTED       = 0x00000010;
        /// Flag denoting if Dvar has latched value. If set, the Dvar does
        /// have a latched value, and that value can be loaded with, e.g.,
        /// dvar::make_latched_value_current()
        const LATCHED               = 0x00000020;
        /// Flag denoting if Dvar is read-only. If set, the Dvar cannot be
        /// modified
        const READ_ONLY             = 0x00000040;
        /// Flag denoting if Dvar is cheat-protected. Prevents Dvar from
        /// being modified if sv_cheats is not set to true
        const CHEAT_PROTECTED       = 0x00000080;
        /// Flag with unknown purpose. Never(?) manually set by a function,
        /// but some Dvars are registered with it set by default
        const UNKNOWN_00000100_D    = 0x00000100;
        /// Flag denoting if Dvar's reset value may be changed
        const CHANGEABLE_RESET      = 0x00000200;
        /// Flag with unknown purpose. Never(?) manually set by a function,
        /// but some Dvars are registered with it set by default
        const UNKNOWN_00000400      = 0x00000400;
        /// Flag denoting if Dvar can be modified from from Devgui
        const ALLOW_SET_FROM_DEVGUI = 0x00000800;
        /// Flag denoting if Dvar is saved. If set, the Dvar will be preserved
        /// across game launches
        const SAVED                 = 0x00001000;
        /// Flag with unknown purpose. Never(?) manually set by a function,
        /// but some Dvars are registered with it set by default
        const UNKNOWN_00002000      = 0x00002000;
        /// Flag denoting if Dvar is external. It seems to mostly apply to
        /// Dvars created from the Devgui, and some dynamically created by
        /// other modules of the engine
        const EXTERNAL              = 0x00004000;
        /// Flag with unknown purpose. Nothing is known besides the name
        const AUTOEXEC              = 0x00008000;
        /// Flag to allow Dvar to be accessed when Con access is restricted
        const RESTRICTED            = 0x00010000;
    }
}

lazy_static! {
    static ref MODIFIED_FLAGS: Arc<RwLock<DvarFlags>> =
        Arc::new(RwLock::new(DvarFlags::empty()));
}

// Finally, the Dvar itself
#[derive(Clone)]
pub struct Dvar {
    // Name of Dvar
    pub name: String,
    // Description of Dvar (optional)
    pub description: String,
    // Flags for Dvar (write-protected, cheat-protected, read-only, etc.)
    pub flags: DvarFlags,
    // Flag to check if the Dvar has been modified
    // Not included in the actual flags for some reason
    pub modified: bool,
    // Flag to check if Dvar was loaded from a saved game
    // Also not in the actual flags for some reason
    pub loaded_from_save_game: bool,
    // Domain of Dvar
    pub domain: DvarLimits,
    // Current value of Dvar
    pub current: DvarValue,
    // Latched value of Dvar
    // (seems to be the value it defaults to on restart)
    latched: DvarValue,
    // Reset value of Dvar
    // (seems to be used when a Dvar is manually reset,
    //  or when the current value flags the Dvar as a cheat
    //  and cheats are subsequently disabled)
    reset: DvarValue,
    // Saved value of Dvar
    // (value used on loading a save game?)
    saved: DvarValue,
}

impl Display for Dvar {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} - {} - {}", self.name, self.description, self.current)
    }
}

// Dvars should only be compared by name, to prevent multiple commands
// with the same name but different remaining fields from being allowed in
// associative containers
impl PartialEq for Dvar {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

// impl Eq to keep the compiler and clippy happy
impl Eq for Dvar {}

// Hash only the name for the same reason as PartialEq
impl Hash for Dvar {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Dvar {
    // Clamp a supplied value to the supplied domain if possible
    fn clamp_value_to_domain(
        value: &mut DvarValue,
        domain: DvarLimits,
    ) -> DvarValue {
        match value {
            DvarValue::Bool(_) => value.clone(),
            DvarValue::Float(f) => DvarValue::Float(f.clamp(
                domain.as_float_limits().unwrap().min,
                domain.as_float_limits().unwrap().max,
            )),
            DvarValue::Vector2(v) => DvarValue::Vector2((
                v.0.clamp(
                    domain.as_vector2_limits().unwrap().min,
                    domain.as_vector2_limits().unwrap().max,
                ),
                v.1.clamp(
                    domain.as_vector2_limits().unwrap().min,
                    domain.as_vector2_limits().unwrap().max,
                ),
            )),
            DvarValue::Vector3(v) => DvarValue::Vector3((
                v.0.clamp(
                    domain.as_vector3_limits().unwrap().min,
                    domain.as_vector3_limits().unwrap().max,
                ),
                v.1.clamp(
                    domain.as_vector3_limits().unwrap().min,
                    domain.as_vector3_limits().unwrap().max,
                ),
                v.2.clamp(
                    domain.as_vector3_limits().unwrap().min,
                    domain.as_vector3_limits().unwrap().max,
                ),
            )),
            DvarValue::Vector4(v) => DvarValue::Vector4((
                v.0.clamp(
                    domain.as_vector4_limits().unwrap().min,
                    domain.as_vector4_limits().unwrap().max,
                ),
                v.1.clamp(
                    domain.as_vector4_limits().unwrap().min,
                    domain.as_vector4_limits().unwrap().max,
                ),
                v.2.clamp(
                    domain.as_vector4_limits().unwrap().min,
                    domain.as_vector4_limits().unwrap().max,
                ),
                v.3.clamp(
                    domain.as_vector4_limits().unwrap().min,
                    domain.as_vector4_limits().unwrap().max,
                ),
            )),
            DvarValue::Int(i) => {
                let min: i32 = domain.as_int_limits().unwrap().min;
                let max: i32 = domain.as_int_limits().unwrap().max;
                if *i < min {
                    DvarValue::Int(min)
                } else if *i > max {
                    DvarValue::Int(max)
                } else {
                    DvarValue::Int(*i)
                }
            }
            DvarValue::String(_) => value.clone(),
            DvarValue::Enumeration(_) => value.clone(),
            DvarValue::Color(_) => value.clone(),
            DvarValue::Int64(i) => {
                let min: i64 = domain.as_int64_limits().unwrap().min;
                let max: i64 = domain.as_int64_limits().unwrap().max;
                if *i < min {
                    DvarValue::Int64(min)
                } else if *i > max {
                    DvarValue::Int64(max)
                } else {
                    DvarValue::Int64(*i)
                }
            }
            DvarValue::LinearColorRGB(v) => DvarValue::LinearColorRGB((
                v.0.clamp(
                    domain.as_linear_color_rgb_limits().unwrap().min,
                    domain.as_linear_color_rgb_limits().unwrap().max,
                ),
                v.1.clamp(
                    domain.as_linear_color_rgb_limits().unwrap().min,
                    domain.as_linear_color_rgb_limits().unwrap().max,
                ),
                v.2.clamp(
                    domain.as_linear_color_rgb_limits().unwrap().min,
                    domain.as_linear_color_rgb_limits().unwrap().max,
                ),
            )),
            DvarValue::ColorXYZ(v) => DvarValue::ColorXYZ((
                v.0.clamp(
                    domain.as_color_xyz_limits().unwrap().min,
                    domain.as_color_xyz_limits().unwrap().max,
                ),
                v.1.clamp(
                    domain.as_color_xyz_limits().unwrap().min,
                    domain.as_color_xyz_limits().unwrap().max,
                ),
                v.2.clamp(
                    domain.as_color_xyz_limits().unwrap().min,
                    domain.as_color_xyz_limits().unwrap().max,
                ),
            )),
        }
    }

    fn clamp_current_value_to_domain(&mut self) {
        Self::clamp_value_to_domain(&mut self.current, self.domain.clone());
    }

    fn clamp_latched_value_to_domain(&mut self) {
        Self::clamp_value_to_domain(&mut self.latched, self.domain.clone());
    }

    fn clamp_reset_value_to_domain(&mut self) {
        Self::clamp_value_to_domain(&mut self.reset, self.domain.clone());
    }

    fn clamp_all_values_to_domain(&mut self) {
        self.clamp_current_value_to_domain();
        self.clamp_latched_value_to_domain();
        self.clamp_reset_value_to_domain();
    }

    fn has_latched_value(&self) -> bool {
        self.current != self.latched
    }

    fn any_latched_values() -> bool {
        for (_, d) in DVARS.try_read().expect("dvar::any_latched_values: failed to acquire reader lock. Probably still held by calling function.").iter() {
            if d.has_latched_value() {
                return true;
            }
        }
        false
    }

    fn can_change_value(
        &self,
        value: DvarValue,
        set_source: SetSource,
    ) -> bool {
        if value == self.reset {
            return true;
        }

        if self.flags.contains(DvarFlags::READ_ONLY) {
            com::println(&format!("{} is read only.", self.name));
            return false;
        }

        if self.flags.contains(DvarFlags::WRITE_PROTECTED) {
            com::println(&format!(
                "{} is write protected protected.",
                self.name
            ));
            return false;
        }

        if self.flags.contains(DvarFlags::CHEAT_PROTECTED)
            && (find("sv_cheats").unwrap().current.as_bool().unwrap() == false)
        {
            true
        } else {
            if (set_source == SetSource::External)
                || (set_source == SetSource::Script)
            {
                com::println(&format!("{} is cheat protected.", self.name));
            }
            false
        }
    }

    // Overwrite latched value with current
    fn clear_latched_value(&mut self) {
        if self.has_latched_value() {
            self.latched = self.current.clone();
        }
    }

    fn set_value(&mut self, value: DvarValue, source: SetSource) {
        if source == SetSource::External || source == SetSource::Script {
            if self.can_change_value(value.clone(), source) == false {
                return;
            }
            if self.flags.contains(DvarFlags::LATCHED) {
                self.latched = value.clone();
                if self.current != self.latched {
                    com::println(&format!(
                        "{} will be changed upon restarting.",
                        self.name
                    ));
                    return;
                }
            }
        } else if source == SetSource::Devgui
            && self.flags.contains(DvarFlags::ALLOW_SET_FROM_DEVGUI)
        {
            self.latched = value;
            return;
        }

        if value != self.current {
            self.current = value;
            self.modified = true;
        } else {
            self.latched = value;
        }
    }

    fn add_flags(&mut self, flags: DvarFlags) {
        self.flags |= flags;
    }

    fn clear_flags(&mut self, flags: DvarFlags) {
        self.flags &= !flags;
    }

    // Helper function to check if supplied value is within supplied domain
    fn value_is_in_domain(domain: DvarLimits, value: DvarValue) -> bool {
        match value {
            DvarValue::Bool(_) => true,
            DvarValue::Float(f) => {
                f < domain.as_float_limits().unwrap().min
                    || f > domain.as_float_limits().unwrap().max
            }
            DvarValue::Vector2(v) => {
                v.0 < domain.as_vector2_limits().unwrap().min
                    || v.0 > domain.as_vector2_limits().unwrap().max
                    || v.1 < domain.as_vector2_limits().unwrap().min
                    || v.1 > domain.as_vector2_limits().unwrap().max
            }
            DvarValue::Vector3(v) => {
                v.0 < domain.as_vector3_limits().unwrap().min
                    || v.0 > domain.as_vector3_limits().unwrap().max
                    || v.1 < domain.as_vector3_limits().unwrap().min
                    || v.1 > domain.as_vector3_limits().unwrap().max
                    || v.2 < domain.as_vector3_limits().unwrap().min
                    || v.2 > domain.as_vector3_limits().unwrap().max
            }
            DvarValue::Vector4(v) => {
                v.0 < domain.as_vector4_limits().unwrap().min
                    || v.0 > domain.as_vector4_limits().unwrap().max
                    || v.1 < domain.as_vector4_limits().unwrap().min
                    || v.1 > domain.as_vector4_limits().unwrap().max
                    || v.2 < domain.as_vector4_limits().unwrap().min
                    || v.2 > domain.as_vector4_limits().unwrap().max
                    || v.3 < domain.as_vector4_limits().unwrap().min
                    || v.3 > domain.as_vector4_limits().unwrap().max
            }
            DvarValue::Int(i) => {
                i < domain.as_int_limits().unwrap().min
                    || i > domain.as_int_limits().unwrap().max
            }
            DvarValue::String(_) => true,
            DvarValue::Enumeration(v) => {
                for s in domain.as_enumeration_limits().unwrap().strings.iter()
                {
                    if v == *s {
                        return true;
                    }
                }
                false
            }
            DvarValue::Color(_) => true,
            DvarValue::Int64(i) => {
                i < domain.as_int64_limits().unwrap().min
                    || i > domain.as_int64_limits().unwrap().max
            }
            DvarValue::LinearColorRGB(v) => {
                v.0 < domain.as_linear_color_rgb_limits().unwrap().min
                    || v.0 > domain.as_linear_color_rgb_limits().unwrap().max
                    || v.1 < domain.as_linear_color_rgb_limits().unwrap().min
                    || v.1 > domain.as_linear_color_rgb_limits().unwrap().max
                    || v.2 < domain.as_linear_color_rgb_limits().unwrap().min
                    || v.2 > domain.as_linear_color_rgb_limits().unwrap().max
            }
            DvarValue::ColorXYZ(v) => {
                v.0 < domain.as_color_xyz_limits().unwrap().min
                    || v.0 > domain.as_color_xyz_limits().unwrap().max
                    || v.1 < domain.as_color_xyz_limits().unwrap().min
                    || v.1 > domain.as_color_xyz_limits().unwrap().max
                    || v.2 < domain.as_color_xyz_limits().unwrap().min
                    || v.2 > domain.as_color_xyz_limits().unwrap().max
            }
        }
    }

    fn set_variant(&mut self, value: DvarValue, source: SetSource) {
        if self.name.is_empty() {
            return;
        }

        if com::log_file_open() && self.current != value {
            com::println(&format!(
                "      dvar set {} {}",
                self.name, self.current
            ));
        }

        if !Self::value_is_in_domain(self.domain.clone(), value.clone()) {
            com::println(&format!(
                "\'{}\' is not a valid value for dvar \'{}\'",
                value, self.name
            ));
            com::println(&format!("{}", self.domain));
            if let DvarValue::Enumeration(_) = value {
                self.set_variant(self.reset.to_owned(), source);
            }
            return;
        }

        if source == SetSource::External || source == SetSource::Script {
            if self.can_change_value(value.clone(), source)
                && self.flags.contains(DvarFlags::LATCHED)
            {
                self.latched = value;
                if self.latched != self.current {
                    com::println(&format!(
                        "{} will be changed upon restarting.",
                        self.name
                    ))
                }
            }
            return;
        } else if source == SetSource::Devgui
            && self.flags.contains(DvarFlags::ALLOW_SET_FROM_DEVGUI)
        {
            self.latched = value;
            return;
        }

        if self.current != value {
            let modified_flags = *MODIFIED_FLAGS.try_read().expect("dvar::Dvar::set_variant: failed to acquire reader lock. Probably still held by calling function.");
            MODIFIED_FLAGS
                .try_write()
                .expect("dvar::Dvar::set_variant: failed to acquire writer lock. Probably still held by calling function.")
                .insert(modified_flags | self.flags);
            self.current = value;
            self.modified = true;
        } else {
            self.latched = self.current.clone();
        }
    }

    fn make_latched_value_current(&mut self) {
        self.set_variant(self.latched.clone(), SetSource::Internal);
    }

    fn reset(&mut self, source: SetSource) {
        self.set_variant(self.reset.clone(), source);
    }
}

macro_rules! zero_variant_enum {
    ($e:ident) => {
        #[derive(Copy, Clone, Default, PartialEq, Eq, Hash)]
        enum $e {
            #[default]
            __,
        }
    };
}

// Enums for [`DvarBuilder`]'s various typestates.
zero_variant_enum!(DvarBuilderStartState);
zero_variant_enum!(DvarBuilderDataState);
zero_variant_enum!(DvarBuilderTypeState);
zero_variant_enum!(DvarBuilderTypeBoolCurrentValueState);
zero_variant_enum!(DvarBuilderTypeBoolOtherValuesState);
zero_variant_enum!(DvarBuilderTypeFloatDomainState);
zero_variant_enum!(DvarBuilderTypeFloatCurrentValueState);
zero_variant_enum!(DvarBuilderTypeFloatOtherValuesState);
zero_variant_enum!(DvarBuilderTypeVector2DomainState);
zero_variant_enum!(DvarBuilderTypeVector2CurrentValueState);
zero_variant_enum!(DvarBuilderTypeVector2OtherValuesState);
zero_variant_enum!(DvarBuilderTypeVector3DomainState);
zero_variant_enum!(DvarBuilderTypeVector3CurrentValueState);
zero_variant_enum!(DvarBuilderTypeVector3OtherValuesState);
zero_variant_enum!(DvarBuilderTypeVector4DomainState);
zero_variant_enum!(DvarBuilderTypeVector4CurrentValueState);
zero_variant_enum!(DvarBuilderTypeVector4OtherValuesState);
zero_variant_enum!(DvarBuilderTypeIntDomainState);
zero_variant_enum!(DvarBuilderTypeIntCurrentValueState);
zero_variant_enum!(DvarBuilderTypeIntOtherValuesState);
zero_variant_enum!(DvarBuilderTypeStringCurrentValueState);
zero_variant_enum!(DvarBuilderTypeStringOtherValuesState);
zero_variant_enum!(DvarBuilderTypeEnumerationDomainState);
zero_variant_enum!(DvarBuilderTypeEnumerationCurrentValueState);
zero_variant_enum!(DvarBuilderTypeEnumerationOtherValuesState);
zero_variant_enum!(DvarBuilderTypeColorCurrentValueState);
zero_variant_enum!(DvarBuilderTypeColorOtherValuesState);
zero_variant_enum!(DvarBuilderTypeInt64DomainState);
zero_variant_enum!(DvarBuilderTypeInt64CurrentValueState);
zero_variant_enum!(DvarBuilderTypeInt64OtherValuesState);
zero_variant_enum!(DvarBuilderTypeLinearColorRGBDomainState);
zero_variant_enum!(DvarBuilderTypeLinearColorRGBCurrentValueState);
zero_variant_enum!(DvarBuilderTypeLinearColorRGBOtherValuesState);
zero_variant_enum!(DvarBuilderTypeColorXYZDomainState);
zero_variant_enum!(DvarBuilderTypeColorXYZCurrentValueState);
zero_variant_enum!(DvarBuilderTypeColorXYZOtherValuesState);
zero_variant_enum!(DvarBuilderTypeUnsafe);

// Helper impl to make constructing Dvars easier
pub struct DvarBuilder<T> {
    dvar: Dvar,
    extra: T,
}

impl DvarBuilder<DvarBuilderStartState> {
    pub fn new() -> DvarBuilder<DvarBuilderDataState> {
        unsafe {
            DvarBuilder::<DvarBuilderDataState> {
                dvar: Dvar {
                    name: "".to_string(),
                    description: "".to_string(),
                    flags: DvarFlags::empty(),
                    modified: false,
                    loaded_from_save_game: false,
                    domain: MaybeUninit::uninit().assume_init(),
                    current: MaybeUninit::uninit().assume_init(),
                    latched: MaybeUninit::uninit().assume_init(),
                    reset: MaybeUninit::uninit().assume_init(),
                    saved: MaybeUninit::uninit().assume_init(),
                },
                extra: Default::default(),
            }
        }
    }
}

impl DvarBuilder<DvarBuilderDataState> {
    pub fn name(mut self, name: &str) -> DvarBuilder<DvarBuilderDataState> {
        self.dvar.name = name.to_string();
        self
    }

    pub fn description(
        mut self,
        description: String,
    ) -> DvarBuilder<DvarBuilderDataState> {
        self.dvar.description = description;
        self
    }

    pub fn flags(
        mut self,
        flags: DvarFlags,
    ) -> DvarBuilder<DvarBuilderDataState> {
        self.dvar.flags = flags;
        self
    }

    pub fn loaded_from_save_game(
        mut self,
        b: bool,
    ) -> DvarBuilder<DvarBuilderDataState> {
        self.dvar.loaded_from_save_game = b;
        DvarBuilder::<DvarBuilderDataState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub fn type_bool(
        self,
    ) -> DvarBuilder<DvarBuilderTypeBoolCurrentValueState> {
        DvarBuilder::<DvarBuilderTypeBoolCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub fn type_float(self) -> DvarBuilder<DvarBuilderTypeFloatDomainState> {
        DvarBuilder::<DvarBuilderTypeFloatDomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub fn type_vector2(
        self,
    ) -> DvarBuilder<DvarBuilderTypeVector2DomainState> {
        DvarBuilder::<DvarBuilderTypeVector2DomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub fn type_vector3(
        self,
    ) -> DvarBuilder<DvarBuilderTypeVector3DomainState> {
        DvarBuilder::<DvarBuilderTypeVector3DomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub fn type_vector4(
        self,
    ) -> DvarBuilder<DvarBuilderTypeVector4DomainState> {
        DvarBuilder::<DvarBuilderTypeVector4DomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub fn type_int(self) -> DvarBuilder<DvarBuilderTypeIntDomainState> {
        DvarBuilder::<DvarBuilderTypeIntDomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub fn type_string(
        self,
    ) -> DvarBuilder<DvarBuilderTypeStringCurrentValueState> {
        DvarBuilder::<DvarBuilderTypeStringCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub fn type_enumeration(
        self,
    ) -> DvarBuilder<DvarBuilderTypeEnumerationDomainState> {
        DvarBuilder::<DvarBuilderTypeEnumerationDomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub fn type_color(
        self,
    ) -> DvarBuilder<DvarBuilderTypeColorCurrentValueState> {
        DvarBuilder::<DvarBuilderTypeColorCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub fn type_int64(self) -> DvarBuilder<DvarBuilderTypeInt64DomainState> {
        DvarBuilder::<DvarBuilderTypeInt64DomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub fn type_linear_color_rgb(
        self,
    ) -> DvarBuilder<DvarBuilderTypeLinearColorRGBDomainState> {
        DvarBuilder::<DvarBuilderTypeLinearColorRGBDomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub fn type_color_xyz(
        self,
    ) -> DvarBuilder<DvarBuilderTypeColorXYZDomainState> {
        DvarBuilder::<DvarBuilderTypeColorXYZDomainState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }

    pub fn type_unsafe(self) -> DvarBuilder<DvarBuilderTypeUnsafe> {
        DvarBuilder::<DvarBuilderTypeUnsafe> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeFloatDomainState> {
    fn domain(
        mut self,
        min: f32,
        max: f32,
    ) -> DvarBuilder<DvarBuilderTypeFloatCurrentValueState> {
        self.dvar.domain = DvarLimits::Float(DvarLimitsFloat::new(min, max));
        DvarBuilder::<DvarBuilderTypeFloatCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeVector2DomainState> {
    fn domain(
        mut self,
        min: f32,
        max: f32,
    ) -> DvarBuilder<DvarBuilderTypeVector2CurrentValueState> {
        self.dvar.domain =
            DvarLimits::Vector2(DvarLimitsVector2::new(min, max));
        DvarBuilder::<DvarBuilderTypeVector2CurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeVector3DomainState> {
    fn domain(
        mut self,
        min: f32,
        max: f32,
    ) -> DvarBuilder<DvarBuilderTypeVector3CurrentValueState> {
        self.dvar.domain =
            DvarLimits::Vector3(DvarLimitsVector3::new(min, max));
        DvarBuilder::<DvarBuilderTypeVector3CurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeVector4DomainState> {
    fn domain(
        mut self,
        min: f32,
        max: f32,
    ) -> DvarBuilder<DvarBuilderTypeVector4CurrentValueState> {
        self.dvar.domain =
            DvarLimits::Vector4(DvarLimitsVector4::new(min, max));
        DvarBuilder::<DvarBuilderTypeVector4CurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeIntDomainState> {
    fn domain(
        mut self,
        min: i32,
        max: i32,
    ) -> DvarBuilder<DvarBuilderTypeIntCurrentValueState> {
        self.dvar.domain = DvarLimits::Int(DvarLimitsInt::new(min, max));
        DvarBuilder::<DvarBuilderTypeIntCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeEnumerationDomainState> {
    fn domain(
        mut self,
        domain: Vec<String>,
    ) -> DvarBuilder<DvarBuilderTypeEnumerationCurrentValueState> {
        self.dvar.domain =
            DvarLimits::Enumeration(DvarLimitsEnumeration::new(&domain));
        DvarBuilder::<DvarBuilderTypeEnumerationCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeInt64DomainState> {
    fn domain(
        mut self,
        min: i64,
        max: i64,
    ) -> DvarBuilder<DvarBuilderTypeInt64CurrentValueState> {
        self.dvar.domain = DvarLimits::Int64(DvarLimitsInt64::new(min, max));
        DvarBuilder::<DvarBuilderTypeInt64CurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeLinearColorRGBDomainState> {
    fn domain(
        mut self,
        min: f32,
        max: f32,
    ) -> DvarBuilder<DvarBuilderTypeLinearColorRGBCurrentValueState> {
        self.dvar.domain =
            DvarLimits::LinearColorRGB(DvarLimitsLinearColorRGB::new(min, max));
        DvarBuilder::<DvarBuilderTypeLinearColorRGBCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeColorXYZDomainState> {
    fn domain(
        mut self,
        min: f32,
        max: f32,
    ) -> DvarBuilder<DvarBuilderTypeColorXYZCurrentValueState> {
        self.dvar.domain =
            DvarLimits::ColorXYZ(DvarLimitsColorXYZ::new(min, max));
        DvarBuilder::<DvarBuilderTypeColorXYZCurrentValueState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeBoolCurrentValueState> {
    fn value(
        mut self,
        value: bool,
    ) -> DvarBuilder<DvarBuilderTypeBoolOtherValuesState> {
        self.dvar.domain = DvarLimits::Bool(DvarLimitsBool::new());
        self.dvar.current = DvarValue::Bool(value);
        self.dvar.latched = DvarValue::Bool(value);
        self.dvar.saved = DvarValue::Bool(value);
        self.dvar.reset = DvarValue::Bool(value);
        DvarBuilder::<DvarBuilderTypeBoolOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeFloatCurrentValueState> {
    fn value(
        mut self,
        value: f32,
    ) -> DvarBuilder<DvarBuilderTypeFloatOtherValuesState> {
        self.dvar.current = DvarValue::Float(value);
        self.dvar.latched = DvarValue::Float(value);
        self.dvar.saved = DvarValue::Float(value);
        self.dvar.reset = DvarValue::Float(value);
        DvarBuilder::<DvarBuilderTypeFloatOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeVector2CurrentValueState> {
    fn value(
        mut self,
        value: Vec2f32,
    ) -> DvarBuilder<DvarBuilderTypeVector2OtherValuesState> {
        self.dvar.current = DvarValue::Vector2(value);
        self.dvar.latched = DvarValue::Vector2(value);
        self.dvar.saved = DvarValue::Vector2(value);
        self.dvar.reset = DvarValue::Vector2(value);
        DvarBuilder::<DvarBuilderTypeVector2OtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeVector3CurrentValueState> {
    fn value(
        mut self,
        value: Vec3f32,
    ) -> DvarBuilder<DvarBuilderTypeVector3OtherValuesState> {
        self.dvar.current = DvarValue::Vector3(value);
        self.dvar.latched = DvarValue::Vector3(value);
        self.dvar.saved = DvarValue::Vector3(value);
        self.dvar.reset = DvarValue::Vector3(value);
        DvarBuilder::<DvarBuilderTypeVector3OtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeVector4CurrentValueState> {
    fn value(
        mut self,
        value: Vec4f32,
    ) -> DvarBuilder<DvarBuilderTypeVector4OtherValuesState> {
        self.dvar.current = DvarValue::Vector4(value);
        self.dvar.latched = DvarValue::Vector4(value);
        self.dvar.saved = DvarValue::Vector4(value);
        self.dvar.reset = DvarValue::Vector4(value);
        DvarBuilder::<DvarBuilderTypeVector4OtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeIntCurrentValueState> {
    fn value(
        mut self,
        value: i32,
    ) -> DvarBuilder<DvarBuilderTypeIntOtherValuesState> {
        self.dvar.current = DvarValue::Int(value);
        self.dvar.latched = DvarValue::Int(value);
        self.dvar.saved = DvarValue::Int(value);
        self.dvar.reset = DvarValue::Int(value);
        DvarBuilder::<DvarBuilderTypeIntOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeStringCurrentValueState> {
    fn value(
        mut self,
        value: String,
    ) -> DvarBuilder<DvarBuilderTypeStringOtherValuesState> {
        self.dvar.domain = DvarLimits::String(DvarLimitsString::new());
        self.dvar.current = DvarValue::String(value.clone());
        self.dvar.latched = DvarValue::String(value.clone());
        self.dvar.saved = DvarValue::String(value.clone());
        self.dvar.reset = DvarValue::String(value);
        DvarBuilder::<DvarBuilderTypeStringOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeEnumerationCurrentValueState> {
    fn value(
        mut self,
        value: String,
    ) -> DvarBuilder<DvarBuilderTypeEnumerationOtherValuesState> {
        self.dvar.current = DvarValue::Enumeration(value.clone());
        self.dvar.latched = DvarValue::Enumeration(value.clone());
        self.dvar.saved = DvarValue::Enumeration(value.clone());
        self.dvar.reset = DvarValue::Enumeration(value);
        DvarBuilder::<DvarBuilderTypeEnumerationOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeColorCurrentValueState> {
    fn value(
        mut self,
        value: Vec4f32,
    ) -> DvarBuilder<DvarBuilderTypeColorOtherValuesState> {
        self.dvar.domain = DvarLimits::Color(DvarLimitsColor::new());
        self.dvar.current = DvarValue::Color(value);
        self.dvar.latched = DvarValue::Color(value);
        self.dvar.saved = DvarValue::Color(value);
        self.dvar.reset = DvarValue::Color(value);
        DvarBuilder::<DvarBuilderTypeColorOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeInt64CurrentValueState> {
    fn value(
        mut self,
        value: i64,
    ) -> DvarBuilder<DvarBuilderTypeInt64OtherValuesState> {
        self.dvar.current = DvarValue::Int64(value);
        self.dvar.latched = DvarValue::Int64(value);
        self.dvar.saved = DvarValue::Int64(value);
        self.dvar.reset = DvarValue::Int64(value);
        DvarBuilder::<DvarBuilderTypeInt64OtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeLinearColorRGBCurrentValueState> {
    fn value(
        mut self,
        value: Vec3f32,
    ) -> DvarBuilder<DvarBuilderTypeLinearColorRGBOtherValuesState> {
        self.dvar.current = DvarValue::LinearColorRGB(value);
        self.dvar.latched = DvarValue::LinearColorRGB(value);
        self.dvar.saved = DvarValue::LinearColorRGB(value);
        self.dvar.reset = DvarValue::LinearColorRGB(value);
        DvarBuilder::<DvarBuilderTypeLinearColorRGBOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeColorXYZCurrentValueState> {
    fn value(
        mut self,
        value: Vec3f32,
    ) -> DvarBuilder<DvarBuilderTypeColorXYZOtherValuesState> {
        self.dvar.current = DvarValue::ColorXYZ(value);
        self.dvar.latched = DvarValue::ColorXYZ(value);
        self.dvar.saved = DvarValue::ColorXYZ(value);
        self.dvar.reset = DvarValue::ColorXYZ(value);
        DvarBuilder::<DvarBuilderTypeColorXYZOtherValuesState> {
            dvar: self.dvar,
            extra: Default::default(),
        }
    }
}

impl DvarBuilder<DvarBuilderTypeBoolOtherValuesState> {
    fn latched(mut self, value: bool) -> Self {
        self.dvar.latched = DvarValue::Bool(value);
        self
    }

    fn saved(mut self, value: bool) -> Self {
        self.dvar.saved = DvarValue::Bool(value);
        self
    }

    fn reset(mut self, value: bool) -> Self {
        self.dvar.reset = DvarValue::Bool(value);
        self
    }

    fn build(self) -> Dvar {
        self.dvar
    }
}

impl DvarBuilder<DvarBuilderTypeFloatOtherValuesState> {
    fn latched(mut self, value: f32) -> Self {
        self.dvar.latched = DvarValue::Float(value);
        self
    }

    fn saved(mut self, value: f32) -> Self {
        self.dvar.saved = DvarValue::Float(value);
        self
    }

    fn reset(mut self, value: f32) -> Self {
        self.dvar.reset = DvarValue::Float(value);
        self
    }

    fn build(self) -> Dvar {
        self.dvar
    }
}

impl DvarBuilder<DvarBuilderTypeVector2OtherValuesState> {
    fn latched(mut self, value: Vec2f32) -> Self {
        self.dvar.latched = DvarValue::Vector2(value);
        self
    }

    fn saved(mut self, value: Vec2f32) -> Self {
        self.dvar.saved = DvarValue::Vector2(value);
        self
    }

    fn reset(mut self, value: Vec2f32) -> Self {
        self.dvar.reset = DvarValue::Vector2(value);
        self
    }

    fn build(self) -> Dvar {
        self.dvar
    }
}

impl DvarBuilder<DvarBuilderTypeVector3OtherValuesState> {
    fn latched(mut self, value: Vec3f32) -> Self {
        self.dvar.latched = DvarValue::Vector3(value);
        self
    }

    fn saved(mut self, value: Vec3f32) -> Self {
        self.dvar.saved = DvarValue::Vector3(value);
        self
    }

    fn reset(mut self, value: Vec3f32) -> Self {
        self.dvar.reset = DvarValue::Vector3(value);
        self
    }

    fn build(self) -> Dvar {
        self.dvar
    }
}

impl DvarBuilder<DvarBuilderTypeVector4OtherValuesState> {
    fn latched(mut self, value: Vec4f32) -> Self {
        self.dvar.latched = DvarValue::Vector4(value);
        self
    }

    fn saved(mut self, value: Vec4f32) -> Self {
        self.dvar.saved = DvarValue::Vector4(value);
        self
    }

    fn reset(mut self, value: Vec4f32) -> Self {
        self.dvar.reset = DvarValue::Vector4(value);
        self
    }

    fn build(self) -> Dvar {
        self.dvar
    }
}

impl DvarBuilder<DvarBuilderTypeIntOtherValuesState> {
    fn latched(mut self, value: i32) -> Self {
        self.dvar.latched = DvarValue::Int(value);
        self
    }

    fn saved(mut self, value: i32) -> Self {
        self.dvar.saved = DvarValue::Int(value);
        self
    }

    fn reset(mut self, value: i32) -> Self {
        self.dvar.reset = DvarValue::Int(value);
        self
    }

    fn build(self) -> Dvar {
        self.dvar
    }
}

impl DvarBuilder<DvarBuilderTypeStringOtherValuesState> {
    fn latched(mut self, value: String) -> Self {
        self.dvar.latched = DvarValue::String(value);
        self
    }

    fn saved(mut self, value: String) -> Self {
        self.dvar.saved = DvarValue::String(value);
        self
    }

    fn reset(mut self, value: String) -> Self {
        self.dvar.reset = DvarValue::String(value);
        self
    }

    fn build(self) -> Dvar {
        self.dvar
    }
}

impl DvarBuilder<DvarBuilderTypeEnumerationOtherValuesState> {
    fn latched(mut self, value: String) -> Self {
        self.dvar.latched = DvarValue::Enumeration(value);
        self
    }

    fn saved(mut self, value: String) -> Self {
        self.dvar.saved = DvarValue::Enumeration(value);
        self
    }

    fn reset(mut self, value: String) -> Self {
        self.dvar.reset = DvarValue::Enumeration(value);
        self
    }

    fn build(self) -> Dvar {
        self.dvar
    }
}

impl DvarBuilder<DvarBuilderTypeColorOtherValuesState> {
    fn latched(mut self, value: Vec4f32) -> Self {
        self.dvar.latched = DvarValue::Color(value);
        self
    }

    fn saved(mut self, value: Vec4f32) -> Self {
        self.dvar.saved = DvarValue::Color(value);
        self
    }

    fn reset(mut self, value: Vec4f32) -> Self {
        self.dvar.reset = DvarValue::Color(value);
        self
    }

    fn build(self) -> Dvar {
        self.dvar
    }
}

impl DvarBuilder<DvarBuilderTypeInt64OtherValuesState> {
    fn latched(mut self, value: i64) -> Self {
        self.dvar.latched = DvarValue::Int64(value);
        self
    }

    fn saved(mut self, value: i64) -> Self {
        self.dvar.saved = DvarValue::Int64(value);
        self
    }

    fn reset(mut self, value: i64) -> Self {
        self.dvar.reset = DvarValue::Int64(value);
        self
    }

    fn build(self) -> Dvar {
        self.dvar
    }
}

impl DvarBuilder<DvarBuilderTypeLinearColorRGBOtherValuesState> {
    fn latched(mut self, value: Vec3f32) -> Self {
        self.dvar.latched = DvarValue::LinearColorRGB(value);
        self
    }

    fn saved(mut self, value: Vec3f32) -> Self {
        self.dvar.saved = DvarValue::LinearColorRGB(value);
        self
    }

    fn reset(mut self, value: Vec3f32) -> Self {
        self.dvar.reset = DvarValue::LinearColorRGB(value);
        self
    }

    fn build(self) -> Dvar {
        self.dvar
    }
}

impl DvarBuilder<DvarBuilderTypeColorXYZOtherValuesState> {
    fn latched(mut self, value: Vec3f32) -> Self {
        self.dvar.latched = DvarValue::ColorXYZ(value);
        self
    }

    fn saved(mut self, value: Vec3f32) -> Self {
        self.dvar.saved = DvarValue::ColorXYZ(value);
        self
    }

    fn reset(mut self, value: Vec3f32) -> Self {
        self.dvar.reset = DvarValue::ColorXYZ(value);
        self
    }

    fn build(self) -> Dvar {
        self.dvar
    }
}

impl DvarBuilder<DvarBuilderTypeUnsafe> {
    fn domain(mut self, domain: DvarLimits) -> Self {
        self.dvar.domain = domain;
        self
    }

    fn value(mut self, value: DvarValue) -> Self {
        self.dvar.current = value;
        self
    }

    fn latched(mut self, value: DvarValue) -> Self {
        self.dvar.latched = value;
        self
    }

    fn saved(mut self, value: DvarValue) -> Self {
        self.dvar.saved = value;
        self
    }

    fn reset(mut self, value: DvarValue) -> Self {
        self.dvar.reset = value;
        self
    }

    fn build(self) -> Dvar {
        self.dvar
    }
}

const DVAR_COUNT_MAX: usize = 4096;

lazy_static! {
    pub static ref DVARS: Arc<RwLock<HashMap<String, Box<Dvar>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

/// Finds a previously-registered [`Dvar`] by name and returns a copy if present.
///
/// Since this function returns a copy, it is not useful for modifying an existing [`Dvar`].
/// (See [`register_variant`] or [`reregister`] for modification purposes).
/// Instead, it may be used to query if a [`Dvar`] exists, or to query the
/// state of said [`Dvar`] at the point in time at which the copy is made.
///
/// # Arguments
///
/// * `name` - A [`String`] that holds the name of the [`Dvar`] to be searched for.
///
/// # Return Value
///
/// Returns [`Some`] if a Dvar with the name `name` exists,
/// otherwise [`None`].
///
/// # Panics
///
/// Panics if the read lock for [`DVARS`] can't be acquired (usually because
/// the write lock is held by a function farther up the call stack).
///
/// Example
/// ```
/// let dvar_name = "sv_test".to_string();
/// match find(dvar_name) {
///    Some(d) => println!("found dvar {} with value {}", d.name, d.current),
///    None => panic!("dvar {} not found", dvar_name)
/// };
/// ```
pub fn find(name: &str) -> Option<Dvar> {
    let lock = DVARS.clone();
    let reader = lock.try_read().expect(
        "dvar::find: failed to acquire reader lock. Probably still held by calling function.",
    );

    if !reader.contains_key(name) {
        return None;
    }

    return Some(*reader.get(name).unwrap().clone());
}

/// Checks if a [`Dvar`] with name `name` exists.
///
/// # Arguments
///
/// * `name` - A [`String`] that holds the name of the [`Dvar`] to be searched for.
///
/// # Return Value
///
/// Returns [`true`] if [`Dvar`] exists, [`false`] otherwise.
///
/// # Panics
///
/// Panics if the read lock for [`DVARS`] can't be acquired (usually because
/// the write lock is held by a function farther up the call stack).
///
/// Example
/// ```
/// let dvar_name = "sv_test".to_string();
/// match exists(dvar_name) {
///    Some(d) => println!("found dvar {}", d.name),
///    None => panic!("dvar {} not found", dvar_name)
/// };
/// ```
pub fn exists(name: &str) -> bool {
    let lock = DVARS.clone();
    let reader = lock.try_read().expect(
        "dvar::exists: failed to acquire reader lock. Probably still held by calling function.",
    );

    reader.contains_key(name)
}

/// Registers a [`Dvar`] of type [`DvarValue::Bool`], using the provided name,
/// value, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`bool`] representing the value to register the [`Dvar`] with
/// * `flags` - The [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - A [`String`] containing a description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// if find(name).is_none {
///     let value = false;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type bool"
///     register_bool(name, value, flags, description);
/// }
/// ```
pub fn register_bool(
    name: &str,
    value: bool,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    let b: bool;
    let other_name: &str;
    {
        let lock = DVARS.clone();
        let mut writer = lock.try_write().expect("dvar::register_new: failed to acquire writer lock. Probably still held by calling function.");

        if writer.len() + 1 > DVAR_COUNT_MAX {
            com::errorln(
                com::ErrorParm::FATAL,
                format!(
                    "Can\'t create dvar \'{}\': {} dvars already exist",
                    name, DVAR_COUNT_MAX
                ),
            );
            return false;
        }

        let dvar = DvarBuilder::new()
            .name(name)
            .description(description.unwrap_or_default().to_string())
            .flags(flags.unwrap_or_default())
            .type_bool()
            .value(value)
            .build();
        b = writer.insert(name.to_string(), Box::new(dvar)).is_some();
        other_name = &writer.get(name).unwrap().name;
        if b {
            com::errorln(com::ErrorParm::FATAL, format!("dvar name hash collision between \'{}\' and \'{}\' Please change one of these names to remove the hash collision", name, other_name));
            return false;
        }
    }

    true
}

pub fn register_new_bool(
    name: &str,
    value: bool,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        return false;
    }

    register_bool(name, value, flags, description)
}

pub fn reregister_bool(
    name: &str,
    value: bool,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if !exists(name) {
        return false;
    }

    register_bool(name, value, flags, description)
}

/// Registers a Dvar of type [`DvarValue::Float`].
///
/// Essentially an easier-to-use wrapper around [`register_variant`] to
/// register a Dvar with type [`DvarValue::Float`], using the provided name,
/// value, domain, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An [`f32`] representing the value to register the [`Dvar`] with
/// * `min` - An [`f32`] representing the lower bound
/// of the domain for the [`Dvar`]
/// * `max` - An [`f32`] representing the upper bound
/// of the domain for the [`Dvar`]
/// * `flags` - The [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - A [`String`] containing a description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// if find(name).is_none {
///     let value = 121.3;
///     let min = -462.0;
///     let max = 592.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type float"
///     register_float(name, value, min, max, flags, description);
/// }
/// ```
pub fn register_float(
    name: &str,
    value: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    let b: bool;
    let other_name: &str;
    {
        let lock = DVARS.clone();
        let mut writer = lock.try_write().expect("dvar::register_new: failed to acquire writer lock. Probably still held by calling function.");

        if writer.len() + 1 > DVAR_COUNT_MAX {
            com::errorln(
                com::ErrorParm::FATAL,
                format!(
                    "Can\'t create dvar \'{}\': {} dvars already exist",
                    name, DVAR_COUNT_MAX
                ),
            );
            return false;
        }

        let dvar = DvarBuilder::new()
            .name(name)
            .description(description.unwrap_or_default().to_string())
            .flags(flags.unwrap_or_default())
            .type_float()
            .domain(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX))
            .value(value)
            .build();
        b = writer.insert(name.to_string(), Box::new(dvar)).is_some();
        other_name = &writer.get(name).unwrap().name;
        if b {
            com::errorln(com::ErrorParm::FATAL, format!("dvar name hash collision between \'{}\' and \'{}\' Please change one of these names to remove the hash collision", name, other_name));
            return false;
        }
    }

    true
}

pub fn register_new_float(
    name: &str,
    value: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        return false;
    }

    register_float(name, value, min, max, flags, description)
}

pub fn reregister_float(
    name: &str,
    value: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if !exists(name) {
        return false;
    }

    register_float(name, value, min, max, flags, description)
}

/// Registers a Dvar of type [`DvarValue::Vector2`].
///
/// Essentially an easier-to-use wrapper around [`register_variant`] to
/// register a Dvar with type [`DvarValue::Vector2`], using the provided name,
/// value, domain, flags, and description. The domain applies to both elements
/// of the value (i.e. `value.0` and `value.1` can't have separate domains).
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An ([`f32`], [`f32`]) representing the value
/// to register the [`Dvar`] with
/// * `min` - An [`f32`] representing the lower bound
/// of the domain for all elements of `value`
/// * `max` - An [`f32`] representing the upper bound
/// of the domain for all elements of `value`
/// * `flags` - The [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - A [`String`] containing a description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// if find(name).is_none {
///     let value = (121.3, -267.4);
///     let min = -462.0;
///     let max = 592.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type vector2"
///     register_vector2(name, value, min, max, flags, description);
/// }
/// ```
pub fn register_vector2(
    name: &str,
    value: Vec2f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    let b: bool;
    let other_name: &str;
    {
        let lock = DVARS.clone();
        let mut writer = lock.try_write().expect("dvar::register_new: failed to acquire writer lock. Probably still held by calling function.");

        if writer.len() + 1 > DVAR_COUNT_MAX {
            com::errorln(
                com::ErrorParm::FATAL,
                format!(
                    "Can\'t create dvar \'{}\': {} dvars already exist",
                    name, DVAR_COUNT_MAX
                ),
            );
            return false;
        }

        let dvar = DvarBuilder::new()
            .name(name)
            .description(description.unwrap_or_default().to_string())
            .flags(flags.unwrap_or_default())
            .type_vector2()
            .domain(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX))
            .value(value)
            .build();
        b = writer.insert(name.to_string(), Box::new(dvar)).is_some();
        other_name = &writer.get(name).unwrap().name;
        if b {
            com::errorln(com::ErrorParm::FATAL, format!("dvar name hash collision between \'{}\' and \'{}\' Please change one of these names to remove the hash collision", name, other_name));
            return false;
        }
    }

    true
}

pub fn register_new_vector2(
    name: &str,
    value: Vec2f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        return false;
    }

    register_vector2(name, value, min, max, flags, description)
}

pub fn reregister_vector2(
    name: &str,
    value: Vec2f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if !exists(name) {
        return false;
    }

    register_vector2(name, value, min, max, flags, description)
}

/// Registers a Dvar of type [`DvarValue::Vector3`].
///
/// Essentially an easier-to-use wrapper around [`register_variant`] to
/// register a Dvar with type [`DvarValue::Vector3`], using the provided name,
/// value, domain, flags, and description. The domain applies to both elements
/// of the value (i.e. `value.0` and `value.1` can't have separate domains).
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An ([`f32`], [`f32`], [`f32`]) representing the value
/// to register the [`Dvar`] with
/// * `min` - An [`f32`] representing the lower bound
/// of the domain for all elements of `value`
/// * `max` - An [`f32`] representing the upper bound
/// of the domain for all elements of `value`
/// * `flags` - The [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - A [`String`] containing a description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// if find(name).is_none {
///     let value = (121.3, -267.4, -143.0);
///     let min = -462.0;
///     let max = 592.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type vector3"
///     register_vector3(name, value, min, max, flags, description);
/// }
/// ```
pub fn register_vector3(
    name: &str,
    value: Vec3f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    let b: bool;
    let other_name: &str;
    {
        let lock = DVARS.clone();
        let mut writer = lock.try_write().expect("dvar::register_new: failed to acquire writer lock. Probably still held by calling function.");

        if writer.len() + 1 > DVAR_COUNT_MAX {
            com::errorln(
                com::ErrorParm::FATAL,
                format!(
                    "Can\'t create dvar \'{}\': {} dvars already exist",
                    name, DVAR_COUNT_MAX
                ),
            );
            return false;
        }

        let dvar = DvarBuilder::new()
            .name(name)
            .description(description.unwrap_or_default().to_string())
            .flags(flags.unwrap_or_default())
            .type_vector3()
            .domain(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX))
            .value(value)
            .build();
        b = writer.insert(name.to_string(), Box::new(dvar)).is_some();
        other_name = &writer.get(name).unwrap().name;
        if b {
            com::errorln(com::ErrorParm::FATAL, format!("dvar name hash collision between \'{}\' and \'{}\' Please change one of these names to remove the hash collision", name, other_name));
            return false;
        }
    }

    true
}

pub fn register_new_vector3(
    name: &str,
    value: Vec3f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        return false;
    }

    register_vector3(name, value, min, max, flags, description)
}

pub fn reregister_vector3(
    name: &str,
    value: Vec3f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if !exists(name) {
        return false;
    }

    register_vector3(name, value, min, max, flags, description)
}

/// Registers a Dvar of type [`DvarValue::Vector4`].
///
/// Essentially an easier-to-use wrapper around [`register_variant`] to
/// register a Dvar with type [`DvarValue::Vector4`], using the provided name,
/// value, domain, flags, and description. The domain applies to both elements
/// of the value (i.e. `value.0` and `value.1` can't have separate domains).
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An ([`f32`], [`f32`], [`f32`], [`f32`]) representing the value
/// to register the [`Dvar`] with
/// * `min` - An [`f32`] representing the lower bound
/// of the domain for all elements of `value`
/// * `max` - An [`f32`] representing the upper bound
/// of the domain for all elements of `value`
/// * `flags` - The [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - A [`String`] containing a description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// if find(name).is_none {
///     let value = (121.3, -267.4, -143.0, 71.2);
///     let min = -462.0;
///     let max = 592.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type vector4"
///     register_vector4(name, value, min, max, flags, description);
/// }
/// ```
pub fn register_vector4(
    name: &str,
    value: Vec4f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    let b: bool;
    let other_name: &str;
    {
        let lock = DVARS.clone();
        let mut writer = lock.try_write().expect("dvar::register_new: failed to acquire writer lock. Probably still held by calling function.");

        if writer.len() + 1 > DVAR_COUNT_MAX {
            com::errorln(
                com::ErrorParm::FATAL,
                format!(
                    "Can\'t create dvar \'{}\': {} dvars already exist",
                    name, DVAR_COUNT_MAX
                ),
            );
            return false;
        }

        let dvar = DvarBuilder::new()
            .name(name)
            .description(description.unwrap_or_default().to_string())
            .flags(flags.unwrap_or_default())
            .type_vector4()
            .domain(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX))
            .value(value)
            .build();
        b = writer.insert(name.to_string(), Box::new(dvar)).is_some();
        other_name = &writer.get(name).unwrap().name;
        if b {
            com::errorln(com::ErrorParm::FATAL, format!("dvar name hash collision between \'{}\' and \'{}\' Please change one of these names to remove the hash collision", name, other_name));
            return false;
        }
    }

    true
}

pub fn register_new_vector4(
    name: &str,
    value: Vec4f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        return false;
    }

    register_vector4(name, value, min, max, flags, description)
}

pub fn reregister_vector4(
    name: &str,
    value: Vec4f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if !exists(name) {
        return false;
    }

    register_vector4(name, value, min, max, flags, description)
}

/// Registers a Dvar of type [`DvarValue::Int`].
///
/// Essentially an easier-to-use wrapper around [`register_variant`] to
/// register a Dvar with type [`DvarValue::Int`], using the provided name,
/// value, domain, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An [`i32`] representing the value to register the [`Dvar`] with
/// * `min` - An [`i32`] representing the lower bound
/// for the domain for `value`
/// * `max` - An [`i32`] representing the upper bound
/// for the domain for `value`
/// * `flags` - The [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - A [`String`] containing a description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// if find(name).is_none {
///     let value = 64867969;
///     let min = i32::MIN;
///     let max = i32::MAX;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type int"
///     register_int(name, value, min, max, flags, description);
/// }
/// ```
pub fn register_int(
    name: &str,
    value: i32,
    min: Option<i32>,
    max: Option<i32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    let b: bool;
    let other_name: &str;
    {
        let lock = DVARS.clone();
        let mut writer = lock.try_write().expect("dvar::register_new: failed to acquire writer lock. Probably still held by calling function.");

        if writer.len() + 1 > DVAR_COUNT_MAX {
            com::errorln(
                com::ErrorParm::FATAL,
                format!(
                    "Can\'t create dvar \'{}\': {} dvars already exist",
                    name, DVAR_COUNT_MAX
                ),
            );
            return false;
        }

        let dvar = DvarBuilder::new()
            .name(name)
            .description(description.unwrap_or_default().to_string())
            .flags(flags.unwrap_or_default())
            .type_int()
            .domain(min.unwrap_or(i32::MIN), max.unwrap_or(i32::MAX))
            .value(value)
            .build();
        b = writer.insert(name.to_string(), Box::new(dvar)).is_some();
        other_name = &writer.get(name).unwrap().name;
        if b {
            com::errorln(com::ErrorParm::FATAL, format!("dvar name hash collision between \'{}\' and \'{}\' Please change one of these names to remove the hash collision", name, other_name));
            return false;
        }
    }

    true
}

pub fn register_new_int(
    name: &str,
    value: i32,
    min: Option<i32>,
    max: Option<i32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        return false;
    }

    register_int(name, value, min, max, flags, description)
}

pub fn reregister_int(
    name: &str,
    value: i32,
    min: Option<i32>,
    max: Option<i32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if !exists(name) {
        return false;
    }

    register_int(name, value, min, max, flags, description)
}

/// Registers a Dvar of type [`DvarValue::String`].
///
/// Essentially an easier-to-use wrapper around [`register_variant`] to
/// register a Dvar with type [`DvarValue::String`], using the provided name,
/// value, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`String`] representing the value
/// to register the [`Dvar`] with
/// * `flags` - The [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - A [`String`] containing a description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// if find(name).is_none {
///     let value = "this is a test".to_string();
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type string"
///     register_string(name, value, flags, description);
/// }
/// ```
pub fn register_string(
    name: &str,
    value: String,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    let b: bool;
    let other_name: &str;
    {
        let lock = DVARS.clone();
        let mut writer = lock.try_write().expect("dvar::register_new: failed to acquire writer lock. Probably still held by calling function.");

        if writer.len() + 1 > DVAR_COUNT_MAX {
            com::errorln(
                com::ErrorParm::FATAL,
                format!(
                    "Can\'t create dvar \'{}\': {} dvars already exist",
                    name, DVAR_COUNT_MAX
                ),
            );
            return false;
        }

        let dvar = DvarBuilder::new()
            .name(name)
            .description(description.unwrap_or_default().to_string())
            .flags(flags.unwrap_or_default())
            .type_string()
            .value(value)
            .build();
        b = writer.insert(name.to_string(), Box::new(dvar)).is_some();
        other_name = &writer.get(name).unwrap().name;
        if b {
            com::errorln(com::ErrorParm::FATAL, format!("dvar name hash collision between \'{}\' and \'{}\' Please change one of these names to remove the hash collision", name, other_name));
            return false;
        }
    }

    true
}

pub fn register_new_string(
    name: &str,
    value: String,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        return false;
    }

    register_string(name, value, flags, description)
}

pub fn reregister_string(
    name: &str,
    value: String,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if !exists(name) {
        return false;
    }

    register_string(name, value, flags, description)
}

/// Registers a Dvar of type [`DvarValue::Enumeration`].
///
/// Essentially an easier-to-use wrapper around [`register_variant`] to
/// register a Dvar with type [`DvarValue::Enumeration`], using the provided
/// name, value, domain, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`String`] representing the value
/// to register the [`Dvar`] with
/// * `domain` - A [`Vec`] of [`String`]s containing
/// all valid values for `value`
/// * `flags` - The [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - A [`String`] containing a description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// if find(name).is_none {
///     let value = "DEF".to_string();
///     let domain = vec!["ABC".to_string(), "DEF".to_string(), "GHI".to_string()];
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type string"
///     register_enum(name, value, domain, flags, description);
/// }
/// ```
pub fn register_enumeration(
    name: &str,
    value: String,
    domain: Option<Vec<String>>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    let b: bool;
    let other_name: &str;
    {
        let lock = DVARS.clone();
        let mut writer = lock.try_write().expect("dvar::register_new: failed to acquire writer lock. Probably still held by calling function.");

        if writer.len() + 1 > DVAR_COUNT_MAX {
            com::errorln(
                com::ErrorParm::FATAL,
                format!(
                    "Can\'t create dvar \'{}\': {} dvars already exist",
                    name, DVAR_COUNT_MAX
                ),
            );
            return false;
        }

        let dvar = DvarBuilder::new()
            .name(name)
            .description(description.unwrap_or_default().to_string())
            .flags(flags.unwrap_or_default())
            .type_enumeration()
            .domain(domain.unwrap_or_default())
            .value(value)
            .build();
        b = writer.insert(name.to_string(), Box::new(dvar)).is_some();
        other_name = &writer.get(name).unwrap().name;
        if b {
            com::errorln(com::ErrorParm::FATAL, format!("dvar name hash collision between \'{}\' and \'{}\' Please change one of these names to remove the hash collision", name, other_name));
            return false;
        }
    }

    true
}

pub fn register_new_enumeration(
    name: &str,
    value: String,
    domain: Option<Vec<String>>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        return false;
    }

    register_enumeration(name, value, domain, flags, description)
}

pub fn reregister_enumeration(
    name: &str,
    value: String,
    domain: Option<Vec<String>>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if !exists(name) {
        return false;
    }

    register_enumeration(name, value, domain, flags, description)
}

/// Registers a Dvar of type [`DvarValue::Color`].
///
/// Essentially an easier-to-use wrapper around [`register_variant`] to
/// register a Dvar with type [`DvarValue::Color`], using the provided name,
/// values, flags, and description. Each color component is represented by
/// an [`f32`] with a domain of (0.0, 1.0).
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `red` - An [`f32`] representing the R component of an RGBA-format color.
/// Must be within the domain (0.0, 1.0).
/// * `green` - An [`f32`] representing the G component of an RGBA-format color.
/// Must be within the domain (0.0, 1.0).
/// * `blue` - An [`f32`] representing the B component of an RGBA-format color.
/// Must be within the domain (0.0, 1.0).
/// * `alpha` - An [`f32`] representing the A component of an RGBA-format color.
/// Must be within the domain (0.0, 1.0).
/// * `flags` - The [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - A [`String`] containing a description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// if find(name).is_none {
///     let red = 1.0;
///     let blue = 1.0;
///     let green = 0.0;
///     let alpha = 1.0;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type color"
///     register_color(name, red, green, blue,
///                    alpha, flags, description);
/// }
/// ```
pub fn register_color(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    let r = if red < 0.0 {
        0.0
    } else if red > 1.0 {
        1.0
    } else {
        red
    } * 255.0
        + 0.001
        + 9.313226e-10;

    let g = if green < 0.0 {
        0.0
    } else if green > 1.0 {
        1.0
    } else {
        green
    } * 255.0
        + 0.001
        + 9.313226e-10;

    let b = if blue < 0.0 {
        0.0
    } else if blue > 1.0 {
        1.0
    } else {
        blue
    } * 255.0
        + 0.001
        + 9.313226e-10;

    let a = if alpha < 0.0 {
        0.0
    } else if alpha > 1.0 {
        1.0
    } else {
        alpha
    } * 255.0
        + 0.001
        + 9.313226e-10;

    if exists(name) {
        return false;
    }

    let b_2: bool;
    let other_name: &str;
    {
        let lock = DVARS.clone();
        let mut writer = lock.try_write().expect("dvar::register_new: failed to acquire writer lock. Probably still held by calling function.");

        if writer.len() + 1 > DVAR_COUNT_MAX {
            com::errorln(
                com::ErrorParm::FATAL,
                format!(
                    "Can\'t create dvar \'{}\': {} dvars already exist",
                    name, DVAR_COUNT_MAX
                ),
            );
            return false;
        }

        let dvar = DvarBuilder::new()
            .name(name)
            .description(description.unwrap_or_default().to_string())
            .flags(flags.unwrap_or_default())
            .type_color()
            .value((r, g, b, a))
            .build();
        b_2 = writer.insert(name.to_string(), Box::new(dvar)).is_some();
        other_name = &writer.get(name).unwrap().name;
        if b_2 {
            com::errorln(com::ErrorParm::FATAL, format!("dvar name hash collision between \'{}\' and \'{}\' Please change one of these names to remove the hash collision", name, other_name));
            return false;
        }
    }

    true
}

pub fn register_new_color(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        return false;
    }

    register_color(name, red, green, blue, alpha, flags, description)
}

pub fn reregister_color(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if !exists(name) {
        return false;
    }

    register_color(name, red, green, blue, alpha, flags, description)
}

/// Registers a Dvar of type [`DvarValue::Int64`].
///
/// Essentially an easier-to-use wrapper around [`register_variant`] to
/// register a Dvar with type [`DvarValue::Int64`], using the provided name,
/// value, domain, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An [`i64`] representing the value to register the [`Dvar`] with
/// * `min` - An [`i64`] representing the lower bound
/// for the domain for `value`
/// * `max` - An [`i64`] representing the upper bound
/// for the domain for `value`
/// * `flags` - The [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - A [`String`] containing a description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// if find(name).is_none {
///     let value = 67894867969;
///     let min = i64::MIN;
///     let max = i64::MAX;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type int64"
///     register_int64(name, value, min, max, flags, description);
/// }
/// ```
pub fn register_int64(
    name: &str,
    value: i64,
    min: Option<i64>,
    max: Option<i64>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        return false;
    }

    let b: bool;
    let other_name: &str;
    {
        let lock = DVARS.clone();
        let mut writer = lock.try_write().expect("dvar::register_new: failed to acquire writer lock. Probably still held by calling function.");

        if writer.len() + 1 > DVAR_COUNT_MAX {
            com::errorln(
                com::ErrorParm::FATAL,
                format!(
                    "Can\'t create dvar \'{}\': {} dvars already exist",
                    name, DVAR_COUNT_MAX
                ),
            );
            return false;
        }

        let dvar = DvarBuilder::new()
            .name(name)
            .description(description.unwrap_or_default().to_string())
            .flags(flags.unwrap_or_default())
            .type_int64()
            .domain(min.unwrap_or(i64::MIN), max.unwrap_or(i64::MAX))
            .value(value)
            .build();
        b = writer.insert(name.to_string(), Box::new(dvar)).is_some();
        other_name = &writer.get(name).unwrap().name;

        if b {
            com::errorln(com::ErrorParm::FATAL, format!("dvar name hash collision between \'{}\' and \'{}\' Please change one of these names to remove the hash collision", name, other_name));
            return false;
        }
    }

    true
}

pub fn register_new_int64(
    name: &str,
    value: i64,
    min: Option<i64>,
    max: Option<i64>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        return false;
    }

    register_int64(name, value, min, max, flags, description)
}

pub fn reregister_int64(
    name: &str,
    value: i64,
    min: Option<i64>,
    max: Option<i64>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if !exists(name) {
        return false;
    }

    register_int64(name, value, min, max, flags, description)
}

/// Registers a Dvar of type [`DvarValue::LinearColorRGB`].
///
/// Essentially an easier-to-use wrapper around [`register_variant`] to
/// register a Dvar with type [`DvarValue::LinearColorRGB`], using the provided name,
/// values, flags, and description. Each color component is represented by
/// an [`f32`] with a domain of (0.0, 1.0).
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `red` - An [`f32`] representing the R component of an RGB-format color.
/// Must be within the domain (0.0, 1.0).
/// * `green` - An [`f32`] representing the G component of an RGB-format color.
/// Must be within the domain (0.0, 1.0).
/// * `blue` - An [`f32`] representing the B component of an RGB-format color.
/// Must be within the domain (0.0, 1.0).
/// * `min` - An [`f32`] representing the lower bound
/// of the domain for `red`, `green`, and `blue`.
/// * `max` - An [`f32`] representing the upper bound
/// of the domain for `red`, `green`, and `blue`.
/// * `flags` - The [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - A [`String`] containing a description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// if find(name).is_none {
///     let red = 0.6;
///     let blue = 0.2;
///     let green = 0.3;
///     let min = 0.1;
///     let max = 0.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type color"
///     register_linear_color_rgb(name, red, green, blue,
///                               min, max flags, description);
/// }
/// ```
#[allow(clippy::too_many_arguments)]
pub fn register_linear_color_rgb(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    let b: bool;
    let other_name: &str;
    {
        let lock = DVARS.clone();
        let mut writer = lock.try_write().expect("dvar::register_new: failed to acquire writer lock. Probably still held by calling function.");

        if writer.len() + 1 > DVAR_COUNT_MAX {
            com::errorln(
                com::ErrorParm::FATAL,
                format!(
                    "Can\'t create dvar \'{}\': {} dvars already exist",
                    name, DVAR_COUNT_MAX
                ),
            );
            return false;
        }

        let dvar = DvarBuilder::new()
            .name(name)
            .description(description.unwrap_or_default().to_string())
            .flags(flags.unwrap_or_default())
            .type_linear_color_rgb()
            .domain(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX))
            .value((red, green, blue))
            .build();
        b = writer.insert(name.to_string(), Box::new(dvar)).is_some();
        other_name = &writer.get(name).unwrap().name;
        if b {
            com::errorln(com::ErrorParm::FATAL, format!("dvar name hash collision between \'{}\' and \'{}\' Please change one of these names to remove the hash collision", name, other_name));
            return false;
        }
    }

    true
}

#[allow(clippy::too_many_arguments)]
pub fn register_new_linear_color_rgb(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        return false;
    }

    register_linear_color_rgb(
        name,
        red,
        green,
        blue,
        min,
        max,
        flags,
        description,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn reregister_linear_color_rgb(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if !exists(name) {
        return false;
    }

    register_linear_color_rgb(
        name,
        red,
        green,
        blue,
        min,
        max,
        flags,
        description,
    )
}

/// Registers a Dvar of type [`DvarValue::ColorXYZ`].
///
/// Essentially an easier-to-use wrapper around [`register_variant`] to
/// register a Dvar with type [`DvarValue::ColorXYZ`], using the provided name,
/// values, flags, and description. Each color component is represented by
/// an [`f32`] with a domain of (0.0, 1.0).
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `x` - An [`f32`] the x-value of the Dvar.
/// * `y` - An [`f32`] the y-value of the Dvar.
/// * `z` - An [`f32`] the z-value of the Dvar.
/// * `min` - An [`f32`] representing the lower bound
/// of the domain for `red`, `green`, and `blue`.
/// * `max` - An [`f32`] representing the upper bound
/// of the domain for `red`, `green`, and `blue`.
/// * `flags` - The [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - A [`String`] containing a description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// if find(name).is_none {
///     let x = 0.6;
///     let y = 0.2;
///     let z = 0.3;
///     let min = 0.1;
///     let max = 0.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type color"
///     register_color_xyz(name, x, y, z, min, max, flags, description);
/// }
/// ```
#[allow(clippy::too_many_arguments)]
pub fn register_color_xyz(
    name: &str,
    x: f32,
    y: f32,
    z: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    let b: bool;
    let other_name: &str;
    {
        let lock = DVARS.clone();
        let mut writer = lock.try_write().expect("dvar::register_new: failed to acquire writer lock. Probably still held by calling function.");

        if writer.len() + 1 > DVAR_COUNT_MAX {
            com::errorln(
                com::ErrorParm::FATAL,
                format!(
                    "Can\'t create dvar \'{}\': {} dvars already exist",
                    name, DVAR_COUNT_MAX
                ),
            );
            return false;
        }

        let dvar = DvarBuilder::new()
            .name(name)
            .description(description.unwrap_or_default().to_string())
            .flags(flags.unwrap_or_default())
            .type_color_xyz()
            .domain(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX))
            .value((x, y, z))
            .build();
        b = writer.insert(name.to_string(), Box::new(dvar)).is_some();
        other_name = &writer.get(name).unwrap().name;

        if b {
            com::errorln(com::ErrorParm::FATAL, format!("dvar name hash collision between \'{}\' and \'{}\' Please change one of these names to remove the hash collision", name, other_name));
            return false;
        }
    }

    true
}

#[allow(clippy::too_many_arguments)]
pub fn register_new_color_xyz(
    name: &str,
    x: f32,
    y: f32,
    z: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        return false;
    }

    register_color_xyz(name, x, y, z, min, max, flags, description)
}

#[allow(clippy::too_many_arguments)]
pub fn reregister_color_xyz(
    name: &str,
    x: f32,
    y: f32,
    z: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if !exists(name) {
        return false;
    }

    register_color_xyz(name, x, y, z, min, max, flags, description)
}

/// Sets the value of an existing [`Dvar`] from the supplied [`SetSource`]
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source `source`. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The [`DvarValue`] to update the [`Dvar`] with.
/// Must be within the domain supplied by `domain`.
/// * `source` - The [`SetSource`] to set the value with.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = DvarValue::Float(372.1);
///     let source = SetSource::Internal;
///     set_variant_from_source(name, DvarValue::Float(value), source);
/// }
/// ```
fn set_variant_from_source(
    name: &str,
    value: DvarValue,
    source: SetSource,
) -> bool {
    match find(name) {
        Some(_) => {
            let lock = DVARS.clone();
            let mut writer = lock.try_write().expect("");
            writer.get_mut(name).unwrap().set_variant(value, source);
            true
        }
        None => false,
    }
}

/// Sets the value of an existing [`Dvar`] from the supplied [`SetSource`]
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source `source`. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `source` - The [`SetSource`] to set the value with.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = false;
///     let source = SetSource::Internal;
///     set_bool_from_source(name, value, source);
/// }
/// ```
pub fn set_bool_from_source(
    name: &str,
    value: bool,
    source: SetSource,
) -> bool {
    set_variant_from_source(name, DvarValue::Bool(value), source)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::Internal`]. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = false;
///     set_bool(name, value);
/// }
/// ```
pub fn set_bool(name: &str, value: bool) -> bool {
    set_bool_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`], or registers a new one with
/// said value.
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name`, or registers a new [`Dvar`] with said parameters.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
/// * `description` - Optional description to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// let value = false;
/// let flags = DvarFlags::External;
/// let description = "External Dvar".to_string();
/// set_or_register_bool(name, value, Some(flags), Some(description));
/// ```
pub fn set_or_register_bool(
    name: &str,
    value: bool,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        set_bool(name, value)
    } else {
        register_bool(name, value, flags, description)
    }
}

/// Sets the value of an existing [`Dvar`] from the supplied [`SetSource`]
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source `source`. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `source` - The [`SetSource`] to set the value with.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = 420.49f32;
///     let source = SetSource::Internal;
///     set_float_from_source(name, value, source);
/// }
/// ```
pub fn set_float_from_source(
    name: &str,
    value: f32,
    source: SetSource,
) -> bool {
    set_variant_from_source(name, DvarValue::Float(value), source)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::Internal`]. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = 420.49f;
///     set_float(name, value);
/// }
/// ```
pub fn set_float(name: &str, value: f32) -> bool {
    set_float_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`], or registers a new one with
/// said value.
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name`, or registers a new [`Dvar`] with said parameters.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
/// * `description` - Optional description to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// let value = 420.69f32;
/// let min = 400.0f32;
/// let max = 800.0f32;
/// let flags = DvarFlags::External;
/// let description = "External Dvar".to_string();
/// set_or_register_float(name, value, Some(min), Some(max), Some(flags), Some(description));
/// ```
pub fn set_or_register_float(
    name: &str,
    value: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        set_float(name, value)
    } else {
        register_float(name, value, min, max, flags, description)
    }
}

/// Sets the value of an existing [`Dvar`] from the supplied [`SetSource`]
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source `source`. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `source` - The [`SetSource`] to set the value with.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (420.49f32, 694.20f32);
///     let source = SetSource::Internal;
///     set_vector2_from_source(name, value, source);
/// }
/// ```
pub fn set_vector2_from_source(
    name: &str,
    value: Vec2f32,
    source: SetSource,
) -> bool {
    set_variant_from_source(name, DvarValue::Vector2(value), source)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::Internal`]. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (420.69f, 694.20f32);
///     set_vector2(name, value);
/// }
/// ```
pub fn set_vector2(name: &str, value: Vec2f32) -> bool {
    set_vector2_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`], or registers a new one with
/// said value.
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name`, or registers a new [`Dvar`] with said parameters.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
/// * `description` - Optional description to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// let value = (420.69f32, 694.20f32);
/// let min = 400.0f32;
/// let max = 800.0f32;
/// let flags = DvarFlags::External;
/// let description = "External Dvar".to_string();
/// set_or_register_vector2(name, value, Some(min), Some(max), Some(flags), Some(description));
/// ```
pub fn set_or_register_vector2(
    name: &str,
    value: Vec2f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        set_vector2(name, value)
    } else {
        register_vector2(name, value, min, max, flags, description)
    }
}

/// Sets the value of an existing [`Dvar`] from the supplied [`SetSource`]
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source `source`. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `source` - The [`SetSource`] to set the value with.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (420.49f32, 694.20f32, -165.0f32);
///     let source = SetSource::Internal;
///     set_vector3_from_source(name, value, source);
/// }
/// ```
pub fn set_vector3_from_source(
    name: &str,
    value: Vec3f32,
    source: SetSource,
) -> bool {
    set_variant_from_source(name, DvarValue::Vector3(value), source)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::Internal`]. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (420.69f, 694.20f32, -165.0f32);
///     set_vector3(name, value);
/// }
/// ```
pub fn set_vector3(name: &str, value: Vec3f32) -> bool {
    set_vector3_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`], or registers a new one with
/// said value.
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name`, or registers a new [`Dvar`] with said parameters.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
/// * `description` - Optional description to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// let value = (420.69f32, 694.20f32, -165.0f32);
/// let min = -200.0f32;
/// let max = 800.0f32;
/// let flags = DvarFlags::External;
/// let description = "External Dvar".to_string();
/// set_or_register_vector3(name, value, Some(min), Some(max), Some(flags), Some(description));
/// ```
pub fn set_or_register_vector3(
    name: &str,
    value: Vec3f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        set_vector3(name, value)
    } else {
        register_vector3(name, value, min, max, flags, description)
    }
}

/// Sets the value of an existing [`Dvar`] from the supplied [`SetSource`]
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source `source`. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `source` - The [`SetSource`] to set the value with.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (420.49f32, 694.20f32, -165.0f32, 196.23f32);
///     let source = SetSource::Internal;
///     set_vector4_from_source(name, value, source);
/// }
/// ```
pub fn set_vector4_from_source(
    name: &str,
    value: Vec4f32,
    source: SetSource,
) -> bool {
    set_variant_from_source(name, DvarValue::Vector4(value), source)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::Internal`]. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (420.69f, 694.20f32, -165.0f32, 196.23f32);
///     set_vector4(name, value);
/// }
/// ```
pub fn set_vector4(name: &str, value: Vec4f32) -> bool {
    set_vector4_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`], or registers a new one with
/// said value.
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name`, or registers a new [`Dvar`] with said parameters.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
/// * `description` - Optional description to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// let value = (420.69f32, 694.20f32, -165.0f32, 196.23f32);
/// let min = -400.0f32;
/// let max = 800.0f32;
/// let flags = DvarFlags::External;
/// let description = "External Dvar".to_string();
/// set_or_register_vector4(name, value, Some(min), Some(max), Some(flags), Some(description));
/// ```
pub fn set_or_register_vector4(
    name: &str,
    value: Vec4f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        set_vector4(name, value)
    } else {
        register_vector4(name, value, min, max, flags, description)
    }
}

/// Sets the value of an existing [`Dvar`] from the supplied [`SetSource`]
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source `source`. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `source` - The [`SetSource`] to set the value with.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = 420i32;
///     let source = SetSource::Internal;
///     set_int_from_source(name, value, source);
/// }
/// ```
pub fn set_int_from_source(name: &str, value: i32, source: SetSource) -> bool {
    set_variant_from_source(name, DvarValue::Int(value), source)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::Internal`]. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = 420;
///     set_int(name, value);
/// }
/// ```
pub fn set_int(name: &str, value: i32) -> bool {
    set_int_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`], or registers a new one with
/// said value.
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name`, or registers a new [`Dvar`] with said parameters.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
/// * `description` - Optional description to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// let value = 420i32;
/// let min = 200i32;
/// let max = 400i32;
/// let flags = DvarFlags::External;
/// let description = "External Dvar".to_string();
/// set_or_register_int(name, value, Some(min), Some(max), Some(flags), Some(description));
/// ```
pub fn set_or_register_int(
    name: &str,
    value: i32,
    min: Option<i32>,
    max: Option<i32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        set_int(name, value)
    } else {
        register_int(name, value, min, max, flags, description)
    }
}

/// Sets the value of an existing [`Dvar`] from the supplied [`SetSource`]
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source `source`. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `source` - The [`SetSource`] to set the value with.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = "test".to_string();
///     let source = SetSource::Internal;
///     set_string_from_source(name, value, source);
/// }
/// ```
pub fn set_string_from_source(
    name: &str,
    value: String,
    source: SetSource,
) -> bool {
    set_variant_from_source(name, DvarValue::String(value), source)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::Internal`]. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = "test".to_string();
///     set_string(name, value);
/// }
/// ```
fn set_string(name: &str, value: String) -> bool {
    set_string_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`], or registers a new one with
/// said value.
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name`, or registers a new [`Dvar`] with said parameters.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
/// * `description` - Optional description to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// let value = "test".to_string();
/// let flags = DvarFlags::External;
/// let description = "External Dvar".to_string();
/// set_or_register_string(name, value, Some(flags), Some(description));
/// ```
pub fn set_or_register_string(
    name: &str,
    value: String,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        set_string(name, value)
    } else {
        register_string(name, value, flags, description)
    }
}

/// Sets the value of an existing [`Dvar`] from the supplied [`SetSource`]
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source `source`. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `source` - The [`SetSource`] to set the value with.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = "test".to_string();
///     let source = SetSource::Internal;
///     set_enumeration_from_source(name, value, source);
/// }
/// ```
pub fn set_enumeration_from_source(
    name: &str,
    value: String,
    source: SetSource,
) -> bool {
    set_variant_from_source(name, DvarValue::Enumeration(value), source)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::Internal`]. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = "test".to_string();
///     set_enumeration(name, value);
/// }
/// ```
fn set_enumeration(name: &str, value: String) -> bool {
    set_enumeration_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`], or registers a new one with
/// said value.
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name`, or registers a new [`Dvar`] with said parameters.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
/// * `description` - Optional description to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// let value = "test".to_string();
/// let domain = vec!["test".to_string(), "test2".to_string()];
/// let flags = DvarFlags::External;
/// let description = "External Dvar".to_string();
/// set_or_register_enumeration(name, value, Some(domain), Some(flags), Some(description));
/// ```
pub fn set_or_register_enumeration(
    name: &str,
    value: String,
    domain: Option<Vec<String>>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        set_enumeration(name, value)
    } else {
        register_enumeration(name, value, domain, flags, description)
    }
}

/// Sets the value of an existing [`Dvar`] from the supplied [`SetSource`]
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source `source`. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `source` - The [`SetSource`] to set the value with.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (0.957f32, 0.66f32, 0.71875f32, 1.0f);
///     let source = SetSource::Internal;
///     set_color_from_source(name, value, source);
/// }
/// ```
pub fn set_color_from_source(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    source: SetSource,
) -> bool {
    set_variant_from_source(
        name,
        DvarValue::Color((red, green, blue, alpha)),
        source,
    )
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::Internal`]. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (0.957f32, 0.66f32, 0.71875f32, 1.0f);
///     set_color(name, value);
/// }
/// ```
pub fn set_color(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
) -> bool {
    set_color_from_source(name, red, green, blue, alpha, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`], or registers a new one with
/// said value.
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name`, or registers a new [`Dvar`] with said parameters.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
/// * `description` - Optional description to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// let value = (0.957f32, 0.66f32, 0.71875f32, 1.0f);
/// let flags = DvarFlags::External;
/// let description = "External Dvar".to_string();
/// set_or_register_color(name, value, Some(flags), Some(description));
/// ```
pub fn set_or_register_color(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        set_color(name, red, green, blue, alpha)
    } else {
        register_color(name, red, green, blue, alpha, flags, description)
    }
}

/// Sets the value of an existing [`Dvar`] from the supplied [`SetSource`]
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source `source`. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `source` - The [`SetSource`] to set the value with.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (18_446_744_073_709_551_616 / 2) - 1;
///     let source = SetSource::Internal;
///     set_int64_from_source(name, value, source);
/// }
/// ```
pub fn set_int64_from_source(
    name: &str,
    value: i64,
    source: SetSource,
) -> bool {
    set_variant_from_source(name, DvarValue::Int64(value), source)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::Internal`]. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (18_446_744_073_709_551_616 / 2) - 1;
///     set_int64(name, value);
/// }
/// ```
pub fn set_int64(name: &str, value: i64) -> bool {
    set_int64_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`], or registers a new one with
/// said value.
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name`, or registers a new [`Dvar`] with said parameters.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
/// * `description` - Optional description to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// let value = (18_446_744_073_709_551_616 / 2) - 1;
/// let min = 0;
/// let max = i64::MAX;
/// let flags = DvarFlags::External;
/// let description = "External Dvar".to_string();
/// set_or_register_int64(name, value, Some(min), Some(max), Some(flags), Some(description));
/// ```
pub fn set_or_register_int64(
    name: &str,
    value: i64,
    min: Option<i64>,
    max: Option<i64>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        set_int64(name, value)
    } else {
        register_int64(name, value, min, max, flags, description)
    }
}

/// Sets the value of an existing [`Dvar`] from the supplied [`SetSource`]
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source `source`. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `source` - The [`SetSource`] to set the value with.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (0.957f32, 0.66f32, 0.71875f32);
///     let source = SetSource::Internal;
///     set_linear_color_rgb_from_source(name, value, source);
/// }
/// ```
pub fn set_linear_color_rgb_from_source(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    source: SetSource,
) -> bool {
    set_variant_from_source(
        name,
        DvarValue::LinearColorRGB((red, green, blue)),
        source,
    )
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::Internal`]. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (0.957f32, 0.66f32, 0.71875f32);
///     set_linear_color_rgb(name, value);
/// }
/// ```
pub fn set_linear_color_rgb(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
) -> bool {
    set_linear_color_rgb_from_source(
        name,
        red,
        green,
        blue,
        SetSource::Internal,
    )
}

/// Sets the value of an existing [`Dvar`], or registers a new one with
/// said value.
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name`, or registers a new [`Dvar`] with said parameters.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
/// * `description` - Optional description to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// # Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// let value = (0.957f32, 0.66f32, 0.71875f32);
/// let min = 0.5f32;
/// let max = 0.97f32;
/// let flags = DvarFlags::External;
/// let description = "External Dvar".to_string();
/// set_or_register_linear_color_rgb(name, value, Some(min), Some(max), Some(flags), Some(description));
/// ```
#[allow(clippy::too_many_arguments)]
pub fn set_or_register_linear_color_rgb(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        set_linear_color_rgb(name, red, green, blue)
    } else {
        register_linear_color_rgb(
            name,
            red,
            green,
            blue,
            min,
            max,
            flags,
            description,
        )
    }
}

/// Sets the value of an existing [`Dvar`] from the supplied [`SetSource`]
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source `source`. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `source` - The [`SetSource`] to set the value with.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (0.957f32, 0.66f32, 0.71875f32);
///     let source = SetSource::Internal;
///     set_color_xyz_from_source(name, value, source);
/// }
/// ```
pub fn set_color_xyz_from_source(
    name: &str,
    x: f32,
    y: f32,
    z: f32,
    source: SetSource,
) -> bool {
    set_variant_from_source(name, DvarValue::ColorXYZ((x, y, z)), source)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::Internal`]. Does nothing if a [`Dvar`] with `name`
/// doesn't exist.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (0.957f32, 0.66f32, 0.71875f32);
///     set_color_xyz(name, value);
/// }
/// ```
pub fn set_color_xyz(name: &str, x: f32, y: f32, z: f32) -> bool {
    set_color_xyz_from_source(name, x, y, z, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`], or registers a new one with
/// said value.
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name`, or registers a new [`Dvar`] with said parameters.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be updated.
/// * `value` - The value to set the [`Dvar`]'s value to.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
/// * `description` - Optional description to register the [`Dvar`] with if
/// said [`Dvar`] doesn't exist. Does nothing it if does exist.
///
/// # Return Value
///
/// Returns true if set was successful, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let name = "sv_test".to_string();
/// let dvar = find(&name);
/// let value = (0.957f32, 0.66f32, 0.71875f32);
/// let min = 0.5f32;
/// let max = 0.97f32;
/// let flags = DvarFlags::External;
/// let description = "External Dvar".to_string();
/// set_or_register_color_xyz(name, value, Some(min), Some(max), Some(flags), Some(description));
/// ```
#[allow(clippy::too_many_arguments)]
pub fn set_or_register_color_xyz(
    name: &str,
    x: f32,
    y: f32,
    z: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> bool {
    if exists(name) {
        set_color_xyz(name, x, y, z)
    } else {
        register_color_xyz(name, x, y, z, min, max, flags, description)
    }
}

/// Retrieves a [`bool`] value from a [`Dvar`].
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Bool`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let b = get_bool("sv_test".to_string()).unwrap();
/// ```
pub fn get_bool(name: &str) -> Option<bool> {
    return match find(name) {
        Some(d) => d.current.as_bool(),
        None => None,
    };
}

/// Retrieves a [`bool`] value from a [`Dvar`] if said [`Dvar`] exists,
/// registers a [`Dvar`] of type [`DvarValue::Bool`] with the supplied parameters 
/// otherwise.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved or registered.
/// * `value` - The value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `flags` - Optional flags to register the [`Dvar`] with if it
/// doesn't already exist.
/// * `description` - Optional description to register the [`Dvar`] with if it
/// doesn't already exist.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Bool`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let b = get_or_register_bool("sv_test", false, None, None).unwrap();
/// ```
pub fn get_or_register_bool(
    name: &str,
    value: bool,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> Option<bool> {
    get_bool(name).or_else(|| {
        if !register_bool(name, value, flags, description) {
            return None;
        }
        Some(value)
    })
}

/// Retrieves an [`f32`] value from a [`Dvar`].
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Float`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let f = get_float("sv_test".to_string()).unwrap();
/// ```
pub fn get_float(name: &str) -> Option<f32> {
    return match find(name) {
        Some(d) => d.current.as_float(),
        None => None,
    };
}

/// Retrieves an [`f32`] value from a [`Dvar`] if said [`Dvar`] exists,
/// registers a [`Dvar`] of type [`DvarValue::Float`] with the supplied parameters 
/// otherwise.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved or registered.
/// * `value` - The value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `min` - An optional minimum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `max` - An optional maximum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `flags` - Optional flags to register the [`Dvar`] with if it
/// doesn't already exist.
/// * `description` - Optional description to register the [`Dvar`] with if it
/// doesn't already exist.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Float`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let f = get_or_register_float("sv_test", 32.6f, None, None, None, None).unwrap();
/// ```
pub fn get_or_register_float(
    name: &str,
    value: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> Option<f32> {
    get_float(name).or_else(|| {
        if !register_float(name, value, min, max, flags, description) {
            return None;
        }
        Some(value)
    })
}

/// Retrieves a [`Vec2f32`] value from a [`Dvar`].
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Vector2`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let v2 = get_vector2("sv_test".to_string()).unwrap();
/// ```
pub fn get_vector2(name: &str) -> Option<Vec2f32> {
    return match find(name) {
        Some(d) => d.current.as_vector2(),
        None => None,
    };
}

/// Retrieves a [`Vec2f32`] value from a [`Dvar`] if said [`Dvar`] exists,
/// registers a [`Dvar`] of type [`DvarValue::Vector2`] with the supplied parameters 
/// otherwise.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved or registered.
/// * `value` - The value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `min` - An optional minimum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `max` - An optional maximum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `flags` - Optional flags to register the [`Dvar`] with if it
/// doesn't already exist.
/// * `description` - Optional description to register the [`Dvar`] with if it
/// doesn't already exist.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Vector2`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let v2 = get_or_register_vector2("sv_test", (32.6f, 68.9f), None, None, None, None).unwrap();
/// ```
pub fn get_or_register_vector2(
    name: &str,
    value: Vec2f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> Option<Vec2f32> {
    get_vector2(name).or_else(|| {
        if !register_vector2(name, value, min, max, flags, description) {
            return None;
        }
        Some(value)
    })
}

/// Retrieves a [`Vec3f32`] value from a [`Dvar`].
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Vector3`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let v3 = get_vector3("sv_test".to_string()).unwrap();
/// ```
pub fn get_vector3(name: &str) -> Option<Vec3f32> {
    return match find(name) {
        Some(d) => d.current.as_vector3(),
        None => None,
    };
}

/// Retrieves a [`Vec3f32`] value from a [`Dvar`] if said [`Dvar`] exists,
/// registers a [`Dvar`] of type [`DvarValue::Vector3`] with the supplied parameters 
/// otherwise.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved or registered.
/// * `value` - The value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `min` - An optional minimum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `max` - An optional maximum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `flags` - Optional flags to register the [`Dvar`] with if it
/// doesn't already exist.
/// * `description` - Optional description to register the [`Dvar`] with if it
/// doesn't already exist.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Vector3`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let v3 = get_or_register_vector3("sv_test", (32.6f, 68.9f, 410.87f), None, None None, None).unwrap();
/// ```
pub fn get_or_register_vector3(
    name: &str,
    value: Vec3f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> Option<Vec3f32> {
    get_vector3(name).or_else(|| {
        if !register_vector3(name, value, min, max, flags, description) {
            return None;
        }
        Some(value)
    })
}

/// Retrieves a [`Vec4f32`] value from a [`Dvar`].
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Vector4`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let v4 = get_vector4("sv_test".to_string()).unwrap();
/// ```
pub fn get_vector4(name: &str) -> Option<Vec4f32> {
    return match find(name) {
        Some(d) => d.current.as_vector4(),
        None => None,
    };
}

/// Retrieves a [`Vec4f32`] value from a [`Dvar`] if said [`Dvar`] exists,
/// registers a [`Dvar`] of type [`DvarValue::Vector4`] with the supplied parameters 
/// otherwise.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved or registered.
/// * `value` - The value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `min` - An optional minimum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `max` - An optional maximum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `flags` - Optional flags to register the [`Dvar`] with if it
/// doesn't already exist.
/// * `description` - Optional description to register the [`Dvar`] with if it
/// doesn't already exist.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Vector4`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let v4 = get_or_register_vector4("sv_test", (32.6f, 68.9f, 410.87f, 683.72f), None, None, None, None).unwrap();
/// ```
pub fn get_or_register_vector4(
    name: &str,
    value: Vec4f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> Option<Vec4f32> {
    get_vector4(name).or_else(|| {
        if !register_vector4(name, value, min, max, flags, description) {
            return None;
        }
        Some(value)
    })
}

/// Retrieves an [`i32`] value from a [`Dvar`].
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Int`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let v2 = get_int("sv_test".to_string()).unwrap();
/// ```
pub fn get_int(name: &str) -> Option<i32> {
    return match find(name) {
        Some(d) => d.current.as_int(),
        None => None,
    };
}

/// Retrieves an [`i32`] value from a [`Dvar`] if said [`Dvar`] exists,
/// registers a [`Dvar`] of type [`DvarValue::Int`] with the supplied parameters 
/// otherwise.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved or registered.
/// * `value` - The value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `min` - An optional minimum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `max` - An optional maximum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `flags` - Optional flags to register the [`Dvar`] with if it
/// doesn't already exist.
/// * `description` - Optional description to register the [`Dvar`] with if it
/// doesn't already exist.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Int`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let i = get_or_register_int("sv_test", 32, None, None, None, None).unwrap();
/// ```
pub fn get_or_register_int(
    name: &str,
    value: i32,
    min: Option<i32>,
    max: Option<i32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> Option<i32> {
    get_int(name).or_else(|| {
        if !register_int(name, value, min, max, flags, description) {
            return None;
        }
        Some(value)
    })
}

/// Retrieves a [`String`] value from a [`Dvar`].
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::String`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let v2 = get_string("sv_test".to_string()).unwrap();
/// ```
pub fn get_string(name: &str) -> Option<String> {
    return match find(name) {
        Some(d) => d.current.as_string(),
        None => None,
    };
}

/// Retrieves a [`String`] value from a [`Dvar`] if said [`Dvar`] exists,
/// registers a [`Dvar`] of type [`DvarValue::String`] with the supplied parameters 
/// otherwise.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved or registered.
/// * `value` - The value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `flags` - Optional flags to register the [`Dvar`] with if it
/// doesn't already exist.
/// * `description` - Optional description to register the [`Dvar`] with if it
/// doesn't already exist.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::String`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let s = get_or_register_string("sv_test", false, None, None).unwrap();
/// ```
pub fn get_or_register_string(
    name: &str,
    value: String,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> Option<String> {
    get_string(name).or_else(|| {
        if !register_string(name, value.clone(), flags, description) {
            return None;
        }
        Some(value)
    })
}

/// Retrieves a [`String`] value from a [`Dvar`].
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Enumeration`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let v2 = get_enumeration("sv_test".to_string()).unwrap();
/// ```
pub fn get_enumeration(name: &str) -> Option<String> {
    return match find(name) {
        Some(d) => d.current.as_enumeration(),
        None => None,
    };
}

/// Retrieves an [`String`] value from a [`Dvar`] if said [`Dvar`] exists,
/// registers a [`Dvar`] of type [`DvarValue::Enumeration`] with the supplied parameters 
/// otherwise.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved or registered.
/// * `value` - The value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `domain` - An optional domain to supply for the [`Dvar`]
/// if it doesn't already exist.
/// * `flags` - Optional flags to register the [`Dvar`] with if it
/// doesn't already exist.
/// * `description` - Optional description to register the [`Dvar`] with if it
/// doesn't already exist.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Enumeration`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let i = get_or_register_enumeration("sv_test", "test", None, None, None).unwrap();
/// ```
pub fn get_or_register_enumeration(
    name: &str,
    value: String,
    domain: Option<Vec<String>>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> Option<String> {
    get_enumeration(name).or_else(|| {
        if !register_enumeration(
            name,
            value.clone(),
            domain,
            flags,
            description,
        ) {
            return None;
        }
        Some(value)
    })
}

/// Retrieves a [`Vec4f32`] value from a [`Dvar`].
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Color`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let c = get_color("sv_test".to_string()).unwrap();
/// ```
pub fn get_color(name: &str) -> Option<Vec4f32> {
    return match find(name) {
        Some(d) => d.current.as_color(),
        None => None,
    };
}

/// Retrieves a [`Vec4f32`] value from a [`Dvar`] if said [`Dvar`] exists,
/// registers a [`Dvar`] of type [`DvarValue::Color`] with the supplied parameters 
/// otherwise.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved or registered.
/// * `red` - The red component of the [`Dvar`]'s value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `green` - The red component of the [`Dvar`]'s value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `blue` - The red component of the [`Dvar`]'s value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `alpha` - The red component of the [`Dvar`]'s value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `flags` - Optional flags to register the [`Dvar`] with if it
/// doesn't already exist.
/// * `description` - Optional description to register the [`Dvar`] with if it
/// doesn't already exist.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Color`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let c = get_or_register_color("sv_test", 0.1f, 0.3f, 0.5f, 0.2f, None, None).unwrap();
/// ```
pub fn get_or_register_color(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> Option<Vec4f32> {
    get_color(name).or_else(|| {
        if !register_color(name, red, green, blue, alpha, flags, description) {
            return None;
        }
        Some((red, green, blue, alpha))
    })
}

/// Retrieves an [`i64`] value from a [`Dvar`].
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Int64`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let i = get_int64("sv_test".to_string()).unwrap();
/// ```
pub fn get_int64(name: &str) -> Option<i64> {
    return match find(name) {
        Some(d) => d.current.as_int64(),
        None => None,
    };
}

/// Retrieves a [`i64`] value from a [`Dvar`] if said [`Dvar`] exists,
/// registers a [`Dvar`] of type [`DvarValue::Int64`] with the supplied parameters 
/// otherwise.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved or registered.
/// * `value` - The value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `min` - An optional minimum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `max` - An optional maximum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `flags` - Optional flags to register the [`Dvar`] with if it
/// doesn't already exist.
/// * `description` - Optional description to register the [`Dvar`] with if it
/// doesn't already exist.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::Int64`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let i = get_or_register_int64("sv_test", 364757258672, None, None, None, None).unwrap();
/// ```
pub fn get_or_register_int64(
    name: &str,
    value: i64,
    min: Option<i64>,
    max: Option<i64>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> Option<i64> {
    get_int64(name).or_else(|| {
        if !register_int64(name, value, min, max, flags, description) {
            return None;
        }
        Some(value)
    })
}

/// Retrieves a [`Vec3f32`] value from a [`Dvar`].
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::LinearColorRGB`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let lc = get_linear_color_rgb("sv_test".to_string()).unwrap();
/// ```
pub fn get_linear_color_rgb(name: &str) -> Option<Vec3f32> {
    return match find(name) {
        Some(d) => d.current.as_linear_color_rgb(),
        None => None,
    };
}

/// Retrieves a [`Vec3f32`] value from a [`Dvar`] if said [`Dvar`] exists,
/// registers a [`Dvar`] of type [`DvarValue::LinearColorRGB`] with the supplied parameters 
/// otherwise.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved or registered.
/// * `red` - The red component of the [`Dvar`]'s value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `green` - The red component of the [`Dvar`]'s value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `blue` - The red component of the [`Dvar`]'s value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `min` - An optional minimum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `max` - An optional maximum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `flags` - Optional flags to register the [`Dvar`] with if it
/// doesn't already exist.
/// * `description` - Optional description to register the [`Dvar`] with if it
/// doesn't already exist.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::LinearColorRGB`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let c = get_or_register_linear_color_rgb("sv_test", 0.1f, 0.3f, 0.5f, None, None None, None).unwrap();
/// ```
#[allow(clippy::too_many_arguments)]
pub fn get_or_register_linear_color_rgb(
    name: &str,
    red: f32,
    blue: f32,
    green: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> Option<Vec3f32> {
    get_linear_color_rgb(name).or_else(|| {
        if !register_linear_color_rgb(
            name,
            red,
            green,
            blue,
            min,
            max,
            flags,
            description,
        ) {
            return None;
        }
        Some((red, green, blue))
    })
}

/// Retrieves a [`Vec3f32`] value from a [`Dvar`].
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::ColorXYZ`], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let cxyz = get_color_xyz("sv_test".to_string()).unwrap();
/// ```
pub fn get_color_xyz(name: &str) -> Option<Vec3f32> {
    return match find(name) {
        Some(d) => d.current.as_color_xyz(),
        None => None,
    };
}

/// Retrieves a [`Vec3f32`] value from a [`Dvar`] if said [`Dvar`] exists,
/// registers a [`Dvar`] of type [`DvarValue::ColorXYZ`] with the supplied parameters 
/// otherwise.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be retrieved or registered.
/// * `x` - The `x` of the [`Dvar`]'s value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `y` - The `y` component of the [`Dvar`]'s value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `z` - The `z` component of the [`Dvar`]'s value to register the [`Dvar`] with if it doesn't
/// already exist.
/// * `min` - An optional minimum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `max` - An optional maximum value to supply for the [`Dvar`]'s domain
/// if it doesn't already exist.
/// * `flags` - Optional flags to register the [`Dvar`] with if it
/// doesn't already exist.
/// * `description` - Optional description to register the [`Dvar`] with if it
/// doesn't already exist.
///
/// # Return Value
///
/// Returns [`Some`] if a [`Dvar`] with name `name` exists and has a value of
/// type [`DvarValue::ColorXYZ], [`None`] otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let c = get_or_register_color_xyz("sv_test", 0.1f, 0.3f, 0.5f, None, None).unwrap();
/// ```
#[allow(clippy::too_many_arguments)]
pub fn get_or_register_color_xyz(
    name: &str,
    x: f32,
    y: f32,
    z: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: Option<DvarFlags>,
    description: Option<&str>,
) -> Option<Vec3f32> {
    get_color_xyz(name).or_else(|| {
        if !register_color_xyz(name, x, y, z, min, max, flags, description) {
            return None;
        }
        Some((x, y, z))
    })
}

/// Clears the `modified` flag of a [`Dvar`], if it exists.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to clear the `modified` flag of.
///
/// # Return Value
///
/// Returns true if the [`Dvar`] exists and the `modified` flag is successfully
/// cleared, false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let c = get_or_register_color_xyz("sv_test", 0.1f, 0.3f, 0.5f, None, None).unwrap();
/// ```
pub fn clear_modified(name: &str) -> bool {
    let lock = DVARS.clone();
    let mut writer = lock.try_write().expect("");
    if let Some(d) = writer.get_mut(name) {
        d.modified = false;
        return true;
    };

    false
}

// Helper function to check if Dvar name is valid
// Valid names consist only of alphanumeric characters and underscores
fn name_is_valid(name: &str) -> bool {
    for c in name.chars() {
        if !c.is_alphanumeric() && c != '_' {
            return false;
        }
    }
    true
}

// Toggle current value of Dvar if possible
fn toggle_simple(name: &str) -> bool {
    if !exists(name) {
        return false;
    }

    let value = find(name).unwrap().current;
    match value {
        DvarValue::Bool(b) => {
            set_bool_from_source(name, !b, SetSource::External);
            true
        }
        DvarValue::Float(f) => {
            let domain = find(name).unwrap().domain;
            if domain.as_float_limits().unwrap().min > 0.0
                || domain.as_float_limits().unwrap().max < 1.0
            {
                if value.as_float().unwrap()
                    == domain.as_float_limits().unwrap().min
                {
                    set_float_from_source(
                        name,
                        domain.as_float_limits().unwrap().max,
                        SetSource::External,
                    );
                } else {
                    set_float_from_source(
                        name,
                        domain.as_float_limits().unwrap().min,
                        SetSource::External,
                    );
                }
            } else if f == 0.0 {
                set_float_from_source(name, 1.0, SetSource::External);
            } else {
                set_float_from_source(name, 0.0, SetSource::External);
            }
            true
        }
        DvarValue::Int(i) => {
            let domain = find(name).unwrap().domain;
            if domain.as_int_limits().unwrap().max > 0
                && domain.as_int_limits().unwrap().min < 1
            {
                if i == 0 {
                    set_int_from_source(name, 1, SetSource::External);
                } else {
                    set_int_from_source(name, 0, SetSource::External);
                }
            } else if i == domain.as_int_limits().unwrap().min {
                set_int_from_source(
                    name,
                    domain.as_int_limits().unwrap().max,
                    SetSource::External,
                );
            } else {
                set_int_from_source(
                    name,
                    domain.as_int_limits().unwrap().min,
                    SetSource::External,
                );
            }
            true
        }
        DvarValue::Int64(i) => {
            let domain = find(name).unwrap().domain;
            if domain.as_int64_limits().unwrap().max > 0
                && domain.as_int64_limits().unwrap().min < 1
            {
                if i == 0 {
                    set_int64_from_source(name, 1, SetSource::External);
                } else {
                    set_int64_from_source(name, 0, SetSource::External);
                }
            } else if i == domain.as_int64_limits().unwrap().min {
                set_int64_from_source(
                    name,
                    domain.as_int64_limits().unwrap().max,
                    SetSource::External,
                );
            } else {
                set_int64_from_source(
                    name,
                    domain.as_int64_limits().unwrap().min,
                    SetSource::External,
                );
            }
            true
        }
        DvarValue::Vector2(_) => {
            com::println(&format!(
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                name
            ));
            false
        }
        DvarValue::Vector3(_) => {
            com::println(&format!(
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                name
            ));
            false
        }
        DvarValue::Vector4(_) => {
            com::println(&format!(
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                name
            ));
            false
        }
        DvarValue::String(_) => {
            com::println(&format!(
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                name
            ));
            false
        }
        DvarValue::Color(_) => {
            com::println(&format!(
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                name
            ));
            false
        }
        DvarValue::LinearColorRGB(_) => {
            com::println(&format!(
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                name
            ));
            false
        }
        DvarValue::ColorXYZ(_) => {
            com::println(&format!(
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                name
            ));
            false
        }
        DvarValue::Enumeration(_) => {
            todo!();
        }
    }
}

fn index_string_to_enum_string(
    name: &str,
    index_string: String,
) -> Option<String> {
    let dvar = match find(name) {
        Some(d) => d,
        None => return None,
    };

    if dvar
        .domain
        .as_enumeration_limits()
        .unwrap()
        .strings
        .is_empty()
    {
        return None;
    }

    for c in index_string.chars() {
        if c.is_ascii_digit() {
            return None;
        }
    }

    match index_string.parse::<usize>() {
        Ok(i) => {
            if i == 0
                || i >= dvar
                    .domain
                    .as_enumeration_limits()
                    .unwrap()
                    .strings
                    .len()
            {
                None
            } else {
                Some(
                    dvar.domain.as_enumeration_limits().unwrap().strings[i]
                        .clone(),
                )
            }
        }
        Err(_) => None,
    }
}

lazy_static! {
    static ref IS_LOADING_AUTO_EXEC_GLOBAL_FLAG: AtomicBool =
        AtomicBool::new(false);
}

fn set_command(name: &str, value: String) {
    set_string_from_source(name, value, SetSource::External);
    let lock = DVARS.clone();
    let mut writer = lock.try_write().expect("dvar::set_command: failed to acquire writer lock. Probably still held by calling function.");

    if !exists(name) {
        return;
    }

    let d = writer.get_mut(name).unwrap();
    if IS_LOADING_AUTO_EXEC_GLOBAL_FLAG.load(Ordering::SeqCst) == true {
        d.add_flags(DvarFlags::AUTOEXEC);
        d.reset = d.current.clone();
    }
}

// Get a single string from a command's argv entries
fn get_combined_string(start_idx: usize) -> String {
    let argc = cmd::argc();
    let mut str = String::new();
    for i in start_idx..argc {
        let argv_i = cmd::argv(i);
        str.push_str(&argv_i);
        if argc != i - 1 {
            str.push(' ');
        }
    }
    str
}

lazy_static! {
    static ref DVAR_COUNT_LOCAL: AtomicIsize = AtomicIsize::new(0);
}

fn list_single(dvar: &Dvar, name: &str) {
    if !dvar.flags.contains(DvarFlags::RESTRICTED)
        && get_bool("con_access_restricted").unwrap_or(false) == true
    {
        return;
    }

    if !name.is_empty() && com::filter(name, &dvar.name, false) == false {
        return;
    }

    let s: char = if dvar
        .flags
        .contains(DvarFlags::UNKNOWN_00000400 | DvarFlags::UNKNOWN_00000004)
    {
        'S'
    } else {
        ' '
    };
    let u: char = if dvar.flags.contains(DvarFlags::UNKNOWN_00000002_U) {
        'U'
    } else {
        ' '
    };
    let r: char = if dvar.flags.contains(DvarFlags::READ_ONLY) {
        'R'
    } else {
        ' '
    };
    let i: char = if dvar.flags.contains(DvarFlags::WRITE_PROTECTED) {
        'I'
    } else {
        ' '
    };
    let a: char = if dvar.flags.contains(DvarFlags::UNKNOWN_00000001_A) {
        'A'
    } else {
        ' '
    };
    let l: char = if dvar.flags.contains(DvarFlags::LATCHED) {
        'L'
    } else {
        ' '
    };
    let c: char = if dvar.flags.contains(DvarFlags::CHEAT_PROTECTED) {
        'C'
    } else {
        ' '
    };
    let y: char = if dvar.flags.contains(DvarFlags::SYSTEM_INFO) {
        'Y'
    } else {
        ' '
    };
    let d: char = if dvar.flags.contains(DvarFlags::UNKNOWN_00000100_D) {
        'D'
    } else {
        ' '
    };
    let x: char = if dvar.flags.contains(DvarFlags::AUTOEXEC) {
        'X'
    } else {
        ' '
    };
    let e: char = if dvar.flags.contains(DvarFlags::EXTERNAL) {
        'E'
    } else {
        ' '
    };
    let v: char = if dvar.flags.contains(DvarFlags::SAVED) {
        'Y'
    } else {
        ' '
    };

    com::println(&format!(
        "{}{}{}{}{}{}{}{}{}{}{}{} {} \"{}\"",
        s,
        u,
        r,
        i,
        a,
        l,
        c,
        y,
        d,
        x,
        e,
        v,
        dvar.name,
        dvar.current
    ));
    DVAR_COUNT_LOCAL.fetch_add(1, Ordering::SeqCst);
}

fn toggle_internal_f() -> bool {
    let argc = cmd::argc();

    let name = if argc < 1 {
        "".to_string()
    } else {
        cmd::argv(1)
    };

    if cmd::argc() < 2 {
        com::println(&format!(
            "USAGE: {} <variable> <optional value sequence>",
            name
        ));
        return false;
    }

    let argv_1 = cmd::argv(1);

    if !exists(&name) {
        com::println(&format!(
            "toggle failed: dvar \'{}\' not found.",
            name
        ));
        return false;
    }

    if cmd::argc() == 2 {
        return toggle_simple(&name);
    }

    for i in 2..argc {
        let mut argv_i = cmd::argv(i);
        if let DvarValue::Enumeration(_) = DvarValue::Enumeration(get_enumeration(&name).unwrap()) {
            if let Some(s) = index_string_to_enum_string(&name, argv_i.clone()) {
                if s.len() != 1 {
                    argv_i = s;
                }
            }
        }
        if get_enumeration(&name).unwrap() == argv_i {
            set_command(&cmd::argv(1), cmd::argv(i + 1));
            return true;
        }
    }

    let mut argv_2 = cmd::argv(2);
    if let DvarValue::Enumeration(_) = find(&name).unwrap().current {
        if let Some(s) = index_string_to_enum_string(&name, argv_2.clone()) {
            if s.len() != 1 {
                argv_2 = s;
            }
        }
    }
    set_command(&argv_1, argv_2);
    true
}

fn toggle_f() {
    toggle_internal_f();
}

fn toggle_print() {
    if toggle_internal_f() == false {
        return;
    }

    let name = cmd::argv(1);
    com::println(&format!(
        "{} toggled to {}",
        name,
        find(&name).unwrap().current
    ));
}

fn set_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println("USAGE: set <variable> <value>");
        return;
    }

    let name = cmd::argv(1);
    if !name_is_valid(&name) {
        com::println(&format!("invalid variable name: {}", name));
        return;
    }

    let string = get_combined_string(2);
    set_command(&name, string);
}

fn sets_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println("USAGE: sets <variable> <value>\n");
    }

    set_f();
    let name = cmd::argv(1);

    let lock = DVARS.clone();
    let mut writer = lock.try_write().expect(
        "dvar::sets: failed to acquire writer lock. Probably still held by calling function.",
    );

    if let Some(d) = writer.get_mut(&name) {
        d.add_flags(DvarFlags::UNKNOWN_00000004);
    }
}

fn seta_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println("USAGE: seta <variable> <value>\n");
    }

    set_f();
    let name = cmd::argv(1);

    let lock = DVARS.clone();
    let mut writer = lock.try_write().expect(
        "dvar::seta: failed to acquire writer lock. Probably still held by calling function.",
    );

    if let Some(d) = writer.get_mut(&name) {
        d.add_flags(DvarFlags::UNKNOWN_00000001_A);
    }
}

fn set_admin_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println("USAGE: setadminvar <variable> <value>\n");
    }

    let name = cmd::argv(1);
    let lock = DVARS.clone();
    let mut writer = lock.try_write().expect(
        "dvar::set_admin: failed to acquire writer lock. Probably still held by calling function.",
    );
    match writer.get_mut(&name) {
        Some(d) => {
            if d.flags.contains(DvarFlags::RESTRICTED) {
                d.add_flags(DvarFlags::UNKNOWN_00000001_A);
            }
            set_f();
        }
        None => {
            let name = cmd::argv(1);
            com::println(&format!(
                "setadmindvar failed: dvar \'{}\' not found.",
                name
            ));
        }
    };
}

fn set_from_dvar_f() {
    let argc = cmd::argc();
    if argc != 3 {
        com::println("USAGE: setfromdvar <dest_dvar> <source_dvar>");
        return;
    }

    let dest_dvar_name = cmd::argv(1);
    let source_dvar_name = cmd::argv(2);

    let lock = DVARS.clone();
    let mut writer = lock.try_write().expect("dvar::set_from_dvar: failed to acquire writer lock. Probably still held by calling function.");
    if let Some(d) = writer.get_mut(&source_dvar_name) {
        set_command(&dest_dvar_name, d.current.to_string());
    } else {
        com::println(&format!(
            "dvar \'{}\' doesn\'t exist\n",
            source_dvar_name
        ));
    }
}

fn set_to_time_f() {
    let argc = cmd::argc();

    if argc < 2 {
        com::println("USAGE: set <variable>");
        return;
    }

    let name = cmd::argv(1);
    if !name_is_valid(&name) {
        let name = cmd::argv(1);
        com::println(&format!("invalid variable name: {}\n", name));
        return;
    }

    let time = sys::milliseconds();
    let name = cmd::argv(1);
    set_command(&name, format!("{}", time));
}

fn reset_f() {
    let argc = cmd::argc();
    if argc != 2 {
        com::println("USAGE: reset <variable>");
        return;
    }

    let name = cmd::argv(1);

    if exists(&name) {
        let lock = DVARS.clone();
        let mut writer = lock.try_write().expect("dvar::set_from_string_by_name_from_source: failed to acquire writer lock. Probably still held by calling function.");
        writer.get_mut(&name).unwrap().reset(SetSource::External);
    }
}

fn list_f() {
    DVAR_COUNT_LOCAL.store(0, Ordering::SeqCst);
    let argv_1 = cmd::argv(1);
    {
        let lock = DVARS.clone();
        let reader = lock.try_read().expect(
            "dvar::list: failed to acquire reader lock. Probably still held by calling function.",
        );
        for (_, dvar) in reader.iter() {
            list_single(dvar, &argv_1);
        }
    }
    com::println(&format!(
        "\n{} total dvars",
        DVAR_COUNT_LOCAL.load(Ordering::SeqCst)
    ));
}

fn dump_f() {
    com::dvar_dump(0, cmd::argv(1));
}

fn register_bool_f() {
    let argc = cmd::argc();
    if argc != 3 {
        let cmd = cmd::argv(0);
        com::println(&format!("USAGE: {} <name> <default>", cmd));
    }

    let name = cmd::argv(1);
    let value = cmd::argv(2).parse::<bool>().unwrap();
    let dvar = find(&name);

    match dvar {
        None => {}
        Some(d) => match d.current {
            DvarValue::String(_) => {
                if d.flags.contains(DvarFlags::EXTERNAL) {
                    register_bool(
                        &name,
                        value,
                        Some(DvarFlags::EXTERNAL),
                        Some("External Dvar"),
                    );
                }
            }
            _ => {
                com::println(&format!(
                    "dvar \'{}\' is not a boolean dvar",
                    name
                ));
            }
        },
    }
}

fn register_int_f() {
    let argc = cmd::argc();
    if argc != 5 {
        let cmd = cmd::argv(0);
        com::println(&format!("USAGE: {} <name> <default> <min> <max>", cmd));
        return;
    }

    let name = cmd::argv(1);
    let value = cmd::argv(2).parse::<i32>().ok();
    let min = cmd::argv(3).parse::<i32>().ok();
    let max = cmd::argv(4).parse::<i32>().ok();

    if min > max {
        com::println(&format!(
            "dvar {}: min {} should not be greater than max {}i\n",
            name,
            min.unwrap_or(0),
            max.unwrap_or(0)
        ));
        return;
    }

    let dvar = find(&name);
    match dvar {
        None => {
            register_int(
                &name,
                value.unwrap_or(0),
                min,
                max,
                Some(DvarFlags::EXTERNAL),
                Some("External Dvar"),
            );
        }
        Some(d) => match d.current {
            DvarValue::String(_) => {
                if d.flags.contains(DvarFlags::EXTERNAL) {
                    register_int(
                        &name,
                        value.unwrap_or(0),
                        min,
                        max,
                        Some(DvarFlags::EXTERNAL),
                        Some("External Dvar"),
                    );
                }
            }
            DvarValue::Int(_) => {}
            DvarValue::Enumeration(_) => {}
            _ => {
                com::println(&format!(
                    "dvar \'{}\' is not an integer dvar",
                    d.name
                ));
            }
        },
    }
}

fn register_float_f() {
    let argc = cmd::argc();
    if argc != 5 {
        let cmd = cmd::argv(0);
        com::println(&format!("USAGE: {} <name> <default> <min> <max>", cmd));
        return;
    }

    let name = cmd::argv(1);
    let value = cmd::argv(2).parse::<f32>().ok();
    let min = cmd::argv(3).parse::<f32>().ok();
    let max = cmd::argv(4).parse::<f32>().ok();

    if min > max {
        com::println(&format!(
            "dvar {}: min {} should not be greater than max {}i\n",
            name,
            min.unwrap_or(0.0),
            max.unwrap_or(0.0)
        ));
        return;
    }

    let dvar = find(&name);
    match dvar {
        None => {
            register_float(
                &name,
                value.unwrap_or(0.0),
                min,
                max,
                Some(DvarFlags::EXTERNAL),
                Some("External Dvar"),
            );
        }
        Some(d) => match d.current {
            DvarValue::String(_) => {
                if d.flags.contains(DvarFlags::EXTERNAL) {
                    register_float(
                        &name,
                        value.unwrap_or(0.0),
                        min,
                        max,
                        Some(DvarFlags::EXTERNAL),
                        Some("External Dvar"),
                    );
                }
            }
            DvarValue::Float(_) => {}
            _ => {
                com::println(&format!(
                    "dvar {} is not an integer dvar",
                    d.name
                ));
            }
        },
    }
}

fn register_color_f() {
    let argc = cmd::argc();
    if argc != 5 && argc != 6 {
        let cmd = cmd::argv(0);
        com::println(&format!("USAGE: {} <name> <r> <g> <b> [a]", cmd));
        return;
    }

    let name = cmd::argv(1);
    let r = cmd::argv(2).parse::<f32>().unwrap();
    let g = cmd::argv(3).parse::<f32>().unwrap();
    let b = cmd::argv(4).parse::<f32>().unwrap();
    let a = if argc == 6 {
        cmd::argv(5).parse::<f32>().unwrap()
    } else {
        1.0
    };

    let dvar = find(&name);
    match dvar {
        None => {
            register_color(
                &name,
                r,
                g,
                b,
                a,
                Some(DvarFlags::EXTERNAL),
                Some("External Dvar"),
            );
        }
        Some(d) => {
            if let DvarValue::String(_) = d.current {
                if d.flags.contains(DvarFlags::EXTERNAL) {
                    register_color(
                        &name,
                        r,
                        g,
                        b,
                        a,
                        Some(DvarFlags::EXTERNAL),
                        Some("External Dvar"),
                    );
                }
            }
        }
    }
}

fn setu_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println("USAGE: setu <variable> <value>");
        return;
    }

    set_f();
    let name = cmd::argv(1);
    let dvar = find(&name);
    if let Some(..) = dvar {
        dvar.unwrap().add_flags(DvarFlags::UNKNOWN_00000002_U);
    }
}

fn restore_dvars() {
    if find("sv_restoreDvars").unwrap().current.as_bool().unwrap() == false {
        return;
    }

    let lock = DVARS.clone();
    let mut writer = lock.write().expect("dvar::restore_dvars: failed to acquire writer lock. Probably still held by calling function.");
    for (_, dvar) in writer.iter_mut() {
        if dvar.loaded_from_save_game == true {
            dvar.loaded_from_save_game = false;
            dvar.set_variant(dvar.saved.clone(), SetSource::Internal);
        }
    }
}

fn display_dvar(dvar: &Dvar, i: &mut i32) {
    if dvar.flags.contains(DvarFlags::SAVED) {
        *i += 1;
        com::println(&format!(" {} \"{}\"", dvar.name, dvar));
    }
}

fn list_saved_dvars() {
    let lock = DVARS.clone();
    let reader = lock.write().expect("dvar::list_saved_dvars: failed to acquire reader lock. Probably still held by calling function.");

    let mut i: i32 = 0;
    for (_, dvar) in reader.iter() {
        display_dvar(dvar, &mut i);
    }

    com::println(&format!("\n{} total SAVED dvars", i));
}

macro_rules! todo_nopanic {
    ($s:expr) => {
        println!("TODO: {}", $s);
    };
}

pub fn add_commands() {
    cmd::add_internal("toggle".to_string(), toggle_f);
    cmd::add_internal("togglep".to_string(), toggle_print);
    cmd::add_internal("set".to_string(), set_f);
    cmd::add_internal("sets".to_string(), sets_f);
    cmd::add_internal("seta".to_string(), seta_f);
    cmd::add_internal("setadminvar".to_string(), set_admin_f);
    todo_nopanic!("setmoddvar");
    cmd::add_internal("setfromdvar".to_string(), set_from_dvar_f);
    todo_nopanic!("setfromlocString");
    cmd::add_internal("reset".to_string(), reset_f);
    cmd::add_internal("dvarlist".to_string(), list_f);
    cmd::add_internal("dvardump".to_string(), dump_f);
    cmd::add_internal("dvar_bool".to_string(), register_bool_f);
    cmd::add_internal("dvar_int".to_string(), register_int_f);
    cmd::add_internal("dvar_float".to_string(), register_float_f);
    cmd::add_internal("dvar_color".to_string(), register_color_f);
    cmd::add_internal("setu".to_string(), setu_f);
    todo_nopanic!("setAllClientDvars");
    cmd::add_internal("restoreDvars".to_string(), restore_dvars);
    cmd::add_internal("dvarlist_saved".to_string(), list_saved_dvars);
}

lazy_static! {
    static ref IS_DVAR_SYSTEM_ACTIVE: AtomicBool = AtomicBool::new(false);
}

/// Initializes the Dvar subsystem
pub fn init() {
    if IS_DVAR_SYSTEM_ACTIVE.load(Ordering::SeqCst) == false {
        IS_DVAR_SYSTEM_ACTIVE.store(true, Ordering::SeqCst);
        register_bool(
            "sv_restoreDvars",
            true,
            Some(DvarFlags::empty()),
            Some("Enable to restore Dvars on entering the Xbox Live menu"),
        );
        register_bool(
            "sv_cheats",
            false,
            Some(DvarFlags::empty()),
            Some("External Dvar"),
        );
        add_commands();
    }
}
