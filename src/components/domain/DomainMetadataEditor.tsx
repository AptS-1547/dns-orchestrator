import { ChevronDown, Edit, X } from "lucide-react"
import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { useShallow } from "zustand/react/shallow"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Textarea } from "@/components/ui/textarea"
import type { DomainColorValue } from "@/constants/colors"
import { extractErrorMessage } from "@/lib/error"
import { cn } from "@/lib/utils"
import { useDomainStore } from "@/stores"
import type { DomainMetadata } from "@/types"
import { DomainColorPicker } from "./DomainColorPicker"
import { TagInputCombobox } from "./TagInputCombobox"

interface DomainMetadataEditorProps {
  accountId: string
  domainId: string
  currentMetadata?: DomainMetadata
  children?: React.ReactNode
}

export function DomainMetadataEditor({
  accountId,
  domainId,
  currentMetadata,
  children,
}: DomainMetadataEditorProps) {
  const { t } = useTranslation()
  const { updateMetadata, getAllUsedTags, domainsByAccount } = useDomainStore(
    useShallow((state) => ({
      updateMetadata: state.updateMetadata,
      getAllUsedTags: state.getAllUsedTags,
      domainsByAccount: state.domainsByAccount,
    }))
  )

  const [open, setOpen] = useState(false)
  const [isFavorite, setIsFavorite] = useState(false)
  const [tags, setTags] = useState<string[]>([])
  const [color, setColor] = useState<DomainColorValue>("none")
  const [note, setNote] = useState("")
  const [tagInput, setTagInput] = useState("")
  const [isLoading, setIsLoading] = useState(false)
  const [isColorPickerOpen, setIsColorPickerOpen] = useState(false)

  // 打开时初始化
  const handleOpenChange = (newOpen: boolean) => {
    setOpen(newOpen)
    if (newOpen) {
      setIsFavorite(currentMetadata?.isFavorite ?? false)
      setTags(currentMetadata?.tags ?? [])
      setColor((currentMetadata?.color as DomainColorValue) || "none")
      setNote(currentMetadata?.note ?? "")
      setTagInput("")
    }
  }

  // 添加标签
  const handleAddTag = () => {
    const trimmed = tagInput.trim()
    if (!trimmed) return

    // 支持英文逗号和全角逗号分隔批量输入
    const newTags = trimmed
      .split(/[,，]/)
      .map((t) => t.trim())
      .filter((t) => t.length > 0 && t.length <= 50)

    // 去重合并
    const merged = Array.from(new Set([...tags, ...newTags]))

    if (merged.length > 10) {
      toast.error(t("domain.tags.maxTagsError"))
      return
    }

    setTags(merged)
    setTagInput("")
  }

  // 从已有标签选择
  const handleSelectTag = (tag: string) => {
    if (tags.includes(tag)) return
    if (tags.length >= 10) {
      toast.error(t("domain.tags.maxTagsError"))
      return
    }
    setTags([...tags, tag])
  }

  // 获取所有已使用的标签（memoized）
  const allTags = useMemo(() => getAllUsedTags(), [domainsByAccount, getAllUsedTags])

  // 移除标签
  const handleRemoveTag = (tag: string) => {
    setTags(tags.filter((t) => t !== tag))
  }

  // 保存
  const handleSave = async () => {
    // 备注长度验证
    if (note.length > 500) {
      toast.error(t("domain.note.maxLength"))
      return
    }

    setIsLoading(true)
    try {
      await updateMetadata(accountId, domainId, {
        isFavorite,
        tags,
        color,
        note: note.trim() === "" ? null : note,
      })
      toast.success(t("domain.metadata.saveSuccess"))
      setOpen(false)
    } catch (error) {
      toast.error(extractErrorMessage(error))
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div onClick={(e) => e.stopPropagation()}>
      <Dialog open={open} onOpenChange={handleOpenChange}>
        <DialogTrigger asChild>
          {children || (
            <Button variant="outline" size="sm">
              <Edit className="mr-1 h-4 w-4" />
              {t("domain.metadata.edit")}
            </Button>
          )}
        </DialogTrigger>
        <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle>{t("domain.metadata.editTitle")}</DialogTitle>
            <DialogDescription>{t("domain.metadata.editDescription")}</DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            {/* 收藏开关 */}
            <div className="flex items-center justify-between">
              <Label htmlFor="favorite">{t("favorites.title")}</Label>
              <Switch id="favorite" checked={isFavorite} onCheckedChange={setIsFavorite} />
            </div>

            {/* 标签编辑 */}
            <div className="space-y-2">
              <Label htmlFor="tag-input">{t("domain.tags.inputLabel")}</Label>
              <TagInputCombobox
                inputId="tag-input"
                value={tagInput}
                onChange={setTagInput}
                onAddTag={handleAddTag}
                onSelectTag={handleSelectTag}
                currentTags={tags}
                allTags={allTags}
                placeholder={t("domain.tags.inputPlaceholder")}
                maxLength={50}
              />
              <div className="flex items-center justify-between text-xs">
                <p className="text-muted-foreground">{t("domain.tags.inputHint")}</p>
                <div className="flex gap-3 text-muted-foreground">
                  <span className={cn(tagInput.length > 45 && "text-destructive")}>
                    {tagInput.length}/50
                  </span>
                  <span>{tags.length}/10</span>
                </div>
              </div>

              {/* 已添加的标签列表 */}
              {tags.length > 0 && (
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <Label>{t("domain.tags.currentTags")}</Label>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => setTags([])}
                      className="h-auto p-1 text-muted-foreground text-xs hover:text-destructive"
                    >
                      {t("common.clearAll")}
                    </Button>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {tags.map((tag) => (
                      <Badge key={tag} variant="secondary" className="group relative pr-6">
                        <span className="inline-block max-w-[100px] truncate align-bottom text-xs">
                          {tag}
                        </span>
                        <Button
                          variant="ghost"
                          size="icon"
                          className="absolute top-0 right-0 h-full w-5 p-0 opacity-100 transition-opacity [@media(hover:hover)]:opacity-0 [@media(hover:hover)]:group-hover:opacity-100"
                          onClick={() => handleRemoveTag(tag)}
                          aria-label={`Remove ${tag}`}
                        >
                          <X className="h-3 w-3" />
                        </Button>
                      </Badge>
                    ))}
                  </div>
                </div>
              )}
            </div>

            {/* 颜色选择器（可折叠） */}
            <Collapsible open={isColorPickerOpen} onOpenChange={setIsColorPickerOpen}>
              <CollapsibleTrigger asChild>
                <Button
                  variant="ghost"
                  className="w-full justify-between px-0 font-normal hover:bg-transparent"
                >
                  <Label className="cursor-pointer">{t("domain.color.label")}</Label>
                  <ChevronDown
                    className={cn(
                      "h-4 w-4 transition-transform",
                      isColorPickerOpen && "rotate-180"
                    )}
                  />
                </Button>
              </CollapsibleTrigger>
              <CollapsibleContent className="pt-2">
                <DomainColorPicker value={color} onChange={setColor} />
              </CollapsibleContent>
            </Collapsible>

            {/* 备注输入 */}
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label htmlFor="note">{t("domain.note.label")}</Label>
                <span
                  className={cn(
                    "text-muted-foreground text-xs",
                    note.length > 450 && "text-yellow-600 dark:text-yellow-500",
                    note.length > 500 && "text-destructive"
                  )}
                >
                  {note.length}/500
                </span>
              </div>
              <Textarea
                id="note"
                placeholder={t("domain.note.placeholder")}
                value={note}
                onChange={(e) => setNote(e.target.value)}
                rows={4}
                maxLength={550}
                className="resize-none"
              />
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setOpen(false)} disabled={isLoading}>
              {t("common.cancel")}
            </Button>
            <Button onClick={handleSave} disabled={isLoading || note.length > 500}>
              {t("common.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
