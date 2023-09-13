#![allow(dead_code)]

use std::{sync::RwLock, time::Duration};
extern crate alloc;
use alloc::sync::Arc;

use arrayvec::{ArrayString, ArrayVec};
use bitflags::bitflags;
use lazy_static::lazy_static;

use crate::common::{Vec2f32, Vec4f32};

#[derive(Clone, Debug)]
pub struct PrintChannel {
    name: ArrayString<32>,
    allow_script: bool,
}

impl PrintChannel {
    pub const fn new() -> Self {
        Self {
            name: ArrayString::new_const(),
            allow_script: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PrintChannelGlob {
    open_channels: ArrayVec<PrintChannel, 256>,
    filters: [[u32; 8]; 6],
}

impl PrintChannelGlob {
    pub const fn new() -> Self {
        Self {
            open_channels: ArrayVec::new_const(),
            filters: [[0u32; 8]; 6],
        }
    }
}

lazy_static! {
    static ref PC_GLOB: Arc<RwLock<PrintChannelGlob>> =
        Arc::new(RwLock::new(PrintChannelGlob::new()));
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum PrintMessageDest {
    CONSOLE = 0x0,
    MINICON = 0x1,
    ERROR = 0x2,
    GAME1 = 0x3,
    GAME2 = 0x4,
    GAME3 = 0x5,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum Channel {
    /// A catch-all for when other channels don't apply or for when the message
    /// shoudldn't be filtered.
    DONT_FILTER,
    ERROR,
    GAMENOTIFY,
    BOLDGAME,
    SUBTITLE,
    LOGFILEONLY,
    /// For graphics-related functions - mostly used in `render` and `rb`
    GFX,
    /// Used in `snd`
    SOUND,
    /// Used in `fs`
    FILES,
    DEVGUI,
    PROFILE,
    UI,
    CLIENT,
    SERVER,
    SYSTEM,
    ANIM,
    FX,
    LIVE,
    PARSER_SCRIPT,
    TASK,
}

impl Channel {
    const fn as_i32(self) -> i32 {
        self as _
    }
}

#[allow(clippy::indexing_slicing)]
pub fn is_channel_visible(
    mut msg_dest: PrintMessageDest,
    channel: Channel,
    param_3: i32,
) -> bool {
    let lock = PC_GLOB.clone();
    let pcglob = lock.read().unwrap();

    if pcglob.open_channels[channel.as_i32() as usize]
        .name
        .is_empty()
    {
        return false;
    }

    if msg_dest == PrintMessageDest::MINICON {
        if channel == Channel::GAMENOTIFY
            || channel == Channel::BOLDGAME
            || channel == Channel::SUBTITLE
        {
            return false;
        }
        msg_dest = PrintMessageDest::CONSOLE;
    }

    if msg_dest == PrintMessageDest::CONSOLE && channel.as_i32() == 0 {
        return true;
    }

    if (pcglob.filters[msg_dest as usize][channel.as_i32() as usize >> 5]
        & 1 << channel.as_i32())
    .trailing_zeros()
        >= 5
        && ((param_3 >> 5 & 0x1F != 3 && param_3 >> 5 & 0x1F != 2)
            || pcglob.filters[msg_dest as usize][0] & 2 != 0)
    {
        return false;
    }

    true
}

pub struct ConsoleBuffer {
    buf: ArrayVec<ArrayString<256>, 100>,
    current_line: usize,
}

impl ConsoleBuffer {
    pub fn new() -> Self {
        Self {
            buf: ArrayVec::new(),
            current_line: 0,
        }
    }

    pub fn append_line(&mut self, line: &ArrayString<256>) {
        *self.buf.get_mut(self.current_line).unwrap() = *line;
        self.current_line = if self.current_line == 256 {
            0
        } else {
            self.current_line + 1
        };
    }

    pub fn set_line(&mut self, line_num: u8, line: &ArrayString<256>) {
        *self.buf.get_mut(line_num as usize).unwrap() = *line;
    }
}

#[derive(Copy, Clone, Default)]
pub struct Message {
    start_time: Duration,
    end_time: Duration,
}

bitflags! {
    #[derive(Default)]
    pub struct MessageLineFlags: u32 {

    }
}

#[derive(Copy, Clone, Default)]
pub struct MessageLine {
    message_index: usize,
    text_buf_pos: usize,
    text_buf_size: usize,
    typing_start_time: Duration,
    last_typing_sound_time: Duration,
    flags: MessageLineFlags,
}

#[derive(Clone, Default)]
pub struct MessageWindow {
    lines: Vec<MessageLine>,
    messages: Vec<Message>,
    circular_text_buffer: String,
    scroll_time: Duration,
    fade_in: Duration,
    fade_out: Duration,
    text_buf_pos: usize,
    first_line_index: usize,
    active_line_count: usize,
    message_index: usize,
}

#[derive(Clone, Default)]
pub struct MessageBuffer {
    gamemsg_text: ArrayVec<ArrayString<2048>, 3>,
    gamemsg_windows: ArrayVec<MessageWindow, 3>,
    gamemsg_lines: ArrayVec<ArrayVec<MessageLine, 12>, 3>,
    gamemsg_messages: ArrayVec<ArrayVec<Message, 12>, 3>,
    minicon_text: ArrayString<4096>,
    minicon_window: MessageWindow,
    minicon_lines: ArrayVec<Message, 100>,
    minicon_messages: ArrayVec<Message, 100>,
    error_text: ArrayString<1024>,
    error_window: MessageWindow,
    error_lines: ArrayVec<MessageLine, 5>,
    error_messages: ArrayVec<Message, 5>,
}

#[derive(Clone, Default)]
pub struct Console {
    initialized: bool,
    console_window: MessageWindow,
    console_lines: ArrayVec<MessageLine, 1024>,
    console_messages: ArrayVec<Message, 1024>,
    console_text: ArrayString<32768>,
    text_temp_line: ArrayString<512>,
    line_offset: usize,
    display_line_offset: usize,
    prev_channel: i32,
    output_visible: bool,
    font_height: i32,
    visible_line_count: usize,
    visible_pixel_width: usize,
    screen_min: Vec2f32,
    screen_max: Vec2f32,
    message_buffer: MessageBuffer,
    color: Vec4f32,
}

lazy_static! {
    static ref CON: RwLock<Console> = RwLock::new(Console::default());
}

pub fn get_text_copy(_len: usize) -> String {
    String::new()
    // let con = CON.read().unwrap();

    // if con.console_window.active_line_count == 0 {
    //     return String::new();
    // }

    // let mut line_pos =
    // con.console_window.lines.get(con.console_window.first_line_index).
    // unwrap().text_buf_pos; let mut end = con.console_window.text_buf_pos
    // - line_pos; if end < 0 { end =
    //   con.console_window.circular_text_buffer.len();
    // }

    // if len - 1 < end {
    //     line_pos = end - len - 1 + line_pos;
    //     if con.console_window.circular_text_buffer.len() < line_pos {
    //         line_pos -= con.console_window.circular_text_buffer.len();
    //     }
    //     end = len - 1;
    // }

    // if line_pos < con.console_window.text_buf_pos {
    //     con.console_window.circular_text_buffer[line_pos..con.console_window.
    // text_buf_pos] } else {

    // }
}
