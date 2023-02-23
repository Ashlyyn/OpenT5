#![feature(never_type)]
#![feature(io_error_more)]
#![feature(const_option)]
#![feature(int_roundings)]

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::iter_nth_zero)]
#![deny(missing_debug_implementations)]

use discord_rich_presence::activity::{Activity};
use lazy_static::lazy_static;
use std::{sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
}};
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
mod pb;
mod console;
mod conbuf;
mod discord_rpc;

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
    locale::init();

    #[allow(clippy::collapsible_if)]
    if &cmdline[0..9] != "allowdupe"
        || cmdline.chars().nth(9).unwrap_or(' ') > ' '
    {
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

    discord_rpc::set_activity(Activity::new().state("Testing...")).unwrap();

    // ========================================================================
    // This is probably the most opaque part of the program so far, so some
    // explanation is in order
    //
    // winit requires the window to be spawned on the main thread due to
    // restrictions on certain platforms (e.g. macOS). One's first instinct
    // then might be to simply thread the rest of the program off to a
    // different thread and let winit consume the main thread. However,
    // the window creation is buried fairly deep in the initialization code,
    // so we can't create the window without the requisite initialization being
    // complete beforehand. One might then think that we should just remove the
    // actual window creation (i.e. the call to render::create_window_2) from
    // com::init's code path, and call it here in main after com::init returns.
    // However, com::init has *a lot* of work to do after the window is
    // created, *and* later initialization code might require that the window
    // already be created.
    //
    // So, we must have the window created before initialization continues.
    //
    // How do we do that then?
    //
    // The answer: Synchronization.
    //
    // Specifically, we thread off com::init and let its thread hit the point
    // where render::create_window_2 should be called, blocking the main thread
    // until then. Instead of calling render::create_window_2 in com::init's
    // code path, we have that thread signal the main thread that the window is
    // ready to be initialized. We then call render::create_window_2 from the
    // main thread. render::create_window_2 will initialize the window, and
    // then signal the other thread that the initialization is complete, at
    // which point the other thread will continue its execution and the main
    // thread will be consumed by the window.
    //
    // The general sequence order goes something like the following, if it's
    // more sensible than my ramblings here:
    //
    // main thread: ... -> spawn init thread -> wait for signal  ->
    // init thread: ........................ -> com::init -> ... ->
    //
    // main thread cont 1: ........................................... ->
    // init thread cont 1: render::create_window -> signal main thread ->
    //
    // main thread cont 2: signaled, render::create_window_2 -> ... ->
    // init thread cont 2: wait for signal -> ........................
    //
    // main thread cont 3: signal init thread -> window stuff forever
    // init thread cont 3: .................. -> continue init
    //
    // (just imagine the main thread and init thread lines are all one
    // continuous line, I just split them to avoid passing the
    // 80 character limit)

    // Here we spawn com::init
    std::thread::spawn(|| {
        com::init();
    });

    com::println(0.into(), &format!(
        "{}: com::init spawned, looping until ready for window init...",
        std::thread::current().name().unwrap_or("main")
    ));

    // The loop here is necessary so that the lock isn't continuously held,
    // otherwise we run into a deadlock where render::create_window tries to
    // access render::WINDOW_AWATING_INIT to signal to the main thread that
    // it's ready for render::create_window_2 to be called, but the lock on
    // render::AWAITING_WINDOW_INIT is already held in main.
    loop {
        {
            let lock = render::WINDOW_AWAITING_INIT.clone();
            let mut writer = lock.write().unwrap();
            // loop until it's ready
            if writer.try_acknowledge().is_some() {
                break;
            }
        }
    }

    // render::create_window_2 needs a gfx::WindowParms to be passed to it.
    // Normally, a gfx::WindowParms is created early in the initialization of
    // the render module and passed through the call chain until it hits
    // render::create_window_2. However, since we're calling
    // render::create_window_2 from main, we need to retrieve that structure
    // manually. render::create_window stores it in render::WND_PARMS right
    // before it signals the main thread to call render::create_window_2.
    // Thus we can just take it from there.
    let lock = render::WND_PARMS.clone();
    let mut wnd_parms = *lock.write().unwrap();

    com::println(0.into(), &format!(
        "{}: ready for window init, creating window...",
        std::thread::current().name().unwrap_or("main")
    ));

    // Finally, we send the main thread off to die in render::create_window_2.
    // Anything past this point will only execute if window creation fails.
    // If it succeeds, winit will call std::process::exit when the
    // window is destroyed, due to another set of platform restrictions
    match render::create_window_2(&mut wnd_parms) {
        Ok(_) => unreachable!(),
        Err(_) => panic!("failed to create window")
    }
    // ========================================================================
}
