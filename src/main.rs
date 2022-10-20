#![allow(non_snake_case, clippy::bool_comparison)]
#![feature(thread_spawn_unchecked)]
#![feature(never_type)]

use lazy_static::lazy_static;
use std::fmt::Write;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
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
mod render;
mod seh;
mod sys;
mod vid;
mod rb;

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

    /*
    if &cmdline[0..9] != "allowdupe" || cmdline.chars().nth(9).unwrap_or(' ') > ' ' {
        if !cmdline.contains("g_connectpaths 3") {
            if sys::check_crash_or_rerun() == false {
                return;
            }
        }
    }
    */

    if cmdline.contains("nosnd") {
        S_NOSND.store(true, Ordering::SeqCst)
    }

    dvar::init();
    //println!("{}", pollster::block_on(sys::find_info()));

    std::thread::spawn(|| {
        com::init();
    });

    println!("com::init spawned, looping until ready for window init...");
    loop {
        {
            let reader = render::AWAITING_WINDOW_INIT.read().expect("");
            let guard = reader.0.lock().unwrap();
            reader.1.wait(guard).unwrap();
            if *reader.0.lock().unwrap() == true {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    
    println!("ready for window init, getting wnd_parms...");
    let mut wnd_parms = {
        let wnd_parms_lock = render::WND_PARMS.clone();
        let wnd_parms_writer = wnd_parms_lock.read().expect("");
        *wnd_parms_writer
    };

    println!("wnd_parms retrieved, creating window...");
    render::create_window_2(&mut wnd_parms);

    {
        let wnd_parms_lock = render::WND_PARMS.clone();
        let mut wnd_parms_writer = wnd_parms_lock.write().expect("");
        *wnd_parms_writer = wnd_parms;
    };
}
