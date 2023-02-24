#![allow(dead_code)]

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

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
        use std::ffi::c_char;
    }
}

pub struct SmpEvent<T: Sized> {
    manual_reset: bool,
    condvar: Condvar,
    mutex: Mutex<(bool, T)>,
}

impl<T: Sized + Clone> SmpEvent<T> {
    /// Creates a new SmpEvent
    ///
    /// # Arguments
    /// * `state` - The initial internal state for the object.
    /// * `signaled` - Whether or not to initialize
    /// the event in the signaled state.
    /// * `manual_reset` - Whether or not the event has to be manually-reset.
    /// If [`false`], [`Self::acknowledge`] and its variants will clear the
    /// signaled state. If not, this must be done manually.
    pub fn new(state: T, signaled: bool, manual_reset: bool) -> Self {
        Self {
            manual_reset,
            condvar: Condvar::new(),
            mutex: Mutex::new((signaled, state)),
        }
    }

    pub fn signaled(&self) -> Option<bool> {
        match self.mutex.lock() {
            Ok(g) => Some(g.0),
            Err(_) => None,
        }
    }

    fn set_signaled(&mut self) -> Result<(), ()> {
        match &mut self.mutex.lock() {
            Ok(g) => {
                g.0 = true;
                Ok(())
            }
            Err(_) => Err(()),
        }
    }

    fn clear_signaled(&mut self) -> Result<(), ()> {
        match &mut self.mutex.lock() {
            Ok(g) => {
                g.0 = false;
                Ok(())
            }
            Err(_) => Err(()),
        }
    }

    pub fn get_state(&self) -> Result<T, ()> {
        match self.mutex.lock() {
            Ok(g) => Ok(g.1.clone()),
            Err(_) => Err(()),
        }
    }

    pub fn try_get_state(&self) -> Result<T, ()> {
        match self.mutex.try_lock() {
            Ok(g) => Ok(g.1.clone()),
            Err(_) => Err(()),
        }
    }

    fn set_state(&mut self, state: T) -> Result<(), ()> {
        match &mut self.mutex.lock() {
            Ok(g) => {
                g.1 = state;
                Ok(())
            }
            Err(_) => Err(()),
        }
    }

    fn try_set_state(&mut self, state: T) -> Result<(), ()> {
        match &mut self.mutex.try_lock() {
            Ok(g) => {
                g.1 = state;
                Ok(())
            }
            Err(_) => Err(()),
        }
    }

    fn wait(&self) -> Result<(), ()> {
        let guard = match self.mutex.lock() {
            Ok(g) => g,
            Err(_) => return Err(()),
        };

        #[allow(unused_must_use)]
        {
            self.condvar.wait(guard).unwrap();
        }
        Ok(())
    }

    fn try_wait(&self) -> Result<(), ()> {
        let guard = match self.mutex.try_lock() {
            Ok(g) => g,
            Err(_) => return Err(()),
        };

        #[allow(unused_must_use)]
        {
            self.condvar.wait(guard).unwrap();
        }
        Ok(())
    }

    fn wait_timeout(&self, timeout: Duration) -> Result<(), ()> {
        let guard = match self.mutex.lock() {
            Ok(g) => g,
            Err(_) => return Err(()),
        };

        #[allow(unused_must_use)]
        {
            self.condvar.wait_timeout(guard, timeout).unwrap();
        }
        Ok(())
    }

    fn try_wait_timeout(&self, timeout: Duration) -> Result<(), ()> {
        let guard = match self.mutex.try_lock() {
            Ok(g) => g,
            Err(_) => return Err(()),
        };

        #[allow(unused_must_use)]
        {
            self.condvar.wait_timeout(guard, timeout).unwrap();
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
    pub fn acknowledge(&mut self) -> Option<T> {
        let res = match self.wait() {
            Ok(_) => self.get_state(),
            Err(_) => Err(()),
        };

        if !self.manual_reset {
            self.clear_signaled().unwrap();
        }

        res.ok()
    }

    /// Tries to acknowledge the event, and retrieves its internal state
    /// if successful.
    #[allow(dead_code)]
    pub fn try_acknowledge(&mut self) -> Option<T> {
        if match self.signaled() {
            Some(s) => s,
            None => return None,
        } == false
        {
            return None;
        }

        if !self.manual_reset {
            self.clear_signaled().unwrap();
        }

        match self.try_get_state() {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    /// Acknowledges the event within [`duration`], and retrieves its
    /// internal state if successful.
    pub fn acknowledge_timeout(&mut self, duration: Duration) -> Option<T> {
        let res = match self.wait_timeout(duration) {
            Ok(_) => self.get_state(),
            Err(_) => Err(()),
        };

        if !self.manual_reset {
            self.clear_signaled().unwrap();
        }

        res.ok()
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
            self.clear_signaled().unwrap();
        }

        res.ok()
    }

    /// Sets the event's internal state and sets the signaled state.
    pub fn send(&mut self, state: T) -> Result<(), ()> {
        self.set_state(state)?;
        self.set_signaled()?;
        self.notify_one();
        Ok(())
    }

    /// Tries to set the event's internal state, and sets the signaled state
    /// if successful.
    #[allow(dead_code)]
    pub fn try_send(&mut self, state: T) -> Result<(), ()> {
        self.try_set_state(state)?;
        self.set_signaled()?;
        self.notify_one();
        Ok(())
    }

    /// Sets the event's internal state, and clears the signaled state
    /// if successful.
    pub fn send_cleared(&mut self, state: T) -> Result<(), ()> {
        self.set_state(state)?;
        self.clear_signaled()?;
        self.notify_one();
        Ok(())
    }

    /// Tries to set the event's internal state, and clears the signaled state
    /// if successful.
    #[allow(dead_code)]
    pub fn try_send_cleared(&mut self, state: T) -> Result<(), ()> {
        self.try_set_state(state)?;
        self.clear_signaled()?;
        self.notify_one();
        Ok(())
    }
}

/// Wrapper for a dynamic library loaded at runtime
pub struct Module {
    // In the future, I woud like to make the inner member a &[u8] rather than
    // a thin pointer, but Windows doesn't make getting the size of a loaded
    // library easier, so we're just going to use a pointer (which will work 
    // on both Windows and Unix platforms)
    ptr: *mut ()
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

                unsafe { LoadLibraryW(PCWSTR(name)) }.ok().map(|h| Module { ptr: h.0 as *mut () })
            }

            /// Unloads the library loaded by [`Module::load`]. Should only be 
            /// used when dropped.
            fn unload(&mut self) {
                unsafe { FreeLibrary(HINSTANCE(self.ptr as _)) };
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
                let name = name.as_ptr() as *const c_char;

                let ptr = unsafe { dlopen(name, RTLD_NOW) } as *mut ();
                if ptr.is_null() {
                    None
                } else {
                    Some(Module { ptr })
                }
            }

            /// Unloads the library loaded by [`Module::load`]. Should only be 
            /// used when dropped.
            fn unload(&mut self) {
                unsafe { dlclose(self.ptr as *mut _) };
            }
        } else {
            pub fn load(name: Path) -> Option<Self> {
                todo!()
            }

            fn unload(&mut self) {
                todo!()
            }
        }
    }
}

impl Drop for Module {
    /// Unloads the module when dropped.
    fn drop(&mut self) {
        self.unload()
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

// Made this because I got tired of importing std::sync::atomic::Ordering
// and passing the exact same Ordering (Ordering::Relaxed) 99% of the time.
// Purely a convenience thing, absolutely meaningless in terms of 
// functionality
pub trait EasierAtomic {
    type ValueType;
    fn load_relaxed(&self) -> Self::ValueType;
    fn store_relaxed(&self, value: Self::ValueType) -> Self::ValueType;
}

impl EasierAtomic for AtomicBool {
    type ValueType = bool;
    fn load_relaxed(&self) -> Self::ValueType {
        self.load(Ordering::Relaxed)
    }

    fn store_relaxed(&self, value: Self::ValueType) -> Self::ValueType {
        self.store(value, Ordering::Relaxed);
        value
    }
}