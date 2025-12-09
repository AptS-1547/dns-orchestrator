import { toast } from "sonner"
import { TIMING, UI } from "@/constants"
import { ENV } from "@/lib/env"
import { useSettingsStore } from "@/stores/settingsStore"

const originalError = console.error

export function initDebugMode() {
  // 生产环境不初始化调试模式
  if (!ENV.isDev) return

  console.error = (...args: unknown[]) => {
    // 始终保留原始控制台输出
    originalError(...args)

    // 如果开启 debug 模式，则通过 Toast 显示
    if (useSettingsStore.getState().debugMode) {
      const message = args
        .map((a) => (typeof a === "object" ? JSON.stringify(a, null, 2) : String(a)))
        .join(" ")

      toast.error("控制台错误", {
        description: message.substring(0, UI.MAX_ERROR_MESSAGE_LENGTH),
        duration: TIMING.TOAST_DURATION,
      })
    }
  }
}
