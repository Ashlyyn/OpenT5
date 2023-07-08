use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(windows, not(feature = "windows_use_wgpu")))] {

    } else {
        pub mod wgpu;
    }
}