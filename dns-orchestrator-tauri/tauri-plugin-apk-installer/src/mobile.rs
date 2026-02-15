use tauri::{
    AppHandle, Runtime,
    plugin::{PluginApi, PluginHandle},
};

use crate::models::{InstallApkRequest, InstallApkResponse};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "net.esaps.tauri_plugin.apkinstaller";

/// APK 安装器移动端实现
pub struct ApkInstaller<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> ApkInstaller<R> {
    pub fn new(_app: &AppHandle<R>, api: PluginApi<R, ()>) -> crate::Result<Self> {
        #[cfg(target_os = "android")]
        let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "ApkInstallerPlugin")?;

        #[cfg(target_os = "ios")]
        let handle = api.register_ios_plugin(())?;

        Ok(Self(handle))
    }

    /// 安装 APK 文件
    pub fn install_apk(&self, path: String) -> crate::Result<()> {
        let response: InstallApkResponse = self
            .0
            .run_mobile_plugin("installApk", InstallApkRequest { path })?;

        if response.success {
            Ok(())
        } else {
            Err(crate::Error::InstallFailed(
                "Installation was not successful".to_string(),
            ))
        }
    }
}
