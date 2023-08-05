# OpenT5
## An open-source Rust reimplementation of T5 (the engine running Call of Duty Black Ops)

There isn't really a whole lot to say about this project yet. Reimplementing a (fairly) modern game engine isn't exactly a trivial nor quick process. As of writing this, the actual game logic remains entirely untouched, everything done so far has been with the low-level internals of the engine. The most "high level" thing so far is probably either creating a (blank) window or integrating Discord Rich Presence (which took all of like 50 lines).

So far, this has been an entirely one-woman project, and as nice as contributions would be, I don't exactly anticipate attracting contributors anytime soon. I've really just written this for completeness's sake (what project *doesn't* have a README.md?), and to give some sort of synopsis of it for anyone who's stumbled across it somehow.

If you're wondering what the ultimate goals of this project are (beyond just "a working reimplementation", of course):
1. **Portability.** I'm most certainly not reimplementing an entire game engine just to have it bound to Microsoft's shitty OS and Microsoft's shitty libraries (i.e. T5 heavily uses D3D9 and XAudio2, among others). Unfortunately, this is going to incur *a lot* of work (specifically, writing at least the entire renderer and sound system from scratch), but I'd rather contend with that than the alternative. To that end, the entire codebase is designed to be as platform-agnostic as possible, using the standard library where possible, and abstracting third-party crates that might only work on certain platforms, so that implementations for other platforms can just be dropped in place without massive rewrites. 64-bit Windows 10/11 and Linux are the current "first class" targets, but we don't want to rule out any targets that could feasibly run the game. At the very least, the game should theoretically be runnable on all targets that T5 ran on (Windows XP+, Xbox 360/PS3) and equivalents (e.g. macOS, BSDs, and Linux from roughly the same era onwards). That's not to say that all platforms must be supported equally well (obviously we'd end up with less maintainers of the macOS-specific parts of the engine than the Windows-specific parts, for example), just that no design decisions should be made that would either categorically rule out supporting them or require significant rewriting to support them. To make life simple, code in this codebase should not assume simple things like endianness or size of `isize`/`usize`/pointers, etc., and no platform-specific code should generally be exposed outside of the function it's used in. Create wrappers, even if they're very thin (see: `sys::message_box` for a prime example of this). The main exception to this rule is if the crate in question will work on all platforms we may target (e.g. `gilrs` for gamepad input, `raw_window_handle` for window handles), or if the functionality implemented by the crate might not be present on all platforms (e.g. Discord Rich Presence).
2. **Clean, safe code.** Part of what convinced me to write this in Rust instead of the very "C with classes"-style C++ used by T5 (which would have been much easier in some respects) is not wanting to deal with segfaults, UB, and every other C-ism that Rust prevents. Basically, just follow normal Rust guidelines. Use `unsafe` only when necessary, don't pass pointers around if you can avoid it, etc.

(More might be added later.)

If, after reading all of the above, you're actually interested in contributing, please don't hesitate. Any help would be *very* appreciated!

## Building
Until I come up with a better way to do this, certain features will have to be enabled manually on all platforms. Currently, they consist of the display server (X, Cocoa) and the rendering API (d3d9, wgpu). 

* On Windows, you'll only have to enable `windows_use_d3d9` OR `windows_use_wgpu` since there's only one Windows display server. 
* On Linux, only support for WGPU and Xlib are currently implemented (Wayland and Vulkan will come soon), so you'll have to specify `linux_use_xlib` and `linux_use_wgpu`.
* On macOS, Xlib and AppKit support are both implemented, so you'll have to specify `macos_use_xlib` OR `macos_use_appkit` and `macos_use_wgpu` (Metal support will also come soon).

(If it were possible to set default features on a per-platform basis, none of this would be an issue, but alas...)

Building also requires a nightly toolchain, so once you have your features figured out, building is as simple as
```bash
    $ git clone https://github.com/Ashlyyn/OpenT5.git
    $ cd OpenT5
    $ cargo +nightly build --features <...>
```
Or you can set your default toolchain to nightly and just run `cargo build --features <...>` without the `+nightly`.

None of the game files are required yet (you will get some weird-looking localization references if `localization.txt` isn't present though).

Linux builds currently require `libgtk4`, so you'll want to grab that from your package manager if you don't have it installed (might swap it out for something else or implement the necessary functionality from scratch later). Windows and macOS don't require anything special on their own.

For builds with Xlib, you'll need `libx11` and `libxrandr` (and XQuartz on macOS). I'm not sure what dependencies Wayland will have yet.

The project will currently *build* for WASM, but it's entirely untested, and there are some things that will *definitely* need to be changed (e.g. use of stdlib threads, blocking the main thread, etc.) to get it to work correctly in the browser, so it's by no means functional.
