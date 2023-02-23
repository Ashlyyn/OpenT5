#![allow(dead_code)]

use std::sync::{Arc, RwLock};

use arrayvec::{ArrayString, ArrayVec};
use lazy_static::lazy_static;

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

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(usize)]
pub enum PrintMessageDest {
    Console = 0x0,
    Minicon = 0x1,
    Error = 0x2,
    Game1 = 0x3,
    Game2 = 0x4,
    Game3 = 0x5,
}

pub struct Channel(u8);

impl Channel {
    pub fn new(value: u8) -> Self {
        Self(value)
    }

    pub fn get(&self) -> u8 {
        self.0
    }
}

macro_rules! channel_from {
    ($t:tt) => {
        impl core::convert::From<$t> for $crate::console::Channel {
            fn from(value: $t) -> Self {
                Self(value as u8)
            }
        }
    };
}

channel_from!(u8);
channel_from!(u16);
channel_from!(u32);
channel_from!(u64);
channel_from!(usize);
channel_from!(i16);
channel_from!(i32);
channel_from!(i64);
channel_from!(isize);

pub fn is_channel_visible(
    mut msg_dest: PrintMessageDest,
    channel: Channel,
    param_3: i32,
) -> bool {
    let lock = PC_GLOB.clone();
    let pcglob = lock.read().unwrap();

    if pcglob.open_channels[channel.get() as usize].name.is_empty() {
        return false;
    }

    if msg_dest == PrintMessageDest::Minicon {
        if channel.get() == 2 || channel.get() == 3 || channel.get() == 4 {
            return false;
        }
        msg_dest = PrintMessageDest::Console;
    }

    if msg_dest == PrintMessageDest::Console && channel.get() == 0 {
        return true;
    }

    if (pcglob.filters[msg_dest as usize][channel.get() as usize >> 5]
        & 1 << channel.get()
        & 0x1F)
        == 0
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

    pub fn append_line(&mut self, line: ArrayString<256>) {
        self.buf[self.current_line] = line;
        self.current_line = if self.current_line == 256 {
            0
        } else {
            self.current_line + 1
        };
    }

    pub fn set_line(&mut self, line_num: u8, line: ArrayString<256>) {
        self.buf[line_num as usize] = line;
    }
}
