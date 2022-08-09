#![allow(dead_code)]

use arrayvec::ArrayVec;
use std::{
    cell::RefCell,
    collections::HashMap,
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

// CmdFunctions should only be compared by name, to prevent multiple commands
// with the same name but different remaining fields from being allowed in
// associative containers
impl PartialEq for CmdFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

// Hash only the name for the same reason
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
    static ref CMD_FUNCTIONS: Arc<RwLock<HashMap<String, CmdFunction>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

pub fn find(name: String) -> Option<CmdFunction> {
    let lock = CMD_FUNCTIONS.clone();
    let reader = lock.read().unwrap();
    reader.get(&name).cloned()
}

pub fn add_internal(name: String, func: fn()) {
    match find(name.clone()) {
        Some(_) => com::println(format!(
            "cmd::add_internal: {} is already defined",
            name
        )),
        None => {
            CMD_FUNCTIONS.write().unwrap().insert(
                name.clone(),
                CmdFunction::new(name, "".to_string(), "".to_string(), func),
            );
        }
    }
}

thread_local! {
    // Use Rc/RefCell instead of Arc/RwLock since ARGS is thread-local
    static ARGS: Rc<RefCell<CmdArgs>> = Rc::new(RefCell::new(CmdArgs::new()));
}

pub fn argc() -> usize {
    // Temporary to take/replace ARGS
    let mut args = CmdArgs::new();

    // Janky take/replace for ARGS/RefCell, try to fix later
    ARGS.with(|arg| {
        args = (*arg).take();
    });

    // Get argc
    let argc = args.argc[args.nesting];

    // Replace
    ARGS.with(|arg| {
        arg.replace(args);
    });

    // And return argc
    argc
}

pub fn argv(idx: usize) -> String {
    // Return "" if idx is out of range
    if idx >= argc() {
        return "".to_string();
    }

    // Create temporary to use for take/replace
    let mut args = CmdArgs::new();

    ARGS.with(|arg| {
        args = (*arg).take();
    });

    // Get actual arg
    let argv = args.argv[args.nesting][idx].clone();

    // Replace ARGS
    ARGS.with(|arg| {
        arg.replace(args);
    });

    // And return acquired arg
    argv
}
