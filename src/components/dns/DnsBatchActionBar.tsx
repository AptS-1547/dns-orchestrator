import { Loader2, Trash2, X } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import { useIsMobile } from "@/hooks/useMediaQuery"

interface DnsBatchActionBarProps {
  /** 选中的记录数 */
  selectedCount: number
  /** 是否正在批量删除 */
  isDeleting: boolean
  /** 清除选择 */
  onClearSelection: () => void
  /** 删除选中记录 */
  onDelete: () => void
}

export function DnsBatchActionBar({
  selectedCount,
  isDeleting,
  onClearSelection,
  onDelete,
}: DnsBatchActionBarProps) {
  const { t } = useTranslation()
  const isMobile = useIsMobile()

  if (selectedCount === 0) return null

  return (
    <div className="fixed inset-x-0 bottom-14 z-50 mx-auto flex w-fit items-center gap-2 rounded-full border bg-background px-3 py-2 shadow-lg md:gap-3 md:px-4">
      <span className="text-muted-foreground text-sm">
        {isMobile
          ? t("dns.selectedCountShort", { count: selectedCount })
          : t("dns.selectedCount", { count: selectedCount })}
      </span>

      {isMobile ? (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button variant="ghost" size="icon" className="h-8 w-8" onClick={onClearSelection}>
                <X className="h-4 w-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>{t("common.deselectAll")}</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="destructive"
                size="icon"
                className="h-8 w-8"
                onClick={onDelete}
                disabled={isDeleting}
              >
                {isDeleting ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Trash2 className="h-4 w-4" />
                )}
              </Button>
            </TooltipTrigger>
            <TooltipContent>{t("dns.batchDelete")}</TooltipContent>
          </Tooltip>
        </TooltipProvider>
      ) : (
        <>
          <Button variant="ghost" size="sm" onClick={onClearSelection}>
            {t("common.deselectAll")}
          </Button>
          <Button variant="destructive" size="sm" onClick={onDelete} disabled={isDeleting}>
            {isDeleting ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Trash2 className="mr-2 h-4 w-4" />
            )}
            {t("dns.batchDelete")}
          </Button>
        </>
      )}
    </div>
  )
}
