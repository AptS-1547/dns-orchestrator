import { ENV } from "@/lib/env"

type LogLevel = "debug" | "info" | "warn" | "error"

interface LoggerOptions {
  /** 模块名称，用于日志前缀 */
  module: string
}

/**
 * 创建一个带模块前缀的 logger
 * 生产环境下 debug 和 info 级别的日志会被静默
 *
 * @example
 * const log = createLogger({ module: "Updater" })
 * log.debug("Checking for updates...")  // [Updater] Checking for updates...
 * log.error("Failed:", error)           // [Updater] Failed: ...
 */
export function createLogger({ module }: LoggerOptions) {
  const prefix = `[${module}]`

  const shouldLog = (level: LogLevel): boolean => {
    // 生产环境只输出 warn 和 error
    if (ENV.isProd) {
      return level === "warn" || level === "error"
    }
    return true
  }

  return {
    debug: (...args: unknown[]) => {
      if (shouldLog("debug")) {
        console.log(prefix, ...args)
      }
    },
    info: (...args: unknown[]) => {
      if (shouldLog("info")) {
        console.info(prefix, ...args)
      }
    },
    warn: (...args: unknown[]) => {
      if (shouldLog("warn")) {
        console.warn(prefix, ...args)
      }
    },
    error: (...args: unknown[]) => {
      if (shouldLog("error")) {
        console.error(prefix, ...args)
      }
    },
  }
}

/**
 * 全局 logger，用于没有特定模块的场景
 */
export const logger = createLogger({ module: "App" })
