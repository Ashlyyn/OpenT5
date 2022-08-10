#![allow(dead_code)]

use crate::*;
use bitflags::bitflags;
use common::*;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, RwLock};
use std::vec::Vec;

// DvarLimitsXXXXX hold the domain for each possible type of Dvar
// Display is impl'ed to print said domain
// Default is impl'ed where possible, and should always resolve to the largest
// possible domain

// bool has no custom-definable domain, it'll always be 0 or 1/true or false
// DvarLimitsBool still needs to be defined for printing the domain
#[derive(Copy, Clone, Default, PartialEq)]
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
#[derive(Copy, Clone, PartialEq)]
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
#[derive(Copy, Clone, Default, PartialEq)]
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
#[derive(Clone, Default, PartialEq)]
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
#[derive(Copy, Clone, Default, PartialEq)]
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
#[derive(Copy, Clone, PartialEq)]
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
    None,
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
            Self::None => {
                write!(f, "")
            }
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
    None,
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
            Self::None => write!(f, ""),
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

    pub fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match self {
            Self::Bool(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f32> {
        match self {
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_float_mut(&mut self) -> Option<&mut f32> {
        match self {
            Self::Float(f) => Some(f),
            _ => None,
        }
    }

    pub fn as_vector2(&self) -> Option<Vec2f32> {
        match self {
            Self::Vector2(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_vector2_mut(&mut self) -> Option<&mut Vec2f32> {
        match self {
            Self::Vector2(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_vector3(&self) -> Option<Vec3f32> {
        match self {
            Self::Vector3(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_vector3_mut(&mut self) -> Option<&mut Vec3f32> {
        match self {
            Self::Vector3(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_vector4(&self) -> Option<Vec4f32> {
        match self {
            Self::Vector4(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_vector4_mut(&mut self) -> Option<&mut Vec4f32> {
        match self {
            Self::Vector4(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i32> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_int_mut(&mut self) -> Option<&mut i32> {
        match self {
            Self::Int(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::String(s) => Some(s.to_string()),
            _ => None,
        }
    }

    pub fn as_string_mut(&mut self) -> Option<String> {
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

    pub fn as_color_mut(&mut self) -> Option<&mut Vec4f32> {
        match self {
            Self::Color(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_int64(&self) -> Option<i64> {
        match self {
            Self::Int64(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_int64_mut(&mut self) -> Option<&mut i64> {
        match self {
            Self::Int64(i) => Some(i),
            _ => None,
        }
    }

    pub fn as_linear_color_rgb(&self) -> Option<Vec3f32> {
        match self {
            Self::LinearColorRGB(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_linear_color_rgb_mut(&mut self) -> Option<&mut Vec3f32> {
        match self {
            Self::LinearColorRGB(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_color_xyz(&self) -> Option<Vec3f32> {
        match self {
            Self::ColorXYZ(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_color_xyz_mut(&mut self) -> Option<&mut Vec3f32> {
        match self {
            Self::ColorXYZ(v) => Some(v),
            _ => None,
        }
    }
}

// Enum for the possible sources a Dvar may be set from
#[derive(PartialEq)]
pub enum SetSource {
    Internal,
    External,
    Script,
    Devgui,
}

// Flags for Dvars
bitflags! {
    pub struct DvarFlags: u32 {
        const UNKNOWN_00000001_A      = 0x00000001;
        const UNKNOWN_00000002_U      = 0x00000002;
        const UNKNOWN_00000004      = 0x00000004;
        const UNKNOWN_00000008_Y      = 0x00000008;
        const WRITE_PROTECTED       = 0x00000010;
        const LATCHED               = 0x00000020;
        const READ_ONLY             = 0x00000040;
        const CHEAT_PROTECTED       = 0x00000080;
        const UNKNOWN_00000100_D      = 0x00000100;
        const UNKNOWN_00000200      = 0x00000200;
        const UNKNOWN_00000400      = 0x00000400;
        const ALLOW_SET_FROM_DEVGUI = 0x00000800;
        const UNKNOWN_00001000_Y      = 0x00001000;
        const UNKNOWN_00002000      = 0x00002000;
        const UNKNOWN_00004000_E      = 0x00004000;
        const UNKNOWN_00008000_X      = 0x00008000;
        const UNKNOWN_00010000      = 0x00010000;
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
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> DvarBuilder {
        DvarBuilder::new()
    }

    // Clamp a supplied value to the supplied domain if possible
    fn clamp_value_to_domain(
        value: &mut DvarValue,
        domain: DvarLimits,
    ) -> DvarValue {
        match value {
            DvarValue::None => {
                panic!("Dvar::clamp_value_to_domain: value is None")
            }
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

    pub fn value(&self) -> &DvarValue {
        &self.current
    }

    pub fn value_mut(&mut self) -> &mut DvarValue {
        &mut self.current
    }

    pub fn clamp_current_value_to_domain(&mut self) {
        Self::clamp_value_to_domain(&mut self.current, self.domain.clone());
    }

    pub fn clamp_latched_value_to_domain(&mut self) {
        Self::clamp_value_to_domain(&mut self.latched, self.domain.clone());
    }

    pub fn clamp_reset_value_to_domain(&mut self) {
        Self::clamp_value_to_domain(&mut self.reset, self.domain.clone());
    }

    pub fn clamp_all_values_to_domain(&mut self) {
        self.clamp_current_value_to_domain();
        self.clamp_latched_value_to_domain();
        self.clamp_reset_value_to_domain();
    }

    pub fn update_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    pub fn update_description(&mut self, desc: String) -> &mut Self {
        self.description = desc;
        self
    }

    pub fn update_flags(&mut self, flags: DvarFlags) -> &mut Self {
        self.flags = flags;
        self
    }

    pub fn update_loaded_from_save_game(&mut self, b: bool) -> &mut Self {
        self.loaded_from_save_game = b;
        self
    }

    pub fn has_latched_value(&self) -> bool {
        self.current != self.latched
    }

    pub fn any_latched_values() -> bool {
        for (_, d) in DVARS.try_read().expect("dvar::any_latched_values: failed to acquire reader lock. Probably still held by calling function.").iter() {
            if d.has_latched_value() {
                return true;
            }
        }
        false
    }

    pub fn can_change_value(
        &self,
        value: DvarValue,
        set_source: SetSource,
    ) -> bool {
        if value == self.reset {
            return true;
        }

        if self.flags.contains(DvarFlags::READ_ONLY) {
            com::println(format!("{} is read only.", self.name));
            return false;
        }

        if self.flags.contains(DvarFlags::WRITE_PROTECTED) {
            com::println(format!(
                "{} is write protected protected.",
                self.name
            ));
            return false;
        }

        if self.flags.contains(DvarFlags::CHEAT_PROTECTED)
            && (find("sv_cheats".to_string())
                .unwrap()
                .value()
                .as_bool()
                .unwrap()
                == false)
        {
            true
        } else {
            if (set_source == SetSource::External)
                || (set_source == SetSource::Script)
            {
                com::println(format!("{} is cheat protected.", self.name));
            }
            false
        }
    }

    pub fn set_latched_value(&mut self, value: DvarValue) {
        self.latched = value;
    }

    // Overwrite latched value with current
    pub fn clear_latched_value(&mut self) {
        if self.has_latched_value() {
            self.set_latched_value(self.current.clone())
        }
    }

    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    pub fn clear_modified(&mut self) {
        self.modified = false;
    }

    pub fn color_red(&self) -> Option<f32> {
        self.current.as_color().map(|c| c.0)
    }

    pub fn color_green(&self) -> Option<f32> {
        self.current.as_color().map(|c| c.1)
    }

    pub fn color_blue(&self) -> Option<f32> {
        self.current.as_color().map(|c| c.2)
    }

    pub fn color_alpha(&self) -> Option<f32> {
        self.current.as_color().map(|c| c.3)
    }

    pub fn set_value(&mut self, value: DvarValue, source: SetSource) {
        if source == SetSource::External || source == SetSource::Script {
            if self.can_change_value(value.clone(), source) == false {
                return;
            }
            if self.flags.contains(DvarFlags::LATCHED) {
                self.set_latched_value(value.clone());
                if self.current != self.latched {
                    com::println(format!(
                        "{} will be changed upon restarting.",
                        self.name
                    ));
                    return;
                }
            }
        } else if source == SetSource::Devgui
            && self.flags.contains(DvarFlags::ALLOW_SET_FROM_DEVGUI)
        {
            self.set_latched_value(value);
            return;
        }

        if value != self.current {
            self.current = value;
            self.modified = true;
        } else {
            self.latched = value;
        }
    }

    pub fn set_bool(&mut self, b: bool, source: SetSource) {
        com::println(format!("      dvar set {} {}", self.name, b));
        self.set_value(DvarValue::Bool(b), source);
    }

    pub fn set_float(&mut self, f: f32, source: SetSource) {
        com::println(format!("      dvar set {} {}", self.name, f));
        self.set_value(DvarValue::Float(f), source)
    }

    pub fn set_vector2(&mut self, v: Vec2f32, source: SetSource) {
        com::println(format!("      dvar set {} {} {}", self.name, v.0, v.1));
        self.set_value(DvarValue::Vector2(v), source);
    }

    pub fn set_vector3(&mut self, v: Vec3f32, source: SetSource) {
        com::println(format!(
            "      dvar set {} {} {} {}",
            self.name, v.0, v.1, v.2
        ));
        self.set_value(DvarValue::Vector3(v), source);
    }

    pub fn set_vector4(&mut self, v: Vec4f32, source: SetSource) {
        com::println(format!(
            "      dvar set {} {} {} {} {}",
            self.name, v.0, v.1, v.2, v.3
        ));
        self.set_value(DvarValue::Vector4(v), source);
    }

    pub fn set_int(&mut self, i: i32, source: SetSource) {
        com::println(format!("      dvar set {} {}", self.name, i));
        self.set_value(DvarValue::Int(i), source);
    }

    pub fn set_string(&mut self, s: String, source: SetSource) {
        com::println(format!("      dvar set {} {}", self.name, s));
        self.set_value(DvarValue::String(s), source);
    }

    pub fn set_enumeration(&mut self, s: String, source: SetSource) {
        com::println(format!("      dvar set {} {}", self.name, s));
        self.set_value(DvarValue::Enumeration(s), source);
    }

    pub fn set_color(&mut self, v: Vec4f32, source: SetSource) {
        com::println(format!(
            "      dvar set {} {} {} {} {}",
            self.name, v.0, v.1, v.2, v.3
        ));
        self.set_value(DvarValue::Color(v), source);
    }

    pub fn set_int64(&mut self, i: i64, source: SetSource) {
        com::println(format!("      dvar set {} {}", self.name, i));
        self.set_value(DvarValue::Int64(i), source);
    }

    pub fn set_linear_color_rgb(&mut self, v: Vec3f32, source: SetSource) {
        com::println(format!(
            "      dvar set {} {} {} {}",
            self.name, v.0, v.1, v.2
        ));
        self.set_value(DvarValue::LinearColorRGB(v), source);
    }

    pub fn set_color_xyz(&mut self, v: Vec3f32, source: SetSource) {
        com::println(format!(
            "      dvar set {} {} {} {}",
            self.name, v.0, v.1, v.2
        ));
        self.set_value(DvarValue::ColorXYZ(v), source);
    }

    pub fn add_flags(&mut self, flags: DvarFlags) {
        self.flags |= flags;
    }

    pub fn clear_flags(&mut self, flags: DvarFlags) {
        self.flags &= !flags;
    }

    // Helper function to check if supplied value is within supplied domain
    pub fn value_is_in_domain(domain: DvarLimits, value: DvarValue) -> bool {
        match value {
            DvarValue::None => {
                panic!("Dvar::clamp_value_to_domain: value is None")
            }
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

    pub fn set_variant(&mut self, value: DvarValue, source: SetSource) {
        if self.name.is_empty() {
            return;
        }

        if com::log_file_open() && self.current != value {
            com::println(format!(
                "      dvar set {} {}",
                self.name, self.current
            ));
        }

        if !Self::value_is_in_domain(self.domain.clone(), value.clone()) {
            com::println(format!(
                "\'{}\' is not a valid value for dvar \'{}\'",
                value, self.name
            ));
            com::println(format!("{}", self.domain));
            if let DvarValue::Enumeration(_) = value {
                self.set_variant(self.reset.to_owned(), source);
            }
            return;
        }

        if source == SetSource::External || source == SetSource::Script {
            if self.can_change_value(value.clone(), source)
                && self.flags.contains(DvarFlags::LATCHED)
            {
                self.set_latched_value(value);
                if self.latched != self.current {
                    com::println(format!(
                        "{} will be changed upon restarting.",
                        self.name
                    ))
                }
            }
            return;
        } else if source == SetSource::Devgui
            && self.flags.contains(DvarFlags::ALLOW_SET_FROM_DEVGUI)
        {
            self.set_latched_value(value);
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
            self.set_latched_value(self.current.clone());
        }
    }

    pub fn set_domain(&mut self, domain: DvarLimits) {
        self.domain = domain;
    }

    pub fn make_latched_value_current(&mut self) {
        self.set_variant(self.latched.clone(), SetSource::Internal);
    }

    pub fn update_reset_value(&mut self, reset: DvarValue) {
        self.reset = reset;
    }

    pub fn reset(&mut self, source: SetSource) {
        self.set_variant(self.reset.clone(), source);
    }
}

// Helper impl to make constructing Dvars easier
pub struct DvarBuilder {
    dvar: Dvar,
}

impl DvarBuilder {
    pub fn new() -> Self {
        DvarBuilder {
            dvar: Dvar {
                name: "".to_string(),
                description: "".to_string(),
                flags: DvarFlags::empty(),
                modified: false,
                loaded_from_save_game: false,
                domain: DvarLimits::None,
                current: DvarValue::None,
                latched: DvarValue::None,
                reset: DvarValue::None,
                saved: DvarValue::None,
            },
        }
    }

    pub fn name(mut self, name: String) -> Self {
        self.dvar.update_name(name);
        self
    }

    pub fn description(mut self, desc: String) -> Self {
        self.dvar.update_description(desc);
        self
    }

    pub fn flags(mut self, flags: DvarFlags) -> Self {
        self.dvar.update_flags(flags);
        self
    }

    pub fn loaded_from_save_game(mut self, b: bool) -> Self {
        self.dvar.update_loaded_from_save_game(b);
        self
    }

    pub fn value(mut self, value: DvarValue) -> Self {
        self.dvar.set_variant(value, SetSource::Internal);
        self
    }

    pub fn domain(mut self, domain: DvarLimits) -> Self {
        self.dvar.set_domain(domain);
        self
    }

    pub fn build(self) -> Dvar {
        self.dvar
    }
}

const DVAR_COUNT_MAX: usize = 4096;

lazy_static! {
    pub static ref DVARS: Arc<RwLock<HashMap<String, Box<Dvar>>>> =
        Arc::new(RwLock::new(HashMap::from(
            [
                ("sv_restoreDvars".to_string(),
                Box::new(Dvar::new()
                    .name("sv_restoreDvars".to_string())
                    .value(DvarValue::Bool(true))
                    .description(
                        "Enable to restore Dvars on entering the Xbox Live menu"
                        .to_string()
                    )
                    .build())),
                ("sv_cheats".to_string(),
                Box::new(Dvar::new()
                    .name("sv_cheats".to_string())
                    .value(DvarValue::Bool(false))
                    .description("External Dvar".to_string())
                    .flags(DvarFlags::WRITE_PROTECTED | DvarFlags::UNKNOWN_00000008_Y)
                    .build()))
            ])));
}

pub fn find(name: String) -> Option<Dvar> {
    let reader_lock = DVARS.clone();
    let reader = reader_lock.try_read().expect(
        "dvar::find: failed to acquire reader lock. Probably still held by calling function.",
    );

    if !reader.contains_key(&name) {
        return None;
    }

    return Some(*reader.get(&name).unwrap().clone());
}

// Register new Dvar
fn register_new(
    name: String,
    flags: DvarFlags,
    value: DvarValue,
    domain: DvarLimits,
    description: String,
) {
    let b: bool;
    let other_name: String;
    {
        let writer_lock = DVARS.clone();
        let mut writer = writer_lock.try_write().expect("dvar::register_new: failed to acquire writer lock. Probably still held by calling function.");

        if writer.len() + 1 > DVAR_COUNT_MAX {
            com::errorln(
                com::ErrorParm::FATAL,
                format!(
                    "Can\'t create dvar \'{}\': {} dvars already exist",
                    name, DVAR_COUNT_MAX
                ),
            );
        }

        let value = Dvar::new()
            .name(name.clone())
            .flags(flags)
            .value(value)
            .domain(domain)
            .description(description)
            .build();
        b = writer.insert(name.clone(), Box::new(value)).is_some();
        other_name = writer.get(&name).unwrap().name.clone();
    }

    if b {
        com::errorln(com::ErrorParm::FATAL, format!("dvar name hash collision between \'{}\' and \'{}\' Please change one of these names to remove the hash collision", name, other_name));
    }
}

// Reregister (update) existing Dvar
#[allow(unused_variables)]
fn reregister(
    dvar: &mut Dvar,
    name: String,
    flags: DvarFlags,
    value: DvarValue,
    domain: DvarLimits,
    description: Option<String>,
) {
    dvar.add_flags(flags);
    if let Some(..) = description {
        dvar.description = description.unwrap();
    }

    if dvar.flags.contains(DvarFlags::CHEAT_PROTECTED)
        && find("sv_cheats".to_string())
            .unwrap()
            .value()
            .as_bool()
            .unwrap()
            == false
    {
        dvar.set_variant(dvar.reset.clone(), SetSource::Internal);
        dvar.set_latched_value(dvar.reset.clone());
    }

    if dvar.flags.contains(DvarFlags::LATCHED) {
        dvar.make_latched_value_current();
    }
}

// Register new Dvar if no Dvar with name currently exists
// Reregister otherwise
fn register_variant(
    name: String,
    flags: DvarFlags,
    value: DvarValue,
    domain: DvarLimits,
    description: String,
) {
    {
        let writer_lock = DVARS.clone();
        let mut writer = writer_lock.try_write().expect("dvar::register_variant: failed to acquire writer lock. Probably still held by calling function.");
        if writer.contains_key(&name) {
            let dvar = writer.get_mut(&name).unwrap();
            reregister(dvar, name, flags, value, domain, Some(description));
            return;
        }
    }

    register_new(name, flags, value, domain, description);
}

// Helper functions to register Dvars of a specific type
pub fn register_bool(
    name: String,
    value: bool,
    flags: DvarFlags,
    description: String,
) {
    register_variant(
        name,
        flags,
        DvarValue::Bool(value),
        DvarLimits::Bool(DvarLimitsBool::new()),
        description,
    );
}

pub fn register_float(
    name: String,
    value: f32,
    min: f32,
    max: f32,
    flags: DvarFlags,
    description: String,
) {
    register_variant(
        name,
        flags,
        DvarValue::Float(value),
        DvarLimits::Float(DvarLimitsFloat::new(min, max)),
        description,
    );
}

pub fn register_vector2(
    name: String,
    value: Vec2f32,
    min: f32,
    max: f32,
    flags: DvarFlags,
    description: String,
) {
    register_variant(
        name,
        flags,
        DvarValue::Vector2(value),
        DvarLimits::Vector2(DvarLimitsVector2::new(min, max)),
        description,
    );
}

pub fn register_vector3(
    name: String,
    value: Vec3f32,
    min: f32,
    max: f32,
    flags: DvarFlags,
    description: String,
) {
    register_variant(
        name,
        flags,
        DvarValue::Vector3(value),
        DvarLimits::Vector3(DvarLimitsVector3::new(min, max)),
        description,
    );
}

pub fn register_vector4(
    name: String,
    value: Vec4f32,
    min: f32,
    max: f32,
    flags: DvarFlags,
    description: String,
) {
    register_variant(
        name,
        flags,
        DvarValue::Vector4(value),
        DvarLimits::Vector4(DvarLimitsVector4::new(min, max)),
        description,
    );
}

pub fn register_int(
    name: String,
    value: i32,
    min: i32,
    max: i32,
    flags: DvarFlags,
    description: String,
) {
    register_variant(
        name,
        flags,
        DvarValue::Int(value),
        DvarLimits::Int(DvarLimitsInt::new(min, max)),
        description,
    );
}

pub fn register_string(
    name: String,
    value: String,
    flags: DvarFlags,
    description: String,
) {
    register_variant(
        name,
        flags,
        DvarValue::String(value),
        DvarLimits::String(DvarLimitsString::new()),
        description,
    );
}

pub fn register_enum(
    name: String,
    value: String,
    domain: Vec<String>,
    flags: DvarFlags,
    description: String,
) {
    register_variant(
        name,
        flags,
        DvarValue::Enumeration(value),
        DvarLimits::Enumeration(DvarLimitsEnumeration::new(&domain)),
        description,
    );
}

pub fn register_color(
    name: String,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    flags: DvarFlags,
    description: String,
) {
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

    register_variant(
        name,
        flags,
        DvarValue::Color((r, g, b, a)),
        DvarLimits::Color(DvarLimitsColor::new()),
        description,
    );
}

pub fn register_int64(
    name: String,
    value: i64,
    min: i64,
    max: i64,
    flags: DvarFlags,
    description: String,
) {
    register_variant(
        name,
        flags,
        DvarValue::Int64(value),
        DvarLimits::Int64(DvarLimitsInt64::new(min, max)),
        description,
    );
}

pub fn register_linear_color_rgb(
    name: String,
    value: Vec3f32,
    min: f32,
    max: f32,
    flags: DvarFlags,
    description: String,
) {
    register_variant(
        name,
        flags,
        DvarValue::LinearColorRGB(value),
        DvarLimits::LinearColorRGB(DvarLimitsLinearColorRGB::new(min, max)),
        description,
    );
}

pub fn register_color_xyz(
    name: String,
    value: Vec3f32,
    min: f32,
    max: f32,
    flags: DvarFlags,
    description: String,
) {
    register_variant(
        name,
        flags,
        DvarValue::ColorXYZ(value),
        DvarLimits::ColorXYZ(DvarLimitsColorXYZ::new(min, max)),
        description,
    );
}

fn set_from_string_by_name_from_source(
    name: String,
    value: String,
    source: SetSource,
    flags: DvarFlags,
) {
    let writer_lock = DVARS.clone();
    let mut writer = writer_lock.try_write().expect("dvar::set_from_string_by_name_from_source: failed to acquire writer lock. Probably still held by calling function.");
    if writer.contains_key(&name) {
        let dvar = writer.get_mut(&name).unwrap();
        dvar.set_string(value, source);
    } else {
        register_string(
            name,
            value,
            flags | DvarFlags::UNKNOWN_00004000_E,
            "External Dvar".to_string(),
        );
    }
}

fn get_bool(name: String) -> Option<bool> {
    return match find(name) {
        Some(d) => d.value().as_bool(),
        None => None,
    };
}

// Helper function to check if Dvar name is valid
// Valid names consist only of alphanumeric characters and underscores
fn name_is_valid(name: String) -> bool {
    for c in name.chars() {
        if c.is_alphanumeric() || c == '_' {
            return false;
        }
    }
    true
}

// Toggle current value of Dvar if possible
fn toggle_simple(dvar: &mut Dvar) -> bool {
    let value = dvar.value().clone();
    match value {
        DvarValue::None => panic!("toggle_simple: dvar.current == None"),
        DvarValue::Bool(b) => {
            dvar.set_bool(!b, SetSource::External);
            true
        }
        DvarValue::Float(f) => {
            if dvar.domain.as_float_limits().unwrap().min > 0.0
                || dvar.domain.as_float_limits().unwrap().max < 1.0
            {
                if dvar.value().as_float().unwrap()
                    == dvar.domain.as_float_limits().unwrap().min
                {
                    dvar.set_float(
                        dvar.domain.as_float_limits().unwrap().max,
                        SetSource::External,
                    );
                } else {
                    dvar.set_float(
                        dvar.domain.as_float_limits().unwrap().min,
                        SetSource::External,
                    );
                }
            } else if f == 0.0 {
                dvar.set_float(1.0, SetSource::External);
            } else {
                dvar.set_float(0.0, SetSource::External);
            }
            true
        }
        DvarValue::Int(i) => {
            if dvar.domain.as_int_limits().unwrap().max > 0
                && dvar.domain.as_int_limits().unwrap().min < 1
            {
                if i == 0 {
                    dvar.set_int(1, SetSource::External);
                } else {
                    dvar.set_int(0, SetSource::External);
                }
            } else if i == dvar.domain.as_int_limits().unwrap().min {
                dvar.set_int(
                    dvar.domain.as_int_limits().unwrap().max,
                    SetSource::External,
                );
            } else {
                dvar.set_int(
                    dvar.domain.as_int_limits().unwrap().min,
                    SetSource::External,
                );
            }
            true
        }
        DvarValue::Int64(i) => {
            if dvar.domain.as_int64_limits().unwrap().max > 0
                && dvar.domain.as_int64_limits().unwrap().min < 1
            {
                if i == 0 {
                    dvar.set_int64(1, SetSource::External);
                } else {
                    dvar.set_int64(0, SetSource::External);
                }
            } else if i == dvar.domain.as_int64_limits().unwrap().min {
                dvar.set_int64(
                    dvar.domain.as_int64_limits().unwrap().max,
                    SetSource::External,
                );
            } else {
                dvar.set_int64(
                    dvar.domain.as_int64_limits().unwrap().min,
                    SetSource::External,
                );
            }
            true
        }
        DvarValue::Vector2(_) => {
            com::println(format!(
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                dvar.name
            ));
            false
        }
        DvarValue::Vector3(_) => {
            com::println(format!(
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                dvar.name
            ));
            false
        }
        DvarValue::Vector4(_) => {
            com::println(format!(
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                dvar.name
            ));
            false
        }
        DvarValue::String(_) => {
            com::println(format!(
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                dvar.name
            ));
            false
        }
        DvarValue::Color(_) => {
            com::println(format!(
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                dvar.name
            ));
            false
        }
        DvarValue::LinearColorRGB(_) => {
            com::println(format!(
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                dvar.name
            ));
            false
        }
        DvarValue::ColorXYZ(_) => {
            com::println(format!(
                "\'toggle\' with no arguments makes no sense for dvar \'{}\'",
                dvar.name
            ));
            false
        }
        DvarValue::Enumeration(_) => {
            todo!();
        }
    }
}

fn index_string_to_enum_string(
    dvar: &Dvar,
    index_string: String,
) -> Option<String> {
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

fn set_command(name: String, value: String) {
    set_from_string_by_name_from_source(
        name.clone(),
        value,
        SetSource::External,
        DvarFlags::empty(),
    );
    let writer_lock = DVARS.clone();
    let mut writer = writer_lock.try_write().expect("dvar::set_command: failed to acquire writer lock. Probably still held by calling function.");

    if !writer.contains_key(&name) {
        return;
    }

    let d = writer.get_mut(&name).unwrap();
    if IS_LOADING_AUTO_EXEC_GLOBAL_FLAG.load(Ordering::SeqCst) == true {
        d.add_flags(DvarFlags::UNKNOWN_00008000_X);
        d.update_reset_value(d.value().clone());
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

fn list_single(dvar: &Dvar, name: String) {
    if !dvar.flags.contains(DvarFlags::UNKNOWN_00010000)
        && get_bool("con_access_restricted".to_string()).unwrap_or(false)
            == true
    {
        return;
    }

    if !name.is_empty() && com::filter(name, dvar.name.clone(), false) == false
    {
        return;
    }

    let s: char = if dvar.flags.contains(
        DvarFlags::UNKNOWN_00000400 | DvarFlags::UNKNOWN_00000004,
    ) {
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
    let y: char = if dvar.flags.contains(DvarFlags::UNKNOWN_00000008_Y) {
        'Y'
    } else {
        ' '
    };
    let d: char = if dvar.flags.contains(DvarFlags::UNKNOWN_00000100_D) {
        'D'
    } else {
        ' '
    };
    let x: char = if dvar.flags.contains(DvarFlags::UNKNOWN_00008000_X) {
        'X'
    } else {
        ' '
    };
    let e: char = if dvar.flags.contains(DvarFlags::UNKNOWN_00004000_E) {
        'E'
    } else {
        ' '
    };
    let v: char = if dvar.flags.contains(DvarFlags::UNKNOWN_00001000_Y) {
        'Y'
    } else {
        ' '
    };

    com::println(format!(
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
        dvar.value()
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
        com::println(format!(
            "USAGE: {} <variable> <optional value sequence>",
            name
        ));
        return false;
    }

    let argv_1 = cmd::argv(1);

    let writer_lock = DVARS.clone();
    let mut writer = writer_lock.try_write().expect("dvar::toggle_internal: failed to acquire writer lock. Probably still held by calling function.");
    let dvar = writer
        .get_mut(&name)
        .ok_or_else(|| {
            com::println(format!(
                "toggle failed: dvar \'{}\' not found.",
                name
            ));
            false
        })
        .unwrap();

    if cmd::argc() == 2 {
        return toggle_simple(dvar);
    }

    for i in 2..argc {
        let mut argv_i = cmd::argv(i);
        if let DvarValue::Enumeration(_) = dvar.value() {
            if let Some(s) = index_string_to_enum_string(dvar, argv_i.clone()) {
                if s.len() != 1 {
                    argv_i = s;
                }
            }
        }
        if dvar.value().to_string() == argv_i.clone() {
            set_command(cmd::argv(1), cmd::argv(i + 1));
            return true;
        }
    }

    let mut argv_2 = cmd::argv(2);
    if let DvarValue::Enumeration(_) = dvar.value() {
        if let Some(s) = index_string_to_enum_string(dvar, argv_2.clone()) {
            if s.len() != 1 {
                argv_2 = s;
            }
        }
    }
    set_command(argv_1, argv_2);
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
    com::println(format!(
        "{} toggled to {}",
        name.clone(),
        find(name).unwrap().value()
    ));
}

fn set_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println("USAGE: set <variable> <value>".to_string());
        return;
    }

    let name = cmd::argv(1);
    if !name_is_valid(name.clone()) {
        com::println(format!("invalid variable name: {}", name));
        return;
    }

    let string = get_combined_string(2);
    set_command(name, string);
}

fn sets_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println("USAGE: sets <variable> <value>\n".to_string());
    }

    set_f();
    let name = cmd::argv(1);

    let writer_lock = DVARS.clone();
    let mut writer = writer_lock.try_write().expect(
        "dvar::sets: failed to acquire writer lock. Probably still held by calling function.",
    );

    if let Some(d) = writer.get_mut(&name) {
        d.add_flags(DvarFlags::UNKNOWN_00000004);
    }
}

fn seta_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println("USAGE: seta <variable> <value>\n".to_string());
    }

    set_f();
    let name = cmd::argv(1);

    let writer_lock = DVARS.clone();
    let mut writer = writer_lock.try_write().expect(
        "dvar::seta: failed to acquire writer lock. Probably still held by calling function.",
    );

    if let Some(d) = writer.get_mut(&name) {
        d.add_flags(DvarFlags::UNKNOWN_00000001_A);
    }
}

fn set_admin_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println("USAGE: setadminvar <variable> <value>\n".to_string());
    }

    let name = cmd::argv(1);
    let writer_lock = DVARS.clone();
    let mut writer = writer_lock.try_write().expect(
        "dvar::set_admin: failed to acquire writer lock. Probably still held by calling function.",
    );
    match writer.get_mut(&name) {
        Some(d) => {
            if d.flags.contains(DvarFlags::UNKNOWN_00010000) {
                d.add_flags(DvarFlags::UNKNOWN_00000001_A);
            }
            set_f();
        }
        None => {
            let name = cmd::argv(1);
            com::println(format!(
                "setadmindvar failed: dvar \'{}\' not found.",
                name
            ));
        }
    };
}

fn set_from_dvar_f() {
    let argc = cmd::argc();
    if argc != 3 {
        com::println(
            "USAGE: setfromdvar <dest_dvar> <source_dvar>".to_string(),
        );
        return;
    }

    let dest_dvar_name = cmd::argv(1);
    let source_dvar_name = cmd::argv(2);

    let writer_lock = DVARS.clone();
    let mut writer = writer_lock.try_write().expect("dvar::set_from_dvar: failed to acquire writer lock. Probably still held by calling function.");
    if let Some(d) = writer.get_mut(&source_dvar_name) {
        set_command(dest_dvar_name, d.value().to_string());
    } else {
        com::println(format!("dvar \'{}\' doesn\'t exist\n", source_dvar_name));
    }
}

fn set_to_time_f() {
    let argc = cmd::argc();

    if argc < 2 {
        com::println("USAGE: set <variable>".to_owned());
        return;
    }

    let name = cmd::argv(1);
    if !name_is_valid(name) {
        let name = cmd::argv(1);
        com::println(format!("invalid variable name: {}\n", name));
        return;
    }

    let time = sys::milliseconds();
    let name = cmd::argv(1);
    set_command(name, format!("{}", time));
}

fn reset_f() {
    let argc = cmd::argc();
    if argc != 2 {
        com::println("USAGE: reset <variable>".to_string());
        return;
    }

    let name = cmd::argv(1);

    if find(name.clone()).is_some() {
        let writer_lock = DVARS.clone();
        let mut writer = writer_lock.try_write().expect("dvar::set_from_string_by_name_from_source: failed to acquire writer lock. Probably still held by calling function.");
        writer.get_mut(&name).unwrap().reset(SetSource::External);
    }
}

fn list_f() {
    DVAR_COUNT_LOCAL.store(0, Ordering::SeqCst);
    let argv_1 = cmd::argv(1);
    {
        let reader_lock = DVARS.clone();
        let reader = reader_lock.try_read().expect(
            "dvar::list: failed to acquire reader lock. Probably still held by calling function.",
        );
        for (_, dvar) in reader.iter() {
            list_single(dvar, argv_1.clone());
        }
    }
    com::println(format!(
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
        com::println(format!("USAGE: {} <name> <default>", cmd));
    }

    let name = cmd::argv(1);
    let value = cmd::argv(2).parse::<bool>().unwrap();
    let dvar = find(name.clone());

    match dvar {
        None => {}
        Some(d) => match d.value() {
            DvarValue::String(_) => {
                if d.flags.contains(DvarFlags::UNKNOWN_00004000_E) {
                    register_bool(
                        name,
                        value,
                        DvarFlags::UNKNOWN_00004000_E,
                        "External Dvar".to_string(),
                    );
                }
            }
            _ => {
                com::println(format!(
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
        com::println(format!("USAGE: {} <name> <default> <min> <max>", cmd));
        return;
    }

    let name = cmd::argv(1);
    let default = cmd::argv(2).parse::<i32>().unwrap();
    let min = cmd::argv(3).parse::<i32>().unwrap();
    let max = cmd::argv(4).parse::<i32>().unwrap();

    if min > max {
        com::println(format!(
            "dvar {}: min {} should not be greater than max {}i\n",
            name, min, max
        ));
        return;
    }

    let dvar = find(name.clone());
    match dvar {
        None => {
            register_int(
                name,
                default,
                min,
                max,
                DvarFlags::UNKNOWN_00004000_E,
                "External Dvar".to_string(),
            );
        }
        Some(d) => match d.value() {
            DvarValue::String(_) => {
                if d.flags.contains(DvarFlags::UNKNOWN_00004000_E) {
                    register_int(
                        name,
                        default,
                        min,
                        max,
                        DvarFlags::UNKNOWN_00004000_E,
                        "External Dvar".to_string(),
                    );
                }
            }
            DvarValue::Int(_) => {}
            DvarValue::Enumeration(_) => {}
            _ => {
                com::println(format!(
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
        com::println(format!("USAGE: {} <name> <default> <min> <max>", cmd));
        return;
    }

    let name = cmd::argv(1);
    let default = cmd::argv(2).parse::<f32>().unwrap();
    let min = cmd::argv(3).parse::<f32>().unwrap();
    let max = cmd::argv(4).parse::<f32>().unwrap();

    if min > max {
        com::println(format!(
            "dvar {}: min {} should not be greater than max {}i\n",
            name, min, max
        ));
        return;
    }

    let dvar = find(name.clone());
    match dvar {
        None => {
            register_float(
                name,
                default,
                min,
                max,
                DvarFlags::UNKNOWN_00004000_E,
                "External Dvar".to_string(),
            );
        }
        Some(d) => match d.value() {
            DvarValue::String(_) => {
                if d.flags.contains(DvarFlags::UNKNOWN_00004000_E) {
                    register_float(
                        name,
                        default,
                        min,
                        max,
                        DvarFlags::UNKNOWN_00004000_E,
                        "External Dvar".to_string(),
                    );
                }
            }
            DvarValue::Float(_) => {}
            _ => {
                com::println(format!("dvar {} is not an integer dvar", d.name));
            }
        },
    }
}

fn register_color_f() {
    let argc = cmd::argc();
    if argc != 5 && argc != 6 {
        let cmd = cmd::argv(0);
        com::println(format!("USAGE: {} <name> <r> <g> <b> [a]", cmd));
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

    let dvar = find(name.clone());
    match dvar {
        None => {
            register_color(
                name,
                r,
                g,
                b,
                a,
                DvarFlags::UNKNOWN_00004000_E,
                "External Dvar".to_string(),
            );
        }
        Some(d) => {
            if let DvarValue::String(_) = d.value() {
                if d.flags.contains(DvarFlags::UNKNOWN_00004000_E) {
                    register_color(
                        name,
                        r,
                        g,
                        b,
                        a,
                        DvarFlags::UNKNOWN_00004000_E,
                        "External Dvar".to_string(),
                    );
                }
            }
        }
    }
}

fn setu_f() {
    let argc = cmd::argc();
    if argc < 3 {
        com::println("USAGE: setu <variable> <value>".to_string());
        return;
    }

    set_f();
    let name = cmd::argv(1);
    let dvar = find(name);
    if let Some(..) = dvar {
        dvar.unwrap().add_flags(DvarFlags::UNKNOWN_00000002_U);
    }
}

fn restore_dvars() {
    if find("sv_restoreDvars".to_string())
        .unwrap()
        .value()
        .as_bool()
        .unwrap()
        == false
    {
        return;
    }

    let writer_lock = DVARS.clone();
    let mut writer = writer_lock.write().expect("dvar::restore_dvars: failed to acquire writer lock. Probably still held by calling function.");
    for (_, dvar) in writer.iter_mut() {
        if dvar.loaded_from_save_game == true {
            dvar.loaded_from_save_game = false;
            dvar.set_variant(dvar.saved.clone(), SetSource::Internal);
        }
    }
}

fn display_dvar(dvar: &Dvar, i: &mut i32) {
    if dvar.flags.contains(DvarFlags::UNKNOWN_00001000_Y) {
        *i += 1;
        com::println(format!(" {} \"{}\"", dvar.name, dvar));
    }
}

fn list_saved_dvars() {
    let reader_lock = DVARS.clone();
    let reader = reader_lock.write().expect("dvar::list_saved_dvars: failed to acquire reader lock. Probably still held by calling function.");

    let mut i: i32 = 0;
    for (_, dvar) in reader.iter() {
        display_dvar(dvar, &mut i);
    }

    com::println(format!("\n{} total SAVED dvars", i));
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

pub fn init() {
    IS_DVAR_SYSTEM_ACTIVE.store(true, Ordering::SeqCst);
    add_commands();
}
