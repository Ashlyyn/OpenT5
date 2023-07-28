use crate::{
    com,
    common::{Vec2f32, Vec3f32, Vec4f32},
    dvar::{builder::DvarBuilder, DvarFlags},
};

use super::{exists, DVARS, DVAR_COUNT_MAX};

/// Registers a new [`Dvar`] of type [`DvarValue::Bool`],
/// using the provided name, value, flags, and description,
/// if a [`Dvar`] with name `name` doesn't already
/// exist, reregisters said [`Dvar`] if it does.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`bool`] representing the value to
/// register the [`Dvar`] with.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if (re)registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// let value = false;
/// let flags = DvarFlags::empty();
/// let description = "A test Dvar of type bool"
/// register_bool(name, value, flags, Some(description));
/// ```
pub fn register_bool(
    name: &str,
    value: bool,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if DVARS.read().unwrap().len() + 1 > DVAR_COUNT_MAX {
        com::errorln!(
            com::ErrorParm::FATAL,
            "Can\'t create dvar \'{}\': {} dvars already exist", name, DVAR_COUNT_MAX,
        );
        return Err(());
    }

    let dvar = DvarBuilder::new()
        .name(name)
        .description(description.unwrap_or_default().to_owned())
        .flags(flags)
        .type_bool()
        .value(value)
        .build();
    if DVARS
        .write()
        .unwrap()
        .insert(name.to_owned(), Box::new(dvar))
        .is_some()
    {
        let other_name = DVARS.read().unwrap().get(name).unwrap().name.clone();
        com::errorln!(
            com::ErrorParm::FATAL,
                "dvar name hash collision between \'{}\' and \'{}\' Please \
                 change one of these names to remove the hash collision",
                name, other_name,
        );
        Err(())
    } else {
        Ok(())
    }
}

/// Registers a new [`Dvar`] of type [`DvarValue::Bool`],
/// using the provided name, value, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`bool`] representing the value to
/// register the [`Dvar`] with.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] doesn't exist and registration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_none() {
///     let value = false;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type bool"
///     register_new_bool(name, value, flags, Some(description));
/// }
/// ```
pub fn register_new_bool(
    name: &str,
    value: bool,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        return Err(());
    }

    register_bool(name, value, flags, description)
}

/// Reregisters an existing [`Dvar`] with name `name` as
/// type [`DvarValue::Bool`] with the supplied value, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`bool`] representing the value to register the [`Dvar`] with.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a description for the
///   [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] exists and reregistration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_some() {
///     let value = false;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type bool"
///     reregister_bool(name, value, flags, Some(description));
/// }
/// ```
pub fn reregister_bool(
    name: &str,
    value: bool,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if !exists(name) {
        return Err(());
    }

    register_bool(name, value, flags, description)
}

/// Registers a new [`Dvar`] of type [`DvarValue::Float`],
/// using the provided name, value, flags, and description,
/// if a [`Dvar`] with name `name` doesn't already
/// exist, reregisters said [`Dvar`] if it does.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An [`f32`] representing the value to register the [`Dvar`] with
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a description for the
///   [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// let value = 121.3;
/// let min = -462.0;
/// let max = 592.7;
/// let flags = DvarFlags::empty();
/// let description = "A test Dvar of type float"
/// register_float(name, value, Some(min), Some(max),
///                             flags, Some(description));
/// ```
pub fn register_float(
    name: &str,
    value: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if DVARS.read().unwrap().len() + 1 > DVAR_COUNT_MAX {
        com::errorln!(
            com::ErrorParm::FATAL, "Can\'t create dvar \'{}\': {} dvars already exist",
                name, DVAR_COUNT_MAX,
        );
        return Err(());
    }

    let dvar = DvarBuilder::new()
        .name(name)
        .description(description.unwrap_or_default().to_owned())
        .flags(flags)
        .type_float()
        .domain(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX))
        .value(value)
        .build();
    if DVARS
        .write()
        .unwrap()
        .insert(name.to_owned(), Box::new(dvar))
        .is_some()
    {
        let other_name = DVARS.read().unwrap().get(name).unwrap().name.clone();
        com::errorln!(
            com::ErrorParm::FATAL,
                "dvar name hash collision between \'{}\' and \'{}\' Please \
                 change one of these names to remove the hash collision",
                name, other_name
            );
        Err(())
    } else {
        Ok(())
    }
}

/// Registers a new [`Dvar`] of type [`DvarValue::Float`],
/// using the provided name, value, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An [`f32`] representing the value to
/// register the [`Dvar`] with.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] doesn't exist and registration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_none() {
///     let value = 121.3;
///     let min = -462.0;
///     let max = 592.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type float"
///     register_new_float(name, value, Some(min), Some(max),
///                                     flags, Some(description));
/// }
/// ```
pub fn register_new_float(
    name: &str,
    value: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        return Err(());
    }

    register_float(name, value, min, max, flags, description)
}

/// Reregisters an existing [`Dvar`] with name `name` as
/// type [`DvarValue::Float`] with the supplied value, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An [`f32`] representing the value to
/// register the [`Dvar`] with.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] exists and reregistration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_some() {
///     let value = 121.3;
///     let min = -462.0;
///     let max = 592.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type float"
///     reregister_float(name, value, Some(min), Some(max),
///                                   flags, Some(description));
/// }
/// ```
pub fn reregister_float(
    name: &str,
    value: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if !exists(name) {
        return Err(());
    }

    register_float(name, value, min, max, flags, description)
}

/// Registers a new [`Dvar`] of type [`DvarValue::Vector2`],
/// using the provided name, value, flags, and description,
/// if a [`Dvar`] with name `name` doesn't
/// already exist, reregisters said [`Dvar`] if it does.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`Vec2f32`] representing the value to
/// register the [`Dvar`] with.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// let value = (121.3, -267.4);
/// let min = -462.0;
/// let max = 592.7;
/// let flags = DvarFlags::empty();
/// let description = "A test Dvar of type vector2"
/// register_vector2(name, value, Some(min), Some(max),
///                               flags, Some(description));
/// ```
pub fn register_vector2(
    name: &str,
    value: Vec2f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if DVARS.read().unwrap().len() + 1 > DVAR_COUNT_MAX {
        com::errorln!(
            com::ErrorParm::FATAL,
                "Can\'t create dvar \'{}\': {} dvars already exist",
                name, DVAR_COUNT_MAX,
        );
        return Err(());
    }

    let dvar = DvarBuilder::new()
        .name(name)
        .description(description.unwrap_or_default().to_owned())
        .flags(flags)
        .type_vector2()
        .domain(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX))
        .value(value)
        .build();

    if DVARS
        .write()
        .unwrap()
        .insert(name.to_owned(), Box::new(dvar))
        .is_some()
    {
        let other_name = DVARS.read().unwrap().get(name).unwrap().name.clone();
        com::errorln!(
            com::ErrorParm::FATAL,
                "dvar name hash collision between \'{}\' and \'{}\' Please \
                 change one of these names to remove the hash collision",
                name, other_name,
        );
        Err(())
    } else {
        Ok(())
    }
}

/// Registers a new [`Dvar`] of type [`DvarValue::Vector2`],
/// using the provided name, value, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`Vec2f32`] representing the value to
/// register the [`Dvar`] with.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] doesn't exist and registration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_none() {
///     let value = (121.3, -267.4);
///     let min = -462.0;
///     let max = 592.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type vector2"
///     register_new_vector2(name, value, Some(min), Some(max),
///                                       flags, Some(description));
/// }
/// ```
pub fn register_new_vector2(
    name: &str,
    value: Vec2f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        return Err(());
    }

    register_vector2(name, value, min, max, flags, description)
}

/// Reregisters an existing [`Dvar`] with name `name` as
/// type [`DvarValue::Vector2`] with the supplied value, flags, and
/// description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`Vec2f32`] representing the value to
/// register the [`Dvar`] with.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] exists and reregistration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_some() {
///     let value = (121.3, -267.4);
///     let min = -462.0;
///     let max = 592.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type vector2"
///     reregister_vector2(name, value, Some(min), Some(max),
///                                     flags, Some(description));
/// }
/// ```
pub fn reregister_vector2(
    name: &str,
    value: Vec2f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if !exists(name) {
        return Err(());
    }

    register_vector2(name, value, min, max, flags, description)
}

/// Registers a new [`Dvar`] of type [`DvarValue::Vector3`],
/// using the provided name, value, flags, and description,
/// if a [`Dvar`] with name `name` doesn't already
/// exist, reregisters said [`Dvar`] if it does.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`Vec3f32`] representing the value to
/// register the [`Dvar`] with.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// let value = (121.3, -267.4, 462.32);
/// let min = -462.0;
/// let max = 592.7;
/// let flags = DvarFlags::empty();
/// let description = "A test Dvar of type vector2"
/// register_vector3(name, value, Some(min), Some(max),
///                               flags, Some(description));
/// ```
pub fn register_vector3(
    name: &str,
    value: Vec3f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if DVARS.read().unwrap().len() + 1 > DVAR_COUNT_MAX {
        com::errorln!(
            com::ErrorParm::FATAL,
                "Can\'t create dvar \'{}\': {} dvars already exist",
                name, DVAR_COUNT_MAX,
        );
        return Err(());
    }

    let dvar = DvarBuilder::new()
        .name(name)
        .description(description.unwrap_or_default().to_owned())
        .flags(flags)
        .type_vector3()
        .domain(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX))
        .value(value)
        .build();
    if DVARS
        .write()
        .unwrap()
        .insert(name.to_owned(), Box::new(dvar))
        .is_some()
    {
        let other_name = DVARS.read().unwrap().get(name).unwrap().name.clone();
        com::errorln!(
            com::ErrorParm::FATAL,
                "dvar name hash collision between \'{}\' and \'{}\' Please \
                 change one of these names to remove the hash collision",
                name, other_name,
        );
        Err(())
    } else {
        Ok(())
    }
}

/// Registers a new [`Dvar`] of type [`DvarValue::Vector3`],
/// using the provided name, value, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An [`Vec3f32`] representing the value to
/// register the [`Dvar`] with.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] doesn't exist and registration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_none() {
///     let value = (121.3, -267.4, 462.32);
///     let min = -462.0;
///     let max = 592.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type vector2"
///     register_new_vector3(name, value, Some(min), Some(max),
///                                       flags, Some(description));
/// }
/// ```
pub fn register_new_vector3(
    name: &str,
    value: Vec3f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        return Err(());
    }

    register_vector3(name, value, min, max, flags, description)
}

/// Reregisters an existing [`Dvar`] with name `name` as
/// type [`DvarValue::Vector3`] with the supplied value, flags, and
/// description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`Vec3f32`] representing the value to
/// register the [`Dvar`] with.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] exists and reregistration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_some() {
///     let value = (121.3, -267.4, 462.32);
///     let min = -462.0;
///     let max = 592.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type vector2"
///     reregister_vector3(name, value, Some(min), Some(max),
///                                     flags, Some(description));
/// }
/// ```
pub fn reregister_vector3(
    name: &str,
    value: Vec3f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if !exists(name) {
        return Err(());
    }

    register_vector3(name, value, min, max, flags, description)
}

/// Registers a new [`Dvar`] of type [`DvarValue::Vector4`],
/// using the provided name, value, flags, and description,
/// if a [`Dvar`] with name `name` doesn't already
/// exist, reregisters said [`Dvar`] if it does.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`Vec4f32`] representing the value to
/// register the [`Dvar`] with.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for the [`Dvar`]
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for the [`Dvar`]
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// let value = (121.3, -267.4, -143.0, 71.2);
/// let min = -462.0;
/// let max = 592.7;
/// let flags = DvarFlags::empty();
/// let description = "A test Dvar of type vector4"
/// register_vector4(name, value, Some(min), Some(max),
///                                   flags, Some(description));
/// ```
pub fn register_vector4(
    name: &str,
    value: Vec4f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if DVARS.read().unwrap().len() + 1 > DVAR_COUNT_MAX {
        com::errorln!(
            com::ErrorParm::FATAL,
                "Can\'t create dvar \'{}\': {} dvars already exist",
                name, DVAR_COUNT_MAX
        );
        return Err(());
    }

    let dvar = DvarBuilder::new()
        .name(name)
        .description(description.unwrap_or_default().to_owned())
        .flags(flags)
        .type_vector4()
        .domain(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX))
        .value(value)
        .build();

    if DVARS
        .write()
        .unwrap()
        .insert(name.to_owned(), Box::new(dvar))
        .is_some()
    {
        let other_name = DVARS.read().unwrap().get(name).unwrap().name.clone();
        com::errorln!(
            com::ErrorParm::FATAL,
                "dvar name hash collision between \'{}\' and \'{}\' Please \
                 change one of these names to remove the hash collision",
                name, other_name,
        );
        Err(())
    } else {
        Ok(())
    }
}

/// Registers a new [`Dvar`] of type [`DvarValue::Vector4`],
/// using the provided name, value, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An [`Vec4f32`] representing the value to
/// register the [`Dvar`] with.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] doesn't exist and registration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_none() {
///     let value = (121.3, -267.4, -143.0, 71.2);
///     let min = -462.0;
///     let max = 592.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type vector4"
///     register_new_vector4(name, value, Some(min), Some(max),
///                                       flags, Some(description));
/// }
/// ```
pub fn register_new_vector4(
    name: &str,
    value: Vec4f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        return Err(());
    }

    register_vector4(name, value, min, max, flags, description)
}

/// Reregisters an existing [`Dvar`] with name `name` as
/// type [`DvarValue::Vector4`] with the supplied value, flags, and
/// description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`Vec4f32`] representing the value to
/// register the [`Dvar`] with.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] exists and reregistration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_some() {
///     let value = (121.3, -267.4, -143.0, 71.2);
///     let min = -462.0;
///     let max = 592.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type vector4"
///     reregister_vector4(name, value, Some(min), Some(max),
///                                     flags, Some(description));
/// }
/// ```
pub fn reregister_vector4(
    name: &str,
    value: Vec4f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if !exists(name) {
        return Err(());
    }

    register_vector4(name, value, min, max, flags, description)
}

/// Registers a new [`Dvar`] of type [`DvarValue::Int`], using
/// the provided name, value, flags, and description, if a [`Dvar`]
/// with name `name` doesn't already exist,
/// reregisters said [`Dvar`] if it does.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An [`i32`] representing the value to
/// register the [`Dvar`] with.
/// * `min` - Optional [`i32`] representing the lower bound
/// of the domain for the [`Dvar`]
/// * `max` - Optional [`i32`] representing the upper bound
/// of the domain for the [`Dvar`]
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// let value = 64867969;
/// let min = i32::MIN;
/// let max = i32::MAX;
/// let flags = DvarFlags::empty();
/// let description = "A test Dvar of type int"
/// register_int(name, value, Some(min), Some(max),
///                           flags, Some(description));
/// ```
pub fn register_int(
    name: &str,
    value: i32,
    min: Option<i32>,
    max: Option<i32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if DVARS.read().unwrap().len() + 1 > DVAR_COUNT_MAX {
        com::errorln!(
            com::ErrorParm::FATAL,
                "Can\'t create dvar \'{}\': {} dvars already exist",
                name, DVAR_COUNT_MAX,
        );
        return Err(());
    }

    let dvar = DvarBuilder::new()
        .name(name)
        .description(description.unwrap_or_default().to_owned())
        .flags(flags)
        .type_int()
        .domain(min.unwrap_or(i32::MIN), max.unwrap_or(i32::MAX))
        .value(value)
        .build();

    if DVARS
        .write()
        .unwrap()
        .insert(name.to_owned(), Box::new(dvar))
        .is_some()
    {
        let other_name = DVARS.read().unwrap().get(name).unwrap().name.clone();
        com::errorln!(
            com::ErrorParm::FATAL,
                "dvar name hash collision between \'{}\' and \'{}\' Please \
                 change one of these names to remove the hash collision",
                name, other_name
        );
        Err(())
    } else {
        Ok(())
    }
}

/// Registers a new [`Dvar`] of type [`DvarValue::Int`],
/// using the provided name, value, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An [`i32`] representing the value to register the [`Dvar`] with.
/// * `min` - Optional [`i32`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`i32`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] doesn't exist and registration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_none() {
///     let value = 64867969;
///     let min = i32::MIN;
///     let max = i32::MAX;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type int"
///     register_new_int(name, value, Some(min), Some(max),
///                                   flags, Some(description));
/// }
/// ```
pub fn register_new_int(
    name: &str,
    value: i32,
    min: Option<i32>,
    max: Option<i32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        return Err(());
    }

    register_int(name, value, min, max, flags, description)
}

/// Reregisters an existing [`Dvar`] with name `name` as
/// type [`DvarValue::Int`] with the supplied value, flags, and
/// description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An [`i32`] representing the value to
/// register the [`Dvar`] with.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] exists and reregistration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_some() {
///     let value = 64867969;
///     let min = i32::MIN;
///     let max = i32::MAX;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type int"
///     reregister_int(name, value, Some(min), Some(max),
///                                 flags, Some(description));
/// }
/// ```
pub fn reregister_int(
    name: &str,
    value: i32,
    min: Option<i32>,
    max: Option<i32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if !exists(name) {
        return Err(());
    }

    register_int(name, value, min, max, flags, description)
}

/// Registers a new [`Dvar`] of type [`DvarValue::String`],
/// using the provided name, value, flags, and description,
/// if a [`Dvar`] with name `name` doesn't already
/// exist, reregisters said [`Dvar`] if it does.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`String`] representing the value to register the [`Dvar`]
///   with.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// let value = "this is a test";
/// let flags = DvarFlags::empty();
/// let description = "A test Dvar of type string"
/// register_string(name, value, flags, Some(description));
/// ```
pub fn register_string(
    name: &str,
    value: &str,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if DVARS.read().unwrap().len() + 1 > DVAR_COUNT_MAX {
        com::errorln!(
            com::ErrorParm::FATAL,
                "Can\'t create dvar \'{}\': {} dvars already exist",
                name, DVAR_COUNT_MAX
        );
        return Err(());
    }

    let dvar = DvarBuilder::new()
        .name(name)
        .description(description.unwrap_or_default().to_owned())
        .flags(flags)
        .type_string()
        .value(value.to_owned())
        .build();

    if DVARS
        .write()
        .unwrap()
        .insert(name.to_owned(), Box::new(dvar))
        .is_some()
    {
        let other_name = DVARS.read().unwrap().get(name).unwrap().name.clone();
        com::errorln!(
            com::ErrorParm::FATAL,
                "dvar name hash collision between \'{}\' and \'{}\' Please \
                 change one of these names to remove the hash collision",
                name, other_name
        );
        Err(())
    } else {
        Ok(())
    }
}

/// Registers a new [`Dvar`] of type [`DvarValue::String`], using the
/// provided name, value, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`String`] representing the value to register the [`Dvar`]
///   with.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] doesn't exist and registration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_none() {
///     let value = "this is a test";
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type string"
///     register_new_string(name, value, flags, Some(description));
/// }
/// ```
pub fn register_new_string(
    name: &str,
    value: &str,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        return Err(());
    }

    register_string(name, value, flags, description)
}

/// Reregisters an existing [`Dvar`] with name `name` as
/// type [`DvarValue::String`] with the supplied value, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`bool`] representing the value to register the [`Dvar`] with.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] exists and reregistration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_some() {
///     let value = "this is a test";
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type string"
///     reregister_string(name, value, flags, Some(description));
/// }
/// ```
pub fn reregister_string(
    name: &str,
    value: &str,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if !exists(name) {
        return Err(());
    }

    register_string(name, value, flags, description)
}

/// Registers a new [`Dvar`] of type [`DvarValue::Enumeration`], using the
/// provided name, value, flags, and description, if a [`Dvar`] with name `name`
/// doesn't already exist, reregisters said [`Dvar`] if it does.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`String`] representing the value to register the [`Dvar`]
///   with.
/// * `domain` - Optional [`Vec<String>`] representing the domain
/// to register the [`Dvar`] with.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a description for the
///   [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// let value = "DEF";
/// let domain = vec!["ABC".to_owned(), "DEF".to_owned(), "GHI".to_owned()];
/// let flags = DvarFlags::empty();
/// let description = "A test Dvar of type string"
/// register_enumeration(name, value, Some(domain), flags, Some(description));
/// ```
#[allow(clippy::semicolon_outside_block)]
pub fn register_enumeration(
    name: &str,
    value: String,
    domain: Option<Vec<String>>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if DVARS.read().unwrap().len() + 1 > DVAR_COUNT_MAX {
        com::errorln!(
            com::ErrorParm::FATAL,
                "Can\'t create dvar \'{}\': {} dvars already exist",
                name, DVAR_COUNT_MAX
        );
        return Err(());
    }

    let dvar = DvarBuilder::new()
        .name(name)
        .description(description.unwrap_or_default().to_owned())
        .flags(flags)
        .type_enumeration()
        .domain(&domain.unwrap_or_default())
        .value(value)
        .build();

    if DVARS
        .write()
        .unwrap()
        .insert(name.to_owned(), Box::new(dvar))
        .is_some()
    {
        let other_name = DVARS.read().unwrap().get(name).unwrap().name.clone();
        com::errorln!(
            com::ErrorParm::FATAL,
                "dvar name hash collision between \'{}\' and \'{}\' Please \
                 change one of these names to remove the hash collision",
                name, other_name
        );
        Err(())
    } else {
        Ok(())
    }
}

/// Registers a new [`Dvar`] of type [`DvarValue::Enumeration`], using the
/// provided name, value, flags, domain, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`String`] representing the value to register the [`Dvar`]
///   with.
/// * `domain` - Optional [`Vec<String>`] representing the domain
/// to register the [`Dvar`] with.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] doesn't exist and registration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_none() {
///     let value = "DEF";
///     let domain = vec!["ABC".to_owned(), "DEF".to_owned(), "GHI".to_owned()];
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type string"
///     register_enumeration(name, value, Some(domain), flags, Some(description));
/// }
/// ```
pub fn register_new_enumeration(
    name: &str,
    value: String,
    domain: Option<Vec<String>>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        return Err(());
    }

    register_enumeration(name, value, domain, flags, description)
}

/// Reregisters an existing [`Dvar`] with name `name` as
/// type [`DvarValue::Enumeration`] with the supplied value, domain, flags,
/// and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - A [`bool`] representing the value to register the [`Dvar`] with.
/// * `domain` - Optional [`Vec<String>`] representing the domain
/// to register the [`Dvar`] with.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] exists and reregistration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_some() {
///     let value = "DEF";
///     let domain = vec!["ABC".to_owned(), "DEF".to_owned(), "GHI".to_owned()];
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type string"
///     register_enumeration(name, value, Some(domain), flags, Some(description));
/// }
/// ```
pub fn reregister_enumeration(
    name: &str,
    value: String,
    domain: Option<Vec<String>>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if !exists(name) {
        return Err(());
    }

    register_enumeration(name, value, domain, flags, description)
}

/// Registers a [`Dvar`] of type [`DvarValue::Color`], using the provided name,
/// value, flags, and description, if a [`Dvar`] with name `name` doesn't
/// already exist, reregisters said [`Dvar`] if it does.
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
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a description for the
///   [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// let red = 1.0;
/// let blue = 1.0;
/// let green = 0.0;
/// let alpha = 1.0;
/// let flags = DvarFlags::empty();
/// let description = "A test Dvar of type color"
/// register_color(name, red, green, blue,
///                alpha, flags, Some(description));
/// ```
#[allow(clippy::semicolon_outside_block)]
pub fn register_color(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    let r = red.clamp(0.0, 1.0).mul_add(255.0, 0.001);
    let g = green.clamp(0.0, 1.0).mul_add(255.0, 0.001);
    let b = blue.clamp(0.0, 1.0).mul_add(255.0, 0.001);
    let a = alpha.clamp(0.0, 1.0).mul_add(255.0, 0.001);

    if DVARS.write().unwrap().len() + 1 > DVAR_COUNT_MAX {
        com::errorln!(
            com::ErrorParm::FATAL,
                "Can\'t create dvar \'{}\': {} dvars already exist",
                name, DVAR_COUNT_MAX
        );
        return Err(());
    }

    let dvar = DvarBuilder::new()
        .name(name)
        .description(description.unwrap_or_default().to_owned())
        .flags(flags)
        .type_color()
        .value((r, g, b, a))
        .build();

    if DVARS
        .write()
        .unwrap()
        .insert(name.to_owned(), Box::new(dvar))
        .is_some()
    {
        let other_name = DVARS.write().unwrap().get(name).unwrap().name.clone();
        com::errorln!(
            com::ErrorParm::FATAL,
                "dvar name hash collision between \'{}\' and \'{}\' Please \
                 change one of these names to remove the hash collision",
                name, other_name
        );
        Err(())
    } else {
        Ok(())
    }
}

/// Registers a new [`Dvar`] of type [`DvarValue::Color`], using the
/// provided name, value, flags, domain, and description.
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
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a description for the
///   [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if exists(name).is_none() {
///     let red = 1.0;
///     let blue = 1.0;
///     let green = 0.0;
///     let alpha = 1.0;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type color"
///     register_new_color(name, red, green, blue,
///                        alpha, flags, Some(description));
/// }
/// ```
pub fn register_new_color(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        return Err(());
    }

    register_color(name, red, green, blue, alpha, flags, description)
}

/// Reregisters an existing [`Dvar`] with name `name` as
/// type [`DvarValue::Color`] with the supplied value, flags, and description.
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
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a description for the
///   [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if exists(name).is_some() {
///     let red = 1.0;
///     let blue = 1.0;
///     let green = 0.0;
///     let alpha = 1.0;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type color"
///     reregister_color(name, red, green, blue,
///                        alpha, flags, Some(description));
/// }
/// ```
pub fn reregister_color(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if !exists(name) {
        return Err(());
    }

    register_color(name, red, green, blue, alpha, flags, description)
}

/// Registers a [`Dvar`] of type [`DvarValue::Int64`], using the provided name,
/// value, flags, and description, if a [`Dvar`] with name `name` doesn't
/// already exist, reregisters said [`Dvar`] if it does.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An [`i64`] representing the value to register the [`Dvar`] with
/// * `min` - Optional [`i64`] representing the lower bound
/// for the domain for `value`
/// * `max` - Optional [`i64`] representing the upper bound
/// for the domain for `value`
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a description for the
///   [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// let value = 67894867969;
/// let min = i64::MIN;
/// let max = i64::MAX;
/// let flags = DvarFlags::empty();
/// let description = "A test Dvar of type int64"
/// register_int64(name, value, Some(min), Some(max), flags, Some(description));
/// ```
pub fn register_int64(
    name: &str,
    value: i64,
    min: Option<i64>,
    max: Option<i64>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if DVARS.write().unwrap().len() + 1 > DVAR_COUNT_MAX {
        com::errorln!(
            com::ErrorParm::FATAL,
                "Can\'t create dvar \'{}\': {} dvars already exist",
                name, DVAR_COUNT_MAX
        );
        return Err(());
    }

    let dvar = DvarBuilder::new()
        .name(name)
        .description(description.unwrap_or_default().to_owned())
        .flags(flags)
        .type_int64()
        .domain(min.unwrap_or(i64::MIN), max.unwrap_or(i64::MAX))
        .value(value)
        .build();

    if DVARS
        .write()
        .unwrap()
        .insert(name.to_owned(), Box::new(dvar))
        .is_some()
    {
        let other_name = DVARS.write().unwrap().get(name).unwrap().name.clone();
        com::errorln!(
            com::ErrorParm::FATAL,
                "dvar name hash collision between \'{}\' and \'{}\' Please \
                 change one of these names to remove the hash collision",
                name, other_name
        );
        Err(())
    } else {
        Ok(())
    }
}

/// Registers a new [`Dvar`] of type [`DvarValue::Int64`],
/// using the provided name, value, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An [`i64`] representing the value to register the [`Dvar`] with.
/// * `min` - Optional [`i64`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`i64`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] doesn't exist and registration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_none() {
///     let value = 67894867969;
///     let min = i64::MIN;
///     let max = i64::MAX;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type int64"
///     register_new_int64(name, value, Some(min), Some(max), flags, Some(description));
/// }
/// ```
pub fn register_new_int64(
    name: &str,
    value: i64,
    min: Option<i64>,
    max: Option<i64>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if exists(name) {
        return Err(());
    }

    register_int64(name, value, min, max, flags, description)
}

/// Reregisters an existing [`Dvar`] with name `name` as
/// type [`DvarValue::Int64`] with the supplied value, flags, and
/// description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `value` - An [`i64`] representing the value to
/// register the [`Dvar`] with.
/// * `min` - Optional [`i64`] representing the lower bound
/// of the domain for the [`Dvar`].
/// * `max` - Optional [`i64`] representing the upper bound
/// of the domain for the [`Dvar`].
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a
/// description for the [`Dvar`].
///
/// # Return Value
///
/// Returns true if [`Dvar`] exists and reregistration
/// was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_some() {
///     let value = 67894867969;
///     let min = i64::MIN;
///     let max = i64::MAX;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type int64"
///     reregister_int64(name, value, Some(min), Some(max), flags, Some(description));
/// }
/// ```
pub fn reregister_int64(
    name: &str,
    value: i64,
    min: Option<i64>,
    max: Option<i64>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if !exists(name) {
        return Err(());
    }

    register_int64(name, value, min, max, flags, description)
}

/// Registers a [`Dvar`] of type [`DvarValue::LinearColorRGB`], using the
/// provided name, value, flags, and description, if a [`Dvar`] with name `name`
/// doesn't already exist, reregisters said [`Dvar`] if it does.
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
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for `red`, `green`, and `blue`.
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for `red`, `green`, and `blue`.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a description for the
///   [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// let red = 0.6;
/// let blue = 0.2;
/// let green = 0.3;
/// let min = 0.1;
/// let max = 0.7;
/// let flags = DvarFlags::empty();
/// let description = "A test Dvar of type color"
/// register_linear_color_rgb(name, red, green, blue,
///                           Some(min), Some(max), flags,
///                           Some(description));
/// ```
#[allow(clippy::too_many_arguments)]
pub fn register_linear_color_rgb(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if DVARS.write().unwrap().len() + 1 > DVAR_COUNT_MAX {
        com::errorln!(
            com::ErrorParm::FATAL,
                "Can\'t create dvar \'{}\': {} dvars already exist",
                name, DVAR_COUNT_MAX
        );
        return Err(());
    }

    let dvar = DvarBuilder::new()
        .name(name)
        .description(description.unwrap_or_default().to_owned())
        .flags(flags)
        .type_linear_color_rgb()
        .domain(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX))
        .value((red, green, blue))
        .build();

    if DVARS
        .write()
        .unwrap()
        .insert(name.to_owned(), Box::new(dvar))
        .is_some()
    {
        let other_name = DVARS.write().unwrap().get(name).unwrap().name.clone();
        com::errorln!(
            com::ErrorParm::FATAL,
                "dvar name hash collision between \'{}\' and \'{}\' Please \
                 change one of these names to remove the hash collision",
                name, other_name
        );
        Err(())
    } else {
        Ok(())
    }
}

/// Registers a new [`Dvar`] of type [`DvarValue::LinearColorRGB`],
/// using the provided name, value, flags, and description.
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
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for `red`, `green`, and `blue`.
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for `red`, `green`, and `blue`.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a description for the
///   [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_none() {
///     let red = 0.6;
///     let blue = 0.2;
///     let green = 0.3;
///     let min = 0.1;
///     let max = 0.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type color"
///     register_new_linear_color_rgb(name, red, green, blue,
///                                   Some(min), Some(max), flags,
///                                   Some(description));
/// }
/// ```
#[allow(clippy::too_many_arguments)]
pub fn register_new_linear_color_rgb(
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
        return Err(());
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

/// Reregisters an existing [`Dvar`] with name `name` as
/// type [`DvarValue::LinearColorRGB`] with the supplied value, flags, and
/// description.
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
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for `red`, `green`, and `blue`.
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for `red`, `green`, and `blue`.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a description for the
///   [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_some() {
///     let red = 0.6;
///     let blue = 0.2;
///     let green = 0.3;
///     let min = 0.1;
///     let max = 0.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type color"
///     reregister_linear_color_rgb(name, red, green, blue,
///                                 Some(min), Some(max), flags,
///                                 Some(description));
/// }
/// ```
#[allow(clippy::too_many_arguments)]
pub fn reregister_linear_color_rgb(
    name: &str,
    red: f32,
    green: f32,
    blue: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if !exists(name) {
        return Err(());
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

/// Registers a [`Dvar`] of type [`DvarValue::ColorXYZ`], using the provided
/// name, value, flags, and description, if a [`Dvar`] with name `name` doesn't
/// already exist, reregisters said [`Dvar`] if it does.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `x` - An [`f32`] the x-value of the Dvar.
/// * `y` - An [`f32`] the y-value of the Dvar.
/// * `z` - An [`f32`] the z-value of the Dvar.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for `red`, `green`, and `blue`.
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for `red`, `green`, and `blue`.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a description for the
///   [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// let x = 0.6;
/// let y = 0.2;
/// let z = 0.3;
/// let min = 0.1;
/// let max = 0.7;
/// let flags = DvarFlags::empty();
/// let description = "A test Dvar of type color XYZ"
/// register_color_xyz(name, x, y, z, Some(min), Some(max), flags, Some(description));
/// ```
#[allow(clippy::too_many_arguments)]
pub fn register_color_xyz(
    name: &str,
    x: f32,
    y: f32,
    z: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if DVARS.write().unwrap().len() + 1 > DVAR_COUNT_MAX {
        com::errorln!(
            com::ErrorParm::FATAL,
                "Can\'t create dvar \'{}\': {} dvars already exist",
                name, DVAR_COUNT_MAX
        );
        return Err(());
    }

    let dvar = DvarBuilder::new()
        .name(name)
        .description(description.unwrap_or_default().to_owned())
        .flags(flags)
        .type_color_xyz()
        .domain(min.unwrap_or(f32::MIN), max.unwrap_or(f32::MAX))
        .value((x, y, z))
        .build();
    if DVARS
        .write()
        .unwrap()
        .insert(name.to_owned(), Box::new(dvar))
        .is_some()
    {
        let other_name = DVARS.write().unwrap().get(name).unwrap().name.clone();
        com::errorln!(
            com::ErrorParm::FATAL,
                "dvar name hash collision between \'{}\' and \'{}\' Please \
                 change one of these names to remove the hash collision",
                name, other_name
        );
        Err(())
    } else {
        Ok(())
    }
}

/// Registers a new [`Dvar`] of type [`DvarValue::ColorXYZ`],
/// using the provided name, value, flags, and description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `x` - An [`f32`] the x-value of the Dvar.
/// * `y` - An [`f32`] the y-value of the Dvar.
/// * `z` - An [`f32`] the z-value of the Dvar.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for `red`, `green`, and `blue`.
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for `red`, `green`, and `blue`.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a description for the
///   [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_none() {
///     let x = 0.6;
///     let y = 0.2;
///     let z = 0.3;
///     let min = 0.1;
///     let max = 0.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type color XYZ"
///     register_new_color_xyz(name, x, y, z, Some(min), Some(max), flags, Some(description));
/// }
/// ```
#[allow(clippy::too_many_arguments)]
pub fn register_new_color_xyz(
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
        return Err(());
    }

    register_color_xyz(name, x, y, z, min, max, flags, description)
}

/// Reregisters an existing [`Dvar`] with name `name` as
/// type [`DvarValue::ColorXYZ`] with the supplied value, flags, and
/// description.
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to be registered.
/// * `x` - An [`f32`] the x-value of the Dvar.
/// * `y` - An [`f32`] the y-value of the Dvar.
/// * `z` - An [`f32`] the z-value of the Dvar.
/// * `min` - Optional [`f32`] representing the lower bound
/// of the domain for `red`, `green`, and `blue`.
/// * `max` - Optional [`f32`] representing the upper bound
/// of the domain for `red`, `green`, and `blue`.
/// * `flags` - Optional [`DvarFlags`] to register the [`Dvar`] with.
/// * `description` - Optional [`String`] containing a description for the
///   [`Dvar`].
///
/// # Return Value
///
/// Returns true if registration was successful, false otherwise.
///
/// # Example
/// ```
/// let name = "sv_test";
/// if find(name).is_some() {
///     let x = 0.6;
///     let y = 0.2;
///     let z = 0.3;
///     let min = 0.1;
///     let max = 0.7;
///     let flags = DvarFlags::empty();
///     let description = "A test Dvar of type color XYZ"
///     reregister_color_xyz(name, x, y, z, Some(min), Some(max), flags, Some(description));
/// }
/// ```
#[allow(clippy::too_many_arguments)]
pub fn reregister_color_xyz(
    name: &str,
    x: f32,
    y: f32,
    z: f32,
    min: Option<f32>,
    max: Option<f32>,
    flags: DvarFlags,
    description: Option<&str>,
) -> Result<(), ()> {
    if !exists(name) {
        return Err(());
    }

    register_color_xyz(name, x, y, z, min, max, flags, description)
}
