import { X } from "lucide-react"
import { useTranslation } from "react-i18next"
import { useToolboxStore } from "@/stores"
import type { QueryHistoryItem } from "@/types"

interface HistoryChipsProps {
  type: "whois" | "dns" | "ip" | "ssl" | "http"
  onSelect: (item: QueryHistoryItem) => void
  maxItems?: number
}

export function HistoryChips({ type, onSelect, maxItems = 5 }: HistoryChipsProps) {
  const { t } = useTranslation()
  const { history, removeHistory, clearHistory } = useToolboxStore()

  // 过滤并限制数量
  const filteredHistory = history.filter((item) => item.type === type).slice(0, maxItems)

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
          {item.type === "dns" && item.recordType && (
            <span className="font-medium text-primary">{item.recordType}</span>
          )}
          <span className="max-w-32 truncate">{item.query}</span>
          <span
            className="opacity-0 transition-opacity hover:text-destructive group-hover:opacity-100"
            onClick={(e) => {
              e.stopPropagation()
              removeHistory(item.id)
            }}
          >
            <X className="h-3 w-3" />
          </span>
        </button>
      ))}
      {history.filter((item) => item.type === type).length > maxItems && (
        <button
          className="px-2 py-1 text-muted-foreground text-xs transition-colors hover:text-foreground"
          onClick={() => clearHistory()}
          title={t("toolbox.clearHistory")}
        >
          {t("toolbox.clearHistory")}
        </button>
      )}
    </div>
  )
}
