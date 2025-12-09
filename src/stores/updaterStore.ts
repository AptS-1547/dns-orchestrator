import { Channel, invoke } from "@tauri-apps/api/core"
import { relaunch } from "@tauri-apps/plugin-process"
import { check, type Update } from "@tauri-apps/plugin-updater"
import { create } from "zustand"
import { ENV, getPlatform } from "@/lib/env"
import { logger } from "@/lib/logger"

const SKIPPED_VERSION_KEY = "dns-orchestrator-skipped-version"

// 获取被跳过的版本
const getSkippedVersion = (): string | null => {
  return localStorage.getItem(SKIPPED_VERSION_KEY)
}

// 设置被跳过的版本
const setSkippedVersion = (version: string): void => {
  localStorage.setItem(SKIPPED_VERSION_KEY, version)
}

// 清除被跳过的版本
const clearSkippedVersion = (): void => {
  localStorage.removeItem(SKIPPED_VERSION_KEY)
}

const MAX_RETRIES = 3

// Android 更新信息
interface AndroidUpdate {
  version: string
  notes: string
  url: string
}

// 下载进度事件
interface DownloadProgress {
  event: "Started" | "Progress" | "Finished"
  data: {
    content_length?: number
    chunk_length?: number
  }
}

// 统一的更新信息类型
type UpdateInfo = Update | AndroidUpdate

interface UpdaterState {
  checking: boolean
  downloading: boolean
  progress: number
  available: UpdateInfo | null
  error: string | null
  upToDate: boolean
  isPlatformUnsupported: boolean
  retryCount: number
  maxRetries: number
  showUpdateDialog: boolean
  checkForUpdates: () => Promise<UpdateInfo | null>
  downloadAndInstall: () => Promise<void>
  skipVersion: () => void
  reset: () => void
  resetUpToDate: () => void
  setShowUpdateDialog: (show: boolean) => void
}

// 判断是否为 Android 更新
const isAndroidUpdate = (update: UpdateInfo): update is AndroidUpdate => {
  return "url" in update && !("downloadAndInstall" in update)
}

// 统一获取更新说明
export const getUpdateNotes = (update: UpdateInfo | null): string => {
  if (!update) return ""
  if (isAndroidUpdate(update)) {
    return update.notes || ""
  }
  return update.body || ""
}

export const useUpdaterStore = create<UpdaterState>((set, get) => ({
  checking: false,
  downloading: false,
  progress: 0,
  available: null,
  error: null,
  upToDate: false,
  isPlatformUnsupported: false,
  retryCount: 0,
  maxRetries: MAX_RETRIES,
  showUpdateDialog: false,

  setShowUpdateDialog: (show: boolean) => set({ showUpdateDialog: show }),

  checkForUpdates: async () => {
    // Web 端不支持更新
    if (__PLATFORM__ === "web") {
      set({ isPlatformUnsupported: true })
      return null
    }

    set({ checking: true, error: null, upToDate: false, isPlatformUnsupported: false })
    try {
      const currentPlatform = getPlatform()
      logger.debug("[Updater] Platform:", currentPlatform)

      if (currentPlatform === "android") {
        // Android 平台：调用自定义命令
        logger.debug("[Updater] Checking for Android updates...")
        const update = await invoke<AndroidUpdate | null>("check_android_update", {
          currentVersion: ENV.appVersion,
        })
        logger.debug("[Updater] Android check result:", update)

        if (update) {
          const skippedVersion = getSkippedVersion()
          if (skippedVersion === update.version) {
            logger.debug("[Updater] Version is skipped")
            set({ available: null, checking: false, upToDate: true })
            return null
          }
          logger.debug("[Updater] Update available:", update.version)
          set({ available: update, checking: false, upToDate: false, showUpdateDialog: true })
          return update
        } else {
          logger.debug("[Updater] No update available")
          set({ available: null, checking: false, upToDate: true })
          return null
        }
      } else {
        // 桌面平台：使用 tauri-plugin-updater
        logger.debug("[Updater] Checking for desktop updates...")
        const update = await check()
        logger.debug("[Updater] Check result:", update)

        if (update) {
          const skippedVersion = getSkippedVersion()
          logger.debug("[Updater] Skipped version:", skippedVersion)
          logger.debug("[Updater] Update version:", update.version)

          if (skippedVersion === update.version) {
            logger.debug("[Updater] Version is skipped, treating as no update")
            set({ available: null, checking: false, upToDate: true })
            return null
          }
          logger.debug("[Updater] Update available:", update.version)
          set({ available: update, checking: false, upToDate: false, showUpdateDialog: true })
        } else {
          logger.debug("[Updater] No update available")
          set({ available: null, checking: false, upToDate: true })
        }
        return update
      }
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      console.error("[Updater] Check failed:", errorMessage, e)

      const isPlatformError =
        errorMessage.includes("platform") && errorMessage.includes("was not found")

      set({
        error: errorMessage,
        checking: false,
        upToDate: false,
        isPlatformUnsupported: isPlatformError,
      })
      return null
    }
  },

  downloadAndInstall: async () => {
    const { available } = get()
    if (!available) return

    set({ downloading: true, progress: 0, error: null, retryCount: 0 })

    const currentPlatform = getPlatform()

    if (currentPlatform === "android" && isAndroidUpdate(available)) {
      // Android 平台：下载 APK 并安装
      let lastError: Error | null = null
      let apkPath: string | null = null

      // 阶段 1: 下载 APK（可重试）
      let downloaded = 0
      let contentLength = 0

      for (let attempt = 0; attempt < MAX_RETRIES; attempt++) {
        try {
          if (attempt > 0) {
            logger.debug(`[Updater] Download retry ${attempt}/${MAX_RETRIES - 1}...`)
            set({ retryCount: attempt, progress: 0 })
            await new Promise((resolve) => setTimeout(resolve, 2000))
          }

          logger.debug("[Updater] Starting Android APK download...")
          downloaded = 0
          contentLength = 0

          // 创建进度回调 Channel
          const onProgress = new Channel<DownloadProgress>()
          onProgress.onmessage = (event) => {
            if (event.event === "Started") {
              contentLength = event.data.content_length ?? 0
              logger.debug("[Updater] Download started, size:", contentLength)
            } else if (event.event === "Progress") {
              downloaded += event.data.chunk_length ?? 0
              if (contentLength > 0) {
                const progress = Math.round((downloaded / contentLength) * 100)
                set({ progress })
              }
            } else if (event.event === "Finished") {
              logger.debug("[Updater] Download finished")
              set({ progress: 100 })
            }
          }

          // 下载 APK
          apkPath = await invoke<string>("download_apk", {
            url: available.url,
            onProgress,
          })
          logger.debug("[Updater] APK downloaded to:", apkPath)
          break // 下载成功，跳出重试循环
        } catch (e) {
          lastError = e instanceof Error ? e : new Error(String(e))

          // 详细错误日志
          console.error(`[Updater] Download attempt ${attempt + 1}/${MAX_RETRIES} failed:`, {
            error: lastError.message,
            stack: lastError.stack,
            url: available.url,
            downloaded: `${downloaded} / ${contentLength} bytes`,
            progress:
              contentLength > 0 ? `${Math.round((downloaded / contentLength) * 100)}%` : "unknown",
            errorType: lastError.constructor.name,
          })

          if (attempt === MAX_RETRIES - 1) {
            console.error("[Updater] All download retry attempts failed", {
              totalAttempts: MAX_RETRIES,
              lastError: {
                message: lastError.message,
                stack: lastError.stack,
              },
              url: available.url,
              version: available.version,
            })
            const errorMessage = lastError?.message || "Download failed"
            set({ error: errorMessage, downloading: false, retryCount: MAX_RETRIES })
            throw lastError || new Error(errorMessage)
          }
        }
      }

      // 阶段 2: 安装 APK（不重试下载）
      if (apkPath) {
        try {
          logger.debug("[Updater] Installing APK...")
          await invoke("install_apk", { path: apkPath })
          clearSkippedVersion()
          set({ downloading: false })
        } catch (e) {
          const installError = e instanceof Error ? e : new Error(String(e))
          console.error("[Updater] Install failed:", {
            error: installError.message,
            stack: installError.stack,
            apkPath,
            errorType: installError.constructor.name,
          })
          // 安装失败不重新下载，只提示错误
          set({ error: installError.message, downloading: false })
          throw installError
        }
      }
    } else if (!isAndroidUpdate(available)) {
      // 桌面平台：使用 tauri-plugin-updater
      const attemptDownload = async (): Promise<void> => {
        let downloaded = 0
        let contentLength = 0

        logger.debug("[Updater] Starting download and install...")
        await available.downloadAndInstall((event) => {
          switch (event.event) {
            case "Started":
              contentLength = event.data.contentLength ?? 0
              logger.debug("[Updater] Download started, size:", contentLength)
              break
            case "Progress":
              downloaded += event.data.chunkLength
              if (contentLength > 0) {
                const progress = Math.round((downloaded / contentLength) * 100)
                set({ progress })
                logger.debug("[Updater] Download progress:", `${progress}%`)
              }
              break
            case "Finished":
              logger.debug("[Updater] Download finished")
              set({ progress: 100 })
              break
          }
        })
      }

      let lastError: Error | null = null

      for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
        try {
          if (attempt > 0) {
            logger.debug(`[Updater] Retry attempt ${attempt}/${MAX_RETRIES}...`)
            set({ retryCount: attempt, progress: 0 })
            await new Promise((resolve) => setTimeout(resolve, 2000))
          }

          await attemptDownload()

          logger.debug("[Updater] Install complete, relaunching...")
          clearSkippedVersion()
          await relaunch()
          return
        } catch (e) {
          lastError = e instanceof Error ? e : new Error(String(e))
          console.error(`[Updater] Download attempt ${attempt + 1} failed:`, lastError.message)

          if (attempt === MAX_RETRIES) {
            console.error("[Updater] All retry attempts failed")
            break
          }
        }
      }

      const errorMessage = lastError?.message || "Download failed"
      set({ error: errorMessage, downloading: false, retryCount: MAX_RETRIES })
      throw lastError || new Error(errorMessage)
    }
  },

  skipVersion: () => {
    const { available } = get()
    if (available) {
      setSkippedVersion(available.version)
      set({ available: null, upToDate: true })
    }
  },

  reset: () => {
    set({
      checking: false,
      downloading: false,
      progress: 0,
      available: null,
      error: null,
      upToDate: false,
      isPlatformUnsupported: false,
      retryCount: 0,
      showUpdateDialog: false,
    })
  },

  resetUpToDate: () => {
    set({ upToDate: false })
  },
}))
