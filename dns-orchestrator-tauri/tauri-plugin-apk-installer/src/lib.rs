//! Tauri APK Installer Plugin
//!
//! 用于在 Android 上安装 APK 文件，使用 FileProvider 正确处理 URI 转换。

#[cfg(mobile)]
use tauri::Manager;
use tauri::{
    Runtime,
    plugin::{Builder, TauriPlugin},
};

mod commands;
mod models;

#[cfg(mobile)]
mod mobile;

pub use models::*;

/// 插件错误类型
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(mobile)]
    #[error("Plugin invoke error: {0}")]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),

    #[error("Installation failed: {0}")]
    InstallFailed(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[cfg(mobile)]
use mobile::ApkInstaller;

/// 为 AppHandle 扩展 APK 安装器方法
#[cfg(mobile)]
pub trait ApkInstallerExt<R: Runtime> {
    fn apk_installer(&self) -> &ApkInstaller<R>;
}

#[cfg(mobile)]
impl<R: Runtime, T: Manager<R>> ApkInstallerExt<R> for T {
    fn apk_installer(&self) -> &ApkInstaller<R> {
        self.state::<ApkInstaller<R>>().inner()
    }
}

/// 初始化插件
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("apk-installer")
        .invoke_handler(tauri::generate_handler![commands::install_apk])
        .setup(|_app, _api| {
            #[cfg(mobile)]
            {
                let installer = mobile::ApkInstaller::new(_app, _api)?;
                _app.manage(installer);
            }
            Ok(())
        })
        .build()
}
