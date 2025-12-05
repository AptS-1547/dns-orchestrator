import React from "react"
import ReactDOM from "react-dom/client"
import App from "./App"
import "./index.css"
import "./i18n" // 初始化 i18n

// 在 React 渲染前初始化主题，避免闪烁
const theme = localStorage.getItem("theme") || "system"
const root = document.documentElement
if (theme === "system") {
  const systemDark = window.matchMedia("(prefers-color-scheme: dark)").matches
  root.classList.add(systemDark ? "dark" : "light")
} else {
  root.classList.add(theme)
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
)
