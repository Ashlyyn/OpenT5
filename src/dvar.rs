#![allow(dead_code)]
#![allow(private_in_public)]
#![deny(missing_docs)]
#![allow(
    clippy::missing_trait_methods,
    clippy::unreadable_literal,
    clippy::pub_use
)]

mod builder;
mod limits;
mod value;

pub mod global_fns;
pub use global_fns::*;

mod cmds;

/// This file contains all of code related to the Dvar subsystem, including
/// the [`Dvar`] itself, functions to get, set, and create Dvars, and [`CmdFunctions`]
/// related to the [`Dvar`] subsystem. There is *a lot* of repeated code here
/// due to the different types of value a [`Dvar`] can hold
use crate::*;
use bitflags::bitflags;
use core::fmt::Display;
use core::hash::Hash;
use lazy_static::lazy_static;
use std::sync::RwLock;
extern crate alloc;
use alloc::sync::Arc;

use self::limits::DvarLimits;
use self::value::DvarValue;

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
        const ARCHIVE               = 0x00000001;
        /// Flag with unknown purpose. Never(?) manually set by a function,
        /// but some Dvars are registered with it set by default
        const USER_INFO             = 0x00000002;
        /// Flag with unknown purpose. Only used once or twice
        const SERVER_INFO           = 0x00000004;
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
        /// Flag to allow Dvar to be accessed when ConAccess is restricted
        const CON_ACCESS            = 0x00010000;
    }
}

lazy_static! {
    static ref MODIFIED_FLAGS: Arc<RwLock<DvarFlags>> =
        Arc::new(RwLock::new(DvarFlags::empty()));
}

// Finally, the Dvar itself
#[allow(clippy::partial_pub_fields)]
#[derive(Clone, Debug)]
struct Dvar {
    /// Name of Dvar
    pub name: String,
    /// Description of Dvar (optional)
    pub description: String,
    /// Flags for Dvar (write-protected, cheat-protected, read-only, etc.)
    pub flags: DvarFlags,
    /// Flag to check if the Dvar has been modified
    /// Not included in the actual flags for some reason
    pub modified: bool,
    /// Flag to check if Dvar was loaded from a saved game
    /// Also not in the actual flags for some reason
    pub loaded_from_save_game: bool,
    /// Domain of Dvar
    pub domain: DvarLimits,
    /// Current value of Dvar
    pub current: DvarValue,
    /// Latched value of Dvar
    /// (seems to be the value it defaults to on restart)
    latched: DvarValue,
    /// Reset value of Dvar
    /// (seems to be used when a Dvar is manually reset,
    ///  or when the current value flags the Dvar as a cheat
    ///  and cheats are subsequently disabled)
    reset: DvarValue,
    /// Saved value of Dvar
    /// (value used on loading a save game?)
    saved: DvarValue,
}

impl Display for Dvar {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
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
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl Dvar {
    // Clamp a supplied value to the supplied domain if possible
    #[allow(clippy::match_same_arms, clippy::too_many_lines)]
    fn clamp_value_to_domain(
        value: &mut DvarValue,
        domain: &DvarLimits,
    ) -> DvarValue {
        let clamped_value = match *value {
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
                if i < min {
                    DvarValue::Int(min)
                } else if i > max {
                    DvarValue::Int(max)
                } else {
                    DvarValue::Int(i)
                }
            }
            DvarValue::String(_) => value.clone(),
            DvarValue::Enumeration(_) => value.clone(),
            DvarValue::Color(_) => value.clone(),
            DvarValue::Int64(i) => {
                let min: i64 = domain.as_int64_limits().unwrap().min;
                let max: i64 = domain.as_int64_limits().unwrap().max;
                if i < min {
                    DvarValue::Int64(min)
                } else if i > max {
                    DvarValue::Int64(max)
                } else {
                    DvarValue::Int64(i)
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
        };
        *value = clamped_value.clone();
        clamped_value
    }

    fn clamp_current_value_to_domain(&mut self) {
        Self::clamp_value_to_domain(&mut self.current, &self.domain);
    }

    fn clamp_latched_value_to_domain(&mut self) {
        Self::clamp_value_to_domain(&mut self.latched, &self.domain);
    }

    fn clamp_reset_value_to_domain(&mut self) {
        Self::clamp_value_to_domain(&mut self.reset, &self.domain);
    }

    fn clamp_all_values_to_domain(&mut self) {
        self.clamp_current_value_to_domain();
        self.clamp_latched_value_to_domain();
        self.clamp_reset_value_to_domain();
    }

    fn has_latched_value(&self) -> bool {
        self.current != self.latched
    }

    #[allow(clippy::needless_pass_by_value)]
    fn can_change_value(
        &self,
        value: &DvarValue,
        set_source: SetSource,
    ) -> bool {
        if *value == self.reset {
            return true;
        }

        if self.flags.contains(DvarFlags::READ_ONLY) {
            com::println!(1.into(), "{} is read only.", self.name);
            return false;
        }

        if self.flags.contains(DvarFlags::WRITE_PROTECTED) {
            com::println!(
                1.into(),
                "{} is write protected protected.", 
                self.name,
            );
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
                com::println!(
                    1.into(),
                    "{} is cheat protected.", 
                    self.name,
                );
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
            if self.can_change_value(&value, source) == false {
                return;
            }
            if self.flags.contains(DvarFlags::LATCHED) {
                self.latched = value.clone();
                if self.current != self.latched {
                    com::println!(
                        16.into(),
                        "{} will be changed upon restarting.",
                        self.name,
                    );
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
    #[allow(clippy::match_same_arms)]
    fn value_is_in_domain(domain: &DvarLimits, value: DvarValue) -> bool {
        match value {
            DvarValue::Bool(_) => true,
            DvarValue::Float(f) => {
                f >= domain.as_float_limits().unwrap().min
                    && f <= domain.as_float_limits().unwrap().max
            }
            DvarValue::Vector2(v) => {
                v.0 >= domain.as_vector2_limits().unwrap().min
                    && v.0 <= domain.as_vector2_limits().unwrap().max
                    && v.1 >= domain.as_vector2_limits().unwrap().min
                    && v.1 <= domain.as_vector2_limits().unwrap().max
            }
            DvarValue::Vector3(v) => {
                v.0 >= domain.as_vector3_limits().unwrap().min
                    && v.0 <= domain.as_vector3_limits().unwrap().max
                    && v.1 >= domain.as_vector3_limits().unwrap().min
                    && v.1 <= domain.as_vector3_limits().unwrap().max
                    && v.2 >= domain.as_vector3_limits().unwrap().min
                    && v.2 <= domain.as_vector3_limits().unwrap().max
            }
            DvarValue::Vector4(v) => {
                v.0 >= domain.as_vector4_limits().unwrap().min
                    && v.0 <= domain.as_vector4_limits().unwrap().max
                    && v.1 >= domain.as_vector4_limits().unwrap().min
                    && v.1 <= domain.as_vector4_limits().unwrap().max
                    && v.2 >= domain.as_vector4_limits().unwrap().min
                    && v.2 <= domain.as_vector4_limits().unwrap().max
                    && v.3 >= domain.as_vector4_limits().unwrap().min
                    && v.3 <= domain.as_vector4_limits().unwrap().max
            }
            DvarValue::Int(i) => {
                i >= domain.as_int_limits().unwrap().min
                    && i <= domain.as_int_limits().unwrap().max
            }
            DvarValue::String(_) => true,
            DvarValue::Enumeration(v) => domain
                .as_enumeration_limits()
                .unwrap()
                .strings
                .iter()
                .any(|s| *s == v),
            DvarValue::Color(_) => true,
            DvarValue::Int64(i) => {
                i >= domain.as_int64_limits().unwrap().min
                    && i <= domain.as_int64_limits().unwrap().max
            }
            DvarValue::LinearColorRGB(v) => {
                v.0 >= domain.as_linear_color_rgb_limits().unwrap().min
                    && v.0 <= domain.as_linear_color_rgb_limits().unwrap().max
                    && v.1 >= domain.as_linear_color_rgb_limits().unwrap().min
                    && v.1 <= domain.as_linear_color_rgb_limits().unwrap().max
                    && v.2 >= domain.as_linear_color_rgb_limits().unwrap().min
                    && v.2 <= domain.as_linear_color_rgb_limits().unwrap().max
            }
            DvarValue::ColorXYZ(v) => {
                v.0 >= domain.as_color_xyz_limits().unwrap().min
                    && v.0 <= domain.as_color_xyz_limits().unwrap().max
                    && v.1 >= domain.as_color_xyz_limits().unwrap().min
                    && v.1 <= domain.as_color_xyz_limits().unwrap().max
                    && v.2 >= domain.as_color_xyz_limits().unwrap().min
                    && v.2 <= domain.as_color_xyz_limits().unwrap().max
            }
        }
    }

    fn set_variant(&mut self, value: DvarValue, source: SetSource) {
        if self.name.is_empty() {
            return;
        }

        if com::log_file_open() && self.current != value {
            com::println!(
                16.into(),
                "      dvar set {} {}", 
                self.name, 
                self.current,
            );
        }

        if !Self::value_is_in_domain(&self.domain, value.clone()) {
            com::println!(
                1.into(),
                "\'{}\' is not a valid value for dvar \'{}\'",
                value, 
                self.name,
            );
            com::println!(1.into(), "{}", self.domain);
            if let DvarValue::Enumeration(_) = value {
                self.set_variant(self.reset.clone(), source);
            }
            return;
        }

        if source == SetSource::External || source == SetSource::Script {
            if self.can_change_value(&value, source)
                && self.flags.contains(DvarFlags::LATCHED)
            {
                self.latched = value;
                if self.latched != self.current {
                    com::println!(
                        16.into(),
                        "{} will be changed upon restarting.",
                        self.name,
                    );
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
            let modified_flags = *MODIFIED_FLAGS.read().unwrap();
            MODIFIED_FLAGS
                .write()
                .unwrap()
                .insert(modified_flags | self.flags);
            self.current = value;
            self.modified = true;
        } else {
            self.latched = self.current.clone();
        }
    }

    pub fn make_latched_value_current(&mut self) {
        self.set_variant(self.latched.clone(), SetSource::Internal);
    }

    fn reset(&mut self, source: SetSource) {
        self.set_variant(self.reset.clone(), source);
    }
}

lazy_static! {
    static ref IS_DVAR_SYSTEM_ACTIVE: AtomicBool = AtomicBool::new(false);
}

/// Initializes the Dvar subsystem
///
/// Shouldn't ever be called more than once, but doing so
/// also shouldn't corrupt anything.
pub fn init() {
    if IS_DVAR_SYSTEM_ACTIVE.load(Ordering::SeqCst) == false {
        IS_DVAR_SYSTEM_ACTIVE.store(true, Ordering::SeqCst);
        register_bool(
            "sv_restoreDvars",
            true,
            DvarFlags::empty(),
            Some("Enable to restore Dvars on entering the Xbox Live menu"),
        )
        .unwrap();
        register_bool(
            "sv_cheats",
            false,
            DvarFlags::empty(),
            Some("External Dvar"),
        )
        .unwrap();
        self::cmds::add_commands();
    }
}
