import { Loader2, Trash2 } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"

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

  if (selectedCount === 0) return null

  return (
    <div className="fixed inset-x-0 bottom-4 z-50 mx-auto flex w-fit items-center gap-3 rounded-full border bg-background px-4 py-2 shadow-lg">
      <span className="text-muted-foreground text-sm">
        {t("dns.selectedCount", { count: selectedCount })}
      </span>
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
    </div>
  )
}
