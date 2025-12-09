import { platform } from "@tauri-apps/plugin-os"

// 静态环境变量（编译时确定，不变）
export const ENV = {
  isDev: import.meta.env.DEV,
  isProd: import.meta.env.PROD,
  appVersion: __APP_VERSION__,
} as const

// 平台信息（运行时确定，初始化后不变）
let cachedPlatform: string | null = null

export function getPlatform(): string {
  if (cachedPlatform === null) {
    cachedPlatform = platform()
  }
  return cachedPlatform
}

export function isAndroid(): boolean {
  return cachedPlatform === "android"
}

export function isIOS(): boolean {
  return cachedPlatform === "ios"
}

export function isDesktop(): boolean {
  return cachedPlatform !== "android" && cachedPlatform !== "ios"
}

// 应用启动时调用一次
export function initEnv(): void {
  cachedPlatform = platform()
}
