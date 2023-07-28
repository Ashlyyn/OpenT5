use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(windows, not(feature = "windows_use_wgpu")))] {
        pub mod d3d9;
    } else {
        pub mod wgpu;
    }
}
