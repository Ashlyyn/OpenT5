use std::sync::RwLock;

use crate::*;

use lazy_static::lazy_static;

#[derive(Copy, Clone, Default)]
pub struct Config {
    pub scene_width: u16,
    pub scene_height: u16,
    pub display_width: u16,
    pub display_height: u16,
    pub output_display_width: u16,
    pub output_display_height: u16,
    pub display_frequency: u16,
    pub is_tool_mode: bool,
    pub is_fullscreen: bool,
    pub aspect_ratio_window: f32,
    pub aspect_ratio_scene_pixel: f32,
    pub aspect_ratio_display_pixel: f32,
    pub max_texture_size: usize,
    pub max_texture_maps: usize,
    pub device_supports_gamma: bool,
}

lazy_static! {
    pub static ref CONFIG: RwLock<Config> =
        RwLock::new(Config::default());
}

pub fn config() -> Config {
    *CONFIG.read().unwrap()
}

#[allow(clippy::print_stdout)]
pub fn app_activate(active_app: bool, is_minimized: bool) {
    key::clear_states(0);
    if is_minimized {
        platform::set_minimized();
    } else {
        platform::clear_minimized();
    }

    if active_app == false {
        platform::clear_active_app();
    } else {
        platform::set_active_app();
        println!("TODO: com::touch_memory");
    }

    input::activate(active_app);
    // _DAT_027706dc = 0;
}
