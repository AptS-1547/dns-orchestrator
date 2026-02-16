import { AlertTriangle, X } from "lucide-react"
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

interface TagOperationDialogProps {
  open: boolean
  onClose: () => void
  mode: "add" | "remove" | "set"
  selectedCount: number
  allTags: string[]
  selectedDomainsTags: string[]
  onConfirm: (tags: string[]) => void
}

export function TagOperationDialog({
  open,
  onClose,
  mode,
  selectedCount,
  allTags,
  selectedDomainsTags,
  onConfirm,
}: TagOperationDialogProps) {
  const { t } = useTranslation()
  const [inputTags, setInputTags] = useState<string[]>([])
  const [inputValue, setInputValue] = useState("")
  const [selectedTagsToRemove, setSelectedTagsToRemove] = useState<Set<string>>(new Set())

  const resetAndClose = () => {
    setInputTags([])
    setInputValue("")
    setSelectedTagsToRemove(new Set())
    onClose()
  }

  const handleAddTag = () => {
    const trimmed = inputValue.trim()
    if (!trimmed) return

    const newTags = trimmed
      .split(",")
      .map((t) => t.trim())
      .filter((t) => t.length > 0 && t.length <= 50)

    const merged = Array.from(new Set([...inputTags, ...newTags]))
    if (merged.length > 10) return

    setInputTags(merged)
    setInputValue("")
  }

  const handleSelectTag = (tag: string) => {
    if (inputTags.includes(tag) || inputTags.length >= 10) return
    setInputTags([...inputTags, tag])
  }

  const handleConfirm = () => {
    if (mode === "remove") {
      if (selectedTagsToRemove.size === 0) return
      onConfirm(Array.from(selectedTagsToRemove))
    } else {
      if (inputTags.length === 0) return
      onConfirm(inputTags)
    }
    resetAndClose()
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

  const titleKey = `domain.tags.batch${mode.charAt(0).toUpperCase() + mode.slice(1)}Title`
  const descKey = `domain.tags.batch${mode.charAt(0).toUpperCase() + mode.slice(1)}Description`
  const confirmKey =
    mode === "add"
      ? "domain.tags.addToSelected"
      : mode === "remove"
        ? "domain.tags.removeFromSelected"
        : "domain.tags.setForSelected"

  return (
    <Dialog open={open} onOpenChange={(o) => !o && resetAndClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t(titleKey)}</DialogTitle>
          <DialogDescription>{t(descKey, { count: selectedCount })}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {mode === "set" && (
            <div className="flex items-start gap-2 rounded-md border border-orange-500/50 bg-orange-500/10 p-3">
              <AlertTriangle className="h-5 w-5 flex-shrink-0 text-orange-500" />
              <p className="text-orange-700 text-sm dark:text-orange-400">
                {t("domain.tags.batchSetWarning")}
              </p>
            </div>
          )}

          {mode === "remove" ? (
            selectedDomainsTags.length === 0 ? (
              <p className="text-muted-foreground text-sm">
                {t("domain.tags.noTagsToRemove", { defaultValue: "所选域名暂无标签" })}
              </p>
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
            )
          ) : (
            <>
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
                        onClick={() => setInputTags(inputTags.filter((t) => t !== tag))}
                      >
                        <X className="h-3 w-3" />
                      </Button>
                    </Badge>
                  ))}
                </div>
              )}
            </>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={resetAndClose}>
            {t("common.cancel")}
          </Button>
          <Button
            variant={mode === "remove" || mode === "set" ? "destructive" : "default"}
            onClick={handleConfirm}
            disabled={
              mode === "remove" ? selectedTagsToRemove.size === 0 : inputTags.length === 0
            }
          >
            {t(confirmKey, { count: selectedCount })}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
