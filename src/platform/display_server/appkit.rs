// macOS can use AppKit with the `macos_use_appkit` feature enabled. Otherwise
// we default to Xlib.

#![allow(non_upper_case_globals)]

#[link(name = "Cocoa", kind = "framework")]
extern "C" {}

use core::ptr::{addr_of_mut, NonNull};
use std::ptr::addr_of;

use icrate::{
    AppKit::{
        NSApp, NSApplication, NSApplicationDelegate,
        NSApplicationTerminateReply, NSEvent, NSEventTypeApplicationDefined,
        NSResponder, NSWindowDelegate, NSWindow, NSBitsPerPixelFromDepth, NSEventTypeKeyDown, NSEventTypeKeyUp, NSEventTypeLeftMouseDown, NSEventTypeLeftMouseUp, NSEventTypeRightMouseDown, NSEventTypeRightMouseUp, NSEventTypeOtherMouseDown, NSEventTypeOtherMouseUp, NSEventTypeScrollWheel, NSEventTypeMouseMoved, NSEventTypeFlagsChanged,
    },
    Foundation::{CGPoint, NSDate, NSNotification, NSSize, NSRect, CGSize},
};
use objc2::{
    declare::{Ivar, IvarEncode},
    declare_class,
    ffi::NSInteger,
    msg_send, msg_send_id,
    mutability::InteriorMutable,
    rc::Id,
    runtime::{NSObject, NSObjectProtocol},
    ClassType,
};

use crate::sys::{KeyboardScancode, WindowEvent, self};

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
            sys::set_quit_event();
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
    pub struct WindowDelegate {
        id: IvarEncode<NSInteger, "_window_id">,
    }

    mod ivars;

    unsafe impl ClassType for WindowDelegate {
        #[inherits(NSObject)]
        type Super = NSResponder;
        type Mutability = InteriorMutable;
        const NAME: &'static str = "WindowDelegate";
    }

    unsafe impl WindowDelegate {
        #[method(initWithId:)]
        unsafe fn init_with_id(
            this: *mut Self,
            window_id: NSInteger,
        ) -> Option<NonNull<Self>> {
            let this: Option<&mut Self> =
                unsafe { msg_send![super(this), init] };

            if let Some(this) = this {
                Ivar::write(&mut this.id, window_id);
                NonNull::new(addr_of_mut!(*this))
            } else {
                None
            }
        }
    }

    unsafe impl NSObjectProtocol for WindowDelegate {}

    unsafe impl NSWindowDelegate for WindowDelegate {
        #[method(windowShouldClose:)]
        unsafe fn window_should_close(&self, _sender: &NSWindow) -> bool {
            self.post_event(WindowEvent::CloseRequested);
            true
        }

        #[method(windowWillClose:)]
        unsafe fn window_will_close(&self, _sender: &NSNotification) {
            self.post_event(WindowEvent::Destroyed);
        }

        #[method(windowDidMove:)]
        unsafe fn window_did_move(&self, notification: &NSNotification) {
            let object = &*notification.object().unwrap();
            // The "object" field of NSNotification is a pointer to an NSWindow
            // for this handler, we just have to "upcast" it ourselves. 
            let window = &*(addr_of!(*object) as *mut NSWindow);
            let pos = window.frame().origin;
            self.post_event(
                WindowEvent::Moved { x: pos.x as _, y: pos.y as _ }
            );
        }

        #[method(windowWillResize:toSize:)]
        unsafe fn window_will_resize(
            &self, _sender: &NSWindow, frame_size: NSSize
        ) -> NSSize {
            self.post_event(
                WindowEvent::Resized { 
                    width: frame_size.width as _, 
                    height: frame_size.height as _ 
                }
            );
            frame_size
        }

        #[method(windowDidBecomeKey:)]
        unsafe fn window_did_become_key(&self, _notification: &NSNotification)
        {
            self.post_event(WindowEvent::SetFocus);
            // Until we come up with a better solution, we'll just fire both
            // SetFocus and Activated here.
            self.post_event(WindowEvent::Activate);
        }

        #[method(windowDidResignKey:)]
        unsafe fn window_did_resign_key(&self, _notification: &NSNotification)
        {
            self.post_event(WindowEvent::KillFocus);
            // Same story here, with KillFocus and Deactivate.
            self.post_event(WindowEvent::Deactivate);
        }

        #[method(windowDidChangeScreen:)]
        unsafe fn window_did_change_screen(
            &self, notification: &NSNotification
        ) {
            let object = &*notification.object().unwrap();
            // The "object" field of NSNotification is a pointer to an NSWindow
            // for this handler, we just have to "upcast" it ourselves. 
            let window = &*(addr_of!(*object) as *mut NSWindow);
            let screen = window.screen().unwrap();
            let bits_per_pixel = 
                NSBitsPerPixelFromDepth(screen.depth()) as u32;
            let horz_res = screen.frame().size.width as u32;
            let vert_res = screen.frame().size.height as u32;
            self.post_event(
                WindowEvent::DisplayChange { 
                    bits_per_pixel, horz_res, vert_res 
                }
            );
        }
    }
);

impl WindowDelegate {
    pub fn new(window_id: NSInteger) -> Id<Self> {
        unsafe { msg_send_id![Self::alloc(), initWithId:window_id] }
    }

    pub fn window_id(&self) -> NSInteger {
        *self.id
    }

    fn make_event(ev: WindowEvent, w_num: NSInteger) -> Id<NSEvent> {
        let ev = Box::new(ev);
        unsafe { NSEvent::otherEventWithType_location_modifierFlags_timestamp_windowNumber_context_subtype_data1_data2(
            NSEventTypeApplicationDefined, CGPoint::new(0.0, 0.0), 0,
            NSDate::now().timeIntervalSince1970(), w_num, None, 0,
            Box::into_raw(ev) as _, 0
        ) }.unwrap()
    }

    fn post_event(&self, ev: WindowEvent) {
        unsafe {
            NSApp.unwrap().postEvent_atStart(
                &Self::make_event(ev, self.window_id()),
                false,
            )
        };
    }
}

#[derive(Copy, Clone, Debug)]
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
            0x24 => Ok(KeyboardScancode::Enter),
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

            0x30 => Ok(KeyboardScancode::Tab),
            0x31 => Ok(KeyboardScancode::Space),
            0x32 => Ok(KeyboardScancode::Tilde),
            0x33 => Ok(KeyboardScancode::Backspace),
            0x35 => Ok(KeyboardScancode::Esc),
            0x37 => Ok(KeyboardScancode::LSys),
            0x38 => Ok(KeyboardScancode::LShift),
            0x39 => Ok(KeyboardScancode::CapsLk),
            0x3A => Ok(KeyboardScancode::LAlt),
            0x3B => Ok(KeyboardScancode::LCtrl),
            0x3C => Ok(KeyboardScancode::RShift),
            0x3D => Ok(KeyboardScancode::RAlt),
            0x3E => Ok(KeyboardScancode::RCtrl),
            0x3F => Ok(KeyboardScancode::Fn),
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

            _ => Err(()),
        }
    }
}

struct NSModifiers(u16);

impl TryFrom<NSModifiers> for sys::Modifiers {
    type Error = ();
    fn try_from(value: NSModifiers) -> Result<Self, Self::Error> {
        let mut modifiers = sys::Modifiers::empty();
        if value.0 & 0x02 != 0 {
            modifiers |= sys::Modifiers::LSHIFT;
        }

        if value.0 & 0x04 != 0 {
            modifiers |= sys::Modifiers::LCTRL;
        }

        if value.0 & 0x08 != 0 {
            modifiers |= sys::Modifiers::LALT;
        }

        if value.0 & 0x10 != 0 {
            modifiers |= sys::Modifiers::LSYS;
        }

        Ok(modifiers)
    }
}

impl TryFrom<&NSEvent> for WindowEvent {
    type Error = ();
    fn try_from(value: &NSEvent) -> Result<Self, Self::Error> {
        match unsafe { value.r#type() } {
            // Some events can't be caught in our event loop and are only
            // sent to the window delegate instead. We created handlers above
            // to catch those events and post them to the event loop ourselves.
            // They all have a type of [`NSEventTypeApplicationDefined`].
            NSEventTypeApplicationDefined => {
                // Sanity check - whenever we post events, subtype and d2 are
                // always set to 0. If they aren't zero, we're getting events
                // we shouldn't be getting.
                assert!(unsafe { value.subtype() == 0 && value.data2() == 0 });
                // Additionally, d1 will contain a pointer (obtained in the
                // WindowDelegate handlers from [`Box::into_raw`]) to the
                // translated [`WindowEvent`]. We don't want to deref null,
                // so first we make sure it's not null...
                assert_ne!(unsafe { value.data1() }, 0);
                // And if it's not, and the previous sanity check succeeded
                // (we wouldn't be here if it hadn't), we'll assume the event
                // *is* in fact one we created in the WindowDelegate handlers,
                // and therefore valid. If it's not, I don't know what to say.
                // Maybe we should change subtype and/or d2 to some magic
                // numbers for extra security? 
                let ev = unsafe { Box::<Self>::from_raw(value.data1() as _) };
                // The [`Box`] created in the above line will take ownership of
                // the pointer that we relinquished in the WindowDelegate
                // handlers and deallocate when it's dropped in this function,
                // so no memory will be leaked.
                Ok(*ev)
            }
            // Other events, like keyboard and mouse events are sent straight
            // to the event loop, and we'll translate them here.
            NSEventTypeKeyDown => {
                let code = KeyCode(unsafe { value.keyCode() });
                Ok(WindowEvent::KeyDown { 
                    logical_scancode: code.try_into().unwrap(),
                    physical_scancode: Some(code.try_into().unwrap()),
                })
            },
            NSEventTypeKeyUp => {
                let code = KeyCode(unsafe { value.keyCode() });
                Ok(WindowEvent::KeyUp { 
                    logical_scancode: code.try_into().unwrap(),
                    physical_scancode: Some(code.try_into().unwrap()),
                })
            },
            NSEventTypeLeftMouseDown => Ok(WindowEvent::MouseButtonDown(sys::MouseScancode::LClick)),
            NSEventTypeLeftMouseUp => Ok(WindowEvent::MouseButtonUp(sys::MouseScancode::LClick)),
            NSEventTypeRightMouseDown => Ok(WindowEvent::MouseButtonDown(sys::MouseScancode::RClick)),
            NSEventTypeRightMouseUp => Ok(WindowEvent::MouseButtonUp(sys::MouseScancode::RClick)),
            NSEventTypeOtherMouseDown => match unsafe { value.buttonNumber() } {
                2 => Ok(WindowEvent::MouseButtonDown(sys::MouseScancode::MClick)),
                _ => Err(())
            },
            NSEventTypeOtherMouseUp => match unsafe { value.buttonNumber() } {
                2 => Ok(WindowEvent::MouseButtonUp(sys::MouseScancode::MClick)),
                _ => Err(())
            },
            NSEventTypeScrollWheel => {
                let scroll_factor = if unsafe { value.hasPreciseScrollingDeltas() } {
                    0.1
                } else {
                    1.0
                };
                let dy = unsafe { value.scrollingDeltaY() } * scroll_factor;
                Ok(WindowEvent::MouseWheelScroll(dy as _))
            },
            NSEventTypeMouseMoved => {
                let current_window = unsafe { NSApp.unwrap().keyWindow() }.unwrap();
                    let current_window_content_view =
                        unsafe { current_window.contentView().unwrap() };
                    let adjust_frame = unsafe { current_window_content_view.frame() };
                    let p = unsafe { current_window.mouseLocationOutsideOfEventStream() };
                    let p = CGPoint::new(
                        p.x.clamp(0.0, adjust_frame.size.width),
                        p.y.clamp(0.0, adjust_frame.size.height),
                    );
                    let r = NSRect::new(p, CGSize::new(0.0, 0.0));
                    let r = unsafe { current_window_content_view.convertRectToBacking(r) };
                    let p = r.origin;
                    Ok(WindowEvent::CursorMoved { x: p.x, y: p.y })
            },
            NSEventTypeFlagsChanged => todo!(),
            _ => todo!(),   
        }
    }
}
