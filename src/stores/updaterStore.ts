import { relaunch } from "@tauri-apps/plugin-process"
import { type Update, check } from "@tauri-apps/plugin-updater"
import { create } from "zustand"

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

interface UpdaterState {
  checking: boolean
  downloading: boolean
  progress: number
  available: Update | null
  error: string | null
  upToDate: boolean
  isPlatformUnsupported: boolean
  retryCount: number
  maxRetries: number
  checkForUpdates: () => Promise<Update | null>
  downloadAndInstall: () => Promise<void>
  skipVersion: () => void
  reset: () => void
  resetUpToDate: () => void
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

  checkForUpdates: async () => {
    set({ checking: true, error: null, upToDate: false, isPlatformUnsupported: false })
    try {
      console.log("[Updater] Checking for updates...")
      const update = await check()
      console.log("[Updater] Check result:", update)

      if (update) {
        // 检查该版本是否被跳过
        const skippedVersion = getSkippedVersion()
        console.log("[Updater] Skipped version:", skippedVersion)
        console.log("[Updater] Update version:", update.version)

        if (skippedVersion === update.version) {
          // 版本被跳过，当作无更新处理
          console.log("[Updater] Version is skipped, treating as no update")
          set({ available: null, checking: false, upToDate: true })
          return null
        }
        console.log("[Updater] Update available:", update.version)
        set({ available: update, checking: false, upToDate: false })
      } else {
        console.log("[Updater] No update available")
        set({ available: null, checking: false, upToDate: true })
      }
      return update
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e)
      console.error("[Updater] Check failed:", errorMessage, e)

      // 判断是否为平台不支持错误
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

    const attemptDownload = async (): Promise<void> => {
      let downloaded = 0
      let contentLength = 0

      console.log("[Updater] Starting download and install...")
      await available.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            contentLength = event.data.contentLength ?? 0
            console.log("[Updater] Download started, size:", contentLength)
            break
          case "Progress":
            downloaded += event.data.chunkLength
            if (contentLength > 0) {
              const progress = Math.round((downloaded / contentLength) * 100)
              set({ progress })
              console.log("[Updater] Download progress:", `${progress}%`)
            }
            break
          case "Finished":
            console.log("[Updater] Download finished")
            set({ progress: 100 })
            break
        }
      })
    }

    let lastError: Error | null = null

    for (let attempt = 0; attempt <= MAX_RETRIES; attempt++) {
      try {
        if (attempt > 0) {
          console.log(`[Updater] Retry attempt ${attempt}/${MAX_RETRIES}...`)
          set({ retryCount: attempt, progress: 0 })
          // 等待 2 秒后重试
          await new Promise((resolve) => setTimeout(resolve, 2000))
        }

        await attemptDownload()

        // 安装完成后清除跳过的版本记录，并重启应用
        console.log("[Updater] Install complete, relaunching...")
        clearSkippedVersion()
        await relaunch()
        return // 成功，退出函数
      } catch (e) {
        lastError = e instanceof Error ? e : new Error(String(e))
        console.error(`[Updater] Download attempt ${attempt + 1} failed:`, lastError.message)

        if (attempt === MAX_RETRIES) {
          // 已达到最大重试次数，放弃
          console.error("[Updater] All retry attempts failed")
          break
        }
      }
    }

    // 所有重试都失败了
    const errorMessage = lastError?.message || "Download failed"
    set({ error: errorMessage, downloading: false, retryCount: MAX_RETRIES })
    throw lastError || new Error(errorMessage)
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
    })
  },

  resetUpToDate: () => {
    set({ upToDate: false })
  },
}))
