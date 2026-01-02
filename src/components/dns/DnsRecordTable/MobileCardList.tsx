import { Loader2 } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Checkbox } from "@/components/ui/checkbox"
import { DnsRecordCard } from "../DnsRecordCard"
import type { MobileCardListProps } from "./types"

export function MobileCardList({
  records,
  isLoading,
  isLoadingMore,
  isDeleting,
  isSelectMode,
  selectedRecordIds,
  hasActiveFilters,
  supportsProxy,
  domainName,
  onEdit,
  onDelete,
  onToggleSelect,
  onSelectAll,
  onClearSelection,
  setSentinelRef,
}: MobileCardListProps) {
  const { t } = useTranslation()

  return (
    <div className="flex scroll-pb-safe flex-col gap-3 p-4">
      {/* 选择模式下显示全选行 */}
      {isSelectMode && records.length > 0 && (
        <div className="flex items-center gap-2 rounded-lg border bg-muted/50 p-3">
          <Checkbox
            checked={records.every((r) => selectedRecordIds.has(r.id))}
            onCheckedChange={(checked) => {
              if (checked) onSelectAll()
              else onClearSelection()
            }}
          />
          <span className="text-muted-foreground text-sm">{t("common.selectAll")}</span>
        </div>
      )}

      {records.length === 0 ? (
        isLoading ? (
          <div className="py-8 text-center">
            <Loader2 className="mx-auto h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        ) : (
          <div className="py-8 text-center text-muted-foreground">
            {hasActiveFilters ? t("common.noMatch") : t("dns.noRecords")}
          </div>
        )
      ) : (
        <>
          {records.map((record) => (
            <DnsRecordCard
              key={record.id}
              record={record}
              onEdit={() => onEdit(record)}
              onDelete={() => onDelete(record)}
              disabled={isDeleting}
              showProxy={supportsProxy}
              domainName={domainName}
              isSelectMode={isSelectMode}
              isSelected={selectedRecordIds.has(record.id)}
              onToggleSelect={() => onToggleSelect(record.id)}
            />
          ))}
          <div ref={setSentinelRef} className="h-1" />
          {isLoadingMore && (
            <div className="py-4 text-center">
              <Loader2 className="mx-auto h-5 w-5 animate-spin text-muted-foreground" />
            </div>
          )}
        </>
      )}
    </div>
  )
}
