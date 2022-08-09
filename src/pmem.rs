#![allow(dead_code)]

use std::sync::RwLock;

use lazy_static::lazy_static;

#[cfg(target_os = "windows")]
use windows::Win32::System::Memory::{VirtualAlloc, MEM_COMMIT, PAGE_READWRITE};

#[cfg(any(target_os = "unix", target_os = "linux"))]
use nix::sys::mman::{mmap, MapFlags, ProtFlags};

#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "unix"),
    not(target_os = "linux")
))]
use libc::malloc;

#[derive(Copy, Clone)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum MemTrack {
    DEBUG = 0x00,
    HUNK = 0x01,
    BINARIES = 0x02,
    MISC_SWAP = 0x03,
    DELIMITER1 = 0x04,
    AI = 0x05,
    AI_NODES = 0x06,
    SCRIPT = 0x07,
    FX = 0x08,
    GLASS = 0x09,
    NETWORK_ENTITY = 0x0A,
    MISC = 0x0B,
    FASTFILE = 0x0C,
    ANIMATION = 0x0D,
    WORLD_GLOBALS = 0x0E,
    SOUND_GLOBALS = 0x0F,
    CLIENT_ANIMSCRIPT = 0x10,
    SOUND = 0x11,
    DELIMITER2 = 0x12,
    RENDERER_GLOBALS = 0x13,
    RENDERER_IMAGES = 0x14,
    RENDERER_WORLD = 0x15,
    RENDERER_MODELS = 0x16,
    RENDERER_MISC = 0x17,
    RENDERER_CINEMATICS = 0x18,
    DELIMITER3 = 0x19,
    COLLISION_MISC = 0x1A,
    COLLISION_BRUSH = 0x1B,
    COLLISION_MODEL_TRI = 0x1C,
    COLLISION_TERRAIN = 0x1D,
    PHYSICS = 0x1E,
    MAP_ENTS = 0x1F,
    TEMP = 0x20,
    DELIMITER4 = 0x21,
    LOCALIZATION = 0x22,
    FLAME = 0x23,
    UI = 0x24,
    TL = 0x25,
    ZMEM = 0x26,
    FIREMANAGER = 0x27,
    PROFILE = 0x28,
    WATERSIM = 0x29,
    CLIENT = 0x2A,
    RECORDER = 0x2B,
    RSTREAM = 0x2C,
    RENDERER_STREAMBUFFER = 0x2D,
    RENDERER_STREAMBUFFER_EXTRA = 0x2E,
    GEOSTREAM = 0x2F,
    DDL = 0x30,
    ONLINE = 0x31,
    EMBLEM = 0x32,
    MINSPEC_IMAGES = 0x33,
    DELIMITER5 = 0x34,
    NONE = 0x35,
    COUNT = 0x36,
}

#[derive(Copy, Clone, Default)]
struct PhysicalMemoryAllocation<'a> {
    name: &'a str,
    pos: usize,
}

impl<'a> PhysicalMemoryAllocation<'a> {
    fn new(n: &'a str, p: usize) -> Self {
        PhysicalMemoryAllocation { name: n, pos: p }
    }
}

#[derive(Copy, Clone)]
struct PhysicalMemoryPrim<'a, 'b> {
    alloc_name: &'a str,
    alloc_list_count: usize,
    pos: usize,
    alloc_list: [PhysicalMemoryAllocation<'b>; 32],
    mem_track: MemTrack,
}

impl<'a, 'b> PhysicalMemoryPrim<'a, 'b> {
    fn new(n: &'a str, c: usize, p: usize, m: MemTrack) -> Self {
        PhysicalMemoryPrim {
            alloc_name: n,
            alloc_list_count: c,
            pos: p,
            alloc_list: [PhysicalMemoryAllocation::new("", 0); 32],
            mem_track: m,
        }
    }
}

struct PhysicalMemory<'a, 'b, 'c, 'd> {
    name: &'a str,
    buf: Option<&'b mut [u8]>,
    prim: [PhysicalMemoryPrim<'c, 'd>; 2],
    size: usize,
}

impl<'a, 'b, 'c, 'd> PhysicalMemory<'a, 'b, 'c, 'd> {
    fn new(n: &'a str, b: Option<&'b mut [u8]>, s: usize) -> Self {
        PhysicalMemory {
            name: n,
            buf: b,
            prim: [
                PhysicalMemoryPrim::<'c, 'd>::new("", 0, 0, MemTrack::DEBUG),
                PhysicalMemoryPrim::<'c, 'd>::new("", 0, s, MemTrack::DEBUG),
            ],
            size: s,
        }
    }
}

#[cfg(target_os = "windows")]
fn alloc<'a>(size: usize) -> Option<&'a mut [u8]> {
    let p = unsafe { VirtualAlloc(core::ptr::null(), size, MEM_COMMIT, PAGE_READWRITE) as *mut u8 };
    match p.is_null() {
        true => None,
        false => unsafe { Some(core::slice::from_raw_parts_mut(p, size)) },
    }
}

#[cfg(any(target_os = "unix", target_os = "linux"))]
fn alloc<'a>(size: usize) -> Option<&'a mut [u8]> {
    let p = unsafe {
        mmap(
            core::ptr::null_mut(),
            size,
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            MapFlags::MAP_PRIVATE | MapFlags::MAP_ANON,
            0,
            0,
        )
        .unwrap() as *mut u8
    };
    match p.is_null() {
        true => None,
        false => unsafe { Some(core::slice::from_raw_parts_mut(p, size)) },
    }
}

#[cfg(all(
    not(target_os = "windows"),
    not(target_os = "unix"),
    not(target_os = "linux")
))]
fn alloc<'a>(size: usize) -> Option<&'a mut [u8]> {
    let p = malloc(size);
    match p.is_null() {
        true => None,
        false => Some(p),
    }
}

extern crate core;
lazy_static! {
    static ref G_PHYSICAL_MEMORY_INIT: RwLock<bool> = RwLock::new(false);
    static ref G_MEM: RwLock<PhysicalMemory<'static, 'static, 'static, 'static>> =
        RwLock::new(PhysicalMemory::new("", None, 0));
}

pub fn init() {
    if *G_PHYSICAL_MEMORY_INIT.read().unwrap() == false {
        *G_PHYSICAL_MEMORY_INIT.write().unwrap() = true;

        const SIZE: usize = 0x12C00000;
        *G_MEM.write().unwrap() = PhysicalMemory::new("main", Some(alloc(SIZE).unwrap()), SIZE);
        println!("Successfully allocated {} bytes.", SIZE);
    }
}
