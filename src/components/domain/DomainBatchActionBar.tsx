import { AlertTriangle, Loader2, Plus, Tags, Trash2, X } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Label } from "@/components/ui/label"
import { TagInputCombobox } from "./TagInputCombobox"

interface DomainBatchActionBarProps {
  selectedCount: number
  isOperating: boolean
  onClearSelection: () => void
  onAddTags: (tags: string[]) => void
  onRemoveTags: (tags: string[]) => void
  onSetTags: (tags: string[]) => void
  selectedDomainsTags: string[] // 所有选中域名的标签并集
  allTags: string[] // 所有已使用的标签（用于下拉选择）
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
  const [inputTags, setInputTags] = useState<string[]>([])
  const [inputValue, setInputValue] = useState("")
  const [selectedTagsToRemove, setSelectedTagsToRemove] = useState<Set<string>>(new Set())

  if (selectedCount === 0) return null

  const handleAddTag = () => {
    const trimmed = inputValue.trim()
    if (!trimmed) return

    const newTags = trimmed
      .split(",")
      .map((t) => t.trim())
      .filter((t) => t.length > 0 && t.length <= 50)

    const merged = Array.from(new Set([...inputTags, ...newTags]))

    if (merged.length > 10) {
      return // 超限不添加
    }

    setInputTags(merged)
    setInputValue("")
  }

  const handleRemoveTag = (tag: string) => {
    setInputTags(inputTags.filter((t) => t !== tag))
  }

  // 从已有标签选择
  const handleSelectTag = (tag: string) => {
    if (inputTags.includes(tag)) return
    if (inputTags.length >= 10) return
    setInputTags([...inputTags, tag])
  }

  const handleConfirmAdd = () => {
    if (inputTags.length === 0) return
    onAddTags(inputTags)
    resetDialog()
  }

  const handleConfirmRemove = () => {
    if (selectedTagsToRemove.size === 0) return
    onRemoveTags(Array.from(selectedTagsToRemove))
    resetDialog()
  }

  const handleConfirmSet = () => {
    if (inputTags.length === 0) return
    onSetTags(inputTags)
    resetDialog()
  }

  const resetDialog = () => {
    setDialogType(null)
    setInputTags([])
    setInputValue("")
    setSelectedTagsToRemove(new Set())
  }

  const toggleTagRemoval = (tag: string) => {
    setSelectedTagsToRemove((prev) => {
      const next = new Set(prev)
      if (next.has(tag)) {
        next.delete(tag)
      } else {
        next.add(tag)
      }
      return next
    })
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

      {/* 添加标签对话框 */}
      <Dialog open={dialogType === "add"} onOpenChange={(open) => !open && resetDialog()}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("domain.tags.batchAddTitle")}</DialogTitle>
            <DialogDescription>
              {t("domain.tags.batchAddDescription", { count: selectedCount })}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>{t("domain.tags.inputLabel")}</Label>
              <TagInputCombobox
                value={inputValue}
                onChange={setInputValue}
                onAddTag={handleAddTag}
                onSelectTag={handleSelectTag}
                currentTags={inputTags}
                allTags={allTags}
                placeholder={t("domain.tags.inputPlaceholder")}
                maxLength={50}
              />
              <p className="text-muted-foreground text-xs">{t("domain.tags.inputHint")}</p>
            </div>

            {inputTags.length > 0 && (
              <div className="flex flex-wrap gap-2">
                {inputTags.map((tag) => (
                  <Badge key={tag} variant="secondary" className="group relative pr-6">
                    {tag}
                    <Button
                      variant="ghost"
                      size="icon"
                      className="absolute top-0 right-0 h-full w-5"
                      onClick={() => handleRemoveTag(tag)}
                    >
                      <X className="h-3 w-3" />
                    </Button>
                  </Badge>
                ))}
              </div>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={resetDialog}>
              {t("common.cancel")}
            </Button>
            <Button onClick={handleConfirmAdd} disabled={inputTags.length === 0}>
              {t("domain.tags.addToSelected", { count: selectedCount })}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 移除标签对话框 */}
      <Dialog open={dialogType === "remove"} onOpenChange={(open) => !open && resetDialog()}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("domain.tags.batchRemoveTitle")}</DialogTitle>
            <DialogDescription>
              {t("domain.tags.batchRemoveDescription", { count: selectedCount })}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            {selectedDomainsTags.length === 0 ? (
              <p className="text-muted-foreground text-sm">所选域名暂无标签</p>
            ) : (
              <div className="space-y-2">
                {selectedDomainsTags.map((tag) => (
                  <div key={tag} className="flex items-center space-x-2">
                    <Checkbox
                      id={`remove-${tag}`}
                      checked={selectedTagsToRemove.has(tag)}
                      onCheckedChange={() => toggleTagRemoval(tag)}
                    />
                    <label
                      htmlFor={`remove-${tag}`}
                      className="font-medium text-sm leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                    >
                      {tag}
                    </label>
                  </div>
                ))}
              </div>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={resetDialog}>
              {t("common.cancel")}
            </Button>
            <Button
              variant="destructive"
              onClick={handleConfirmRemove}
              disabled={selectedTagsToRemove.size === 0}
            >
              {t("domain.tags.removeFromSelected", { count: selectedCount })}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 替换标签对话框 */}
      <Dialog open={dialogType === "set"} onOpenChange={(open) => !open && resetDialog()}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t("domain.tags.batchSetTitle")}</DialogTitle>
            <DialogDescription>
              {t("domain.tags.batchSetDescription", { count: selectedCount })}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            {/* 警告提示 */}
            <div className="flex items-start gap-2 rounded-md border border-orange-500/50 bg-orange-500/10 p-3">
              <AlertTriangle className="h-5 w-5 flex-shrink-0 text-orange-500" />
              <p className="text-orange-700 text-sm dark:text-orange-400">
                {t("domain.tags.batchSetWarning")}
              </p>
            </div>

            <div className="space-y-2">
              <Label>{t("domain.tags.inputLabel")}</Label>
              <TagInputCombobox
                value={inputValue}
                onChange={setInputValue}
                onAddTag={handleAddTag}
                onSelectTag={handleSelectTag}
                currentTags={inputTags}
                allTags={allTags}
                placeholder={t("domain.tags.inputPlaceholder")}
                maxLength={50}
              />
              <p className="text-muted-foreground text-xs">{t("domain.tags.inputHint")}</p>
            </div>

            {inputTags.length > 0 && (
              <div className="flex flex-wrap gap-2">
                {inputTags.map((tag) => (
                  <Badge key={tag} variant="secondary" className="group relative pr-6">
                    {tag}
                    <Button
                      variant="ghost"
                      size="icon"
                      className="absolute top-0 right-0 h-full w-5"
                      onClick={() => handleRemoveTag(tag)}
                    >
                      <X className="h-3 w-3" />
                    </Button>
                  </Badge>
                ))}
              </div>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={resetDialog}>
              {t("common.cancel")}
            </Button>
            <Button
              variant="destructive"
              onClick={handleConfirmSet}
              disabled={inputTags.length === 0}
            >
              {t("domain.tags.setForSelected", { count: selectedCount })}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
