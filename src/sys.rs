use lazy_static::lazy_static;
use std::{sync::atomic::{AtomicBool, AtomicIsize, Ordering::SeqCst}, time::SystemTime};

lazy_static! {
    static ref BASE_TIME_ACQUIRED: AtomicBool = AtomicBool::new(false);
    pub static ref TIME_BASE: AtomicIsize = AtomicIsize::new(0);
}

pub fn milliseconds() -> isize {
    if BASE_TIME_ACQUIRED.load(SeqCst) == false {
        let now = SystemTime::now();
        let time = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
        TIME_BASE.store(time.try_into().unwrap(), SeqCst);
    }

    let time: isize = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().try_into().unwrap();
    time - TIME_BASE.load(SeqCst)
}