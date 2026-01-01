import { useCallback } from "react"

/**
 * Enter 键处理 Hook
 * @param callback 按下 Enter 键时执行的回调函数
 * @returns 键盘事件处理函数
 */
export function useEnterKeyHandler(callback: () => void) {
  return useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter") {
        callback()
      }
    },
    [callback]
  )
}
