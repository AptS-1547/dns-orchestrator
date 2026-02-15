//! Android APK 更新模块
//!
//! 仅在 Android 平台编译，提供应用内更新功能：
//! 1. 检查更新 - 解析 latest.json
//! 2. 下载 APK - 带进度回调 + 签名验证
//! 3. 安装 APK - 触发系统安装器（仅允许缓存目录）

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::Manager;

const LATEST_JSON_URL: &str =
    "https://github.com/AptS-1547/dns-orchestrator/releases/latest/download/latest.json";

/// APK 更新签名验证用的公钥（编译时从 tauri.conf.json plugins.updater.pubkey 提取）
const APK_UPDATER_PUBKEY: &str = include_str!(concat!(env!("OUT_DIR"), "/updater_pubkey.txt"));

/// Android 更新信息
#[derive(Debug, Clone, Serialize)]
pub struct AndroidUpdate {
    pub version: String,
    pub notes: String,
    pub url: String,
    pub signature: String,
}

/// latest.json 结构
#[derive(Debug, Deserialize)]
struct LatestJson {
    version: String,
    notes: Option<String>,
    platforms: HashMap<String, Platform>,
}

/// 平台信息
#[derive(Debug, Deserialize)]
struct Platform {
    url: String,
    signature: Option<String>,
}

/// 下载进度事件
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum DownloadProgress {
    Started { content_length: u64 },
    Progress { chunk_length: u64 },
    Finished,
}

/// 比较版本号，返回 true 如果 remote > current
fn is_newer_version(current: &str, remote: &str) -> bool {
    let parse_version = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };

    let current_parts = parse_version(current);
    let remote_parts = parse_version(remote);

    for i in 0..std::cmp::max(current_parts.len(), remote_parts.len()) {
        let c = current_parts.get(i).unwrap_or(&0);
        let r = remote_parts.get(i).unwrap_or(&0);
        if r > c {
            return true;
        }
        if r < c {
            return false;
        }
    }
    false
}

/// 验证 APK 文件的 minisign 签名
fn verify_apk_signature(apk_path: &std::path::Path, signature_str: &str) -> Result<(), String> {
    use minisign_verify::{PublicKey, Signature};

    let pubkey = PublicKey::decode(APK_UPDATER_PUBKEY)
        .map_err(|e| format!("Failed to decode updater pubkey: {e}"))?;

    let signature = Signature::decode(signature_str)
        .map_err(|e| format!("Failed to decode APK signature: {e}"))?;

    let data =
        std::fs::read(apk_path).map_err(|e| format!("Failed to read APK for verification: {e}"))?;

    pubkey.verify(&data, &signature, false).map_err(|_| {
        "APK signature verification failed: file may have been tampered with".to_string()
    })
}

/// 检查 Android 更新
///
/// 解析 latest.json，查找 android 平台的更新信息
#[tauri::command]
pub async fn check_android_update(
    current_version: String,
) -> Result<Option<AndroidUpdate>, String> {
    let client = reqwest::Client::builder()
        .user_agent("DNS-Orchestrator-Updater")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // 获取 latest.json
    let response = client
        .get(LATEST_JSON_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch latest.json: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch latest.json: HTTP {}",
            response.status()
        ));
    }

    let latest: LatestJson = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse latest.json: {}", e))?;

    // 查找 Android 平台（尝试多个可能的 key）
    let android_keys = ["android", "android-aarch64", "android-arm64"];
    let platform = android_keys
        .iter()
        .find_map(|key| latest.platforms.get(*key));

    let Some(platform) = platform else {
        return Ok(None); // 没有 Android 平台的更新
    };

    // 签名必须存在且不能为 "none"
    let signature = match &platform.signature {
        Some(sig) if !sig.is_empty() && sig != "none" => sig.clone(),
        _ => return Err("Android update has no valid signature, refusing update".to_string()),
    };

    // 比较版本
    if !is_newer_version(&current_version, &latest.version) {
        return Ok(None); // 当前已是最新版本
    }

    Ok(Some(AndroidUpdate {
        version: latest.version,
        notes: latest.notes.unwrap_or_default(),
        url: platform.url.clone(),
        signature,
    }))
}

/// 下载 APK 文件到缓存目录并验证签名
#[tauri::command]
pub async fn download_apk(
    app: tauri::AppHandle,
    url: String,
    signature: String,
    on_progress: tauri::ipc::Channel<DownloadProgress>,
) -> Result<String, String> {
    use futures::StreamExt;
    use std::io::Write;

    // 签名不能为空
    if signature.is_empty() || signature == "none" {
        return Err("Cannot download APK without a valid signature".to_string());
    }

    let client = reqwest::Client::builder()
        .user_agent("DNS-Orchestrator-Updater")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // 发起请求
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to download APK: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download APK: HTTP {}",
            response.status()
        ));
    }

    let content_length = response.content_length().unwrap_or(0);

    // 发送开始事件
    let _ = on_progress.send(DownloadProgress::Started { content_length });

    // 获取缓存目录
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to get cache dir: {}", e))?;

    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache dir: {}", e))?;

    let apk_path = cache_dir.join("update.apk");

    // 创建文件
    let mut file = std::fs::File::create(&apk_path)
        .map_err(|e| format!("Failed to create APK file: {}", e))?;

    // 流式下载
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;

        file.write_all(&chunk)
            .map_err(|e| format!("Failed to write APK: {}", e))?;

        // 发送进度事件
        let _ = on_progress.send(DownloadProgress::Progress {
            chunk_length: chunk.len() as u64,
        });
    }

    // 关闭文件确保写入完成
    drop(file);

    // 验证签名
    if let Err(e) = verify_apk_signature(&apk_path, &signature) {
        // 删除不可信文件
        let _ = std::fs::remove_file(&apk_path);
        return Err(e);
    }

    // 发送完成事件（签名验证通过后才发送）
    let _ = on_progress.send(DownloadProgress::Finished);

    Ok(apk_path.to_string_lossy().to_string())
}

/// 触发 APK 安装
///
/// 使用自定义插件通过 FileProvider 正确处理 URI 转换
/// 仅允许安装位于应用缓存目录下的 APK 文件
#[tauri::command]
pub async fn install_apk(app: tauri::AppHandle, path: String) -> Result<(), String> {
    use tauri_plugin_apk_installer::ApkInstallerExt;

    // 路径白名单：只允许安装缓存目录下的文件
    let cache_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| format!("Failed to get cache dir: {}", e))?;

    let apk_path = std::path::Path::new(&path);

    let canonical_path = apk_path
        .canonicalize()
        .map_err(|e| format!("Invalid APK path: {}", e))?;
    let canonical_cache = cache_dir
        .canonicalize()
        .map_err(|e| format!("Failed to resolve cache dir: {}", e))?;

    if !canonical_path.starts_with(&canonical_cache) {
        return Err("Rejected: APK path is outside app cache directory".to_string());
    }

    app.apk_installer()
        .install_apk(path)
        .map_err(|e| format!("Failed to install APK: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("1.0.0", "1.0.1"));
        assert!(is_newer_version("1.0.0", "1.1.0"));
        assert!(is_newer_version("1.0.0", "2.0.0"));
        assert!(is_newer_version("v1.0.0", "v1.0.1"));
        assert!(!is_newer_version("1.0.1", "1.0.0"));
        assert!(!is_newer_version("1.0.0", "1.0.0"));
        assert!(is_newer_version("1.0.7", "1.0.8"));
    }

    #[test]
    fn test_pubkey_decode() {
        use minisign_verify::PublicKey;
        // 确保编译时内嵌的公钥格式正确
        PublicKey::decode(APK_UPDATER_PUBKEY).expect("Embedded pubkey should be valid");
    }

    #[test]
    fn test_version_comparison_empty_strings() {
        // 两个空字符串 → 相等，不是更新
        assert!(!is_newer_version("", ""));
        // 空 vs 有值 → remote 更新
        assert!(is_newer_version("", "1.0.0"));
        // 有值 vs 空 → 不是更新
        assert!(!is_newer_version("1.0.0", ""));
    }

    #[test]
    fn test_version_comparison_single_segment() {
        assert!(is_newer_version("1", "2"));
        assert!(!is_newer_version("2", "1"));
        assert!(!is_newer_version("1", "1"));
    }

    #[test]
    fn test_version_comparison_four_segments() {
        assert!(is_newer_version("1.0.0.0", "1.0.0.1"));
        assert!(!is_newer_version("1.0.0.1", "1.0.0.0"));
        // 不同长度比较：缺失段视为 0
        assert!(is_newer_version("1.0.0", "1.0.0.1"));
        assert!(!is_newer_version("1.0.0.1", "1.0.0"));
    }

    #[test]
    fn test_latest_json_deserialize() {
        let json = r#"{
            "version": "v1.2.0",
            "notes": "Bug fixes",
            "platforms": {
                "android-aarch64": {
                    "url": "https://example.com/app.apk",
                    "signature": "sig123"
                }
            }
        }"#;
        let latest: LatestJson = serde_json::from_str(json).unwrap();
        assert_eq!(latest.version, "v1.2.0");
        assert_eq!(latest.notes.as_deref(), Some("Bug fixes"));
        let platform = latest.platforms.get("android-aarch64").unwrap();
        assert_eq!(platform.url, "https://example.com/app.apk");
        assert_eq!(platform.signature.as_deref(), Some("sig123"));
    }

    #[test]
    fn test_latest_json_deserialize_no_notes() {
        let json = r#"{
            "version": "v1.0.0",
            "platforms": {}
        }"#;
        let latest: LatestJson = serde_json::from_str(json).unwrap();
        assert!(latest.notes.is_none());
        assert!(latest.platforms.is_empty());
    }

    #[test]
    fn test_download_progress_serialize() {
        let started = DownloadProgress::Started {
            content_length: 1024,
        };
        let json = serde_json::to_value(&started).unwrap();
        assert_eq!(json["event"], "Started");
        assert_eq!(json["data"]["content_length"], 1024);

        let progress = DownloadProgress::Progress { chunk_length: 256 };
        let json = serde_json::to_value(&progress).unwrap();
        assert_eq!(json["event"], "Progress");
        assert_eq!(json["data"]["chunk_length"], 256);

        let finished = DownloadProgress::Finished;
        let json = serde_json::to_value(&finished).unwrap();
        assert_eq!(json["event"], "Finished");
        assert!(json.get("data").is_none());
    }

    #[test]
    fn test_android_update_serialize() {
        let update = AndroidUpdate {
            version: "v1.2.0".into(),
            notes: "New feature".into(),
            url: "https://example.com/app.apk".into(),
            signature: "sig123".into(),
        };
        let json = serde_json::to_value(&update).unwrap();
        assert_eq!(json["version"], "v1.2.0");
        assert_eq!(json["notes"], "New feature");
        assert_eq!(json["url"], "https://example.com/app.apk");
        assert_eq!(json["signature"], "sig123");
    }
}
