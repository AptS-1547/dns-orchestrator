#![allow(clippy::expect_used, clippy::unwrap_used)]

use base64::Engine;

fn main() {
    tauri_build::build();
    extract_updater_pubkey();
}

/// 从 tauri.conf.json 提取 updater 公钥，编译时写入 `OUT_DIR` 供 `include_str`! 使用
fn extract_updater_pubkey() {
    println!("cargo:rerun-if-changed=tauri.conf.json");

    let config =
        std::fs::read_to_string("tauri.conf.json").expect("build: failed to read tauri.conf.json");
    let config: serde_json::Value =
        serde_json::from_str(&config).expect("build: failed to parse tauri.conf.json");

    let pubkey_b64 = config
        .pointer("/plugins/updater/pubkey")
        .and_then(|v| v.as_str())
        .expect("build: missing plugins.updater.pubkey in tauri.conf.json");

    let pubkey_bytes = base64::engine::general_purpose::STANDARD
        .decode(pubkey_b64)
        .expect("build: failed to base64 decode updater pubkey");
    let pubkey_str =
        String::from_utf8(pubkey_bytes).expect("build: updater pubkey is not valid UTF-8");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = std::path::Path::new(&out_dir).join("updater_pubkey.txt");
    std::fs::write(&dest, pubkey_str).expect("build: failed to write updater_pubkey.txt");
}
