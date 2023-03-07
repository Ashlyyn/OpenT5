#![allow(dead_code)]

use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use std::path::Path;

use core::sync::atomic::AtomicIsize;
use core::sync::atomic::AtomicUsize;
use std::sync::{Condvar, Mutex};

use raw_window_handle::HasRawWindowHandle;

use crate::platform::WindowHandle;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_os = "windows")] {
        use std::os::windows::prelude::OsStrExt;
        use windows::Win32::System::LibraryLoader::{LoadLibraryW, FreeLibrary};
        use windows::Win32::Foundation::HINSTANCE;
        use windows::core::PCWSTR;
    }
    else if #[cfg(target_family = "unix")] {
        use std::os::unix::prelude::OsStrExt;
        use libc::{dlopen, dlclose, RTLD_NOW};
        use core::ffi::c_char;
    }
}

pub struct SmpEvent<T: Sized> {
    manual_reset: bool,
    condvar: Condvar,
    mutex: Mutex<(bool, T)>,
}

impl<T: Sized + Clone> SmpEvent<T> {
    /// Creates a new [`SmpEvent`]
    ///
    /// # Arguments
    /// * `state` - The initial internal state for the object.
    /// * `signaled` - Whether or not to initialize
    /// the event in the signaled state.
    /// * `manual_reset` - Whether or not the event has to be manually-reset.
    /// If [`false`], [`Self::acknowledge`] and its variants will clear the
    /// signaled state. If not, this must be done manually.
    pub const fn new(state: T, signaled: bool, manual_reset: bool) -> Self {
        Self {
            manual_reset,
            condvar: Condvar::new(),
            mutex: Mutex::new((signaled, state)),
        }
    }

    pub fn signaled(&self) -> bool {
        self.mutex.lock().unwrap().0
    }

    fn set_signaled(&mut self) {
        self.mutex.lock().unwrap().0 = true;
    }

    fn clear_signaled(&mut self) {
        self.mutex.lock().unwrap().0 = false;
    }

    pub fn get_state(&self) -> T {
        self.mutex.lock().unwrap().1.clone()
    }

    pub fn try_get_state(&self) -> Result<T, ()> {
        self.mutex
            .try_lock()
            .map_or_else(|_| Err(()), |g| Ok(g.1.clone()))
    }

    fn set_state(&mut self, state: T) {
        self.mutex.lock().unwrap().1 = state;
    }

    fn try_set_state(&mut self, state: T) -> Result<(), ()> {
        let Ok(guard) = &mut self.mutex.try_lock() else {
            return Err(())
        };

        guard.1 = state;
        Ok(())
    }

    fn wait(&self) {
        let guard = self.mutex.lock().unwrap();

        #[allow(unused_must_use, clippy::semicolon_outside_block)]
        {
            self.condvar.wait(guard).unwrap();
        }
    }

    #[allow(clippy::semicolon_outside_block, clippy::unwrap_in_result)]
    fn try_wait(&self) -> Result<(), ()> {
        let Ok(guard) = self.mutex.try_lock() else {
            return Err(())
        };

        #[allow(unused_must_use)]
        {
            self.condvar.wait(guard).unwrap();
        }
        Ok(())
    }

    fn wait_timeout(&self, timeout: Duration) {
        let guard = self.mutex.lock().unwrap();

        #[allow(unused_must_use, clippy::semicolon_outside_block)]
        {
            self.condvar.wait_timeout(guard, timeout).unwrap();
        }
    }

    #[allow(clippy::map_err_ignore, clippy::semicolon_outside_block)]
    fn try_wait_timeout(&self, timeout: Duration) -> Result<(), ()> {
        let Ok(guard) = self.mutex.try_lock() else { return Err(()) };

        #[allow(unused_must_use)]
        {
            self.condvar.wait_timeout(guard, timeout).map_err(|_| ())?;
        }

        Ok(())
    }

    /*
    fn wait_while<F: FnMut(&mut (bool, T)) -> bool>(
        &self,
        condition: F,
    ) -> bool {
        let guard = match self.mutex.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        #[allow(unused_must_use)]
        {
            self.condvar.wait_while(guard, condition).unwrap();
        }
        true
    }

    fn try_wait_while<F: FnMut(&mut (bool, T)) -> bool>(
        &self,
        condition: F,
    ) -> bool {
        let guard = match self.mutex.try_lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        #[allow(unused_must_use)]
        {
            self.condvar.wait_while(guard, condition).unwrap();
        }
        true
    }

    fn wait_timeout_while<F: FnMut(&mut (bool, T)) -> bool>(
        &self,
        duration: Duration,
        condition: F,
    ) -> bool {
        let guard = match self.mutex.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        #[allow(unused_must_use)]
        {
            self.condvar
                .wait_timeout_while(guard, duration, condition)
                .unwrap();
        }
        true
    }

    fn try_wait_timeout_while<F: FnMut(&mut (bool, T)) -> bool>(
        &self,
        duration: Duration,
        condition: F,
    ) -> bool {
        let guard = match self.mutex.try_lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        #[allow(unused_must_use)]
        {
            self.condvar
                .wait_timeout_while(guard, duration, condition)
                .unwrap();
        }
        true
    }

    fn wait_until_signaled(&self) -> bool {
        self.wait_while(|(signaled, _)| *signaled != true)
    }

    fn try_wait_until_signaled(&self) -> bool {
        self.try_wait_while(|(signaled, _)| *signaled != true)
    }

    fn try_wait_until_signaled_timeout(&self, timeout: Duration) -> bool {
        self.try_wait_timeout_while(timeout, |(signaled, _)| *signaled != true)
    }
    */

    fn notify_one(&self) {
        self.condvar.notify_one();
    }

    #[allow(dead_code)]
    fn notify_all(&self) {
        self.condvar.notify_all();
    }

    /// Acknowledges the event and retrieves the internal state.
    pub fn acknowledge(&mut self) -> T {
        self.wait();

        if !self.manual_reset {
            self.clear_signaled();
        }

        self.get_state()
    }

    /// Tries to acknowledge the event, and retrieves its internal state
    /// if successful.
    #[allow(dead_code)]
    pub fn try_acknowledge(&mut self) -> Option<T> {
        if self.signaled() == false {
            return None;
        }

        if !self.manual_reset {
            self.clear_signaled();
        }

        self.try_get_state().map_or_else(|_| None, |s| Some(s))
    }

    /// Acknowledges the event within [`duration`], and retrieves its
    /// internal state if successful.
    pub fn acknowledge_timeout(&mut self, duration: Duration) -> T {
        self.wait_timeout(duration);

        if !self.manual_reset {
            self.clear_signaled();
        }

        self.get_state()
    }

    /// Tries to acknowledge the event within [`duration`], and retrieves its
    /// internal state if successful.
    #[allow(dead_code)]
    pub fn try_acknowledge_timeout(&mut self, duration: Duration) -> Option<T> {
        let res = match self.try_wait_timeout(duration) {
            Ok(_) => self.try_get_state(),
            Err(_) => Err(()),
        };

        if !self.manual_reset {
            self.clear_signaled();
        }

        res.ok()
    }

    /// Sets the event's internal state and sets the signaled state.
    pub fn send(&mut self, state: T) {
        self.set_state(state);
        self.set_signaled();
        self.notify_one();
    }

    /// Tries to set the event's internal state, and sets the signaled state
    /// if successful.
    #[allow(dead_code)]
    pub fn try_send(&mut self, state: T) {
        if self.try_set_state(state).is_err() {
            return;
        }
        self.set_signaled();
        self.notify_one();
    }

    /// Sets the event's internal state, and clears the signaled state
    /// if successful.
    pub fn send_cleared(&mut self, state: T) {
        self.set_state(state);
        self.clear_signaled();
        self.notify_one();
    }

    /// Tries to set the event's internal state, and clears the signaled state
    /// if successful.
    #[allow(dead_code)]
    pub fn try_send_cleared(&mut self, state: T) {
        if self.try_set_state(state).is_err() {
            return;
        }
        self.clear_signaled();
        self.notify_one();
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
                // [`OsStrExt::encode_wide`] doesn't add the null-terminator that
                // LoadLibraryW is going to expect, so we have to add it
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
                unsafe { LoadLibraryW(PCWSTR(name)) }.ok().map(|h| Self { ptr: h.0 as *mut () })
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
                unsafe { FreeLibrary(HINSTANCE(self.ptr as _)); }
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
                // dlclose itself should never create UB, violate memory
                // safety, etc., regardless of the pointer passed to it,
                // but in any event, the pointer we pass is guaranteed to
                // be valid since it was retrieved via dlopen.
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

impl<T: HasRawWindowHandle> EasierWindowHandle for T {
    fn window_handle(&self) -> WindowHandle {
        WindowHandle::new(self.raw_window_handle())
    }
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
    fn increment_wrapping(&self) -> Self::ValueType {
        self.increment()
            .unwrap_or_else(|| self.store_relaxed(num::zero()))
    }
}

#[allow(clippy::missing_trait_methods)]
impl EasierAtomic for AtomicIsize {
    type ValueType = isize;
    fn load_relaxed(&self) -> Self::ValueType {
        self.load(Ordering::Relaxed)
    }

    fn store_relaxed(&self, value: Self::ValueType) -> Self::ValueType {
        self.store(value, Ordering::Relaxed);
        value
    }

    fn increment(&self) -> Option<isize> {
        self.store_relaxed(self.load_relaxed().checked_add(1)?)
            .into()
    }
}

#[allow(clippy::missing_trait_methods)]
impl EasierAtomic for AtomicUsize {
    type ValueType = usize;
    fn load_relaxed(&self) -> Self::ValueType {
        self.load(Ordering::Relaxed)
    }

    fn store_relaxed(&self, value: Self::ValueType) -> Self::ValueType {
        self.store(value, Ordering::Relaxed);
        value
    }

    fn increment(&self) -> Option<usize> {
        self.store_relaxed(self.load_relaxed().checked_add(1)?)
            .into()
    }
}
