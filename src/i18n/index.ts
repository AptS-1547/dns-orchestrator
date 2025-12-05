import i18n from "i18next"
import { initReactI18next } from "react-i18next"
import enUS from "./locales/en-US"
import zhCN from "./locales/zh-CN"

// 支持的语言列表
export const supportedLanguages = [
  { code: "zh-CN", name: "简体中文" },
  { code: "en-US", name: "English" },
  // { code: "ja-JP", name: "日本語" }, // 后续扩展
] as const

export type LanguageCode = (typeof supportedLanguages)[number]["code"]

// 从 localStorage 获取语言设置，默认中文
const getInitialLanguage = (): LanguageCode => {
  const saved = localStorage.getItem("language")
  if (saved && supportedLanguages.some((l) => l.code === saved)) {
    return saved as LanguageCode
  }
  // 尝试使用系统语言
  const systemLang = navigator.language
  if (systemLang.startsWith("zh")) return "zh-CN"
  if (systemLang.startsWith("en")) return "en-US"
  return "zh-CN"
}

i18n.use(initReactI18next).init({
  resources: {
    "zh-CN": { translation: zhCN },
    "en-US": { translation: enUS },
  },
  lng: getInitialLanguage(),
  fallbackLng: "zh-CN",
  interpolation: {
    escapeValue: false, // React 已经处理了 XSS
  },
})

// 语言切换函数
export const changeLanguage = (lang: LanguageCode) => {
  i18n.changeLanguage(lang)
  localStorage.setItem("language", lang)
}

export default i18n
