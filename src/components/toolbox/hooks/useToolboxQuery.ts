import { useCallback, useState } from "react"
import { toast } from "sonner"
import { getErrorMessage } from "@/lib/error"
import { toolboxService } from "@/services"
import { useToolboxStore } from "@/stores"
import type { ApiResponse, QueryHistoryItem } from "@/types"

export interface UseToolboxQueryReturn<TResult> {
  /** 是否正在加载 */
  isLoading: boolean
  /** 查询结果 */
  result: TResult | null
  /** 执行查询 */
  execute: (
    serviceFn: () => Promise<ApiResponse<TResult>>,
    historyItem: Omit<QueryHistoryItem, "id" | "timestamp">
  ) => Promise<TResult | null>
  /** 重置结果 */
  reset: () => void
}

/**
 * 通用工具箱查询 Hook
 * 统一管理：loading 状态、Service 调用、历史记录添加、错误提示
 */
export function useToolboxQuery<TResult>(): UseToolboxQueryReturn<TResult> {
  const { addHistory } = useToolboxStore()

  const [isLoading, setIsLoading] = useState(false)
  const [result, setResult] = useState<TResult | null>(null)

  const execute = useCallback(
    async (
      serviceFn: () => Promise<ApiResponse<TResult>>,
      historyItem: Omit<QueryHistoryItem, "id" | "timestamp">
    ): Promise<TResult | null> => {
      setIsLoading(true)
      setResult(null)

      try {
        const response = await serviceFn()

        if (response.success && response.data) {
          setResult(response.data)
          addHistory(historyItem)
          return response.data
        }

        toast.error(getErrorMessage(response.error))
        return null
      } catch (err) {
        toast.error(String(err))
        return null
      } finally {
        setIsLoading(false)
      }
    },
    [addHistory]
  )

  const reset = useCallback(() => {
    setResult(null)
  }, [])

  return {
    isLoading,
    result,
    execute,
    reset,
  }
}

// ============ 导出 toolboxService 供组件使用 ============
export { toolboxService }
