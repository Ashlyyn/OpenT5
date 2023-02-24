#![allow(clippy::pub_use)]

use std::{
    collections::HashMap,
    sync::{RwLock},
};

extern crate alloc;
use alloc::sync::Arc;

use lazy_static::lazy_static;

use crate::dvar::Dvar;

use super::DvarFlags;

pub mod register;
pub use register::*;

pub mod set;
pub use set::*;

pub mod get;
pub use get::*;

const DVAR_COUNT_MAX: usize = 4096;

lazy_static! {
    pub(super) static ref DVARS: Arc<RwLock<HashMap<String, Box<Dvar>>>> =
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
/// let dvar_name = "sv_test";
/// match find(dvar_name) {
///     Some(d) => println!("found dvar {} with value {}", d.name, d.current),
///     None => panic!("dvar {} not found", dvar_name),
/// };
/// ```
pub(super) fn find(name: &str) -> Option<Dvar> {
    let lock = DVARS.clone();
    let reader = lock.read().unwrap();

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
/// let dvar_name = "sv_test";
/// match exists(dvar_name) {
///     true => println!("found dvar {}", d.name),
///     false => panic!("dvar {} not found", dvar_name),
/// };
/// ```
pub fn exists(name: &str) -> bool {
    find(name).is_some()
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
/// let name = "sv_test";
/// clear_modified(name);
/// ```
pub fn clear_modified(name: &str) -> Result<(), ()> {
    let lock = DVARS.clone();
    let mut writer = lock.write().unwrap();
    if let Some(d) = writer.get_mut(name) {
        d.modified = false;
        return Ok(());
    };

    Err(())
}

/// Adds flags to an existing [`Dvar`].
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to add the flags to.
/// * `flags` - The [`DvarFlags`] to add to the [`Dvar`].
///
/// # Return Value
///
/// Returns true if the [`Dvar`] exists and the flags were successfully added,
/// false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let name = "sv_test";
/// let flags = DvarFlags::empty();
/// add_flags(name, flags);
/// ```
pub fn add_flags(name: &str, flags: DvarFlags) -> Result<(), ()> {
    let lock = DVARS.clone();
    let mut writer = lock.write().unwrap();
    if let Some(d) = writer.get_mut(name) {
        d.add_flags(flags);
        return Ok(());
    };

    Err(())
}

/// Clears flags from an existing [`Dvar`].
///
/// # Arguments
/// * `name` - A [`String`] that holds the name of the [`Dvar`]
/// to add the flags to.
/// * `flags` - The [`DvarFlags`] to clear from the [`Dvar`].
///
/// # Return Value
///
/// Returns true if the [`Dvar`] exists and the flags were successfully added,
/// false otherwise.
///
/// # Panics
/// Panics if the write lock for [`DVARS`] can't be acquired (usually because
/// the write lock or a read lock is held by a function farther up the
/// call stack).
///
/// Example
/// ```
/// let name = "sv_test";
/// let flags = DvarFlags::EXTERNAL;
/// clear_flags(name, flags);
/// ```
pub fn clear_flags(name: &str, flags: DvarFlags) -> Result<(), ()> {
    let lock = DVARS.clone();
    let mut writer = lock.write().unwrap();
    if let Some(d) = writer.get_mut(name) {
        d.clear_flags(flags);
        return Ok(());
    };

    Err(())
}

pub fn make_latched_value_current(name: &str) -> Result<(), ()> {
    let lock = DVARS.clone();
    let mut writer = lock.write().unwrap();
    if let Some(d) = writer.get_mut(name) {
        d.make_latched_value_current();
        return Ok(());
    };

    Err(())
}

// Helper function to check if Dvar name is valid
// Valid names consist only of alphanumeric characters and underscores
pub fn name_is_valid(name: &str) -> bool {
    !name.chars().any(|c| !c.is_alphanumeric() && c != '_')
}
