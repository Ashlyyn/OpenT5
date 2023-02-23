#![allow(dead_code)]

use std::ffi::c_char;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use std::sync::{Condvar, Mutex};

use libc::dlclose;
use raw_window_handle::HasRawWindowHandle;

use crate::platform::WindowHandle;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_family = "unix")] {
        use std::os::unix::prelude::OsStrExt;
        use libc::{dlopen, RTLD_NOW};
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

pub struct Module {
    ptr: *mut ()
}

impl Module {
    cfg_if! {
        if #[cfg(target_os = "windows")] {
            pub fn load(name: Path) -> Option<Self> {
                todo!()
            }

            pub fn unload(&mut self) {
                todo!()
            }
        } else if #[cfg(target_family = "unix")] {
            pub fn load(name: &Path) -> Option<Self> {
                let ptr = unsafe { dlopen(name.as_os_str().as_bytes().as_ptr() as *const c_char, RTLD_NOW) } as *mut ();
                if ptr.is_null() {
                    None
                } else {
                    Some(Module { ptr })
                }
            }

            fn unload(&mut self) {
                unsafe { dlclose(self.ptr as *mut _) };
            }
        } else {
            pub fn load(name: Path) -> Option<Self> {
                todo!()
            }

            pub fn unload(&mut self) {
                todo!()
            }
        }
    }
}

impl Drop for Module {
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