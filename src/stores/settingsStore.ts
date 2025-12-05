import { type LanguageCode, changeLanguage, supportedLanguages } from "@/i18n"
import { create } from "zustand"

type Theme = "light" | "dark" | "system"

// 获取初始语言（与 i18n 逻辑保持一致）
const getInitialLanguage = (): LanguageCode => {
  const saved = localStorage.getItem("language")
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
  setTheme: (theme: Theme) => void
  setLanguage: (lang: LanguageCode) => void
}

export const useSettingsStore = create<SettingsState>((set) => ({
  theme: (localStorage.getItem("theme") as Theme) || "system",
  language: getInitialLanguage(),

  setTheme: (theme) => {
    set({ theme })
    localStorage.setItem("theme", theme)

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
}))

// 初始化主题
export function initTheme() {
  const theme = (localStorage.getItem("theme") as Theme) || "system"
  const root = document.documentElement

  if (theme === "system") {
    const systemDark = window.matchMedia("(prefers-color-scheme: dark)").matches
    root.classList.add(systemDark ? "dark" : "light")
  } else {
    root.classList.add(theme)
  }
}
