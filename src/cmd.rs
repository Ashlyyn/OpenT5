#![allow(dead_code)]

use arrayvec::ArrayVec;
use std::{
    cell::RefCell,
    collections::HashSet,
    hash::{Hash, Hasher},
    rc::Rc,
    sync::RwLock,
};

use lazy_static::lazy_static;

use crate::*;
use common::ItemDef;

#[derive(Clone, Eq)]
pub struct CmdFunction {
    name: String,
    auto_complete_dir: String,
    auto_complete_ext: String,
    function: fn(),
}

impl PartialEq for CmdFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for CmdFunction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Clone, Default)]
struct CmdArgs {
    nesting: usize,
    local_client_num: [i32; 8],
    controller_index: [i32; 8],
    item_def: ArrayVec<ItemDef, 8>,
    argshift: [i32; 8],
    argc: [usize; 8],
    argv: ArrayVec<Vec<String>, 8>,
    text_pool: ArrayVec<char, 8192>,
    argv_pool: ArrayVec<String, 512>,
    used_text_pool: [i32; 8],
    total_used_argv_pool: i32,
    total_used_text_pool: i32,
}

impl CmdArgs {
    fn new() -> Self {
        CmdArgs {
            nesting: 0,
            local_client_num: [0; 8],
            controller_index: [0; 8],
            item_def: ArrayVec::new(),
            argshift: [0; 8],
            argc: [0; 8],
            argv: ArrayVec::new(),
            text_pool: ArrayVec::new(),
            argv_pool: ArrayVec::new(),
            used_text_pool: [0; 8],
            total_used_argv_pool: 0,
            total_used_text_pool: 0,
        }
    }
}

impl CmdFunction {
    fn new(
        name: String,
        auto_complete_dir: String,
        auto_complete_ext: String,
        function: fn(),
    ) -> Self {
        CmdFunction {
            name,
            auto_complete_dir,
            auto_complete_ext,
            function,
        }
    }
}

lazy_static! {
    static ref CMD_FUNCTIONS: RwLock<HashSet<Option<CmdFunction>>> = RwLock::new(HashSet::new());
}

pub fn find(name: String) -> Option<CmdFunction> {
    for c in CMD_FUNCTIONS.read().unwrap().iter() {
        match c {
            Some(f) => {
                if f.name == name {
                    return Some(f.clone());
                }
            }
            None => {}
        }
    }
    None
}

pub fn add_internal(name: String, func: fn()) {
    match find(name.clone()) {
        Some(_) => com::println(format!("cmd::add_internal: {} is already defined", name)),
        None => {
            CMD_FUNCTIONS.write().unwrap().insert(Some(CmdFunction::new(
                name,
                "".to_string(),
                "".to_string(),
                func,
            )));
        }
    }
}

thread_local! {
    static ARGS: Rc<RefCell<CmdArgs>> = Rc::new(RefCell::new(CmdArgs::new()));
}

pub fn argc() -> usize {
    let mut args = CmdArgs::new();

    ARGS.with(|arg| {
        args = (*arg).take();
    });

    let argc = args.argc[args.nesting];

    ARGS.with(|arg| {
        arg.replace(args);
    });

    argc
}

pub fn argv(idx: usize) -> String {
    if idx >= argc() {
        return "".to_string();
    }

    let mut args = CmdArgs::new();

    ARGS.with(|arg| {
        args = (*arg).take();
    });

    let argv = args.argv[args.nesting][idx].clone();
    ARGS.with(|arg| {
        arg.replace(args);
    });

    argv
}
