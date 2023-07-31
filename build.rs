use cfg_aliases::cfg_aliases;

fn main() {
    // Setup cfg aliases
    cfg_aliases! {
        // OSes
        // windows and unix already defined by rust
        macos: { target_os = "macos" },
        linux: { target_os = "linux" },
        bsd: { any(
            target_os = "freebsd", target_os = "dragonfly",
            target_os = "openbsd", target_os = "netbsd"
        ) },
        other_unix: {
            all(unix, not(target_os = "macos"), not(target_os = "linux"))
        },
        // we'll have to update this if we add support for OSes
        // that aren't Windows or Unix
        // No, this is not the PS3's OtherOS
        other_os: { not(any(windows, unix)) },
        // Display servers
        xlib: { all(
            unix,
            not(feature = "linux_use_wayland"),
            not(feature = "macos_use_appkit"))
        },
        wayland: { all(target_os = "linux", feature = "linux_use_wayland") },
        appkit: { all(target_os = "macos", feature = "macos_use_appkit") },
        // Rendering backends
        d3d9: { all(windows, not(feature = "windows_use_wgpu")) },
        wgpu: { any(not(windows), feature = "windows_use_wgpu") },
        // Arches
        x86: { any(target_arch = "x86", target_arch = "x86_64") },
        i686: { target_arch = "x86" },
        x86_64: { target_arch = "x86_64" },
        wasm: { target_arch = "wasm32" },

        native: { not(target_arch = "wasm32") },
    }
}
