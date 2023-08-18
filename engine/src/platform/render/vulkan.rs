use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use ash::{vk, Entry, Instance};
use lazy_static::lazy_static;

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
