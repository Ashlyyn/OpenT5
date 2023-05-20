// have to do this to deal with warnings created from x11 constants
#![allow(non_upper_case_globals)]

use std::ptr::addr_of_mut;

use cfg_if::cfg_if;
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandWindowHandle, XlibDisplayHandle,
    XlibWindowHandle,
};
use x11::{xlib::{XEvent, CreateNotify, XDefaultVisual, XVisualIDFromVisual, XDefaultScreen, DestroyNotify, ConfigureNotify, FocusIn, FocusOut, Button1, Button2, Button3, Button4, ButtonPress, Button5, ButtonRelease, ShiftMask, ControlMask, LockMask, Mod1Mask, Mod2Mask, Mod3Mask, Mod4Mask, Mod5Mask, KeyPress, XKeycodeToKeysym, XLookupString, KeyRelease, XDefaultDepth}, keysym::{XK_space, XK_exclam, XK_1, XK_at, XK_2, XK_3, XK_numbersign, XK_dollar, XK_4, XK_5, XK_percent, XK_6, XK_asciicircum, XK_7, XK_ampersand, XK_asterisk, XK_8, XK_parenleft, XK_9, XK_0, XK_parenright, XK_hyphen, XK_underscore, XK_F1, XK_F2, XK_F3, XK_F4, XK_F5, XK_F6, XK_F7, XK_F8, XK_F9, XK_F10, XK_F11, XK_F12, XK_Escape, XK_Scroll_Lock, XK_Pause, XK_Sys_Req, XK_Print, XK_Break, XK_asciitilde, XK_grave, XK_equal, XK_plus, XK_BackSpace, XK_Insert, XK_Home, XK_Page_Up, XK_Num_Lock, XK_KP_Divide, XK_KP_Multiply, XK_KP_Subtract, XK_Tab, XK_Q, XK_q, XK_W, XK_E, XK_R, XK_T, XK_Y, XK_U, XK_I, XK_O, XK_P, XK_w, XK_e, XK_r, XK_t, XK_y, XK_u, XK_i, XK_o, XK_p, XK_bracketleft, XK_braceleft, XK_bracketright, XK_braceright, XK_backslash, XK_bar, XK_Delete, XK_End, XK_Page_Down, XK_KP_7, XK_KP_Home, XK_KP_8, XK_KP_9, XK_KP_Up, XK_KP_Page_Down, XK_KP_Add, XK_Caps_Lock, XK_A, XK_a, XK_S, XK_D, XK_F, XK_G, XK_H, XK_J, XK_K, XK_L, XK_semicolon, XK_apostrophe, XK_s, XK_d, XK_f, XK_g, XK_h, XK_j, XK_k, XK_l, XK_colon, XK_quotedbl, XK_Return, XK_KP_4, XK_KP_Left, XK_KP_5, XK_KP_6, XK_KP_Begin, XK_KP_Right, XK_Shift_L, XK_Z, XK_X, XK_C, XK_V, XK_B, XK_N, XK_M, XK_comma, XK_period, XK_slash, XK_Shift_R, XK_z, XK_x, XK_c, XK_v, XK_b, XK_n, XK_m, XK_less, XK_greater, XK_question, XK_uparrow, XK_KP_1, XK_KP_End, XK_KP_2, XK_KP_Down, XK_KP_3, XK_KP_Enter, XK_Control_L, XK_Win_L, XK_Alt_L, XK_Alt_R, XK_function, XK_Menu, XK_Control_R, XK_leftarrow, XK_downarrow, XK_rightarrow, XK_KP_0, XK_KP_Insert, XK_KP_Delete, XK_KP_Decimal, XK_KP_Page_Up}, xrandr::RRScreenChangeNotify};

use crate::{platform::WindowHandle, sys::{WindowEvent, MouseScancode, Modifiers, KeyboardScancode}};

pub fn main() {
    gtk4::init().unwrap();
}

impl WindowHandle {
    pub const fn new(handle: RawWindowHandle) -> Self {
        Self(handle)
    }

    pub const fn get(&self) -> RawWindowHandle {
        self.0
    }

    pub const fn get_wayland(&self) -> Option<WaylandWindowHandle> {
        match self.get() {
            RawWindowHandle::Wayland(handle) => Some(handle),
            _ => None,
        }
    }

    pub const fn get_xlib(&self) -> Option<XlibWindowHandle> {
        match self.get() {
            RawWindowHandle::Xlib(handle) => Some(handle),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MonitorHandle {
    Xlib(XlibDisplayHandle),
    Wayland(()),
}

#[allow(clippy::missing_trait_methods)]
impl Ord for MonitorHandle {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match *self {
            Self::Xlib(handle) => handle
                .display
                .cmp(&other.get_xlib().unwrap().display)
                .then(handle.screen.cmp(&other.get_xlib().unwrap().screen)),
            Self::Wayland(()) => ().cmp(&()),
        }
    }
}

#[allow(clippy::missing_trait_methods)]
impl PartialOrd for MonitorHandle {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// Win32 => Win32
// Linux => Xlib, Wayland
// macOS => Xlib, AppKit, UiKit
// Other Unix => Xlib
impl MonitorHandle {
    cfg_if! {
        if #[cfg(feature = "linux_use_wayland")] {
            pub const fn get(&self) -> RawDisplayHandle {
                match *self {
                    Self::Wayland(handle) => RawDisplayHandle::Wayland(handle),
                    _ => panic!()
                }
            }
        } else {
            pub const fn get(&self) -> RawDisplayHandle {
                match *self {
                    Self::Xlib(handle) => RawDisplayHandle::Xlib(handle),
                    _ => panic!()
                }
            }
        }
    }

    pub const fn get_xlib(&self) -> Option<XlibDisplayHandle> {
        match *self {
            Self::Xlib(handle) => Some(handle),
            _ => None,
        }
    }

    pub const fn get_wayland(&self) -> Option<()> {
        match *self {
            Self::Wayland(_) => Some(()),
            _ => None,
        }
    }
}

cfg_if! {
    if #[cfg(feature = "linux_use_wayland")] {

    } else {
        #[derive(Copy, Clone, Debug)]
        #[repr(transparent)]
        struct XlibMouseButton(u32);

        impl TryFrom<XlibMouseButton> for MouseScancode {
            type Error = ();
            fn try_from(value: XlibMouseButton) -> Result<Self, Self::Error> {
                match value.0 {
                    Button1 => Ok(Self::LClick),
                    Button2 => Ok(Self::MClick),
                    Button3 => Ok(Self::RClick),
                    8 => Ok(Self::Button5),
                    9 => Ok(Self::Button4),
                    _ => Err(()),
                }
            }
        }

        #[derive(Copy, Clone, Debug)]
        #[repr(transparent)]
        struct XlibModifiers(u32);

        impl XlibModifiers {
            fn contains_mod_masks(self) -> bool {
                self.0 & (Mod1Mask | Mod2Mask | Mod3Mask | Mod4Mask | Mod5Mask) != 0 
            }
        }

        // We don't care about the mouse buttons for this, and 
        // Mod1Mask-Mod5Mask might be different between systems,
        // so we can't reliably map them to a certain modifier key here
        impl TryFrom<XlibModifiers> for Modifiers {
            type Error = ();
            fn try_from(value: XlibModifiers) -> Result<Self, Self::Error> {
                let mut modifiers = Modifiers::empty();
                if value.0 & ShiftMask != 0 {
                    modifiers |= Modifiers::LSHIFT;
                }

                if value.0 & ControlMask != 0 {
                    modifiers |= Modifiers::LCTRL;
                }

                if value.0 & LockMask != 0 {
                    modifiers |= Modifiers::CAPSLOCK;
                }

                if modifiers.is_empty() {
                    Err(())
                } else {
                    Ok(modifiers)
                }
            }
        }

        pub struct XlibContext {
            width: i32,
            height: i32, 
            x: i32,
            y: i32,
        }

        struct XlibKeysym(x11::xlib::KeySym);

        impl TryFrom<XlibKeysym> for KeyboardScancode {
            type Error = ();
            fn try_from(value: XlibKeysym) -> Result<Self, Self::Error> {
                match value.0 as u32 {
                    XK_Escape => Ok(Self::Esc),
                    XK_F1 => Ok(Self::F1),
                    XK_F2 => Ok(Self::F2),
                    XK_F3 => Ok(Self::F3),
                    XK_F4 => Ok(Self::F4),
                    XK_F5 => Ok(Self::F5),
                    XK_F6 => Ok(Self::F6),
                    XK_F7 => Ok(Self::F7),
                    XK_F8 => Ok(Self::F8),
                    XK_F9 => Ok(Self::F9),
                    XK_F10 => Ok(Self::F10),
                    XK_F11 => Ok(Self::F11),
                    XK_F12 => Ok(Self::F12),
                    XK_Print | XK_Sys_Req => Ok(Self::PrtScSysRq),
                    XK_Scroll_Lock => Ok(Self::ScrLk),
                    XK_Pause | XK_Break => Ok(Self::PauseBreak),

                    XK_asciitilde | XK_grave => Ok(Self::Tilde),
                    XK_1 | XK_exclam => Ok(Self::Key1),
                    XK_2 | XK_at => Ok(Self::Key2),
                    XK_3 | XK_numbersign => Ok(Self::Key3),
                    XK_4 | XK_dollar => Ok(Self::Key4),
                    XK_5 | XK_percent => Ok(Self::Key5),
                    XK_6 | XK_asciicircum => Ok(Self::Key6),
                    XK_7 | XK_ampersand => Ok(Self::Key7),
                    XK_8 | XK_asterisk => Ok(Self::Key8),
                    XK_9 | XK_parenleft => Ok(Self::Key9),
                    XK_0 | XK_parenright => Ok(Self::Key0),
                    XK_hyphen | XK_underscore => Ok(Self::Hyphen),
                    XK_equal | XK_plus => Ok(Self::Equals),
                    XK_BackSpace => Ok(Self::Backspace),
                    XK_Insert => Ok(Self::Insert),
                    XK_Home => Ok(Self::Home),
                    XK_Page_Up => Ok(Self::PgUp),
                    XK_Num_Lock => Ok(Self::NumLk),
                    XK_KP_Divide => Ok(Self::NumSlash),
                    XK_KP_Multiply => Ok(Self::NumAsterisk),
                    XK_KP_Subtract => Ok(Self::NumHyphen),

                    XK_Tab => Ok(Self::Tab),
                    XK_Q | XK_q => Ok(Self::Q),
                    XK_W | XK_w => Ok(Self::W),
                    XK_E | XK_e => Ok(Self::E),
                    XK_R | XK_r => Ok(Self::R),
                    XK_T | XK_t => Ok(Self::T),
                    XK_Y | XK_y => Ok(Self::Y),
                    XK_U | XK_u => Ok(Self::U),
                    XK_I | XK_i => Ok(Self::I),
                    XK_O | XK_o => Ok(Self::O),
                    XK_P | XK_p => Ok(Self::P),
                    XK_bracketleft | XK_braceleft => Ok(Self::OpenBracket),
                    XK_bracketright | XK_braceright => Ok(Self::CloseBracket),
                    XK_backslash | XK_bar => Ok(Self::BackSlash),
                    XK_Delete => Ok(Self::Del),
                    XK_End => Ok(Self::End),
                    XK_Page_Down => Ok(Self::PgDn),
                    XK_KP_7 | XK_KP_Home => Ok(Self::Num7),
                    XK_KP_8 | XK_KP_Up => Ok(Self::Num8),
                    XK_KP_9 | XK_KP_Page_Up => Ok(Self::Num9),
                    XK_KP_Add => Ok(Self::NumPlus),

                    XK_Caps_Lock => Ok(Self::CapsLk),
                    XK_A | XK_a => Ok(Self::A),
                    XK_S | XK_s => Ok(Self::S),
                    XK_D | XK_d => Ok(Self::D),
                    XK_F | XK_f => Ok(Self::F),
                    XK_G | XK_g => Ok(Self::G),
                    XK_H | XK_h => Ok(Self::H),
                    XK_J | XK_j => Ok(Self::J),
                    XK_K | XK_k => Ok(Self::K),
                    XK_L | XK_l => Ok(Self::L),
                    XK_semicolon | XK_colon => Ok(Self::Semicolon),
                    XK_apostrophe | XK_quotedbl => Ok(Self::Apostrophe),
                    XK_Return => Ok(Self::Enter),
                    XK_KP_4 | XK_KP_Left => Ok(Self::Num4),
                    XK_KP_5 | XK_KP_Begin => Ok(Self::Num5),
                    XK_KP_6 | XK_KP_Right => Ok(Self::Num6),
                    
                    XK_Shift_L => Ok(Self::LShift),
                    XK_Z | XK_z => Ok(Self::Z),
                    XK_X | XK_x => Ok(Self::X),
                    XK_C | XK_c => Ok(Self::C),
                    XK_V | XK_v => Ok(Self::V),
                    XK_B | XK_b => Ok(Self::B),
                    XK_N | XK_n => Ok(Self::N),
                    XK_M | XK_m => Ok(Self::M),
                    XK_comma | XK_less => Ok(Self::Comma),
                    XK_period | XK_greater => Ok(Self::Period),
                    XK_slash | XK_question => Ok(Self::ForwardSlash),
                    XK_Shift_R => Ok(Self::RShift),
                    XK_uparrow => Ok(Self::ArrowUp),
                    XK_KP_1 | XK_KP_End => Ok(Self::Num1),
                    XK_KP_2 | XK_KP_Down => Ok(Self::Num2),
                    XK_KP_3 | XK_KP_Page_Down => Ok(Self::Num3),
                    XK_KP_Enter => Ok(Self::NumEnter),
                    
                    XK_Control_L => Ok(Self::LCtrl),
                    XK_Win_L => Ok(Self::LSys),
                    XK_Alt_L => Ok(Self::LAlt),
                    XK_space => Ok(Self::Space),
                    XK_Alt_R => Ok(Self::RAlt),
                    XK_function => Ok(Self::Fn),
                    XK_Menu => Ok(Self::Menu),
                    XK_Control_R => Ok(Self::RCtrl),
                    XK_leftarrow => Ok(Self::ArrowLeft),
                    XK_downarrow => Ok(Self::ArrowDown),
                    XK_rightarrow => Ok(Self::ArrowRight),
                    XK_KP_0 | XK_KP_Insert => Ok(Self::Num0),
                    XK_KP_Decimal | XK_KP_Delete => Ok(Self::NumPeriod),

                    _ => Err(())
                }
            }
        }

        pub trait WindowEventExtXlib {
            type Iter: Iterator<Item = WindowEvent>;
            fn try_from_xevent(ev: XEvent, context: XlibContext) -> Result<(Self::Iter, Option<XlibContext>), ()>;
        }

        impl WindowEventExtXlib for WindowEvent {
            type Iter = impl Iterator<Item = WindowEvent>;
            fn try_from_xevent(ev: XEvent, context: XlibContext) -> Result<(Self::Iter, Option<XlibContext>),()> {
                let any = unsafe { ev.any };
                match any.type_ {
                    CreateNotify => {
                        let mut handle = XlibWindowHandle::empty();
                        let ev = unsafe { ev.any };
                        handle.window = ev.window;
                        let screen = unsafe { XDefaultScreen(ev.display) };
                        let visual = unsafe { XDefaultVisual(ev.display, screen) };
                        let visual_id = unsafe { XVisualIDFromVisual(visual) };
                        handle.visual_id = visual_id;
                        Ok((vec![Self::Created(WindowHandle::new(RawWindowHandle::Xlib(handle)))].into_iter(), None))
                    },
                    DestroyNotify => Ok((vec![Self::Destroyed].into_iter(), None)),
                    ConfigureNotify => {
                        let ev = unsafe { ev.configure };
                        let x = ev.x;
                        let y = ev.y;
                        let width = ev.width;
                        let height = ev.height;
                        let new_context = XlibContext { width, height, x, y };
                        let mut evs = Vec::new();
                        if width != context.width || height != context.height {
                            evs.push(Self::Resized { width: width as _, height: height as _ });
                        } else if x != context.x || y != context.y {
                            evs.push(Self::Moved { x: x as _, y: y as _ });
                        }

                        if evs.is_empty() {
                            Err(())
                        } else {
                            Ok((evs.into_iter(), Some(new_context)))
                        }
                    },
                    FocusIn => Ok((vec![Self::SetFocus].into_iter(), None)),
                    FocusOut => Ok((vec![Self::KillFocus].into_iter(), None)),
                    ButtonPress => {
                        let ev = unsafe { ev.button };
                        let button = XlibMouseButton(ev.button);
                        
                        if let Ok(b) = MouseScancode::try_from(button) {
                            Ok((vec![Self::MouseButtonDown(b)].into_iter(), None))
                        } else if button.0 == Button4 {
                            Ok((vec![Self::MouseWheelScroll(120.0)].into_iter(), None))
                        } else if button.0 == Button5 {
                            Ok((vec![Self::MouseWheelScroll(-120.0)].into_iter(), None))
                        } else {
                            Err(())
                        }
                    },
                    ButtonRelease => {
                        let ev = unsafe { ev.button };
                        let button = XlibMouseButton(ev.button);
                        if let Ok(b) = MouseScancode::try_from(button) {
                            Ok((vec![Self::MouseButtonUp(b)].into_iter(), None))
                        } else {
                            Err(())
                        }
                    },
                    KeyPress | KeyRelease => {
                        let down = any.type_ == KeyPress;
                        let mut ev = unsafe { ev.key };
                        let keycode = ev.keycode;
                        let physical_keysym = unsafe { 
                            XKeycodeToKeysym(ev.display, keycode as _, 0) 
                        };
                        let mut c = 0i8;
                        let mut logical_keysym = 0;
                        unsafe { 
                            XLookupString(
                                addr_of_mut!(ev), addr_of_mut!(c), 
                                core::mem::size_of_val(&c) as _, 
                                addr_of_mut!(logical_keysym), 
                                core::ptr::null_mut()
                            ); 
                        };

                        let physical_scancode: Option<KeyboardScancode> = XlibKeysym(physical_keysym).try_into().ok();
                        let logical_scancode = if let Ok(k) = XlibKeysym(logical_keysym).try_into() {
                            k
                        } else { 
                            return Err(())
                        };

                        if down {
                            Ok((vec![Self::KeyDown {
                                logical_scancode,
                                physical_scancode,
                            }].into_iter(), None))
                        } else {
                            Ok((vec![Self::KeyUp {
                                logical_scancode,
                                physical_scancode,
                            }].into_iter(), None))
                        }
                    },
                    RRScreenChangeNotify => {
                        let ev = unsafe { ev.xrr_screen_change_notify };
                        let screen = unsafe { XDefaultScreen(ev.display) };
                        let depth = unsafe { XDefaultDepth(ev.display, screen) };
                        Ok((vec![Self::DisplayChange { 
                            bits_per_pixel: depth as _, 
                            horz_res: ev.width as _,
                            vert_res: ev.height as _,
                        }].into_iter(), None))
                    }
                    _ => Err(())
                }
            }
        }
    }
}