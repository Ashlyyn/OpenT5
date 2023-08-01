// macOS can use AppKit with the `macos_use_appkit` feature enabled. Otherwise
// we default to Xlib.

#[link(name = "Cocoa", kind = "framework")]
extern "C" {}

use core::ptr::NonNull;

use icrate::{
    AppKit::{
        NSApplication, NSApplicationDelegate, NSApplicationTerminateReply,
        NSResponder, NSWindowDelegate, NSEvent,
    },
    Foundation::NSNotification,
};
use objc2::{
    declare_class, msg_send, msg_send_id,
    mutability::InteriorMutable,
    rc::Id,
    runtime::{NSObject, NSObjectProtocol},
    ClassType,
};

use crate::sys::{self, WindowEvent};

pub fn init() {}

pub struct AppKitGlobals {
    app: Id<NSApplication>,
}

declare_class!(
    #[derive(Debug)]
    pub struct AppDelegate;

    unsafe impl ClassType for AppDelegate {
        #[inherits(NSObject)]
        type Super = NSResponder;
        type Mutability = InteriorMutable;
        const NAME: &'static str = "AppDelegate";
    }

    unsafe impl AppDelegate {
        #[method(init)]
        unsafe fn init(this: *mut Self) -> Option<NonNull<Self>> {
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl NSApplicationDelegate for AppDelegate {
        #[method(applicationShouldTerminate:)]
        #[allow(non_snake_case)]
        unsafe fn applicationShouldTerminate(
            &self,
            _sender: &NSApplication,
        ) -> NSApplicationTerminateReply {
            0
        }
    }
);

impl AppDelegate {
    pub fn new() -> Id<Self> {
        unsafe { msg_send_id![Self::alloc(), init] }
    }
}

declare_class!(
    #[derive(Debug)]
    pub struct WindowDelegate;

    unsafe impl ClassType for WindowDelegate {
        #[inherits(NSObject)]
        type Super = NSResponder;
        type Mutability = InteriorMutable;
        const NAME: &'static str = "WindowDelegate";
    }

    unsafe impl WindowDelegate {
        #[method(init)]
        unsafe fn init(this: *mut Self) -> Option<NonNull<Self>> {
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for WindowDelegate {}

    unsafe impl NSWindowDelegate for WindowDelegate {
        #[method(windowWillClose:)]
        unsafe fn window_will_close(&self, _sender: &NSNotification) {
            sys::set_quit_event();
        }
    }
);

impl WindowDelegate {
    pub fn new() -> Id<Self> {
        unsafe { msg_send_id![Self::alloc(), init] }
    }
}

impl TryFrom<&NSEvent> for WindowEvent {
    type Error = ();
    fn try_from(value: &NSEvent) -> Result<Self, Self::Error> {
        match unsafe { value.r#type() } {
            _=> todo!()
        }
    }
}