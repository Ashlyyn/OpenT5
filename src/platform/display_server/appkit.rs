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

use crate::sys::{self, WindowEvent, KeyboardScancode};

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

struct KeyCode(u16);

impl TryFrom<KeyCode> for KeyboardScancode {
    type Error = ();
    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        match value.0 {
            0x00 => Ok(KeyboardScancode::A),
            0x01 => Ok(KeyboardScancode::S),
            0x02 => Ok(KeyboardScancode::D),
            0x03 => Ok(KeyboardScancode::F),
            0x04 => Ok(KeyboardScancode::H),
            0x05 => Ok(KeyboardScancode::G),
            0x06 => Ok(KeyboardScancode::Z),
            0x07 => Ok(KeyboardScancode::X),
            0x08 => Ok(KeyboardScancode::C),
            0x09 => Ok(KeyboardScancode::V),
            0x0B => Ok(KeyboardScancode::B),
            0x0C => Ok(KeyboardScancode::Q),
            0x0D => Ok(KeyboardScancode::W),
            0x0E => Ok(KeyboardScancode::E),
            0x0F => Ok(KeyboardScancode::R),
            0x10 => Ok(KeyboardScancode::Y),
            0x11 => Ok(KeyboardScancode::T),
            0x12 => Ok(KeyboardScancode::Key1),
            0x13 => Ok(KeyboardScancode::Key2),
            0x14 => Ok(KeyboardScancode::Key3),
            0x15 => Ok(KeyboardScancode::Key4),
            0x16 => Ok(KeyboardScancode::Key6),
            0x17 => Ok(KeyboardScancode::Key5),
            0x18 => Ok(KeyboardScancode::Equals),
            0x19 => Ok(KeyboardScancode::Key9),
            0x1A => Ok(KeyboardScancode::Key7),
            0x1B => Ok(KeyboardScancode::Hyphen),
            0x1C => Ok(KeyboardScancode::Key8),
            0x1D => Ok(KeyboardScancode::Key0),
            0x1E => Ok(KeyboardScancode::OpenBracket),
            0x1F => Ok(KeyboardScancode::O),
            0x20 => Ok(KeyboardScancode::U),
            0x21 => Ok(KeyboardScancode::CloseBracket),
            0x22 => Ok(KeyboardScancode::I),
            0x23 => Ok(KeyboardScancode::P),
            0x25 => Ok(KeyboardScancode::L),
            0x26 => Ok(KeyboardScancode::J),
            0x27 => Ok(KeyboardScancode::Apostrophe),
            0x28 => Ok(KeyboardScancode::K),
            0x29 => Ok(KeyboardScancode::Semicolon),
            0x2A => Ok(KeyboardScancode::BackSlash),
            0x2B => Ok(KeyboardScancode::Comma),
            0x2C => Ok(KeyboardScancode::ForwardSlash),
            0x2D => Ok(KeyboardScancode::N),
            0x2E => Ok(KeyboardScancode::M),
            0x2F => Ok(KeyboardScancode::Period),
            0x32 => Ok(KeyboardScancode::Tilde),
            0x41 => Ok(KeyboardScancode::NumPeriod),
            0x43 => Ok(KeyboardScancode::NumAsterisk),
            0x45 => Ok(KeyboardScancode::NumPlus),
            0x47 => Ok(KeyboardScancode::NumLk),
            0x4B => Ok(KeyboardScancode::NumSlash),
            0x4C => Ok(KeyboardScancode::NumEnter),
            0x4E => Ok(KeyboardScancode::NumHyphen),
            0x52 => Ok(KeyboardScancode::Num0),
            0x53 => Ok(KeyboardScancode::Num1),
            0x55 => Ok(KeyboardScancode::Num3),
            0x56 => Ok(KeyboardScancode::Num4),
            0x57 => Ok(KeyboardScancode::Num5),
            0x58 => Ok(KeyboardScancode::Num6),
            0x59 => Ok(KeyboardScancode::Num7),
            0x5B => Ok(KeyboardScancode::Num8),
            0x5C => Ok(KeyboardScancode::Num9),

            0x24 => Ok(KeyboardScancode::Enter),
            0x30 => Ok(KeyboardScancode::Tab),
            0x31 => Ok(KeyboardScancode::Space),
            0x33 => Ok(KeyboardScancode::Backspace),
            0x35 => Ok(KeyboardScancode::Esc),
            0x37 => Ok(KeyboardScancode::LSys),
            0x38 => Ok(KeyboardScancode::LShift),
            0x39 => Ok(KeyboardScancode::CapsLk),
            0x3A => Ok(KeyboardScancode::LAlt),
            0x3B => Ok(KeyboardScancode::LCtrl),
            0x3C => Ok(KeyboardScancode::RShift),
            0x3D => Ok(KeyboardScancode::Fn),
            0x60 => Ok(KeyboardScancode::F5),
            0x61 => Ok(KeyboardScancode::F6),
            0x62 => Ok(KeyboardScancode::F7),
            0x63 => Ok(KeyboardScancode::F3),
            0x64 => Ok(KeyboardScancode::F8),
            0x65 => Ok(KeyboardScancode::F9),
            0x66 => Ok(KeyboardScancode::F11),
            0x6D => Ok(KeyboardScancode::F10),
            0x6F => Ok(KeyboardScancode::F12),
            0x72 => Ok(KeyboardScancode::Insert),
            0x73 => Ok(KeyboardScancode::Home),
            0x74 => Ok(KeyboardScancode::PgUp),
            0x75 => Ok(KeyboardScancode::Del),
            0x76 => Ok(KeyboardScancode::F4),
            0x77 => Ok(KeyboardScancode::End),
            0x78 => Ok(KeyboardScancode::F2),
            0x79 => Ok(KeyboardScancode::PgDn),
            0x7A => Ok(KeyboardScancode::F1),
            0x7B => Ok(KeyboardScancode::ArrowLeft),
            0x7C => Ok(KeyboardScancode::ArrowRight),
            0x7D => Ok(KeyboardScancode::ArrowDown),
            0x7E => Ok(KeyboardScancode::ArrowUp),

            _ => Err(())
        }
    }
}

impl TryFrom<&NSEvent> for WindowEvent {
    type Error = ();
    fn try_from(value: &NSEvent) -> Result<Self, Self::Error> {
        match unsafe { value.r#type() } {
            _ => todo!()
        }
    }
}