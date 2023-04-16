use crate::{
    common::{Vec2f32, Vec3f32, Vec4f32},
    dvar::{limits::DvarLimits, value::DvarValue, DvarFlags, SetSource},
};

use super::{
    exists, find, get_enumeration, register_bool, register_color,
    register_color_xyz, register_enumeration, register_float, register_int,
    register_int64, register_linear_color_rgb, register_string,
    register_vector2, register_vector3, register_vector4, DVARS,
};

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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = DvarValue::Float(372.1);
///     let source = SetSource::Internal;
///     set_variant_from_source(name, DvarValue::Float(value), source);
/// }
/// ```
pub fn set_variant_from_source(
    name: &str,
    value: DvarValue,
    source: SetSource,
) -> Result<(), ()> {
    match find(name) {
        Some(_) => {
            DVARS
                .write()
                .unwrap()
                .get_mut(name)
                .unwrap()
                .set_variant(value, source);
            Ok(())
        }
        None => Err(()),
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
/// let name = "sv_test";
/// let dvar = find(name);
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
) -> Result<(), ()> {
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = false;
///     set_bool_internal(name, value);
/// }
/// ```
pub fn set_bool_internal(name: &str, value: bool) -> Result<(), ()> {
    set_bool_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::External`]. Does nothing if a [`Dvar`] with `name`
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = false;
///     set_bool(name, value);
/// }
/// ```
pub fn set_bool(name: &str, value: bool) -> Result<(), ()> {
    set_bool_from_source(name, value, SetSource::External)
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
/// let name = "sv_test";
/// let dvar = find(name);
/// let value = false;
/// let flags = DvarFlags::External;
/// let description = "External Dvar";
/// set_or_register_bool(name, value, flags, Some(description));
/// ```
pub fn set_or_register_bool(
    name: &str,
    value: bool,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        set_bool_internal(name, value)
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
/// let name = "sv_test";
/// let dvar = find(name);
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
) -> Result<(), ()> {
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = 420.49f;
///     set_float_internal(name, value);
/// }
/// ```
pub fn set_float_internal(name: &str, value: f32) -> Result<(), ()> {
    set_float_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::External`]. Does nothing if a [`Dvar`] with `name`
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = 420.49f;
///     set_float(name, value);
/// }
/// ```
pub fn set_float(name: &str, value: f32) -> Result<(), ()> {
    set_float_from_source(name, value, SetSource::External)
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
/// let name = "sv_test";
/// let dvar = find(name);
/// let value = 420.69f32;
/// let min = 400.0f32;
/// let max = 800.0f32;
/// let flags = DvarFlags::External;
/// let description = "External Dvar";
/// set_or_register_float(
///     name,
///     value,
///     Some(min),
///     Some(max),
///     flags,
///     Some(description),
/// );
/// ```
pub fn set_or_register_float(
    name: &str,
    value: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        set_float_internal(name, value)
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
/// let name = "sv_test";
/// let dvar = find(name);
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
) -> Result<(), ()> {
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (420.69f, 694.20f32);
///     set_vector2_internal(name, value);
/// }
/// ```
pub fn set_vector2_internal(name: &str, value: Vec2f32) -> Result<(), ()> {
    set_vector2_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::External`]. Does nothing if a [`Dvar`] with `name`
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (420.69f, 694.20f32);
///     set_vector2(name, value);
/// }
/// ```
pub fn set_vector2(name: &str, value: Vec2f32) -> Result<(), ()> {
    set_vector2_from_source(name, value, SetSource::External)
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
/// let name = "sv_test";
/// let dvar = find(name);
/// let value = (420.69f32, 694.20f32);
/// let min = 400.0f32;
/// let max = 800.0f32;
/// let flags = DvarFlags::External;
/// let description = "External Dvar";
/// set_or_register_vector2(
///     name,
///     value,
///     Some(min),
///     Some(max),
///     flags,
///     Some(description),
/// );
/// ```
pub fn set_or_register_vector2(
    name: &str,
    value: Vec2f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        set_vector2_internal(name, value)
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
/// let name = "sv_test";
/// let dvar = find(name);
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
) -> Result<(), ()> {
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (420.69f, 694.20f32, -165.0f32);
///     set_vector3_internal(name, value);
/// }
/// ```
pub fn set_vector3_internal(name: &str, value: Vec3f32) -> Result<(), ()> {
    set_vector3_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::External`]. Does nothing if a [`Dvar`] with `name`
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (420.69f, 694.20f32, -165.0f32);
///     set_vector3(name, value);
/// }
/// ```
pub fn set_vector3(name: &str, value: Vec3f32) -> Result<(), ()> {
    set_vector3_from_source(name, value, SetSource::External)
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
/// let name = "sv_test";
/// let dvar = find(name);
/// let value = (420.69f32, 694.20f32, -165.0f32);
/// let min = -200.0f32;
/// let max = 800.0f32;
/// let flags = DvarFlags::External;
/// let description = "External Dvar";
/// set_or_register_vector3(
///     name,
///     value,
///     Some(min),
///     Some(max),
///     flags,
///     Some(description),
/// );
/// ```
pub fn set_or_register_vector3(
    name: &str,
    value: Vec3f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
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
/// let name = "sv_test";
/// let dvar = find(name);
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
) -> Result<(), ()> {
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (420.69f, 694.20f32, -165.0f32, 196.23f32);
///     set_vector4_internal(name, value);
/// }
/// ```
pub fn set_vector4_internal(name: &str, value: Vec4f32) -> Result<(), ()> {
    set_vector4_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::External`]. Does nothing if a [`Dvar`] with `name`
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (420.69f, 694.20f32, -165.0f32, 196.23f32);
///     set_vector4(name, value);
/// }
/// ```
pub fn set_vector4(name: &str, value: Vec4f32) -> Result<(), ()> {
    set_vector4_from_source(name, value, SetSource::External)
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
/// let name = "sv_test";
/// let dvar = find(name);
/// let value = (420.69f32, 694.20f32, -165.0f32, 196.23f32);
/// let min = -400.0f32;
/// let max = 800.0f32;
/// let flags = DvarFlags::External;
/// let description = "External Dvar";
/// set_or_register_vector4(
///     name,
///     value,
///     Some(min),
///     Some(max),
///     flags,
///     Some(description),
/// );
/// ```
pub fn set_or_register_vector4(
    name: &str,
    value: Vec4f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = 420i32;
///     let source = SetSource::Internal;
///     set_int_from_source(name, value, source);
/// }
/// ```
pub fn set_int_from_source(
    name: &str,
    value: i32,
    source: SetSource,
) -> Result<(), ()> {
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = 420;
///     set_int(name, value);
/// }
/// ```
pub fn set_int_internal(name: &str, value: i32) -> Result<(), ()> {
    set_int_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::External`]. Does nothing if a [`Dvar`] with `name`
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = 420;
///     set_int(name, value);
/// }
/// ```
pub fn set_int(name: &str, value: i32) -> Result<(), ()> {
    set_int_from_source(name, value, SetSource::External)
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
/// let name = "sv_test";
/// let dvar = find(name);
/// let value = 420i32;
/// let min = 200i32;
/// let max = 400i32;
/// let flags = DvarFlags::External;
/// let description = "External Dvar");
/// set_or_register_int(name, value, Some(min), Some(max), flags, Some(description));
/// ```
pub fn set_or_register_int(
    name: &str,
    value: i32,
    min: Option<i32>,
    max: Option<i32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        set_int_internal(name, value)
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = "test";
///     let source = SetSource::Internal;
///     set_string_from_source(name, value, source);
/// }
/// ```
pub fn set_string_from_source(
    name: &str,
    value: &str,
    source: SetSource,
) -> Result<(), ()> {
    set_variant_from_source(name, DvarValue::String(value.to_owned()), source)
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = "test";
///     set_string(name, value);
/// }
/// ```
pub fn set_string_internal(name: &str, value: &str) -> Result<(), ()> {
    set_string_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::External`]. Does nothing if a [`Dvar`] with `name`
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = "test";
///     set_string(name, value);
/// }
/// ```
pub fn set_string(name: &str, value: &str) -> Result<(), ()> {
    set_string_from_source(name, value, SetSource::External)
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
/// let name = "sv_test";
/// let dvar = find(name);
/// let value = "test";
/// let flags = DvarFlags::External;
/// let description = "External Dvar";
/// set_or_register_string(name, value, flags, Some(description));
/// ```
pub fn set_or_register_string(
    name: &str,
    value: &str,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        set_string_internal(name, value)
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = "test";
///     let source = SetSource::Internal;
///     set_enumeration_from_source(name, value, source);
/// }
/// ```
pub fn set_enumeration_from_source(
    name: &str,
    value: &str,
    source: SetSource,
) -> Result<(), ()> {
    match find(name) {
        Some(d) => match d.current {
            DvarValue::Enumeration(_) => {
                if !d
                    .domain
                    .as_enumeration_limits()
                    .unwrap()
                    .strings
                    .iter()
                    .any(|s| *s == value)
                {
                    return Err(());
                }
            }
            _ => return Err(()),
        },
        None => return Err(()),
    };
    set_variant_from_source(
        name,
        DvarValue::Enumeration(value.to_owned()),
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = "test";
///     set_enumeration(name, value);
/// }
/// ```
pub fn set_enumeration_internal(name: &str, value: &str) -> Result<(), ()> {
    set_enumeration_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::External`]. Does nothing if a [`Dvar`] with `name`
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = "test";
///     set_enumeration(name, value);
/// }
/// ```
pub fn set_enumeration(name: &str, value: &str) -> Result<(), ()> {
    set_enumeration_from_source(name, value, SetSource::External)
}

/// Advances an existing [`Dvar`] of type [`DvarValue::Enumeration`]
/// to the next value of its domain.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be advanced.
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     set_enumeration_next(name);
/// }
/// ```
pub fn set_enumeration_next(name: &str) -> Result<(), ()> {
    let Some(current) = get_enumeration(name) else { return Err(()) };

    match find(name) {
        Some(d) => match d.current {
            DvarValue::Enumeration(_) => {
                let domain = d.domain.as_enumeration_limits().unwrap();
                let mut iter = domain.strings.iter();
                loop {
                    match iter.next() {
                        Some(v) => {
                            if *v == current {
                                let next = match iter.next() {
                                    Some(n) => n.clone(),
                                    None => return Err(()),
                                };
                                return set_enumeration(name, &next);
                            }
                        }
                        None => match domain.strings.iter().next() {
                            Some(v) => return set_enumeration(name, v),
                            None => return Err(()),
                        },
                    }
                }
            }
            _ => Err(()),
        },
        None => Err(()),
    }
}

/// Advances an existing [`Dvar`] of type [`DvarValue::Enumeration`]
/// to the previous value of its domain.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be reversed.
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     set_enumeration_prev(name);
/// }
/// ```
pub fn set_enumeration_prev(name: &str) -> Result<(), ()> {
    match find(name) {
        Some(d) => match d.current {
            DvarValue::Enumeration(s) => {
                let DvarLimits::Enumeration(domain) = d.domain else { return Err(()) };

                let Some((i, _)) = domain.strings
                    .iter()
                    .enumerate()
                    .find(|(_, u)| **u == *s
                ) else { return Err(()) };

                if i == 0 {
                    return Err(());
                }

                let value =
                    domain.strings.iter().nth(i - 1).unwrap_or_else(|| {
                        domain.strings.iter().next().unwrap()
                    });
                if set_enumeration_internal(name, value).is_err() {
                    return Err(());
                };
                Ok(())
            }
            _ => Err(()),
        },
        None => Err(()),
    }
}

pub fn add_to_enumeration_domain(
    name: &str,
    domain_str: &str,
) -> Result<(), ()> {
    match find(name) {
        Some(d) => match d.current {
            DvarValue::Enumeration(_) => {
                match &mut DVARS.write().unwrap().get_mut(name).unwrap().domain
                {
                    DvarLimits::Enumeration(l) => {
                        l.strings.insert(domain_str.to_owned());
                        Ok(())
                    }
                    _ => Err(()),
                }
            }
            _ => Err(()),
        },
        None => Err(()),
    }
}

pub fn remove_from_enumeration_domain(
    name: &str,
    domain_str: &str,
) -> Result<(), ()> {
    match find(name) {
        Some(d) => match d.current {
            DvarValue::Enumeration(_) => {
                match &mut DVARS.write().unwrap().get_mut(name).unwrap().domain
                {
                    DvarLimits::Enumeration(l) => {
                        l.strings.remove(&domain_str.to_owned());
                        Ok(())
                    }
                    _ => Err(()),
                }
            }
            _ => Err(()),
        },
        None => Err(()),
    }
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
/// let name = "sv_test";
/// let dvar = find(name);
/// let value = "test";
/// let domain = vec!["test".to_owned(), "test2".to_owned()];
/// let flags = DvarFlags::External;
/// let description = "External Dvar";
/// set_or_register_enumeration(
///     name,
///     value,
///     Some(domain),
///     flags,
///     Some(description),
/// );
/// ```
pub fn set_or_register_enumeration(
    name: &str,
    value: String,
    domain: Option<Vec<String>>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        set_enumeration_internal(name, &value)
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
/// let name = "sv_test";
/// let dvar = find(name);
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
) -> Result<(), ()> {
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (0.957f32, 0.66f32, 0.71875f32, 1.0f);
///     set_color(name, value);
/// }
/// ```
pub fn set_color_internal(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
) -> Result<(), ()> {
    set_color_from_source(name, red, green, blue, alpha, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::External`]. Does nothing if a [`Dvar`] with `name`
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
/// let name = "sv_test";
/// let dvar = find(name);
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
) -> Result<(), ()> {
    set_color_from_source(name, red, green, blue, alpha, SetSource::External)
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
/// let name = "sv_test";
/// let dvar = find(name);
/// let value = (0.957f32, 0.66f32, 0.71875f32, 1.0f);
/// let flags = DvarFlags::External;
/// let description = "External Dvar";
/// set_or_register_color(name, value, flags, Some(description));
/// ```
pub fn set_or_register_color(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        set_color_internal(name, red, green, blue, alpha)
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
/// let name = "sv_test";
/// let dvar = find(name);
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
) -> Result<(), ()> {
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (18_446_744_073_709_551_616 / 2) - 1;
///     set_int64(name, value);
/// }
/// ```
pub fn set_int64_internal(name: &str, value: i64) -> Result<(), ()> {
    set_int64_from_source(name, value, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::External`]. Does nothing if a [`Dvar`] with `name`
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (18_446_744_073_709_551_616 / 2) - 1;
///     set_int64(name, value);
/// }
/// ```
pub fn set_int64(name: &str, value: i64) -> Result<(), ()> {
    set_int64_from_source(name, value, SetSource::External)
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
/// let name = "sv_test";
/// let dvar = find(name);
/// let value = (18_446_744_073_709_551_616 / 2) - 1;
/// let min = 0;
/// let max = i64::MAX;
/// let flags = DvarFlags::External;
/// let description = "External Dvar";
/// set_or_register_int64(
///     name,
///     value,
///     Some(min),
///     Some(max),
///     flags,
///     Some(description),
/// );
/// ```
pub fn set_or_register_int64(
    name: &str,
    value: i64,
    min: Option<i64>,
    max: Option<i64>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        set_int64_internal(name, value)
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
/// let name = "sv_test";
/// let dvar = find(name);
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
) -> Result<(), ()> {
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (0.957f32, 0.66f32, 0.71875f32);
///     set_linear_color_rgb(name, value);
/// }
/// ```
pub fn set_linear_color_rgb_internal(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
) -> Result<(), ()> {
    set_linear_color_rgb_from_source(
        name,
        red,
        green,
        blue,
        SetSource::Internal,
    )
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::External`]. Does nothing if a [`Dvar`] with `name`
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
/// let name = "sv_test";
/// let dvar = find(name);
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
) -> Result<(), ()> {
    set_linear_color_rgb_from_source(
        name,
        red,
        green,
        blue,
        SetSource::External,
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
/// let name = "sv_test";
/// let dvar = find(name);
/// let value = (0.957f32, 0.66f32, 0.71875f32);
/// let min = 0.5f32;
/// let max = 0.97f32;
/// let flags = DvarFlags::External;
/// let description = "External Dvar";
/// set_or_register_linear_color_rgb(
///     name,
///     value,
///     Some(min),
///     Some(max),
///     flags,
///     Some(description),
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn set_or_register_linear_color_rgb(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        set_linear_color_rgb_internal(name, red, green, blue)
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
/// let name = "sv_test";
/// let dvar = find(name);
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
) -> Result<(), ()> {
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (0.957f32, 0.66f32, 0.71875f32);
///     set_color_xyz(name, value);
/// }
/// ```
pub fn set_color_xyz_internal(
    name: &str,
    x: f32,
    y: f32,
    z: f32,
) -> Result<(), ()> {
    set_color_xyz_from_source(name, x, y, z, SetSource::Internal)
}

/// Sets the value of an existing [`Dvar`].
///
/// Uses the supplied parameters to update an existing [`Dvar`] with name
/// `name` from source [`SetSource::External`]. Does nothing if a [`Dvar`] with `name`
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
/// let name = "sv_test";
/// let dvar = find(name);
/// // Make sure Dvar currently exists
/// if dvar.is_some() {
///     let value = (0.957f32, 0.66f32, 0.71875f32);
///     set_color_xyz(name, value);
/// }
/// ```
pub fn set_color_xyz(name: &str, x: f32, y: f32, z: f32) -> Result<(), ()> {
    set_color_xyz_from_source(name, x, y, z, SetSource::External)
}

/// Sets the value of an existing [`Dvar`], or registers a new one with
/// said value
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
/// let name = "sv_test";
/// let dvar = find(name);
/// let value = (0.957f32, 0.66f32, 0.71875f32);
/// let min = 0.5f32;
/// let max = 0.97f32;
/// let flags = DvarFlags::External;
/// let description = "External Dvar";
/// set_or_register_color_xyz(
///     name,
///     value,
///     Some(min),
///     Some(max),
///     flags,
///     Some(description),
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn set_or_register_color_xyz(
    name: &str,
    x: f32,
    y: f32,
    z: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        set_color_xyz_internal(name, x, y, z)
    } else {
        register_color_xyz(name, x, y, z, min, max, flags, description)
    }
}
