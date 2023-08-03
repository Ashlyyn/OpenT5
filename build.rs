use cfg_aliases::cfg_aliases;

fn main() {
    // Setup cfg aliases
    cfg_aliases! {
        // OSes
        // Windows and unix already defined by rust
        macos: { target_os = "macos" },
        linux: { target_os = "linux" },
        bsd: { any(
            target_os = "freebsd", target_os = "dragonfly",
            target_os = "openbsd", target_os = "netbsd"
        ) },
        other_unix: {
            all(unix, not(target_os = "macos"), not(target_os = "linux"))
        },
        // We'll have to update this if we add support for OSes
        // that aren't Windows or Unix
        // No, this is not the PS3's OtherOS
        other_os: { not(any(windows, unix)) },
        // This might have to be updated later, but right now the only even
        // remotely-supported OS-less platform is wasm
        no_os: { target_arch = "wasm32" },
        // Display servers
        xlib: { any(other_unix, feature = "linux_use_xlib", feature = "macos_use_xlib") },
        wayland: { all(target_os = "linux", feature = "linux_use_wayland") },
        appkit: { all(target_os = "macos", feature = "macos_use_appkit") },
        // Rendering backends
        d3d9: { all(windows, feature = "windows_use_d3d9") },
        wgpu: { any(feature = "windows_use_wgpu", feature = "linux_use_wgpu", feature = "macos_use_wgpu") },
        // Arches
        x86: { any(target_arch = "x86", target_arch = "x86_64") },
        i686: { target_arch = "x86" },
        x86_64: { target_arch = "x86_64" },
        wasm: { target_arch = "wasm32" },

        native: { not(target_arch = "wasm32") },
    }
}
