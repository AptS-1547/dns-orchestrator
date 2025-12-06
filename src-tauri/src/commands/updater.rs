//! Android APK 更新模块
//!
//! 仅在 Android 平台编译，提供应用内更新功能：
//! 1. 检查更新 - 解析 latest.json
//! 2. 下载 APK - 带进度回调
//! 3. 安装 APK - 触发系统安装器

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::Manager;

const LATEST_JSON_URL: &str =
    "https://github.com/AptS-1547/dns-orchestrator/releases/latest/download/latest.json";

/// Android 更新信息
#[derive(Debug, Clone, Serialize)]
pub struct AndroidUpdate {
    pub version: String,
    pub notes: String,
    pub url: String,
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

    // 比较版本
    if !is_newer_version(&current_version, &latest.version) {
        return Ok(None); // 当前已是最新版本
    }

    Ok(Some(AndroidUpdate {
        version: latest.version,
        notes: latest.notes.unwrap_or_default(),
        url: platform.url.clone(),
    }))
}

/// 下载 APK 文件到缓存目录
#[tauri::command]
pub async fn download_apk(
    app: tauri::AppHandle,
    url: String,
    on_progress: tauri::ipc::Channel<DownloadProgress>,
) -> Result<String, String> {
    use futures::StreamExt;
    use std::io::Write;

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
        return Err(format!("Failed to download APK: HTTP {}", response.status()));
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

    // 发送完成事件
    let _ = on_progress.send(DownloadProgress::Finished);

    Ok(apk_path.to_string_lossy().to_string())
}

/// 触发 APK 安装
///
/// 使用自定义插件通过 FileProvider 正确处理 URI 转换
#[tauri::command]
pub async fn install_apk(app: tauri::AppHandle, path: String) -> Result<(), String> {
    use tauri_plugin_apk_installer::ApkInstallerExt;

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
}
