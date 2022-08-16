#![allow(non_snake_case, clippy::bool_comparison)]

use lazy_static::lazy_static;
use std::fmt::Write;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
extern crate num_derive;

mod cbuf;
mod cmd;
mod com;
mod common;
mod dvar;
mod fs;
mod gfx;
mod input;
mod locale;
mod platform;
mod pmem;
mod render;
mod sys;
mod vid;

lazy_static! {
    #[allow(dead_code)]
    static ref G_ALLOW_MATURE: AtomicBool = AtomicBool::new(true);
    static ref S_NOSND: AtomicBool = AtomicBool::new(false);
}

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
    println!("{}", pollster::block_on(sys::find_info()));

    // std::thread::spawn(|| {
    let mut window_parms = gfx::WindowParms {
        window_handle: platform::WindowHandle::new(),
        hz: 60,
        fullscreen: false,
        x: 0,
        y: 0,
        scene_height: 480,
        scene_width: 640,
        display_width: 1920,
        display_height: 1080,
        aa_samples: 0,
    };

    render::create_window(&mut window_parms);
    // });
}
