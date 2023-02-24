use std::sync::atomic::{AtomicBool, Ordering};

static NETWORKING_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn networking_enabled() -> bool {
    NETWORKING_ENABLED.load(Ordering::Relaxed)
}

fn config(_enabled: bool) {
    todo!()
}

pub fn restart() {
    config(networking_enabled())
}
