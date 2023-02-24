use crate::{
    common::{Vec2f32, Vec3f32, Vec4f32},
    dvar::DvarFlags,
};

use super::{
    find, register_bool, register_color, register_color_xyz,
    register_enumeration, register_float, register_int, register_int64,
    register_linear_color_rgb, register_string, register_vector2,
    register_vector3, register_vector4,
};

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
/// let b = get_bool("sv_test").unwrap();
/// ```
pub fn get_bool(name: &str) -> Option<bool> {
    match find(name) {
        Some(d) => d.current.as_bool(),
        None => None,
    }
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
    flags: DvarFlags,
    description: Option<&str>,
) -> Option<bool> {
    get_bool(name).or_else(|| {
        if register_bool(name, value, flags, description).is_err() {
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
/// let f = get_float("sv_test").unwrap();
/// ```
pub fn get_float(name: &str) -> Option<f32> {
    match find(name) {
        Some(d) => d.current.as_float(),
        None => None,
    }
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
/// let f = get_or_register_float("sv_test", 32.6f, None, None, None, None)
///     .unwrap();
/// ```
pub fn get_or_register_float(
    name: &str,
    value: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Option<f32> {
    get_float(name).or_else(|| {
        if register_float(name, value, min, max, flags, description).is_err() {
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
/// let v2 = get_vector2("sv_test").unwrap();
/// ```
pub fn get_vector2(name: &str) -> Option<Vec2f32> {
    match find(name) {
        Some(d) => d.current.as_vector2(),
        None => None,
    }
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
/// let v2 = get_or_register_vector2(
///     "sv_test",
///     (32.6f, 68.9f),
///     None,
///     None,
///     None,
///     None,
/// )
/// .unwrap();
/// ```
pub fn get_or_register_vector2(
    name: &str,
    value: Vec2f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Option<Vec2f32> {
    get_vector2(name).or_else(|| {
        if register_vector2(name, value, min, max, flags, description).is_err()
        {
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
/// let v3 = get_vector3("sv_test").unwrap();
/// ```
pub fn get_vector3(name: &str) -> Option<Vec3f32> {
    match find(name) {
        Some(d) => d.current.as_vector3(),
        None => None,
    }
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
    flags: DvarFlags,
    description: Option<&str>,
) -> Option<Vec3f32> {
    get_vector3(name).or_else(|| {
        if register_vector3(name, value, min, max, flags, description).is_err()
        {
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
/// let v4 = get_vector4("sv_test").unwrap();
/// ```
pub fn get_vector4(name: &str) -> Option<Vec4f32> {
    match find(name) {
        Some(d) => d.current.as_vector4(),
        None => None,
    }
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
/// let v4 = get_or_register_vector4(
///     "sv_test",
///     (32.6f, 68.9f, 410.87f, 683.72f),
///     None,
///     None,
///     None,
///     None,
/// )
/// .unwrap();
/// ```
pub fn get_or_register_vector4(
    name: &str,
    value: Vec4f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Option<Vec4f32> {
    get_vector4(name).or_else(|| {
        if register_vector4(name, value, min, max, flags, description).is_err()
        {
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
/// let v2 = get_int("sv_test").unwrap();
/// ```
pub fn get_int(name: &str) -> Option<i32> {
    match find(name) {
        Some(d) => d.current.as_int(),
        None => None,
    }
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
    flags: DvarFlags,
    description: Option<&str>,
) -> Option<i32> {
    get_int(name).or_else(|| {
        if register_int(name, value, min, max, flags, description).is_err() {
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
/// let v2 = get_string("sv_test").unwrap();
/// ```
pub fn get_string(name: &str) -> Option<String> {
    match find(name) {
        Some(d) => d.current.as_string(),
        None => None,
    }
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
    value: &str,
    flags: DvarFlags,
    description: Option<&str>,
) -> Option<String> {
    get_string(name).or_else(|| {
        if register_string(name, value, flags, description).is_err() {
            return None;
        }
        Some(value.to_string())
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
/// let v2 = get_enumeration("sv_test").unwrap();
/// ```
pub fn get_enumeration(name: &str) -> Option<String> {
    match find(name) {
        Some(d) => d.current.as_enumeration(),
        None => None,
    }
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
/// let i = get_or_register_enumeration("sv_test", "test", None, None, None)
///     .unwrap();
/// ```
pub fn get_or_register_enumeration(
    name: &str,
    value: String,
    domain: Option<Vec<String>>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Option<String> {
    get_enumeration(name).or_else(|| {
        if register_enumeration(name, value.clone(), domain, flags, description)
            .is_err()
        {
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
/// let c = get_color("sv_test").unwrap();
/// ```
pub fn get_color(name: &str) -> Option<Vec4f32> {
    match find(name) {
        Some(d) => d.current.as_color(),
        None => None,
    }
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
/// let c =
///     get_or_register_color("sv_test", 0.1f, 0.3f, 0.5f, 0.2f, None, None)
///         .unwrap();
/// ```
pub fn get_or_register_color(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    flags: DvarFlags,
    description: Option<&str>,
) -> Option<Vec4f32> {
    get_color(name).or_else(|| {
        if register_color(name, red, green, blue, alpha, flags, description)
            .is_err()
        {
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
/// let i = get_int64("sv_test").unwrap();
/// ```
pub fn get_int64(name: &str) -> Option<i64> {
    match find(name) {
        Some(d) => d.current.as_int64(),
        None => None,
    }
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
/// let i =
///     get_or_register_int64("sv_test", 364757258672, None, None, None, None)
///         .unwrap();
/// ```
pub fn get_or_register_int64(
    name: &str,
    value: i64,
    min: Option<i64>,
    max: Option<i64>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Option<i64> {
    get_int64(name).or_else(|| {
        if register_int64(name, value, min, max, flags, description).is_err() {
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
/// let lc = get_linear_color_rgb("sv_test").unwrap();
/// ```
pub fn get_linear_color_rgb(name: &str) -> Option<Vec3f32> {
    match find(name) {
        Some(d) => d.current.as_linear_color_rgb(),
        None => None,
    }
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
    flags: DvarFlags,
    description: Option<&str>,
) -> Option<Vec3f32> {
    get_linear_color_rgb(name).or_else(|| {
        if register_linear_color_rgb(
            name,
            red,
            green,
            blue,
            min,
            max,
            flags,
            description,
        )
        .is_err()
        {
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
/// let cxyz = get_color_xyz("sv_test").unwrap();
/// ```
pub fn get_color_xyz(name: &str) -> Option<Vec3f32> {
    match find(name) {
        Some(d) => d.current.as_color_xyz(),
        None => None,
    }
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
/// let c = get_or_register_color_xyz("sv_test", 0.1f, 0.3f, 0.5f, None, None)
///     .unwrap();
/// ```
#[allow(clippy::too_many_arguments)]
pub fn get_or_register_color_xyz(
    name: &str,
    x: f32,
    y: f32,
    z: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Option<Vec3f32> {
    get_color_xyz(name).or_else(|| {
        if register_color_xyz(name, x, y, z, min, max, flags, description)
            .is_err()
        {
            return None;
        }
        Some((x, y, z))
    })
}
