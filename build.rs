use std::path::PathBuf;

use winres::WindowsResource;

fn main() -> std::io::Result<()> {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let assets = root.join("assets");
    let ico = assets.join("BlackOps.ico");

    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        if ico.exists() {
            WindowsResource::new()
                // This path can be absolute, or relative to your crate root.
                .set_icon(&ico.to_string_lossy())
                .set("InternalName", "Open T5 SP")
                .compile()?;
        }
        Ok(())
    } else if std::env::var_os("CARGO_CFG_LINUX").is_some() {
        // TODO - create .desktop file
        Ok(())
    } else if std::env::var_os("CARGO_CFG_MACOS").is_some() {
        let debug_mode = std::env::var("DEBUG").is_ok();
        let profile = if debug_mode { "debug" } else { "release" };
        let build = PathBuf::from(std::env::var("CARGO_TARGET_DIR").unwrap())
            .join(profile);
        let app = build.join("OpenT5 SP.app");
        let contents = app.clone().join("Contents");
        let macos = contents.join("MacOS");
        let bin_name = std::env::var_os("CARGO_BIN_NAME").unwrap();
        let bin = build.join(bin_name.clone());
        match std::fs::create_dir(app.clone()) {
            Ok(_) => {
                std::fs::create_dir(contents.clone())?;
                std::fs::copy(
                    assets.join("Info.plist"),
                    contents.clone().join("Info.plist"),
                )?;
                std::fs::create_dir(macos.clone())?;
                std::fs::copy(bin, macos.join(bin_name))?;
                let resources = contents.join("Resources");
                if ico.exists() {
                    // TODO - make sure we don't have to do any format
                    // conversion for the icon
                    std::fs::copy(
                        assets.join("BlackOps.ico"),
                        resources.join("AppIcons.icns"),
                    )?;
                }
            }
            // If create_dir fails, it might be because the bundle
            // already exists
            Err(e) => {
                match e.kind() {
                    // If the bundle already exists, just copy the new
                    // executable and leave the rest alone
                    std::io::ErrorKind::AlreadyExists => {
                        std::fs::remove_file(macos.join(bin_name.clone()))?;
                        std::fs::copy(bin, macos.join(bin_name))?;
                    }
                    // Otherwise, just fail
                    _ => return Err(e),
                }
            }
        }

        Ok(())
    } else {
        Ok(())
    }
}
