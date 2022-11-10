//#![allow(dead_code)]

use cfg_if::cfg_if;
use std::time::Duration;

cfg_if! {
    if #[cfg(debug_assertions)] {
        use no_deadlocks::{Condvar, Mutex};
    } else {
        use std::sync::{Condvar, Mutex};
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

    fn set_signaled(&mut self) -> bool {
        match &mut self.mutex.lock() {
            Ok(g) => {
                g.0 = true;
                true
            }
            Err(_) => false,
        }
    }

    fn clear_signaled(&mut self) -> bool {
        match &mut self.mutex.lock() {
            Ok(g) => {
                g.0 = false;
                true
            }
            Err(_) => false,
        }
    }

    pub fn get_state(&self) -> Option<T> {
        match self.mutex.lock() {
            Ok(g) => Some(g.1.clone()),
            Err(_) => None,
        }
    }

    pub fn try_get_state(&self) -> Option<T> {
        match self.mutex.try_lock() {
            Ok(g) => Some(g.1.clone()),
            Err(_) => None,
        }
    }

    fn set_state(&mut self, state: T) -> bool {
        match &mut self.mutex.lock() {
            Ok(g) => {
                g.1 = state;
                true
            }
            Err(_) => false,
        }
    }

    fn try_set_state(&mut self, state: T) -> bool {
        match &mut self.mutex.try_lock() {
            Ok(g) => {
                g.1 = state;
                true
            }
            Err(_) => false,
        }
    }

    fn wait(&self) -> bool {
        let guard = match self.mutex.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        #[allow(unused_must_use)]
        {
            self.condvar.wait(guard).unwrap();
        }
        true
    }

    fn try_wait(&self) -> bool {
        let guard = match self.mutex.try_lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        #[allow(unused_must_use)]
        {
            self.condvar.wait(guard).unwrap();
        }
        true
    }

    fn wait_timeout(&self, timeout: Duration) -> bool {
        let guard = match self.mutex.lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        #[allow(unused_must_use)]
        {
            self.condvar.wait_timeout(guard, timeout).unwrap();
        }
        true
    }

    fn try_wait_timeout(&self, timeout: Duration) -> bool {
        let guard = match self.mutex.try_lock() {
            Ok(g) => g,
            Err(_) => return false,
        };

        #[allow(unused_must_use)]
        {
            self.condvar.wait_timeout(guard, timeout).unwrap();
        }
        true
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
            true => self.get_state(),
            false => None,
        };

        if !self.manual_reset {
            self.clear_signaled();
        }

        res
    }

    /// Tries to acknowledge the event, and retrieves its internal state
    /// if successful.
    #[allow(dead_code)]
    pub fn try_acknowledge(&mut self) -> Option<T> {
        let res = match self.try_wait() {
            true => self.try_get_state(),
            false => None,
        };

        if !self.manual_reset {
            self.clear_signaled();
        }

        res
    }

    /// Acknowledges the event within [`duration`], and retrieves its
    /// internal state if successful.
    pub fn acknowledge_timeout(&mut self, duration: Duration) -> Option<T> {
        let res = match self.wait_timeout(duration) {
            true => self.get_state(),
            false => None,
        };

        if !self.manual_reset {
            self.clear_signaled();
        }

        res
    }

    /// Tries to acknowledge the event within [`duration`], and retrieves its
    /// internal state if successful.
    #[allow(dead_code)]
    pub fn try_acknowledge_timeout(&mut self, duration: Duration) -> Option<T> {
        let res = match self.try_wait_timeout(duration) {
            true => self.try_get_state(),
            false => None,
        };

        if !self.manual_reset {
            self.clear_signaled();
        }

        res
    }

    /// Sets the event's internal state and sets the signaled state.
    pub fn send(&mut self, state: T) -> bool {
        if !self.set_state(state) {
            return false;
        }
        if !self.set_signaled() {
            return false;
        }

        self.notify_one();
        true
    }

    /// Tries to set the event's internal state, and sets the signaled state
    /// if successful.
    #[allow(dead_code)]
    pub fn try_send(&mut self, state: T) -> bool {
        if !self.try_set_state(state) {
            return false;
        }
        if !self.set_signaled() {
            return false;
        }

        self.notify_one();
        true
    }

    /// Sets the event's internal state, and clears the signaled state
    /// if successful.
    pub fn send_cleared(&mut self, state: T) -> bool {
        if !self.set_state(state) {
            return false;
        }
        if !self.clear_signaled() {
            return false;
        }

        self.notify_one();
        true
    }

    /// Tries to set the event's internal state, and clears the signaled state
    /// if successful.
    #[allow(dead_code)]
    pub fn try_send_cleared(&mut self, state: T) -> bool {
        if !self.try_set_state(state) {
            return false;
        }
        if !self.clear_signaled() {
            return false;
        }

        self.notify_one();
        true
    }
}
