use cfg_aliases::cfg_aliases;

fn main() {
    // Setup cfg aliases
    cfg_aliases! {
        // OSes
        wasm: { target_arch = "wasm32" },
        native: { not(target_arch = "wasm32") },
        macos: { target_os = "macos" },
        linux: { target_os = "linux" },
        other_unix: { all(unix, not(target_os = "macos"), not(target_os = "linux")) },
        // Display servers
        xlib: { all(unix, not(feature = "linux_use_wayland"), not(feature = "macos_use_appkit")) },
        wayland: { all(target_os = "linux", feature = "linux_use_wayland") },
        appkit: { all(target_os = "macos", feature = "macos_use_appkit") },
        // Rendering backends
        d3d9: { all(windows, not(feature = "windows_use_wgpu")) },
        wgpu: { any(not(windows), feature = "windows_use_wgpu") },
        // Arches
        i686: { target_arch = "x86" },
        x86_64: { target_arch = "x86_64" },
    }
}
