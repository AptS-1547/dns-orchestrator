import { invoke } from "@tauri-apps/api/core"
import { useCallback, useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { getErrorMessage } from "@/lib/error"
import { useToolboxStore } from "@/stores"
import type { ApiResponse, QueryHistoryItem } from "@/types"

export interface UseToolboxQueryOptions<TParams> {
  /** Tauri command 名称 */
  commandName: string
  /** 历史记录类型 */
  historyType: QueryHistoryItem["type"]
  /** 生成历史记录的查询字符串 */
  getHistoryQuery: (params: TParams) => string
  /** 额外的历史记录字段（如 DNS 的 recordType） */
  getHistoryExtra?: (params: TParams) => Partial<QueryHistoryItem>
}

export interface UseToolboxQueryReturn<TParams, TResult> {
  /** 是否正在加载 */
  isLoading: boolean
  /** 查询结果 */
  result: TResult | null
  /** 执行查询 */
  execute: (params: TParams) => Promise<TResult | null>
  /** 重置结果 */
  reset: () => void
}

/**
 * 通用工具箱查询 Hook
 * 统一管理：loading 状态、Tauri invoke 调用、历史记录添加、错误提示
 */
export function useToolboxQuery<TParams extends Record<string, unknown>, TResult>(
  options: UseToolboxQueryOptions<TParams>
): UseToolboxQueryReturn<TParams, TResult> {
  const { commandName, historyType, getHistoryQuery, getHistoryExtra } = options
  const { t } = useTranslation()
  const { addHistory } = useToolboxStore()

  const [isLoading, setIsLoading] = useState(false)
  const [result, setResult] = useState<TResult | null>(null)

  const execute = useCallback(
    async (params: TParams): Promise<TResult | null> => {
      setIsLoading(true)
      setResult(null)

      try {
        const response = await invoke<ApiResponse<TResult>>(commandName, params)

        if (response.success && response.data) {
          setResult(response.data)
          // 添加历史记录
          addHistory({
            type: historyType,
            query: getHistoryQuery(params),
            ...getHistoryExtra?.(params),
          })
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
    [commandName, historyType, getHistoryQuery, getHistoryExtra, addHistory, t]
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
