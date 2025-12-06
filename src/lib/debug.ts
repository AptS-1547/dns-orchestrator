import { toast } from "sonner"
import { useSettingsStore } from "@/stores/settingsStore"

const originalError = console.error

export function initDebugMode() {
  console.error = (...args: unknown[]) => {
    // 始终保留原始控制台输出
    originalError(...args)

    // 如果开启 debug 模式，则通过 Toast 显示
    if (useSettingsStore.getState().debugMode) {
      const message = args
        .map((a) => (typeof a === "object" ? JSON.stringify(a, null, 2) : String(a)))
        .join(" ")

      toast.error("控制台错误", {
        description: message.substring(0, 300),
        duration: 5000,
      })
    }
  }
}
