use tauri::{command, AppHandle, Runtime};

#[cfg(mobile)]
use crate::ApkInstallerExt;

/// 安装 APK 文件
///
/// 在 Android 平台上使用 FileProvider 正确处理 URI 转换
#[command]
pub async fn install_apk<R: Runtime>(app: AppHandle<R>, path: String) -> Result<(), String> {
    #[cfg(mobile)]
    {
        app.apk_installer()
            .install_apk(path)
            .map_err(|e| e.to_string())
    }

    #[cfg(not(mobile))]
    {
        let _ = app;
        let _ = path;
        Err("APK installation is only supported on Android".to_string())
    }
}
