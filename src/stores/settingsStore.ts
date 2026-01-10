import { create } from "zustand"
import { changeLanguage, type LanguageCode, supportedLanguages } from "@/i18n"
import { type PaginationMode, STORAGE_DEFAULTS, storage, type Theme } from "@/services/storage"

// 获取初始语言（与 i18n 逻辑保持一致）
const getInitialLanguage = (): LanguageCode => {
  const saved = storage.get("language")
  if (saved && supportedLanguages.some((l) => l.code === saved)) {
    return saved as LanguageCode
  }
  // 尝试使用系统语言
  const systemLang = navigator.language
  if (systemLang.startsWith("en")) return "en-US"
  if (systemLang.startsWith("zh")) return "zh-CN"
  return "zh-CN"
}

interface SettingsState {
  theme: Theme
  language: LanguageCode
  debugMode: boolean
  sidebarCollapsed: boolean
  paginationMode: PaginationMode
  showRecordHints: boolean
  operationNotifications: boolean
  setTheme: (theme: Theme) => void
  setLanguage: (lang: LanguageCode) => void
  setDebugMode: (enabled: boolean) => void
  setSidebarCollapsed: (collapsed: boolean) => void
  setPaginationMode: (mode: PaginationMode) => void
  setShowRecordHints: (enabled: boolean) => void
  setOperationNotifications: (enabled: boolean) => void
}

export const useSettingsStore = create<SettingsState>((set) => ({
  theme: storage.getWithDefault("theme", STORAGE_DEFAULTS.theme),
  language: getInitialLanguage(),
  debugMode: storage.getWithDefault("debugMode", STORAGE_DEFAULTS.debugMode),
  sidebarCollapsed: storage.getWithDefault("sidebarCollapsed", STORAGE_DEFAULTS.sidebarCollapsed),
  paginationMode: storage.getWithDefault("paginationMode", STORAGE_DEFAULTS.paginationMode),
  showRecordHints: storage.getWithDefault("showRecordHints", STORAGE_DEFAULTS.showRecordHints),
  operationNotifications: storage.getWithDefault(
    "operationNotifications",
    STORAGE_DEFAULTS.operationNotifications
  ),

  setTheme: (theme) => {
    set({ theme })
    storage.set("theme", theme)

    // 应用主题
    const root = document.documentElement
    root.classList.remove("light", "dark")

    if (theme === "system") {
      const systemDark = window.matchMedia("(prefers-color-scheme: dark)").matches
      root.classList.add(systemDark ? "dark" : "light")
    } else {
      root.classList.add(theme)
    }
  },

  setLanguage: (lang) => {
    set({ language: lang })
    changeLanguage(lang)
  },

  setDebugMode: (enabled) => {
    set({ debugMode: enabled })
    storage.set("debugMode", enabled)
  },

  setSidebarCollapsed: (collapsed) => {
    set({ sidebarCollapsed: collapsed })
    storage.set("sidebarCollapsed", collapsed)
  },

  setPaginationMode: (mode) => {
    set({ paginationMode: mode })
    storage.set("paginationMode", mode)
  },

  setShowRecordHints: (enabled) => {
    set({ showRecordHints: enabled })
    storage.set("showRecordHints", enabled)
  },

  setOperationNotifications: (enabled) => {
    set({ operationNotifications: enabled })
    storage.set("operationNotifications", enabled)
  },
}))

// 初始化主题
export function initTheme() {
  const theme = storage.getWithDefault("theme", STORAGE_DEFAULTS.theme)

  // 同步更新 store 状态（确保 store 与 storage 一致）
  useSettingsStore.setState({ theme })

  const root = document.documentElement
  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)")

  // 应用当前主题
  const applyTheme = () => {
    const currentTheme = useSettingsStore.getState().theme
    root.classList.remove("light", "dark")

    if (currentTheme === "system") {
      root.classList.add(mediaQuery.matches ? "dark" : "light")
    } else {
      root.classList.add(currentTheme)
    }
  }

  applyTheme()

  // 监听系统主题变化
  mediaQuery.addEventListener("change", () => {
    const currentTheme = useSettingsStore.getState().theme
    if (currentTheme === "system") {
      applyTheme()
    }
  })
}
