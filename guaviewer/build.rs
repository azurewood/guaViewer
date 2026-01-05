//Build script for Slint
#[cfg(windows)]
extern crate embed_resource;

#[cfg(windows)]
fn main() {
    // if cfg!(target_os = "windows") {
    //     let mut res = winres::WindowsResource::new();
    //     res.set_icon("ui/icon.ico"); // Path to your .ico file
    //     res.compile().unwrap();
    // }
    // slint_build::compile("ui/app.slint").unwrap();

    // 1. Existing Slint compilation
    slint_build::compile("ui/app.slint").expect("Slint build failed");

    // 2. New Executable Icon logic (Windows only)
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_resource::compile("resources.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}

#[cfg(not(windows))]
fn main() {
    slint_build::compile("ui/app.slint").expect("Slint build failed");
}
