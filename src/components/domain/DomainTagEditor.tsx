import { Plus, X } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { toast } from "sonner"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Badge } from "@/components/ui/badge"
import { extractErrorMessage } from "@/lib/error"
import { useDomainStore } from "@/stores"

interface DomainTagEditorProps {
  accountId: string
  domainId: string
  currentTags: string[]
  children?: React.ReactNode
}

export function DomainTagEditor({
  accountId,
  domainId,
  currentTags,
  children,
}: DomainTagEditorProps) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const [tags, setTags] = useState<string[]>(currentTags)
  const [inputValue, setInputValue] = useState("")
  const [isLoading, setIsLoading] = useState(false)

  const setDomainTags = useDomainStore((state) => state.setTags)

  // 重置状态
  const handleOpenChange = (newOpen: boolean) => {
    setOpen(newOpen)
    if (newOpen) {
      setTags(currentTags)
      setInputValue("")
    }
  }

  // 添加标签
  const handleAddTag = () => {
    const trimmed = inputValue.trim()
    if (!trimmed) return

    // 支持逗号分隔批量输入
    const newTags = trimmed
      .split(",")
      .map((t) => t.trim())
      .filter((t) => t.length > 0 && t.length <= 50)

    // 去重合并
    const merged = Array.from(new Set([...tags, ...newTags]))

    if (merged.length > 10) {
      toast.error(t("domain.tags.maxTagsError"))
      return
    }

    setTags(merged)
    setInputValue("")
  }

  // 移除标签
  const handleRemoveTag = (tag: string) => {
    setTags(tags.filter((t) => t !== tag))
  }

  // 保存
  const handleSave = async () => {
    setIsLoading(true)
    try {
      await setDomainTags(accountId, domainId, tags)
      toast.success(t("domain.tags.saveSuccess"))
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
              <Plus className="mr-1 h-4 w-4" />
              {t("domain.tags.edit")}
            </Button>
          )}
        </DialogTrigger>
        <DialogContent className="sm:max-w-[425px]">
          <DialogHeader>
            <DialogTitle>{t("domain.tags.editTitle")}</DialogTitle>
            <DialogDescription>{t("domain.tags.editDescription")}</DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            {/* 输入框 */}
            <div className="space-y-2">
              <Label htmlFor="tag-input">{t("domain.tags.inputLabel")}</Label>
              <div className="flex gap-2">
                <Input
                  id="tag-input"
                  placeholder={t("domain.tags.inputPlaceholder")}
                  value={inputValue}
                  onChange={(e) => setInputValue(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      e.preventDefault()
                      handleAddTag()
                    }
                  }}
                  maxLength={50}
                />
                <Button onClick={handleAddTag} size="sm" disabled={!inputValue.trim()}>
                  <Plus className="h-4 w-4" />
                </Button>
              </div>
              <p className="text-muted-foreground text-xs">
                {t("domain.tags.inputHint")} ({tags.length}/10)
              </p>
            </div>

            {/* 标签列表 */}
            {tags.length > 0 && (
              <div className="space-y-2">
                <Label>{t("domain.tags.currentTags")}</Label>
                <div className="flex flex-wrap gap-2">
                  {tags.map((tag) => (
                    <Badge key={tag} variant="secondary" className="group relative pr-6">
                      <span className="text-xs">{tag}</span>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="absolute top-0 right-0 h-full w-5 p-0 opacity-0 group-hover:opacity-100"
                        onClick={() => handleRemoveTag(tag)}
                      >
                        <X className="h-3 w-3" />
                      </Button>
                    </Badge>
                  ))}
                </div>
              </div>
            )}
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setOpen(false)} disabled={isLoading}>
              {t("common.cancel")}
            </Button>
            <Button onClick={handleSave} disabled={isLoading}>
              {t("common.save")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
