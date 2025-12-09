import { Globe, Loader2 } from "lucide-react"
import { useCallback, useEffect, useRef } from "react"
import { useTranslation } from "react-i18next"
import { Badge } from "@/components/ui/badge"
import { cn } from "@/lib/utils"
import type { Domain, DomainStatus } from "@/types"

interface DomainListProps {
  domains: Domain[]
  selectedId: string | null
  onSelect: (id: string | null) => void
  hasMore?: boolean
  isLoadingMore?: boolean
  onLoadMore?: () => void
}

export function DomainList({
  domains,
  selectedId,
  onSelect,
  hasMore = false,
  isLoadingMore = false,
  onLoadMore,
}: DomainListProps) {
  const { t } = useTranslation()
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
      {domains.map((domain) => (
        <button
          type="button"
          key={domain.id}
          onClick={() => onSelect(selectedId === domain.id ? null : domain.id)}
          className={cn(
            "flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors",
            "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
            selectedId === domain.id && "bg-sidebar-accent text-sidebar-accent-foreground"
          )}
        >
          <Globe className="h-4 w-4 shrink-0 text-muted-foreground" />
          <div className="flex-1 truncate text-left">
            <div className="truncate font-medium">{domain.name}</div>
            {domain.recordCount !== undefined && (
              <div className="text-muted-foreground text-xs">
                {t("domain.recordCount", { count: domain.recordCount })}
              </div>
            )}
          </div>
          <Badge variant={statusConfig[domain.status]?.variant ?? "secondary"} className="text-xs">
            {t(statusConfig[domain.status]?.labelKey ?? "domain.status.active")}
          </Badge>
        </button>
      ))}
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
