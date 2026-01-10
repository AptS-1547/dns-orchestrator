import React from "react"
import ReactDOM from "react-dom/client"
import { initEnv, isAndroid, isIOS } from "@/lib/env"
import { storage, STORAGE_DEFAULTS } from "@/services/storage"
import App from "./App"
import "./index.css"
import "./i18n" // 初始化 i18n

// 初始化环境（获取平台信息）
initEnv()

// 添加平台类名（用于移动端特定样式）
if (isAndroid() || isIOS()) {
  document.documentElement.classList.add("platform-mobile")
}

// 在 React 渲染前初始化主题，避免闪烁
const theme = storage.getWithDefault("theme", STORAGE_DEFAULTS.theme)
const root = document.documentElement
if (theme === "system") {
  const systemDark = window.matchMedia("(prefers-color-scheme: dark)").matches
  root.classList.add(systemDark ? "dark" : "light")
} else {
  root.classList.add(theme)
}

// 渲染应用
// 注意：StrictMode 在开发环境下会让 useEffect 执行两次，用于检测副作用问题
// 这会导致 toast 等提示出现两次，但生产环境不会有此问题
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
)
