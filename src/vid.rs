use crate::*;

pub fn app_activate(active_app: bool, is_minimized: bool) {
    if is_minimized {
        platform::set_minimized();
    } else {
        platform::clear_minimized();
    }

    println!("TODO: key::clear_states");

    if active_app == false {
        platform::clear_active_app();
    } else {
        platform::set_active_app();
        println!("TODO: com::touch_memory");
    }

    // _DAT_027706dc = 0;
}
