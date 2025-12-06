package net.esaps.tauri_plugin.apkinstaller

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Build
import androidx.core.content.FileProvider
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.File

@InvokeArg
class InstallApkArgs {
    lateinit var path: String
}

@TauriPlugin
class ApkInstallerPlugin(private val activity: Activity) : Plugin(activity) {

    @Command
    fun installApk(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(InstallApkArgs::class.java)
            val apkFile = File(args.path)

            if (!apkFile.exists()) {
                invoke.reject("APK file not found: ${args.path}")
                return
            }

            val apkUri: Uri = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
                // Android 7.0+ 需要使用 FileProvider
                FileProvider.getUriForFile(
                    activity,
                    "${activity.packageName}.fileprovider",
                    apkFile
                )
            } else {
                // Android 7.0 以下可以直接使用 file:// URI
                Uri.fromFile(apkFile)
            }

            val intent = Intent(Intent.ACTION_VIEW).apply {
                setDataAndType(apkUri, "application/vnd.android.package-archive")
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }

            activity.startActivity(intent)

            val result = JSObject()
            result.put("success", true)
            invoke.resolve(result)

        } catch (e: Exception) {
            invoke.reject("Failed to install APK: ${e.message}")
        }
    }
}
