// Puts the app's icon (Pip) into the viewer's exe as resource 1, for File Explorer, the title bar
// and the taskbar (main.rs loads it from there).

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon_with_id("../../src-tauri/icons/icon.ico", "1");
        res.compile().expect("the viewer's icon");
    }
    println!("cargo:rerun-if-changed=../../src-tauri/icons/icon.ico");
}
