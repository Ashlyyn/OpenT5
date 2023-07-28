#![allow(dead_code)]

use core::f32::consts::PI;
use std::path::Path;

use core::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicIsize,
    AtomicU16, AtomicU32, AtomicU64, AtomicU8, AtomicUsize, Ordering,
};
extern crate alloc;
use alloc::sync::Arc;
use std::sync::{Condvar, Mutex};

use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};

use crate::platform::{os::target::MonitorHandle, WindowHandle};

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        use std::os::windows::prelude::OsStrExt;
        use windows::Win32::System::LibraryLoader::{LoadLibraryW, FreeLibrary};
        use windows::Win32::Foundation::HMODULE;
        use windows::core::PCWSTR;
        use windows::Win32::Foundation::{WPARAM, LPARAM};
        use windows::Win32::Graphics::Direct3D9::D3DFORMAT;
    }
    else if #[cfg(target_family = "unix")] {
        use std::os::unix::prelude::OsStrExt;
        use libc::{dlopen, dlclose, RTLD_NOW};
        use core::ffi::c_char;
    }
}

#[derive(Clone, Debug)]
pub struct SmpEvent {
    manual_reset: bool,
    inner: Arc<(Mutex<SignalState>, Condvar)>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SignalState {
    Signaled,
    Cleared,
}

impl SmpEvent {
    /// Creates a new [`SmpEvent`]
    ///
    /// # Arguments
    /// * `initial_state` - The initial [`SignalState`] for the event (cleared
    /// or signaled).
    /// * `manual_reset` - Whether or not the event has to be manually-reset.
    /// If [`false`], [`SmpEvent::wait`] will clear the event. If [`true`], the
    /// event will have to be reset manually with [`SmpEvent::clear`].
    pub fn new(initial_state: SignalState, manual_reset: bool) -> Self {
        Self {
            manual_reset,
            inner: Arc::new((Mutex::new(initial_state), Condvar::new())),
        }
    }

    pub fn wait(&mut self) {
        if *self.inner.0.lock().unwrap() == SignalState::Signaled {
            if !self.manual_reset {
                *self.inner.0.lock().unwrap() = SignalState::Cleared;
            }
            return;
        }

        #[allow(unused_must_use)]
        {
            self.inner.1.wait(self.inner.0.lock().unwrap());
        }
        if !self.manual_reset {
            *self.inner.0.lock().unwrap() = SignalState::Cleared;
        }
    }

    pub fn query(&mut self) -> SignalState {
        *self.inner.0.lock().unwrap()
    }

    pub fn clear(&mut self) {
        *self.inner.0.lock().unwrap() = SignalState::Cleared;
    }

    pub fn set(&mut self) {
        *self.inner.0.lock().unwrap() = SignalState::Signaled;

        if self.manual_reset {
            self.notify_all();
        } else {
            self.notify_one();
        }
    }

    fn notify_one(&self) {
        self.inner.1.notify_one();
    }

    #[allow(dead_code)]
    fn notify_all(&self) {
        self.inner.1.notify_all();
    }
}

/// Wrapper for a dynamic library loaded at runtime
pub struct Module {
    // In the future, I woud like to make the inner member a &[u8] rather than
    // a thin pointer, but Windows doesn't make getting the size of a loaded
    // library easier, so we're just going to use a pointer (which will work
    // on both Windows and Unix platforms)
    ptr: *mut (),
}

impl Module {
    cfg_if! {
        if #[cfg(target_os = "windows")] {
            /// Loads a library from the supplied path using [`LoadLibraryW`].
            /// Refer to [`LoadLibraryW`]'s documentation for what paths are
            /// valid.
            ///
            /// # Arguments
            ///
            /// * `name` - the name or or path of the library to be loaded.
            ///
            /// # Return Value
            ///
            /// Returns [`Some`] if the library was successfully loaded,
            /// [`None`] if not.
            pub fn load(name: &Path) -> Option<Self> {
                // [`OsStrExt::encode_wide`] doesn't add the null-terminator
                // that LoadLibraryW is going to expect, so we have to add it
                // manually
                let mut name =
                    name.as_os_str().encode_wide().collect::<Vec<_>>();
                name.push(0x0000);
                let name = name.as_ptr();

                // SAFETY:
                // LoadLibraryW is an FFI function, requiring use of unsafe.
                // LoadLibraryW itself should never create UB, violate memory
                // safety, etc., regardless of the name or path passed to it
                // in any scenario.
                unsafe {
                    LoadLibraryW(PCWSTR(name))
                }.ok().map(|h| Self { ptr: h.0 as *mut () })
            }

            /// Unloads the library loaded by [`Module::load`]. Should only be
            /// used when dropped.
            fn unload(&mut self) {
                // SAFETY:
                // FreeLibrary is an FFI function, requiring use of unsafe.
                // FreeLibrary itself should never create UB, violate memory
                // safety, etc., regardless of the pointer passed to it,
                // but in any event, the pointer we pass is guaranteed to
                // be valid since it was retrieved via LoadLibraryW.
                unsafe { FreeLibrary(HMODULE(self.ptr as _)); }
            }
        } else if #[cfg(target_family = "unix")] {
            /// Loads a library from the supplied path using [`dlopen`].
            /// Refer to [`dlopen`]'s documentation for what paths are
            /// valid.
            ///
            /// # Arguments
            ///
            /// * `name` - the name or or path of the library to be loaded.
            ///
            /// # Return Value
            ///
            /// Returns [`Some`] if the library was successfully loaded,
            /// [`None`] if not.
            pub fn load(name: &Path) -> Option<Self> {
                // [`OsStrExt::as_bytes`] doesn't yield a null-terminated string
                // like dlopen is going to expect, so we have to add it
                // manually
                let mut name = name.as_os_str().as_bytes().to_vec();
                name.push(b'\0');
                let name = name.as_ptr().cast::<c_char>();

                // SAFETY:
                // dlopen is an FFI function, requiring use of unsafe.
                // dlopen itself should never create UB, violate memory
                // safety, etc., regardless of the name or path passed to it
                // in any scenario.
                let ptr = unsafe { dlopen(name, RTLD_NOW) }.cast::<()>();
                if ptr.is_null() {
                    None
                } else {
                    Some(Self { ptr })
                }
            }

            /// Unloads the library loaded by [`Module::load`]. Should only be
            /// used when dropped.
            fn unload(&mut self) {
                // SAFETY:
                // dlclose is an FFI function, requiring use of unsafe.
                // dlclose itself may corrupt the program if it's passed a
                // library currently use by the program, but this function is
                // only ever called when the [`Module`] is dropped, so it
                // shouldn't corrupt anything.
                unsafe { dlclose(self.ptr.cast()); }
            }
        } else {
            #[allow(clippy::unimplemented)]
            pub fn load(_name: &Path) -> Option<Self> {
                unimplemented!()
            }

            #[allow(clippy::unimplemented, clippy::unused_self)]
            fn unload(&mut self) {
                unimplemented!()
            }
        }
    }
}

impl Drop for Module {
    /// Unloads the module when dropped.
    fn drop(&mut self) {
        self.unload();
    }
}

pub trait EasierWindowHandle: HasRawWindowHandle {
    fn window_handle(&self) -> WindowHandle;
}

// Made this because I got tired of importing core::sync::atomic::Ordering
// and passing the exact same Ordering (Ordering::Relaxed) 99% of the time.
// Purely a convenience thing, absolutely meaningless in terms of
// functionality
pub trait EasierAtomicBool {
    type ValueType;
    fn load_relaxed(&self) -> Self::ValueType;
    fn store_relaxed(&self, value: Self::ValueType) -> Self::ValueType;
}

impl EasierAtomicBool for AtomicBool {
    type ValueType = bool;
    fn load_relaxed(&self) -> Self::ValueType {
        self.load(Ordering::Relaxed)
    }

    fn store_relaxed(&self, value: Self::ValueType) -> Self::ValueType {
        self.store(value, Ordering::Relaxed);
        value
    }
}

pub trait EasierAtomic {
    type ValueType: num::Zero;
    fn load_relaxed(&self) -> Self::ValueType;
    fn store_relaxed(&self, value: Self::ValueType) -> Self::ValueType;
    fn increment(&self) -> Option<Self::ValueType>;
    fn decrement(&self) -> Option<Self::ValueType>;
    fn increment_wrapping(&self) -> Self::ValueType {
        self.increment()
            .unwrap_or_else(|| self.store_relaxed(num::zero()))
    }
    fn decrement_wrapping(&self) -> Self::ValueType {
        self.decrement()
            .unwrap_or_else(|| self.store_relaxed(num::zero()))
    }
}

macro_rules! easier_atomic_impl {
    ($t:ty, $vt:ty) => {
        #[allow(clippy::missing_trait_methods)]
        impl EasierAtomic for $t {
            type ValueType = $vt;
            fn load_relaxed(&self) -> Self::ValueType {
                self.load(Ordering::Relaxed)
            }

            fn store_relaxed(&self, value: Self::ValueType) -> Self::ValueType {
                self.store(value, Ordering::Relaxed);
                value
            }

            fn increment(&self) -> Option<Self::ValueType> {
                self.store_relaxed(self.load_relaxed().checked_add(1)?)
                    .into()
            }

            fn decrement(&self) -> Option<Self::ValueType> {
                self.store_relaxed(self.load_relaxed().checked_sub(1)?)
                    .into()
            }
        }
    };
}

easier_atomic_impl!(AtomicI8, i8);
easier_atomic_impl!(AtomicU8, u8);
easier_atomic_impl!(AtomicI16, i16);
easier_atomic_impl!(AtomicU16, u16);
easier_atomic_impl!(AtomicI32, i32);
easier_atomic_impl!(AtomicU32, u32);
easier_atomic_impl!(AtomicI64, i64);
easier_atomic_impl!(AtomicU64, u64);
easier_atomic_impl!(AtomicIsize, isize);
easier_atomic_impl!(AtomicUsize, usize);

pub trait CharFromUtf16Char {
    fn try_as_char(self) -> Option<char>;
}

impl CharFromUtf16Char for u16 {
    fn try_as_char(self) -> Option<char> {
        char::decode_utf16([self])
            .flatten()
            .collect::<Vec<_>>()
            .iter()
            .copied()
            .nth(0)
    }
}

pub trait LowWord {
    fn low_word(self) -> u16;
}

pub trait HighWord {
    fn high_word(self) -> u16;
}

impl LowWord for u32 {
    #[allow(clippy::cast_sign_loss)]
    fn low_word(self) -> u16 {
        (self & 0xFFFF) as _
    }
}

impl HighWord for u32 {
    #[allow(clippy::cast_sign_loss)]
    fn high_word(self) -> u16 {
        ((self >> 16) & 0xFFFF) as _
    }
}

impl LowWord for i32 {
    #[allow(clippy::cast_sign_loss)]
    fn low_word(self) -> u16 {
        (self & 0xFFFF) as _
    }
}

impl HighWord for i32 {
    #[allow(clippy::cast_sign_loss)]
    fn high_word(self) -> u16 {
        ((self >> 16) & 0xFFFF) as _
    }
}

cfg_if! {
    if #[cfg(windows)] {
        impl LowWord for WPARAM {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            fn low_word(self) -> u16 {
                (self.0 as u32).low_word()
            }
        }

        impl HighWord for WPARAM {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            fn high_word(self) -> u16 {
                (self.0 as u32).high_word()
            }
        }

        impl LowWord for LPARAM {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            fn low_word(self) -> u16 {
                (self.0 as u32).low_word()
            }
        }

        impl HighWord for LPARAM {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            fn high_word(self) -> u16 {
                (self.0 as u32).high_word()
            }
        }
    }
}

const UNITS_TO_METERS: f64 = 0.0254;
const METERS_TO_UNITS: f64 = 1.0 / 0.0254;

#[derive(Clone, Debug)]
pub struct WgpuSurface {
    pub window_handle: WindowHandle,
    pub monitor_handle: MonitorHandle,
}

// SAFETY: Should be safe since it's just grabbing the monitor handle
unsafe impl HasRawWindowHandle for WgpuSurface {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window_handle.get()
    }
}
// SAFETY: Should be safe since it's just grabbing the monitor handle
unsafe impl HasRawDisplayHandle for WgpuSurface {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.monitor_handle.get()
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Units(pub f64);

impl Units {
    const ZERO: Self = Self(0f64);
    const ONE: Self = Self(1f64);
    const MAX: Self = Self(f64::MAX);

    pub const fn new(units: f64) -> Self {
        Self(units)
    }

    pub const fn from_meters(meters: Meters) -> Self {
        Self(meters.0 * METERS_TO_UNITS)
    }

    pub const fn as_meters(self) -> Meters {
        Meters(self.0 * UNITS_TO_METERS)
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Meters(pub f64);

impl Meters {
    const ZERO: Self = Self(0f64);
    const ONE: Self = Self(1f64);
    const MAX: Self = Self(f64::MAX);

    pub const fn new(meters: f64) -> Self {
        Self(meters)
    }

    pub const fn from_units(units: Units) -> Self {
        Self(units.0 * UNITS_TO_METERS)
    }

    pub const fn as_units(self) -> Units {
        Units(self.0 * METERS_TO_UNITS)
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Distance(Units);

impl Distance {
    const ZERO: Self = Self(Units::ZERO);
    const ONE: Self = Self(Units::ONE);
    const MAX: Self = Self(Units::MAX);

    pub const fn from_units(units: Units) -> Self {
        Self(units)
    }

    pub const fn from_meters(meters: Meters) -> Self {
        Self(Units::from_meters(meters))
    }

    pub const fn as_units(self) -> Units {
        self.0
    }

    pub const fn as_meters(self) -> Meters {
        self.0.as_meters()
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct ScalarSpeed(Units);

impl ScalarSpeed {
    const ZERO: Self = Self(Units::ZERO);
    const ONE: Self = Self(Units::ONE);
    const MAX: Self = Self(Units::MAX);

    pub const fn from_units_per_second(units_per_second: Units) -> Self {
        Self(units_per_second)
    }

    pub const fn from_meters_per_second(meters_per_second: Meters) -> Self {
        Self(Units::from_meters(meters_per_second))
    }

    pub const fn as_units_per_second(self) -> Units {
        self.0
    }

    pub const fn as_meters_per_second(self) -> Meters {
        self.0.as_meters()
    }
}

const DEGREES_TO_RADIANS: f32 = PI / 180.0f32;
const RADIANS_TO_DEGREES: f32 = 180.0f32 / PI;

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Degrees(f32);

impl Degrees {
    const ZERO: Self = Self(0f32);
    const ONE: Self = Self(1f32);
    const MAX: Self = Self(360.0f32);

    pub const fn new(degrees: f32) -> Self {
        Self(degrees)
    }

    pub const fn from_radians(radians: Radians) -> Self {
        Self(radians.0 * RADIANS_TO_DEGREES)
    }

    pub const fn as_radians(self) -> Radians {
        Radians(self.0)
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Radians(f32);

impl Radians {
    const ZERO: Self = Self(0f32);
    const MAX: Self = Self(2.0f32 * PI);

    pub const fn new(degrees: f32) -> Self {
        Self(degrees)
    }

    pub const fn from_degrees(degrees: Degrees) -> Self {
        Self(degrees.0 * DEGREES_TO_RADIANS)
    }

    pub const fn as_degrees(self) -> Degrees {
        Degrees(self.0 * RADIANS_TO_DEGREES)
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Angle(Radians);

impl Angle {
    pub const fn from_degrees(degrees: Degrees) -> Self {
        Self(Radians::from_degrees(degrees))
    }

    pub const fn from_radians(radians: Radians) -> Self {
        Self(radians)
    }

    pub const fn as_degrees(self) -> Degrees {
        self.0.as_degrees()
    }

    pub const fn as_radians(self) -> Radians {
        self.0
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Velocity((Units, Units, Units));

impl Velocity {
    pub const fn from_units(x: Units, y: Units, z: Units) -> Self {
        Self((x, y, z))
    }

    pub const fn from_meters(x: Meters, y: Meters, z: Meters) -> Self {
        Self((x.as_units(), y.as_units(), z.as_units()))
    }

    pub const fn as_units(&self) -> (Units, Units, Units) {
        self.0
    }

    pub const fn as_meters(&self) -> (Meters, Meters, Meters) {
        (
            self.0 .1.as_meters(),
            self.0 .1.as_meters(),
            self.0 .2.as_meters(),
        )
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point {
    pub const ORIGIN: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct FourCC(u32);

pub const fn make_four_cc(a: u8, b: u8, c: u8, d: u8) -> FourCC {
    FourCC(a as u32 | ((b as u32) << 8) | ((c as u32) << 16) | ((d as u32) << 24))
}

#[const_trait]
pub trait FourCCExtDX {
    fn as_d3dfmt(self) -> D3DFORMAT;
}

impl const FourCCExtDX for FourCC {
    fn as_d3dfmt(self) -> D3DFORMAT {
        D3DFORMAT(self.0)
    }
}

pub const D3DFMT_NULL: D3DFORMAT = make_four_cc(b'N', b'U', b'L', b'L').as_d3dfmt();
pub const D3DPTFILTERCAPS_MINFANISOTROPIC: u32 = 0x400;
pub const D3DPTFILTERCAPS_MAGFANISOTROPIC: u32 = 0x4000000;