import { Loader2, Plus, Tags, Trash2 } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { Button } from "@/components/ui/button"
import { TagOperationDialog } from "./TagOperationDialog"

interface DomainBatchActionBarProps {
  selectedCount: number
  isOperating: boolean
  onClearSelection: () => void
  onAddTags: (tags: string[]) => void
  onRemoveTags: (tags: string[]) => void
  onSetTags: (tags: string[]) => void
  selectedDomainsTags: string[]
  allTags: string[]
}

export function DomainBatchActionBar({
  selectedCount,
  isOperating,
  onClearSelection,
  onAddTags,
  onRemoveTags,
  onSetTags,
  selectedDomainsTags,
  allTags,
}: DomainBatchActionBarProps) {
  const { t } = useTranslation()
  const [dialogType, setDialogType] = useState<"add" | "remove" | "set" | null>(null)

  if (selectedCount === 0) return null

  const handleConfirm = (tags: string[]) => {
    if (dialogType === "add") onAddTags(tags)
    else if (dialogType === "remove") onRemoveTags(tags)
    else if (dialogType === "set") onSetTags(tags)
  }

  return (
    <>
      <div className="fixed inset-x-0 bottom-4 z-50 mx-auto flex w-fit items-center gap-3 rounded-full border bg-background px-4 py-2 shadow-lg">
        <span className="text-muted-foreground text-sm">
          {t("domain.selectedCount", { count: selectedCount })}
        </span>
        <Button variant="ghost" size="sm" onClick={onClearSelection} disabled={isOperating}>
          {t("common.deselectAll")}
        </Button>
        <Button
          variant="default"
          size="sm"
          onClick={() => setDialogType("add")}
          disabled={isOperating}
        >
          {isOperating ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Plus className="mr-2 h-4 w-4" />
          )}
          {t("domain.tags.batchAdd")}
        </Button>
        <Button
          variant="secondary"
          size="sm"
          onClick={() => setDialogType("remove")}
          disabled={isOperating}
        >
          <Trash2 className="mr-2 h-4 w-4" />
          {t("domain.tags.batchRemove")}
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setDialogType("set")}
          disabled={isOperating}
        >
          <Tags className="mr-2 h-4 w-4" />
          {t("domain.tags.batchSet")}
        </Button>
      </div>

      {dialogType && (
        <TagOperationDialog
          open
          onClose={() => setDialogType(null)}
          mode={dialogType}
          selectedCount={selectedCount}
          allTags={allTags}
          selectedDomainsTags={selectedDomainsTags}
          onConfirm={handleConfirm}
        />
      )}
    </>
  )
}
