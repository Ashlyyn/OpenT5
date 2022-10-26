#![allow(non_snake_case, clippy::bool_comparison)]
#![feature(thread_spawn_unchecked)]
#![feature(never_type)]
#![feature(local_key_cell_methods)]
#![feature(cstr_from_bytes_until_nul)]

use lazy_static::lazy_static;
use std::fmt::Write;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
extern crate num_derive;

mod cbuf;
mod cl;
mod cmd;
mod com;
mod common;
mod dvar;
mod fs;
mod gfx;
mod input;
mod key;
mod locale;
mod platform;
mod pmem;
mod rb;
mod render;
mod seh;
mod sys;
mod util;
mod vid;

lazy_static! {
    #[allow(dead_code)]
    static ref G_ALLOW_MATURE: AtomicBool = AtomicBool::new(true);
    static ref S_NOSND: AtomicBool = AtomicBool::new(false);
}

#[allow(clippy::collapsible_else_if)]
fn main() {
    platform::os::target::main();
    let cmdline = sys::get_cmdline();
    if cmdline.contains("autominidump") {
        sys::start_minidump(false);
    } else {
        if cmdline.contains("minidump") {
            sys::start_minidump(true);
        } else {
            // Windows top-level exception handler bullshit
        }
    }

    pmem::init();

    #[allow(clippy::collapsible_if)]
    if &cmdline[0..9] != "allowdupe" || cmdline.chars().nth(9).unwrap_or(' ') > ' ' {
        if !cmdline.contains("g_connectpaths 3") {
            if sys::check_crash_or_rerun() == false {
                return;
            }
        }
    }

    if cmdline.contains("nosnd") {
        S_NOSND.store(true, Ordering::SeqCst)
    }

    dvar::init();
    //println!("{}", pollster::block_on(sys::find_info()));

    std::thread::spawn(|| {
        com::init();
    });

    com::println(&format!(
        "{}: com::init spawned, looping until ready for window init...",
        std::thread::current().name().unwrap_or("main")
    ));
    {
        let reader = render::AWAITING_WINDOW_INIT.read().expect("");
        reader.wait_until_signaled();
    }

    com::println(&format!(
        "{}: ready for window init, getting wnd_parms...",
        std::thread::current().name().unwrap_or("main")
    ));
    let mut wnd_parms = {
        let wnd_parms_lock = render::WND_PARMS.clone();
        let wnd_parms_writer = wnd_parms_lock.read().expect("");
        *wnd_parms_writer
    };

    com::println(&format!(
        "{}: wnd_parms retrieved, creating window...",
        std::thread::current().name().unwrap_or("main")
    ));
    render::create_window_2(&mut wnd_parms);

    {
        let wnd_parms_lock = render::WND_PARMS.clone();
        let mut wnd_parms_writer = wnd_parms_lock.write().expect("");
        *wnd_parms_writer = wnd_parms;
    };
}
