use core::fmt::Display;

use crate::common::{Vec2f32, Vec3f32, Vec4f32};

// Enum to hold all possible Dvar values
#[derive(Clone, PartialEq, Debug)]
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
    #[allow(clippy::match_same_arms)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
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
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub const fn as_float(&self) -> Option<f32> {
        match self {
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub const fn as_vector2(&self) -> Option<Vec2f32> {
        match self {
            Self::Vector2(v) => Some(*v),
            _ => None,
        }
    }

    pub const fn as_vector3(&self) -> Option<Vec3f32> {
        match self {
            Self::Vector3(v) => Some(*v),
            _ => None,
        }
    }

    pub const fn as_vector4(&self) -> Option<Vec4f32> {
        match self {
            Self::Vector4(v) => Some(*v),
            _ => None,
        }
    }

    pub const fn as_int(&self) -> Option<i32> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }
    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn as_enumeration(&self) -> Option<String> {
        match self {
            Self::Enumeration(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub const fn as_color(&self) -> Option<Vec4f32> {
        match self {
            Self::Color(c) => Some(*c),
            _ => None,
        }
    }

    pub const fn as_int64(&self) -> Option<i64> {
        match self {
            Self::Int64(i) => Some(*i),
            _ => None,
        }
    }

    pub const fn as_linear_color_rgb(&self) -> Option<Vec3f32> {
        match self {
            Self::LinearColorRGB(v) => Some(*v),
            _ => None,
        }
    }

    pub const fn as_color_xyz(&self) -> Option<Vec3f32> {
        match self {
            Self::ColorXYZ(v) => Some(*v),
            _ => None,
        }
    }
}
