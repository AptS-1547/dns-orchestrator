const COMMANDS: &[&str] = &["install_apk"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .build();
}
