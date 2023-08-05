use std::sync::{RwLock, RwLockWriteGuard, RwLockReadGuard};

use lazy_static::lazy_static;
use ash::{Entry, Instance, vk};

#[derive(Default)]
pub struct VkGlobals {
    pub entry: Option<Entry>,
    pub instance: Option<Instance>,
    pub physical_device: Option<vk::PhysicalDevice>,
    pub device: Option<vk::Device>,
}

lazy_static! {
    static ref VK: RwLock<VkGlobals> = RwLock::new(VkGlobals::default());
}

pub fn vk() -> RwLockReadGuard<'static, VkGlobals> {
    VK.read().unwrap()
}

pub fn vk_mut() -> RwLockWriteGuard<'static, VkGlobals> {
    VK.write().unwrap()
}