import { Globe, Loader2 } from "lucide-react"
import { useCallback, useEffect, useRef } from "react"
import { useTranslation } from "react-i18next"
import { Badge } from "@/components/ui/badge"
import { Checkbox } from "@/components/ui/checkbox"
import { type DomainColorKey, getDomainColor } from "@/constants/colors"
import { cn } from "@/lib/utils"
import { useSettingsStore } from "@/stores"
import type { Domain, DomainStatus } from "@/types"

interface DomainListProps {
  domains: Domain[]
  selectedId: string | null
  onSelect: (id: string | null) => void
  hasMore?: boolean
  isLoadingMore?: boolean
  onLoadMore?: () => void
  // 批量模式支持
  isBatchMode?: boolean
  selectedKeys?: Set<string>
  onToggleSelection?: (accountId: string, domainId: string) => void
  accountId?: string
}

export function DomainList({
  domains,
  selectedId,
  onSelect,
  hasMore = false,
  isLoadingMore = false,
  onLoadMore,
  isBatchMode = false,
  selectedKeys = new Set(),
  onToggleSelection,
  accountId = "",
}: DomainListProps) {
  const { t } = useTranslation()
  const theme = useSettingsStore((state) => state.theme)
  const isDark =
    theme === "dark" ||
    (theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches)
  const sentinelRef = useRef<HTMLDivElement>(null)

  // 设置 IntersectionObserver 用于无限滚动
  const handleObserver = useCallback(
    (entries: IntersectionObserverEntry[]) => {
      const [entry] = entries
      if (entry.isIntersecting && hasMore && !isLoadingMore && onLoadMore) {
        onLoadMore()
      }
    },
    [hasMore, isLoadingMore, onLoadMore]
  )

  useEffect(() => {
    const sentinel = sentinelRef.current
    if (!(sentinel && onLoadMore)) return

    const observer = new IntersectionObserver(handleObserver, {
      rootMargin: "100px",
    })
    observer.observe(sentinel)

    return () => observer.disconnect()
  }, [handleObserver, onLoadMore])

  const statusConfig: Record<
    DomainStatus,
    { labelKey: string; variant: "default" | "secondary" | "destructive" | "outline" }
  > = {
    active: { labelKey: "domain.status.active", variant: "default" },
    paused: { labelKey: "domain.status.paused", variant: "secondary" },
    pending: { labelKey: "domain.status.pending", variant: "outline" },
    error: { labelKey: "domain.status.error", variant: "destructive" },
    unknown: { labelKey: "domain.status.unknown", variant: "outline" },
  }

  if (domains.length === 0) {
    return <div className="py-2 text-muted-foreground text-sm">{t("domain.noDomains")}</div>
  }

  return (
    <div className="space-y-1">
      {domains.map((domain) => {
        const domainKey = `${accountId}::${domain.id}`
        const isSelected = isBatchMode ? selectedKeys.has(domainKey) : selectedId === domain.id

        return (
          <button
            type="button"
            key={domain.id}
            onClick={() => {
              if (isBatchMode && onToggleSelection && accountId) {
                onToggleSelection(accountId, domain.id)
              } else {
                onSelect(selectedId === domain.id ? null : domain.id)
              }
            }}
            className={cn(
              "flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors",
              "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
              isSelected && "bg-sidebar-accent text-sidebar-accent-foreground",
              isBatchMode && isSelected && "ring-2 ring-primary"
            )}
          >
            {isBatchMode && (
              <Checkbox
                checked={isSelected}
                onCheckedChange={() => {
                  // Checkbox change handler
                }}
                onClick={(e) => e.stopPropagation()}
              />
            )}
            {/* 颜色色块（固定占位以保持对齐） */}
            <div
              className="h-3 w-0.5 shrink-0 rounded-full"
              style={{
                backgroundColor: domain.metadata?.color
                  ? getDomainColor(domain.metadata.color as DomainColorKey, isDark)
                  : "transparent",
              }}
            />
            <Globe className="h-4 w-4 shrink-0 text-muted-foreground" />
            <div className="flex-1 truncate text-left">
              <div className="truncate font-medium">{domain.name}</div>
              {domain.recordCount !== undefined && (
                <div className="text-muted-foreground text-xs">
                  {t("domain.recordCount", { count: domain.recordCount })}
                </div>
              )}
            </div>
            <Badge
              variant={statusConfig[domain.status]?.variant ?? "secondary"}
              className="text-xs"
            >
              {t(statusConfig[domain.status]?.labelKey ?? "domain.status.active")}
            </Badge>
          </button>
        )
      })}
      {/* 无限滚动触发点 */}
      <div ref={sentinelRef} className="h-1" />
      {isLoadingMore && (
        <div className="flex justify-center py-2">
          <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
        </div>
      )}
    </div>
  )
}
