#![allow(dead_code)]

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
    condvar: Condvar,
    mutex: Mutex<(bool, T)>,
}

impl<T: Sized + Clone> SmpEvent<T> {
    pub fn new(state: T, signaled: bool) -> Self {
        Self {
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

    pub fn set_signaled(&mut self) -> bool {
        match &mut self.mutex.lock() {
            Ok(g) => {
                g.0 = true;
                true
            }
            Err(_) => false,
        }
    }

    pub fn clear_signaled(&mut self) -> bool {
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

    pub fn set_state(&mut self, state: T) -> bool {
        match &mut self.mutex.lock() {
            Ok(g) => {
                g.1 = state;
                true
            }
            Err(_) => false,
        }
    }

    pub fn try_set_state(&mut self, state: T) -> bool {
        match &mut self.mutex.try_lock() {
            Ok(g) => {
                g.1 = state;
                true
            }
            Err(_) => false,
        }
    }

    pub fn wait(&self) -> bool {
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

    pub fn try_wait(&self) -> bool {
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

    pub fn wait_timeout(&self, timeout: Duration) -> bool {
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

    pub fn try_wait_timeout(&self, timeout: Duration) -> bool {
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

    pub fn wait_while<F: FnMut(&mut (bool, T)) -> bool>(
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

    pub fn try_wait_while<F: FnMut(&mut (bool, T)) -> bool>(
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

    pub fn wait_timeout_while<F: FnMut(&mut (bool, T)) -> bool>(
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

    pub fn try_wait_timeout_while<F: FnMut(&mut (bool, T)) -> bool>(
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

    pub fn wait_until_signaled(&self) -> bool {
        self.wait_while(|(signaled, _)| *signaled != true)
    }

    pub fn try_wait_until_signaled(&self) -> bool {
        self.try_wait_while(|(signaled, _)| *signaled != true)
    }

    pub fn try_wait_until_signaled_timeout(&self, timeout: Duration) -> bool {
        self.try_wait_timeout_while(timeout, |(signaled, _)| *signaled != true)
    }

    pub fn notify_one(&self) {
        self.condvar.notify_one();
    }

    pub fn notify_all(&self) {
        self.condvar.notify_all();
    }
}
