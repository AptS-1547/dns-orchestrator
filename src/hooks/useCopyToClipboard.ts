import { useCallback } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"

/**
 * 复制到剪贴板 Hook
 * @returns 复制函数
 */
export function useCopyToClipboard() {
  const { t } = useTranslation()

  return useCallback(
    async (text: string, successMessage?: string) => {
      try {
        await navigator.clipboard.writeText(text)
        toast.success(successMessage || t("common.copied"))
      } catch (err) {
        toast.error(t("common.copyFailed"))
      }
    },
    [t]
  )
}
