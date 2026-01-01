import { X } from "lucide-react"
import { memo, useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog"
import { useToolboxStore } from "@/stores"
import type { QueryHistoryItem } from "@/types"

interface HistoryChipsProps {
  type: "whois" | "dns" | "ip" | "ssl" | "http" | "dns-propagation" | "dnssec"
  onSelect: (item: QueryHistoryItem) => void
  maxItems?: number
}

function HistoryChipsComponent({ type, onSelect, maxItems = 5 }: HistoryChipsProps) {
  const { t } = useTranslation()
  const { history, removeHistory, clearHistoryByType } = useToolboxStore()
  const [isAlertOpen, setIsAlertOpen] = useState(false)

  // 过滤并限制数量 - 使用 useMemo 缓存
  const filteredHistory = useMemo(
    () => history.filter((item) => item.type === type).slice(0, maxItems),
    [history, type, maxItems]
  )

  // 删除单条历史记录
  const handleRemove = (id: string, e: React.MouseEvent) => {
    e.stopPropagation()
    removeHistory(id)
    toast.success(t("toolbox.historyRemoved"))
  }

  // 清空当前类型的所有历史记录
  const handleConfirmClearAll = () => {
    clearHistoryByType(type)
    toast.success(t("toolbox.historyCleared"))
    setIsAlertOpen(false)
  }

  if (filteredHistory.length === 0) {
    return null
  }

  return (
    <div className="flex flex-wrap items-center gap-2">
      <span className="shrink-0 text-muted-foreground text-xs">{t("toolbox.history")}:</span>
      {filteredHistory.map((item) => (
        <button
          key={item.id}
          className="group inline-flex items-center gap-1 rounded-full bg-muted px-2 py-1 text-xs transition-colors hover:bg-muted/80"
          onClick={() => onSelect(item)}
        >
          {(item.type === "dns" || item.type === "dns-propagation") && item.recordType && (
            <span className="font-medium text-primary">{item.recordType}</span>
          )}
          <span className="max-w-32 truncate">{item.query}</span>
          <span
            className="opacity-0 transition-opacity hover:text-destructive group-hover:opacity-100"
            onClick={(e) => handleRemove(item.id, e)}
          >
            <X className="h-3 w-3" />
          </span>
        </button>
      ))}
      {history.filter((item) => item.type === type).length > maxItems && (
        <AlertDialog open={isAlertOpen} onOpenChange={setIsAlertOpen}>
          <AlertDialogTrigger asChild>
            <button
              type="button"
              className="px-2 py-1 text-muted-foreground text-xs transition-colors hover:text-foreground"
              title={t("toolbox.clearHistory")}
            >
              {t("toolbox.clearHistory")}
            </button>
          </AlertDialogTrigger>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>{t("toolbox.clearHistory")}</AlertDialogTitle>
              <AlertDialogDescription>{t("toolbox.confirmClearHistory")}</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>{t("common.cancel")}</AlertDialogCancel>
              <AlertDialogAction onClick={handleConfirmClearAll}>
                {t("common.confirm")}
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      )}
    </div>
  )
}

export const HistoryChips = memo(HistoryChipsComponent)
