#![allow(dead_code)]

use std::path::PathBuf;

use crate::util::Module;

pub struct StCl {
    cl_id: i32,
    cl_instance: Module,
    ag_instance: Module,
    reload_client: bool,
    msg_prefix: String,
    cwd: PathBuf,
}
