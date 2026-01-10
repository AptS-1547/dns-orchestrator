import { create } from "zustand"
import { TIMING } from "@/constants"
import { ENV, getPlatform } from "@/lib/env"
import { logger } from "@/lib/logger"
import { storage } from "@/services/storage"

const GITHUB_RELEASES_API =
  "https://api.github.com/repos/AptS-1547/dns-orchestrator/releases/latest"

// 获取被跳过的版本
const getSkippedVersion = (): string | null => {
  return storage.get("skippedVersion")
}

// 设置被跳过的版本
const setSkippedVersion = (version: string): void => {
  storage.set("skippedVersion", version)
}

// 清除被跳过的版本
const clearSkippedVersion = (): void => {
  storage.remove("skippedVersion")
}

const MAX_RETRIES = 3

// ============ 更新信息类型 ============

/** Web 端更新信息（来自 GitHub Release） */
interface WebUpdate {
  version: string
  notes: string
  url: string // GitHub Release 页面 URL
}

/** 桌面端更新信息（来自 tauri-plugin-updater） */
interface DesktopUpdate {
  version: string
  body?: string
  downloadAndInstall: (onProgress?: (event: DownloadEvent) => void) => Promise<void>
}

/** Android 更新信息 */
interface AndroidUpdate {
  version: string
  notes: string
  url: string // APK 下载 URL
}

/** 下载进度事件 */
interface DownloadEvent {
  event: "Started" | "Progress" | "Finished"
  data?: {
    contentLength?: number
    chunkLength?: number
  }
}

// 统一的更新信息类型
type UpdateInfo = WebUpdate | DesktopUpdate | AndroidUpdate

// ============ 类型判断 ============

const isWebUpdate = (update: UpdateInfo): update is WebUpdate => {
  return "url" in update && !("downloadAndInstall" in update) && __PLATFORM__ === "web"
}

const isAndroidUpdate = (update: UpdateInfo): update is AndroidUpdate => {
  return "url" in update && !("downloadAndInstall" in update) && __PLATFORM__ !== "web"
}

const isDesktopUpdate = (update: UpdateInfo): update is DesktopUpdate => {
  return "downloadAndInstall" in update
}

// ============ 更新说明获取 ============

export const getUpdateNotes = (update: UpdateInfo | null): string => {
  if (!update) return ""
  if ("notes" in update) return update.notes || ""
  if ("body" in update) return update.body || ""
  return ""
}

// ============ GitHub Release API ============

async function checkGitHubRelease(): Promise<WebUpdate | null> {
  try {
    const response = await fetch(GITHUB_RELEASES_API)
    if (!response.ok) {
      throw new Error(`GitHub API error: ${response.status}`)
    }

    const data = await response.json()
    const latestVersion = data.tag_name?.replace(/^v/, "") || ""

    // 比较版本
    if (latestVersion && latestVersion !== ENV.appVersion) {
      return {
        version: latestVersion,
        notes: data.body || "",
        url: data.html_url || "",
      }
    }

    return null
  } catch (error) {
    logger.error("Failed to check GitHub release:", error)
    throw error
  }
}

// ============ Store 定义 ============

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
    set({ checking: true, error: null, upToDate: false, isPlatformUnsupported: false })

    try {
      // Web 端：使用 GitHub Release API
      if (__PLATFORM__ === "web") {
        logger.debug("Checking for Web updates via GitHub...")
        const update = await checkGitHubRelease()

        if (update) {
          const skippedVersion = getSkippedVersion()
          if (skippedVersion === update.version) {
            logger.debug("Version is skipped")
            set({ available: null, checking: false, upToDate: true })
            return null
          }
          logger.debug("Update available:", update.version)
          set({ available: update, checking: false, upToDate: false, showUpdateDialog: true })
          return update
        }

        logger.debug("No update available")
        set({ available: null, checking: false, upToDate: true })
        return null
      }

      const currentPlatform = getPlatform()
      logger.debug("Platform:", currentPlatform)

      // Android 端
      if (currentPlatform === "android") {
        logger.debug("Checking for Android updates...")
        const { invoke } = await import("@tauri-apps/api/core")
        const update = await invoke<AndroidUpdate | null>("check_android_update", {
          currentVersion: ENV.appVersion,
        })
        logger.debug("Android check result:", update)

        if (update) {
          const skippedVersion = getSkippedVersion()
          if (skippedVersion === update.version) {
            logger.debug("Version is skipped")
            set({ available: null, checking: false, upToDate: true })
            return null
          }
          logger.debug("Update available:", update.version)
          set({ available: update, checking: false, upToDate: false, showUpdateDialog: true })
          return update
        }

        logger.debug("No update available")
        set({ available: null, checking: false, upToDate: true })
        return null
      }

      // 桌面端：使用 tauri-plugin-updater
      logger.debug("Checking for desktop updates...")
      const { check } = await import("@tauri-apps/plugin-updater")
      const update = await check()
      logger.debug("Check result:", update)

      if (update) {
        const skippedVersion = getSkippedVersion()
        logger.debug("Skipped version:", skippedVersion)
        logger.debug("Update version:", update.version)

        if (skippedVersion === update.version) {
          logger.debug("Version is skipped, treating as no update")
          set({ available: null, checking: false, upToDate: true })
          return null
        }
        logger.debug("Update available:", update.version)
        // 使用类型断言，因为 tauri-plugin-updater 的 Update 类型兼容我们的 DesktopUpdate
        set({
          available: update as unknown as DesktopUpdate,
          checking: false,
          upToDate: false,
          showUpdateDialog: true,
        })
      } else {
        logger.debug("No update available")
        set({ available: null, checking: false, upToDate: true })
      }
      return update as unknown as UpdateInfo | null
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      logger.error("Check failed:", errorMessage, e)

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

    // Web 端不支持自动下载安装
    if (__PLATFORM__ === "web") {
      logger.warn("Web platform does not support auto download and install")
      return
    }

    set({ downloading: true, progress: 0, error: null, retryCount: 0 })

    const currentPlatform = getPlatform()

    // Android 端
    if (currentPlatform === "android" && isAndroidUpdate(available)) {
      const { invoke } = await import("@tauri-apps/api/core")
      const { Channel } = await import("@tauri-apps/api/core")

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
            await new Promise((resolve) => setTimeout(resolve, TIMING.UPDATE_RETRY_DELAY))
          }

          logger.debug("Starting Android APK download...")
          downloaded = 0
          contentLength = 0

          // 创建进度回调 Channel
          const onProgress = new Channel<{
            event: "Started" | "Progress" | "Finished"
            data: { content_length?: number; chunk_length?: number }
          }>()
          onProgress.onmessage = (event) => {
            if (event.event === "Started") {
              contentLength = event.data.content_length ?? 0
              logger.debug("Download started, size:", contentLength)
            } else if (event.event === "Progress") {
              downloaded += event.data.chunk_length ?? 0
              if (contentLength > 0) {
                const progress = Math.round((downloaded / contentLength) * 100)
                set({ progress })
              }
            } else if (event.event === "Finished") {
              logger.debug("Download finished")
              set({ progress: 100 })
            }
          }

          // 下载 APK
          apkPath = await invoke<string>("download_apk", {
            url: available.url,
            onProgress,
          })
          logger.debug("APK downloaded to:", apkPath)
          break // 下载成功，跳出重试循环
        } catch (e) {
          lastError = e instanceof Error ? e : new Error(String(e))
          logger.error(`Download attempt ${attempt + 1}/${MAX_RETRIES} failed:`, lastError.message)

          if (attempt === MAX_RETRIES - 1) {
            const errorMessage = lastError?.message || "Download failed"
            set({ error: errorMessage, downloading: false, retryCount: MAX_RETRIES })
            throw lastError || new Error(errorMessage)
          }
        }
      }

      // 阶段 2: 安装 APK（不重试下载）
      if (apkPath) {
        try {
          logger.debug("Installing APK...")
          await invoke("install_apk", { path: apkPath })
          clearSkippedVersion()
          set({ downloading: false })
        } catch (e) {
          const installError = e instanceof Error ? e : new Error(String(e))
          logger.error("Install failed:", installError.message)
          set({ error: installError.message, downloading: false })
          throw installError
        }
      }
    } else if (isDesktopUpdate(available)) {
      // 桌面端：使用 tauri-plugin-updater
      const attemptDownload = async (): Promise<void> => {
        let downloaded = 0
        let contentLength = 0

        logger.debug("Starting download and install...")
        await available.downloadAndInstall((event) => {
          switch (event.event) {
            case "Started":
              contentLength = event.data?.contentLength ?? 0
              logger.debug("Download started, size:", contentLength)
              break
            case "Progress":
              downloaded += event.data?.chunkLength ?? 0
              if (contentLength > 0) {
                const progress = Math.round((downloaded / contentLength) * 100)
                set({ progress })
                logger.debug("Download progress:", `${progress}%`)
              }
              break
            case "Finished":
              logger.debug("Download finished")
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
            await new Promise((resolve) => setTimeout(resolve, TIMING.UPDATE_RETRY_DELAY))
          }

          await attemptDownload()

          logger.debug("Install complete, relaunching...")
          clearSkippedVersion()

          const { relaunch } = await import("@tauri-apps/plugin-process")
          await relaunch()
          return
        } catch (e) {
          lastError = e instanceof Error ? e : new Error(String(e))
          logger.error(`Download attempt ${attempt + 1} failed:`, lastError.message)

          if (attempt === MAX_RETRIES) {
            logger.error("All retry attempts failed")
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

// 导出类型判断函数供 update-dialog 使用
export { isWebUpdate }
